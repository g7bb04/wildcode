use crate::boards::{fallback_boards, Board};
use crate::config::Config;
use crate::events::{BgEvent, EventSender};
use crate::proc;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub const ESP32_INDEX_URL: &str =
    "https://espressif.github.io/arduino-esp32/package_esp32_index.json";
/// Official China index recommended by Espressif. Its `-cn` platform and tool
/// versions point at `dl.espressif.cn`, so installing an explicit `-cn`
/// version keeps every download on the China mirror.
const ESP32_CN_INDEX_URL: &str =
    "https://jihulab.com/esp-mirror/espressif/arduino-esp32/-/raw/gh-pages/package_esp32_index_cn.json";
const CLI_CDN_BASE: &str = "https://downloads.arduino.cc/arduino-cli/";
const CLI_FALLBACK_VERSION: &str = "1.5.1";
const RELEASE_API: &str = "https://api.github.com/repos/arduino/arduino-cli/releases/latest";
const USER_AGENT: &str = "WildIDE/0.1 (+https://github.com/arduino/arduino-cli)";

#[derive(Debug, Clone, Default)]
pub struct Toolchain {
    pub cli: Option<PathBuf>,
    pub version: Option<String>,
}

impl Toolchain {
    /// Look for Arduino CLI in the user setting, next to the app's data, and
    /// finally on PATH.
    pub fn detect(explicit: &str) -> Self {
        let mut candidates = Vec::new();
        let explicit = explicit.trim();
        if !explicit.is_empty() {
            candidates.push(PathBuf::from(explicit));
        }
        candidates.push(local_cli_path());
        candidates.push(PathBuf::from(if cfg!(windows) {
            "arduino-cli.exe"
        } else {
            "arduino-cli"
        }));

        for candidate in candidates {
            if candidate.components().count() > 1 && !candidate.exists() {
                continue;
            }
            if let Some(version) = version_of(&candidate) {
                return Self {
                    cli: Some(candidate),
                    version: Some(version),
                };
            }
        }
        Self::default()
    }

    pub fn display_path(&self) -> String {
        self.cli
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "未找到".to_owned())
    }
}

pub fn local_cli_path() -> PathBuf {
    Config::tools_dir().join(if cfg!(windows) {
        "arduino-cli.exe"
    } else {
        "arduino-cli"
    })
}

pub fn version_of(cli: &Path) -> Option<String> {
    let mut command = Command::new(cli);
    command.arg("version");
    let output = proc::run_quiet(&mut command).ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    // "arduino-cli  Version: 1.1.1 Commit: ..." -> "1.1.1"
    if let Some(rest) = line.split("Version:").nth(1) {
        return Some(rest.split_whitespace().next().unwrap_or(rest).to_owned());
    }
    Some(line.to_owned())
}

pub fn core_installed(cli: &Path) -> bool {
    let mut command = Command::new(cli);
    command.args(["core", "list", "--format", "json"]);
    let Ok(output) = proc::run_quiet(&mut command) else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    parse_core_list(&output.stdout)
}

fn parse_core_list(bytes: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return false;
    };
    let Some(core) = value
        .get("platforms")
        .and_then(Value::as_array)
        .or_else(|| value.as_array())
    else {
        return false;
    };
    core.iter().any(|item| {
        item.get("id")
            .and_then(Value::as_str)
            .map(|id| id == "esp32:esp32" || id.starts_with("esp32:esp32@"))
            .unwrap_or(false)
    })
}

pub fn list_esp32_boards(cli: &Path) -> Vec<Board> {
    let mut command = Command::new(cli);
    command.args(["board", "listall", "esp32", "--format", "json"]);
    let Ok(output) = proc::run_quiet(&mut command) else {
        return fallback_boards();
    };
    if !output.status.success() {
        return fallback_boards();
    }
    let boards = parse_board_list(&output.stdout);
    if boards.is_empty() {
        fallback_boards()
    } else {
        boards
    }
}

fn parse_board_list(bytes: &[u8]) -> Vec<Board> {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return Vec::new();
    };
    let Some(items) = value.get("boards").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut boards = Vec::new();
    for item in items {
        let (Some(name), Some(fqbn)) = (
            item.get("name").and_then(Value::as_str),
            item.get("fqbn").and_then(Value::as_str),
        ) else {
            continue;
        };
        if !fqbn.starts_with("esp32:esp32:") {
            continue;
        }
        boards.push(Board::new(name, fqbn));
    }
    boards.sort_by_key(|board| board.name.to_lowercase());
    boards.dedup_by(|a, b| a.fqbn == b.fqbn);
    boards
}

