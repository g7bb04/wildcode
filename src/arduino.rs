use crate::events::{BgEvent, EventSender};
use crate::proc;
use crate::toolchain::ESP32_INDEX_URL;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Give Windows time to release the CDC handle before esptool opens it.
pub const UPLOAD_PORT_RELEASE_DELAY: Duration = Duration::from_millis(500);
/// How long to wait before retrying an upload that failed to open the port.
pub const UPLOAD_RETRY_DELAY: Duration = Duration::from_millis(1500);
/// Total upload attempts (the first try plus one retry).
pub const UPLOAD_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunOutcome {
    pub success: bool,
    pub code: Option<i32>,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryUse {
    pub used: u64,
    pub total: u64,
}

impl MemoryUse {
    pub fn percent(self) -> u64 {
        self.used
            .saturating_mul(100)
            .checked_div(self.total)
            .unwrap_or(0)
    }
}

/// Facts collected from `arduino-cli` output while a build or upload runs.
#[derive(Debug, Clone, Default)]
pub struct TaskReport {
    pub flash: Option<MemoryUse>,
    pub ram: Option<MemoryUse>,
    pub warnings: usize,
    pub errors: usize,
    pub chip: Option<String>,
    pub mac: Option<String>,
    pub wrote_bytes: Option<u64>,
    pub compressed_bytes: Option<u64>,
    pub speed_kbps: Option<f64>,
    pub verified: bool,
    pub upload_attempts: usize,
}

impl TaskReport {
    pub fn observe(&mut self, line: &str) {
        if line.contains("warning:") {
            self.warnings += 1;
        }
        if line.contains("error:") || line.contains("A fatal error occurred") {
            self.errors += 1;
        }

        if line.starts_with("Sketch uses ") {
            if let (Some(used), Some(total)) = (
                number_after(line, "Sketch uses "),
                number_after(line, "Maximum is "),
            ) {
                self.flash = Some(MemoryUse { used, total });
            }
        }
        if line.starts_with("Global variables use ") {
            if let (Some(used), Some(total)) = (
                number_after(line, "Global variables use "),
                number_after(line, "Maximum is "),
            ) {
                self.ram = Some(MemoryUse { used, total });
            }
        }

        if let Some(rest) = line.strip_prefix("Chip is ") {
            let chip = rest.split(" (").next().unwrap_or(rest).trim();
            if !chip.is_empty() {
                self.chip = Some(chip.to_owned());
            }
        }
        if let Some(rest) = line.strip_prefix("MAC: ") {
            let mac = rest.trim();
            if !mac.is_empty() {
                self.mac = Some(mac.to_owned());
            }
        }
        if line.starts_with("Wrote ") {
            if let Some(bytes) = number_after(line, "Wrote ") {
                self.wrote_bytes = Some(bytes);
            }
            if let Some(open) = line.find('(') {
                let rest = &line[open + 1..];
                self.compressed_bytes = rest
                    .split_whitespace()
                    .next()
                    .and_then(|value| value.parse().ok());
            }
            if let Some(speed) = number_before(line, " kbit/s") {
                self.speed_kbps = Some(speed);
            }
        }
        if line.contains("Hash of data verified") {
            self.verified = true;
        }
        if line.starts_with("上传第 ") {
            self.upload_attempts += 1;
        }
    }

    /// One-line upload summary, e.g. `ESP32-S2 · MAC ... · 962.9 kbit/s`.
    pub fn upload_summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(chip) = &self.chip {
            parts.push(chip.clone());
        }
        if let Some(mac) = &self.mac {
            parts.push(format!("MAC {mac}"));
        }
        if let Some(bytes) = self.wrote_bytes {
            let compressed = self
                .compressed_bytes
                .map(|value| format!("（压缩 {value}）"))
                .unwrap_or_default();
            parts.push(format!("写入 {bytes} 字节{compressed}"));
        }
        if let Some(speed) = self.speed_kbps {
            parts.push(format!("{speed:.1} kbit/s"));
        }
        if self.verified {
            parts.push("校验通过".to_owned());
        }
        if self.upload_attempts > 0 {
            parts.push(format!("尝试 {} 次", self.upload_attempts));
        }
        parts.join(" · ")
    }
}

