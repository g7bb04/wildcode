use crate::events::EventSender;
use crate::proc;
use crate::toolchain::ESP32_INDEX_URL;
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

    run_command(&mut command, tx, task, cancel)
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
}