/// Download the latest stable Arduino CLI and extract it into the app tools folder.
///
/// The official `downloads.arduino.cc` CDN is preferred over GitHub Releases,
/// because it is normally reachable from mainland China. GitHub remains a
/// fallback when the CDN or its version lookup is unavailable.
pub fn install_cli(
    target: &Path,
    tx: &EventSender,
    task: u64,
    cancel: &AtomicBool,
) -> Result<(), String> {
    emit(tx, task, "正在查询 Arduino CLI 最新稳定版本…");
    let release = fetch_latest_release();
    let version = release
        .as_ref()
        .and_then(|value| value.get("tag_name").and_then(Value::as_str))
        .map(|tag| tag.trim_start_matches('v').to_owned())
        .filter(|tag| !tag.is_empty());
    let github_url = release.as_ref().and_then(github_asset_url);

    let mut downloaded: Option<(Vec<u8>, String)> = None;

    if let Some(version) = version.as_deref() {
        if let Some(name) = cdn_asset_name(version) {
            let url = format!("{CLI_CDN_BASE}{name}");
            emit(tx, task, format!("正在从 Arduino 官方 CDN 下载 {version}…"));
            match http_get(&url, 300) {
                Ok(data) => downloaded = Some((data, url)),
                Err(err) => emit(tx, task, format!("官方 CDN 下载失败：{err}")),
            }
        }
    }

    if downloaded.is_none() && version.is_none() {
        emit(
            tx,
            task,
            format!("无法查询最新版本，尝试内置稳定版本 {CLI_FALLBACK_VERSION}…"),
        );
        if let Some(name) = cdn_asset_name(CLI_FALLBACK_VERSION) {
            let url = format!("{CLI_CDN_BASE}{name}");
            match http_get(&url, 300) {
                Ok(data) => downloaded = Some((data, url)),
                Err(err) => emit(tx, task, format!("官方 CDN 下载失败：{err}")),
            }
        }
    }

    if downloaded.is_none() {
        if let Some(url) = github_url {
            emit(tx, task, "正在从 GitHub 下载（备用线路）…");
            let data = http_get(&url, 300)?;
            downloaded = Some((data, url));
        } else {
            return Err(
                "无法下载 Arduino CLI：官方 CDN 和 GitHub 均不可用，请检查网络或在设置中手动指定"
                    .to_owned(),
            );
        }
    }

    let (data, source_url) =
        downloaded.ok_or_else(|| "无法下载 Arduino CLI，请检查网络".to_owned())?;

    emit(
        tx,
        task,
        format!(
            "下载完成（{:.1} MB），正在解压…",
            data.len() as f64 / 1_048_576.0
        ),
    );

    if cancel.load(Ordering::Relaxed) {
        return Err("已取消".to_owned());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建工具目录失败: {err}"))?;
    }
    let temp = target.with_extension("download");
    let _ = fs::remove_file(&temp);

    if source_url.ends_with(".zip") {
        extract_zip(&data, &temp)?;
    } else if source_url.ends_with(".tar.gz") {
        extract_tar_gz(&data, &temp)?;
    } else {
        return Err("未知的 Arduino CLI 压缩格式".to_owned());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&temp, fs::Permissions::from_mode(0o755));
    }

    if target.exists() {
        let _ = fs::remove_file(target);
    }
    fs::rename(&temp, target).map_err(|err| format!("安装 Arduino CLI 失败: {err}"))?;

    if version_of(target).is_none() {
        return Err("安装完成，但无法运行 arduino-cli，请手动安装".to_owned());
    }
    emit(tx, task, "Arduino CLI 安装完成。");
    Ok(())
}

fn fetch_latest_release() -> Option<Value> {
    let response = ureq::get(RELEASE_API)
        .set("User-Agent", USER_AGENT)
        .timeout(Duration::from_secs(20))
        .call()
        .ok()?;
    let body = response.into_string().ok()?;
    serde_json::from_str(&body).ok()
}

fn github_asset_url(release: &Value) -> Option<String> {
    let assets = release.get("assets")?.as_array()?;
    let suffix = release_asset_suffix();
    assets.iter().find_map(|asset| {
        let name = asset.get("name").and_then(Value::as_str)?;
        if name.ends_with(suffix) {
            asset
                .get("browser_download_url")
                .and_then(Value::as_str)
                .map(str::to_owned)
        } else {
            None
        }
    })
}