fn number_after(text: &str, marker: &str) -> Option<u64> {
    let rest = text.split_once(marker)?.1;
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn number_before(text: &str, marker: &str) -> Option<f64> {
    let before = text.split_once(marker)?.0;
    let number: String = before
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    number.parse().ok()
}

/// Compile an Arduino sketch into a stable build directory.
pub fn compile(
    cli: &Path,
    sketch_dir: &Path,
    fqbn: &str,
    build_dir: &Path,
    tx: &EventSender,
    task: u64,
    cancel: &AtomicBool,
) -> Result<RunOutcome, String> {
    fs::create_dir_all(build_dir).map_err(|err| format!("无法创建编译目录: {err}"))?;

    let mut command = Command::new(cli);
    command
        .arg("compile")
        .arg("--fqbn")
        .arg(fqbn)
        .arg("--additional-urls")
        .arg(ESP32_INDEX_URL)
        .arg("--warnings")
        .arg("default")
        .arg("--build-path")
        .arg(build_dir)
        .arg(sketch_dir);

    emit_command(&command, tx, task);
    run_command(&mut command, tx, task, cancel)
}

/// Upload binaries previously produced by [`compile`].
pub fn upload(
    cli: &Path,
    fqbn: &str,
    port: &str,
    build_dir: &Path,
    tx: &EventSender,
    task: u64,
    cancel: &AtomicBool,
) -> Result<RunOutcome, String> {
    let mut command = Command::new(cli);
    command
        .arg("upload")
        .arg("--fqbn")
        .arg(fqbn)
        .arg("--port")
        .arg(port)
        .arg("--input-dir")
        .arg(build_dir)
        .arg("--additional-urls")
        .arg(ESP32_INDEX_URL)
        // Native-USB ESP32-S2/S3 boards can disappear for a moment while
        // esptool performs the 1200-bps touch reset.
        .arg("--discovery-timeout")
        .arg("10s");

    emit_command(&command, tx, task);
    run_command(&mut command, tx, task, cancel)
}

fn emit_command(command: &Command, tx: &EventSender, task: u64) {
    let mut parts = vec![quote_arg(command.get_program())];
    parts.extend(command.get_args().map(quote_arg));
    let _ = tx.send(BgEvent::Output {
        task,
        line: format!("$ {}", parts.join(" ")),
    });
}

fn quote_arg(value: &OsStr) -> String {
    let text = value.to_string_lossy();
    if text.contains(' ') {
        format!("\"{text}\"")
    } else {
        text.into_owned()
    }
}

fn run_command(
    command: &mut Command,
    tx: &EventSender,
    task: u64,
    cancel: &AtomicBool,
) -> Result<RunOutcome, String> {
    let status = proc::run_streaming(command, tx, task, cancel)
        .map_err(|err| format!("无法启动 Arduino CLI: {err}"))?;
    let cancelled = cancel.load(Ordering::Relaxed);
    Ok(RunOutcome {
        success: status.success() && !cancelled,
        code: status.code(),
        cancelled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_is_plain_data() {
        let outcome = RunOutcome {
            success: true,
            code: Some(0),
            cancelled: false,
        };
        assert!(outcome.success);
    }

    #[test]
    fn retry_timing_is_reasonable() {
        assert!(UPLOAD_PORT_RELEASE_DELAY >= Duration::from_millis(300));
        assert!(UPLOAD_RETRY_DELAY >= Duration::from_millis(1000));
        assert_eq!(UPLOAD_ATTEMPTS, 2);
    }

    #[test]
    fn parses_compile_memory_usage() {
        let mut report = TaskReport::default();
        report.observe(
            "Sketch uses 303088 bytes (23%) of program storage space. Maximum is 1310720 bytes.",
        );
        report.observe(
            "Global variables use 48800 bytes (14%) of dynamic memory, leaving 278880 bytes for local variables. Maximum is 327680 bytes.",
        );
        let flash = report.flash.unwrap();
        assert_eq!(flash.used, 303088);
        assert_eq!(flash.total, 1_310_720);
        assert_eq!(flash.percent(), 23);
        let ram = report.ram.unwrap();
        assert_eq!(ram.used, 48_800);
        assert_eq!(ram.total, 327_680);
        assert_eq!(ram.percent(), 14);
    }

    #[test]
    fn parses_upload_details() {
        let mut report = TaskReport::default();
        report.observe("Chip is ESP32-S2 (revision v1.0)");
        report.observe("MAC: 7c:df:a1:12:34:56");
        report.observe("上传第 1/2 次 · COM6");
        report.observe(
            "Wrote 303232 bytes (175042 compressed) at 0x00010000 in 2.5 seconds (963.2 kbit/s).",
        );
        report.observe("Hash of data verified.");
        assert_eq!(report.chip.as_deref(), Some("ESP32-S2"));
        assert_eq!(report.mac.as_deref(), Some("7c:df:a1:12:34:56"));
        assert_eq!(report.wrote_bytes, Some(303_232));
        assert_eq!(report.compressed_bytes, Some(175_042));
        assert!((report.speed_kbps.unwrap() - 963.2).abs() < f64::EPSILON);
        assert_eq!(report.upload_attempts, 1);
        assert!(report.verified);
        assert!(report.upload_summary().contains("校验通过"));
    }

    #[test]
    fn counts_diagnostics() {
        let mut report = TaskReport::default();
        report.observe("warning: unused variable 'x'");
        report.observe("src/main.cpp:1:1: error: expected ';'");
        report.observe("A fatal error occurred: Could not open port");
        assert_eq!(report.warnings, 1);
        assert_eq!(report.errors, 2);
    }
}