/// File name used by the official Arduino CLI CDN for the current platform.
fn cdn_asset_name(version: &str) -> Option<String> {
    if version.is_empty()
        || !version
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
    {
        return None;
    }
    let suffix = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64" | "aarch64") => "Windows_64bit.zip",
        ("linux", "x86_64") => "Linux_64bit.tar.gz",
        ("linux", "aarch64") => "Linux_ARM64.tar.gz",
        ("macos", "x86_64") => "macOS_64bit.tar.gz",
        ("macos", "aarch64") => "macOS_ARM64.tar.gz",
        _ => return None,
    };
    Some(format!("arduino-cli_{version}_{suffix}"))
}

fn http_get(url: &str, timeout_secs: u64) -> Result<Vec<u8>, String> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .timeout(Duration::from_secs(timeout_secs))
        .call()
        .map_err(|err| format!("请求失败: {err}"))?;
    let mut data = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut data)
        .map_err(|err| format!("读取响应失败: {err}"))?;
    if data.is_empty() {
        return Err("响应内容为空".to_owned());
    }
    Ok(data)
}

fn extract_zip(data: &[u8], target: &Path) -> Result<(), String> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(data)).map_err(|err| format!("解压失败: {err}"))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| format!("解压失败: {err}"))?;
        let file_name = entry
            .enclosed_name()
            .and_then(|path| path.file_name().map(|name| name.to_owned()));
        let Some(file_name) = file_name else {
            continue;
        };
        if file_name.to_string_lossy() == "arduino-cli.exe"
            || file_name.to_string_lossy() == "arduino-cli"
        {
            let mut file = File::create(target).map_err(|err| format!("写入失败: {err}"))?;
            io::copy(&mut entry, &mut file).map_err(|err| format!("写入失败: {err}"))?;
            file.flush().ok();
            return Ok(());
        }
    }
    Err("压缩包中未找到 arduino-cli".to_owned())
}

fn extract_tar_gz(data: &[u8], target: &Path) -> Result<(), String> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(data));
    let mut archive = tar::Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|err| format!("解压失败: {err}"))?;
    for entry in entries {
        let mut entry = entry.map_err(|err| format!("解压失败: {err}"))?;
        let file_name = entry
            .path()
            .ok()
            .and_then(|path| path.file_name().map(|name| name.to_owned()));
        if file_name.as_deref() == Some(std::ffi::OsStr::new("arduino-cli")) {
            let mut file = File::create(target).map_err(|err| format!("写入失败: {err}"))?;
            io::copy(&mut entry, &mut file).map_err(|err| format!("写入失败: {err}"))?;
            file.flush().ok();
            return Ok(());
        }
    }
    Err("压缩包中未找到 arduino-cli".to_owned())
}

fn release_asset_suffix() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64" | "aarch64") => "Windows_64bit.zip",
        ("linux", "x86_64") => "Linux_64bit.tar.gz",
        ("linux", "aarch64") => "Linux_ARM64.tar.gz",
        ("macos", "x86_64") => "macOS_64bit.tar.gz",
        ("macos", "aarch64") => "macOS_ARM64.tar.gz",
        _ => "Windows_64bit.zip",
    }
}

pub fn install_esp32_core(
    cli: &Path,
    tx: &EventSender,
    task: u64,
    cancel: &AtomicBool,
    use_china_mirror: bool,
) -> Result<(), String> {
    let (additional_url, spec) = if use_china_mirror {
        emit(tx, task, "正在获取 Espressif 中国镜像索引（Jihulab）…");
        match fetch_latest_cn_version() {
            Ok(version) => {
                emit(
                    tx,
                    task,
                    format!("已启用 Espressif 中国镜像，将安装 esp32:esp32@{version}。"),
                );
                (
                    ESP32_CN_INDEX_URL.to_owned(),
                    format!("esp32:esp32@{version}"),
                )
            }
            Err(err) => {
                emit(
                    tx,
                    task,
                    format!("中国镜像索引不可用：{err}；将使用官方源继续。"),
                );
                (ESP32_INDEX_URL.to_owned(), "esp32:esp32".to_owned())
            }
        }
    } else {
        (ESP32_INDEX_URL.to_owned(), "esp32:esp32".to_owned())
    };

    run_core_install(cli, tx, task, cancel, &additional_url, &spec)
}

fn run_core_install(
    cli: &Path,
    tx: &EventSender,
    task: u64,
    cancel: &AtomicBool,
    additional_url: &str,
    spec: &str,
) -> Result<(), String> {
    emit(tx, task, "正在更新 ESP32 开发板索引（需要联网）…");
    let mut update = Command::new(cli);
    update.args(["core", "update-index", "--additional-urls", additional_url]);
    let status = proc::run_streaming(&mut update, tx, task, cancel)
        .map_err(|err| format!("无法运行 Arduino CLI: {err}"))?;
    if cancel.load(Ordering::Relaxed) {
        return Err("已取消".to_owned());
    }
    if !status.success() {
        return Err("更新开发板索引失败".to_owned());
    }

    emit(
        tx,
        task,
        format!("正在安装 {spec}（首次安装需要下载较多文件，请耐心等待）…"),
    );
    let mut install = Command::new(cli);
    install.args(["core", "install", spec, "--additional-urls", additional_url]);
    let status = proc::run_streaming(&mut install, tx, task, cancel)
        .map_err(|err| format!("无法运行 Arduino CLI: {err}"))?;
    if cancel.load(Ordering::Relaxed) {
        return Err("已取消".to_owned());
    }
    if !status.success() {
        return Err("安装 ESP32 开发支持失败".to_owned());
    }
    emit(tx, task, "ESP32 开发支持安装完成。");
    Ok(())
}

/// Fetch the official China index and pick the newest `-cn` platform version.
fn fetch_latest_cn_version() -> Result<String, String> {
    let bytes = http_get(ESP32_CN_INDEX_URL, 45)?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|err| format!("解析中国镜像索引失败: {err}"))?;
    latest_cn_version(&value).ok_or_else(|| "中国镜像索引中没有找到 -cn 版本".to_owned())
}

fn latest_cn_version(index: &Value) -> Option<String> {
    index
        .get("packages")?
        .as_array()?
        .iter()
        .filter(|package| package.get("name").and_then(Value::as_str) == Some("esp32"))
        .flat_map(|package| {
            package
                .get("platforms")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|platform| platform.get("version").and_then(Value::as_str))
        .filter(|version| version.ends_with("-cn"))
        .max_by_key(|version| version_key(version))
        .map(str::to_owned)
}

fn version_key(version: &str) -> Vec<u64> {
    version
        .trim_end_matches("-cn")
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

fn emit(tx: &EventSender, task: u64, line: impl Into<String>) {
    let _ = tx.send(BgEvent::Output {
        task,
        line: line.into(),
    });
}

/// Used by the UI for a friendly hint when opening the serial monitor.
pub fn wait_after_upload() -> Duration {
    Duration::from_millis(1200)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_core_list() {
        let json = br#"[{"id":"esp32:esp32","installed":"2.0.17"}]"#;
        assert!(parse_core_list(json));
        let json = br#"[{"id":"arduino:avr","installed":"1.8.6"}]"#;
        assert!(!parse_core_list(json));
        let json = br#"{"platforms":[{"id":"esp32:esp32"}]}"#;
        assert!(parse_core_list(json));
    }

    #[test]
    fn parses_board_list() {
        let json = br#"{"boards":[
            {"name":"ESP32 Dev Module","fqbn":"esp32:esp32:esp32"},
            {"name":"Arduino Uno","fqbn":"arduino:avr:uno"},
            {"name":"ESP32-S3 Dev Module","fqbn":"esp32:esp32:esp32s3"}
        ]}"#;
        let boards = parse_board_list(json);
        assert_eq!(boards.len(), 2);
        assert_eq!(boards[0].name, "ESP32 Dev Module");
        assert_eq!(boards[1].fqbn, "esp32:esp32:esp32s3");
    }

    #[test]
    fn picks_latest_cn_version() {
        let index = serde_json::json!({
            "packages": [{
                "name": "esp32",
                "platforms": [
                    {"version": "3.3.9-cn"},
                    {"version": "3.3.10-cn"},
                    {"version": "3.3.12"},
                    {"version": "3.2.1-cn"}
                ]
            }]
        });
        assert_eq!(latest_cn_version(&index).as_deref(), Some("3.3.10-cn"));
        assert_eq!(version_key("3.3.10-cn"), vec![3, 3, 10]);
        assert_eq!(version_key("3.3.9-cn"), vec![3, 3, 9]);
        assert!(latest_cn_version(&serde_json::json!({"packages": []})).is_none());
    }

    #[test]
    fn cdn_names_are_valid() {
        assert!(cdn_asset_name("1.5.1").is_some());
        assert!(cdn_asset_name("../escape").is_none());
        assert!(cdn_asset_name("").is_none());
    }
}
