use crate::arduino;
use crate::boards::{self, Board};
use crate::config::{self, Config, ThemeChoice, APP_NAME, APP_NAME_ZH};
use crate::editor;
use crate::events::{BgEvent, EnvInfo};
use crate::runner::{Runner, TaskKind};
use crate::serial::SerialManager;
use crate::theme::{self, palette, ACCENT, ERROR, OK, WARN};
use crate::toolchain::{self, Toolchain};
use crate::util;
use eframe::egui::{
    self, Align, Button, Color32, Frame, Label, Layout, Margin, RichText, Rounding, ScrollArea,
    Stroke, TextEdit, TextWrapMode, Ui,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_LOG_LINES: usize = 1200;
const BAUD_RATES: &[u32] = &[
    9600, 19200, 38400, 57600, 74_880, 115_200, 230_400, 460_800, 921_600, 1_500_000,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum BottomTab {
    Serial,
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    None,
    Compile,
    Upload,
    ToggleSerial,
    RefreshPorts,
    RefreshBoards,
    Settings,
    Save,
    SaveAs,
    Open,
    InstallCore,
    InstallCli,
    StopTask,
    ClearSerial,
    ClearOutput,
    CopyOutput,
    SendSerial,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusKind {
    Info,
    Success,
    Error,
    Busy,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LogLevel {
    Info,
    Command,
    Success,
    Warn,
    Error,
}

#[derive(Clone)]
struct LogLine {
    text: String,
    level: LogLevel,
}

pub struct WildApp {
    cfg: Config,
    code: String,
    last_saved: String,
    last_autosave: Instant,
    last_config_save: Instant,
    config_dirty: bool,

    toolchain: Toolchain,
    cli_version: Option<String>,
    core_installed: bool,
    env_checking: bool,
    env_generation: u64,
    boards: Vec<Board>,

    ports: Vec<String>,
    serial: SerialManager,
    serial_input: String,
    serial_was_open: bool,

    bottom_tab: BottomTab,
    logs: Vec<LogLine>,
    log_has_error: bool,
    status: String,
    status_kind: StatusKind,

    runner: Option<Runner>,
    next_task_id: u64,
    tx: Sender<BgEvent>,
    rx: Receiver<BgEvent>,

    show_settings: bool,
    settings_cli: String,
    settings_error: Option<String>,

    reopen_at: Option<Instant>,
    reopen_attempts: u8,
    focus_serial_at: Option<Instant>,
    last_port_refresh: Instant,
}

impl WildApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);

        let cfg = Config::load();
        if let Err(err) = cfg.ensure_sketch() {
            eprintln!("failed to create sketch: {err}");
        }
        let code = fs::read_to_string(cfg.sketch_path())
            .unwrap_or_else(|_| config::DEFAULT_CODE.to_owned());
        theme::apply(&cc.egui_ctx, cfg.theme, cfg.font_size);

        let toolchain = Toolchain::detect(&cfg.cli_path);
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            last_saved: code.clone(),
            code,
            cfg,
            last_autosave: Instant::now(),
            last_config_save: Instant::now(),
            config_dirty: false,
            cli_version: toolchain.version.clone(),
            toolchain,
            core_installed: false,
            env_checking: false,
            env_generation: 0,
            boards: Vec::new(),
            ports: Vec::new(),
            serial: SerialManager::default(),
            serial_input: String::new(),
            serial_was_open: false,
            bottom_tab: BottomTab::Output,
            logs: Vec::new(),
            log_has_error: false,
            status: "就绪".to_owned(),
            status_kind: StatusKind::Info,
            runner: None,
            next_task_id: 1,
            tx,
            rx,
            show_settings: false,
            settings_cli: String::new(),
            settings_error: None,
            reopen_at: None,
            reopen_attempts: 0,
            focus_serial_at: None,
            last_port_refresh: Instant::now(),
        };

        app.serial.baud = app.cfg.baud;
        app.refresh_ports();
        app.refresh_env();
        if app.toolchain.cli.is_some() {
            app.push_log(
                format!(
                    "Wild IDE {} · Arduino CLI {}",
                    APP_VERSION,
                    app.toolchain.version.as_deref().unwrap_or("?")
                ),
                LogLevel::Info,
            );
        }
        app
    }

    // ---------------------------------------------------------------- events

    fn poll_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                BgEvent::Output { task, line } => {
                    if self.runner.as_ref().is_some_and(|runner| runner.id == task) {
                        self.push_output(line);
                    }
                }
                BgEvent::Finished {
                    task,
                    success,
                    code,
                    cancelled,
                } => {
                    self.finish_task(task, success, code, cancelled);
                }
                BgEvent::Serial(bytes) => {
                    self.serial.append(&bytes);
                }
                BgEvent::SerialError(message) => {
                    self.serial.close();
                    self.serial_was_open = false;
                    self.reopen_at = None;
                    self.status_error(message.clone());
                    self.push_log(message, LogLevel::Error);
                }
                BgEvent::Env(info) => self.apply_env(info),
            }
        }
    }

    fn apply_env(&mut self, info: EnvInfo) {
        if info.generation != self.env_generation {
            return;
        }
        self.env_checking = false;
        self.cli_version = info.cli_version;
        let was_installed = self.core_installed;
        self.core_installed = info.core_installed;
        self.set_boards(info.boards);
        if !was_installed && self.core_installed {
            self.push_log("ESP32 开发支持已就绪。", LogLevel::Success);
        }
    }

    fn set_boards(&mut self, boards: Vec<Board>) {
        // Before esp32:esp32 is installed there is no meaningful board list.
        // Keep the picker empty instead of showing hard-coded ESP32 boards.
        if boards.is_empty() {
            self.boards.clear();
            return;
        }
        let mut boards = boards;
        if !boards.iter().any(|board| board.fqbn == self.cfg.board) {
            boards.insert(
                0,
                Board::new(
                    format!("自定义开发板 ({})", self.cfg.board),
                    self.cfg.board.clone(),
                ),
            );
        }
        self.boards = boards;
    }

    fn refresh_env(&mut self) {
        self.env_generation = self.env_generation.wrapping_add(1);
        let generation = self.env_generation;
        let Some(cli) = self.toolchain.cli.clone() else {
            self.env_checking = false;
            self.core_installed = false;
            self.set_boards(Vec::new());
            return;
        };

        self.env_checking = true;
        let tx = self.tx.clone();
        thread::spawn(move || {
            let cli_version = toolchain::version_of(&cli);
            let core_installed = toolchain::core_installed(&cli);
            let boards = if core_installed {
                toolchain::list_esp32_boards(&cli)
            } else {
                Vec::new()
            };
            let _ = tx.send(BgEvent::Env(EnvInfo {
                generation,
                cli_version,
                core_installed,
                boards,
            }));
        });
    }

    fn finish_task(&mut self, task: u64, success: bool, code: Option<i32>, cancelled: bool) {
        if !self.runner.as_ref().is_some_and(|runner| runner.id == task) {
            return;
        }
        let runner = self.runner.take().unwrap();
        let kind = runner.kind;
        let elapsed = runner.elapsed_seconds();
        self.focus_serial_at = None;

        match kind {
            TaskKind::Compile => {
                if success {
                    self.status_success(format!("编译成功（{elapsed}s）"));
                    self.push_log(format!("✓ 编译成功，用时 {elapsed}s"), LogLevel::Success);
                } else if cancelled {
                    self.status_info("编译已停止");
                    self.push_log("编译已停止。", LogLevel::Warn);
                } else {
                    self.status_error("编译失败，请查看编译输出");
                    self.push_log(
                        format!(
                            "✗ 编译失败{}",
                            code.map(|c| format!("（退出码 {c}）")).unwrap_or_default()
                        ),
                        LogLevel::Error,
                    );
                    self.bottom_tab = BottomTab::Output;
                }
            }
            TaskKind::Upload => {
                if self.serial_was_open {
                    self.schedule_serial_reopen();
                }
                self.serial_was_open = false;
                if success {
                    self.status_success(format!("上传成功（{elapsed}s）"));
                    self.push_log(format!("✓ 上传成功，用时 {elapsed}s"), LogLevel::Success);
                    self.focus_serial_at = Some(Instant::now() + Duration::from_millis(900));
                } else if cancelled {
                    self.status_info("上传已停止");
                    self.push_log("上传已停止。", LogLevel::Warn);
                    self.bottom_tab = BottomTab::Output;
                } else {
                    let recent: Vec<&LogLine> = self.logs.iter().rev().take(40).collect();
                    let compile_failed = recent
                        .iter()
                        .any(|line| line.text.contains("编译失败，已停止上传"));
                    let port_error = recent.iter().any(|line| is_upload_port_error(&line.text));
                    if compile_failed {
                        self.status_error("编译失败，请查看编译输出");
                    } else if port_error {
                        self.status_error("上传失败：串口被占用或 USB 设备异常");
                        self.push_log(
                            "提示：请拔插开发板 USB（或按 RST），关闭 Arduino IDE、其他串口监视器等占用串口的程序，然后再点击「上传」重试。",
                            LogLevel::Warn,
                        );
                    } else {
                        self.status_error("上传失败，请查看编译输出");
                    }
                    self.push_log(
                        format!(
                            "✗ 上传失败{}",
                            code.map(|c| format!("（退出码 {c}）")).unwrap_or_default()
                        ),
                        LogLevel::Error,
                    );
                    self.bottom_tab = BottomTab::Output;
                }
            }
            TaskKind::CoreInstall => {
                if success {
                    self.core_installed = true;
                    self.env_checking = false;
                    self.status_success("ESP32 开发支持安装完成");
                    self.push_log("✓ ESP32 开发支持安装完成。", LogLevel::Success);
                    self.refresh_env();
                } else if cancelled {
                    self.status_info("已停止安装");
                } else {
                    self.status_error("ESP32 开发支持安装失败");
                    self.push_log("✗ ESP32 开发支持安装失败，请查看输出。", LogLevel::Error);
                }
                self.bottom_tab = BottomTab::Output;
            }
            TaskKind::CliInstall => {
                if success {
                    self.toolchain = Toolchain::detect(&self.cfg.cli_path);
                    self.cli_version = self.toolchain.version.clone();
                    self.env_checking = false;
                    self.status_success("Arduino CLI 安装完成");
                    self.push_log(
                        format!("✓ Arduino CLI 已就绪：{}", self.toolchain.display_path()),
                        LogLevel::Success,
                    );
                    self.refresh_env();
                } else if cancelled {
                    self.status_info("已停止下载");
                } else {
                    self.status_error("Arduino CLI 安装失败");
                }
                self.bottom_tab = BottomTab::Output;
            }
        }
    }

    // ------------------------------------------------------------- lifecycle

    fn tick(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let dirty = self.code != self.last_saved;

        if dirty && now.duration_since(self.last_autosave) > Duration::from_millis(1200) {
            self.save_sketch(false);
        }
        if self.config_dirty && now.duration_since(self.last_config_save) > Duration::from_secs(1) {
            if let Err(err) = self.cfg.save() {
                self.push_log(format!("保存设置失败: {err}"), LogLevel::Warn);
            }
            self.config_dirty = false;
            self.last_config_save = now;
        }

        if let Some(at) = self.reopen_at {
            if now >= at {
                self.reopen_at = None;
                if !self.serial.is_open() {
                    if self.open_serial() {
                        self.reopen_attempts = 0;
                        self.status_success("串口已重新连接");
                    } else {
                        self.reopen_attempts = self.reopen_attempts.saturating_add(1);
                        if self.reopen_attempts < 5 {
                            self.reopen_at = Some(now + Duration::from_millis(900));
                            self.status_info("正在等待串口重新出现…");
                        } else {
                            self.status_error("串口未能自动重新打开，请拔插开发板或点击 ↻");
                            self.push_log(
                                "提示：LOLIN S2 Mini 等原生 USB 板在上传后可能需要拔插一次 USB，串口才会重新出现。",
                                LogLevel::Warn,
                            );
                            self.reopen_attempts = 0;
                        }
                    }
                }
            }
        }

        if let Some(at) = self.focus_serial_at {
            if now >= at {
                self.bottom_tab = BottomTab::Serial;
                self.focus_serial_at = None;
            }
        }

        if !self.serial.is_open()
            && self.reopen_at.is_none()
            && self.runner.is_none()
            && now.duration_since(self.last_port_refresh) > Duration::from_secs(5)
        {
            self.refresh_ports();
        }

        if self.code != self.last_saved
            || self.runner.is_some()
            || self.serial.is_open()
            || self.reopen_at.is_some()
            || self.env_checking
        {
            ctx.request_repaint_after(Duration::from_millis(50));
        } else {
            ctx.request_repaint_after(Duration::from_millis(800));
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context, action: &mut Action) {
        let ctrl = if cfg!(target_os = "macos") {
            egui::Modifiers::COMMAND
        } else {
            egui::Modifiers::CTRL
        };
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F7)) {
            *action = Action::Compile;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F8)) {
            *action = Action::Upload;
        }
        let mut save_as = ctrl;
        save_as.shift = true;
        if ctx.input_mut(|input| input.consume_key(save_as, egui::Key::S)) {
            *action = Action::SaveAs;
        }
        if ctx.input_mut(|input| input.consume_key(ctrl, egui::Key::S)) {
            *action = Action::Save;
        }
        if ctx.input_mut(|input| input.consume_key(ctrl, egui::Key::O)) {
            *action = Action::Open;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.show_settings = false;
        }
    }

    // --------------------------------------------------------------- actions

    fn handle_action(&mut self, action: Action, ctx: &egui::Context) {
        match action {
            Action::None => {}
            Action::Compile => self.start_compile(false),
            Action::Upload => self.start_compile(true),
            Action::ToggleSerial => self.toggle_serial(),
            Action::RefreshPorts => {
                self.refresh_ports();
                if self.ports.is_empty() {
                    self.status_error("未发现串口，请检查 USB 连接");
                } else {
                    self.status_info(format!("发现 {} 个串口", self.ports.len()));
                }
            }
            Action::RefreshBoards => {
                if self.toolchain.cli.is_none() {
                    self.status_error("请先安装 Arduino CLI");
                } else if !self.core_installed {
                    self.status_info("安装 ESP32 开发支持后即可选择开发板");
                } else {
                    self.status_info("正在刷新开发板列表…");
                    self.refresh_env();
                }
            }
            Action::Settings => {
                self.settings_cli = self.cfg.cli_path.clone();
                self.settings_error = None;
                self.show_settings = true;
            }
            Action::Save => {
                self.save_sketch(true);
            }
            Action::SaveAs => self.save_sketch_as(),
            Action::Open => self.open_sketch(),
            Action::InstallCore => self.start_core_install(),
            Action::InstallCli => self.start_cli_install(),
            Action::StopTask => {
                if let Some(runner) = &self.runner {
                    runner.stop();
                    self.status_busy(format!("正在停止{}…", runner.kind.label()));
                }
            }
            Action::ClearSerial => {
                self.serial.clear();
                self.status_info("串口缓冲区已清空");
            }
            Action::ClearOutput => {
                self.logs.clear();
                self.log_has_error = false;
            }
            Action::CopyOutput => {
                let text = self
                    .logs
                    .iter()
                    .map(|line| line.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                ctx.output_mut(|output| output.copied_text = text);
                self.status_success("编译输出已复制");
            }
            Action::SendSerial => self.send_serial(),
        }
    }

    fn start_compile(&mut self, upload: bool) {
        if self.runner.is_some() {
            return;
        }
        let Some(cli) = self.toolchain.cli.clone() else {
            self.need_cli();
            return;
        };
        if !self.core_installed {
            self.need_core();
            return;
        }
        if upload && self.cfg.port.trim().is_empty() {
            self.status_error("请先选择串口");
            self.bottom_tab = BottomTab::Serial;
            return;
        }
        if !self.save_sketch(false) {
            return;
        }

        self.serial_was_open = false;
        let upload_port = if upload {
            if self.serial.is_open() {
                self.serial.close();
                self.serial_was_open = true;
                self.push_log("已关闭串口以便上传。", LogLevel::Info);
            }
            Some(self.cfg.port.clone())
        } else {
            None
        };

        let kind = if upload {
            TaskKind::Upload
        } else {
            TaskKind::Compile
        };
        let id = self.begin_task(kind);
        self.bottom_tab = BottomTab::Output;
        self.status_busy(format!("正在{}…", kind.label()));
        self.push_log(
            format!("▶ 开始{} · {}", kind.label(), self.cfg.sketch_name()),
            LogLevel::Command,
        );

        let tx = self.tx.clone();
        let sketch_dir = self.cfg.sketch_dir();
        let fqbn = self.cfg.board.clone();
        let build_dir = Config::build_dir();
        let cancel = self
            .runner
            .as_ref()
            .map(|runner| runner.cancel.clone())
            .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
        thread::spawn(move || {
            let finish = |success: bool, code: Option<i32>, cancelled: bool| {
                let _ = tx.send(BgEvent::Finished {
                    task: id,
                    success,
                    code,
                    cancelled,
                });
            };

            let compile_result =
                arduino::compile(&cli, &sketch_dir, &fqbn, &build_dir, &tx, id, &cancel);
            let mut outcome = match compile_result {
                Ok(outcome) => outcome,
                Err(message) => {
                    let _ = tx.send(BgEvent::Output {
                        task: id,
                        line: message,
                    });
                    finish(false, None, false);
                    return;
                }
            };

            if !outcome.success || outcome.cancelled {
                if !outcome.cancelled {
                    let _ = tx.send(BgEvent::Output {
                        task: id,
                        line: "编译失败，已停止上传。".to_owned(),
                    });
                }
                finish(outcome.success, outcome.code, outcome.cancelled);
                return;
            }

            if let Some(port) = upload_port.as_deref() {
                let _ = tx.send(BgEvent::Output {
                    task: id,
                    line: format!("编译完成，准备上传到 {port}…"),
                });
                // Let the OS/driver finish releasing the serial handle and let
                // a native-USB board settle before esptool opens it.
                thread::sleep(arduino::UPLOAD_PORT_RELEASE_DELAY);

                let mut attempt = 1;
                loop {
                    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                        outcome = arduino::RunOutcome {
                            success: false,
                            code: None,
                            cancelled: true,
                        };
                        break;
                    }

                    let _ = tx.send(BgEvent::Output {
                        task: id,
                        line: format!(
                            "上传第 {attempt}/{} 次 · {port} · {}",
                            arduino::UPLOAD_ATTEMPTS,
                            build_dir.display()
                        ),
                    });
                    outcome = match arduino::upload(&cli, &fqbn, port, &build_dir, &tx, id, &cancel)
                    {
                        Ok(outcome) => outcome,
                        Err(message) => {
                            let _ = tx.send(BgEvent::Output {
                                task: id,
                                line: message,
                            });
                            arduino::RunOutcome {
                                success: false,
                                code: None,
                                cancelled: false,
                            }
                        }
                    };

                    if outcome.success || outcome.cancelled || attempt >= arduino::UPLOAD_ATTEMPTS {
                        break;
                    }

                    let _ = tx.send(BgEvent::Output {
                        task: id,
                        line: "上传失败，等待串口恢复后重试…".to_owned(),
                    });
                    thread::sleep(arduino::UPLOAD_RETRY_DELAY);
                    attempt += 1;
                }
            }

            finish(outcome.success, outcome.code, outcome.cancelled);
        });
    }

    fn start_core_install(&mut self) {
        if self.runner.is_some() {
            return;
        }
        let Some(cli) = self.toolchain.cli.clone() else {
            self.need_cli();
            return;
        };
        let id = self.begin_task(TaskKind::CoreInstall);
        self.bottom_tab = BottomTab::Output;
        self.status_busy("正在安装 ESP32 开发支持…");
        self.push_log(
            if self.cfg.use_china_mirror {
                "▶ 安装 esp32:esp32 开发支持（Espressif 中国镜像 Jihulab）"
            } else {
                "▶ 安装 esp32:esp32 开发支持（官方源）"
            },
            LogLevel::Command,
        );
        let use_china_mirror = self.cfg.use_china_mirror;
        let tx = self.tx.clone();
        let cancel = self
            .runner
            .as_ref()
            .map(|runner| runner.cancel.clone())
            .unwrap();
        thread::spawn(move || {
            if let Err(message) =
                toolchain::install_esp32_core(&cli, &tx, id, &cancel, use_china_mirror)
            {
                let _ = tx.send(BgEvent::Output {
                    task: id,
                    line: message,
                });
                let _ = tx.send(BgEvent::Finished {
                    task: id,
                    success: false,
                    code: None,
                    cancelled: cancel.load(std::sync::atomic::Ordering::Relaxed),
                });
            } else {
                let _ = tx.send(BgEvent::Finished {
                    task: id,
                    success: true,
                    code: Some(0),
                    cancelled: false,
                });
            }
        });
    }

    fn start_cli_install(&mut self) {
        if self.runner.is_some() {
            return;
        }
        let id = self.begin_task(TaskKind::CliInstall);
        self.bottom_tab = BottomTab::Output;
        self.status_busy("正在下载 Arduino CLI…");
        self.push_log("▶ 下载并安装 Arduino CLI", LogLevel::Command);
        let tx = self.tx.clone();
        let target = toolchain::local_cli_path();
        let cancel = self
            .runner
            .as_ref()
            .map(|runner| runner.cancel.clone())
            .unwrap();
        thread::spawn(move || {
            if let Err(message) = toolchain::install_cli(&target, &tx, id, &cancel) {
                let _ = tx.send(BgEvent::Output {
                    task: id,
                    line: message,
                });
                let _ = tx.send(BgEvent::Finished {
                    task: id,
                    success: false,
                    code: None,
                    cancelled: cancel.load(std::sync::atomic::Ordering::Relaxed),
                });
            } else {
                let _ = tx.send(BgEvent::Finished {
                    task: id,
                    success: true,
                    code: Some(0),
                    cancelled: false,
                });
            }
        });
    }

    fn begin_task(&mut self, kind: TaskKind) -> u64 {
        let id = self.next_task_id;
        self.next_task_id = self.next_task_id.wrapping_add(1);
        self.runner = Some(Runner::new(id, kind, Arc::new(AtomicBool::new(false))));
        id
    }

    fn need_cli(&mut self) {
        self.status_error("未检测到 Arduino CLI");
        self.bottom_tab = BottomTab::Output;
        self.push_log(
            "未检测到 Arduino CLI。请点击顶部提示中的「下载并安装」，或在设置里指定 arduino-cli 路径。",
            LogLevel::Error,
        );
    }

    fn need_core(&mut self) {
        self.status_error("尚未安装 ESP32 开发支持");
        self.bottom_tab = BottomTab::Output;
        self.push_log(
            "编译前请先点击顶部提示中的「安装 ESP32 支持」。",
            LogLevel::Error,
        );
    }

    // ---------------------------------------------------------------- serial

    fn refresh_ports(&mut self) {
        self.last_port_refresh = Instant::now();
        if self.serial.is_open() {
            return;
        }
        match serialport::available_ports() {
            Ok(list) => {
                let mut names: Vec<String> = list.into_iter().map(|port| port.port_name).collect();
                names.sort_by(|a, b| natural_port_cmp(a, b));
                self.ports = names;

                if !self.ports.contains(&self.cfg.port) {
                    let fallback = self.ports.first().cloned().unwrap_or_default();
                    if fallback != self.cfg.port {
                        self.cfg.port = fallback;
                        self.config_dirty = true;
                    }
                }
            }
            Err(err) => {
                self.ports.clear();
                self.push_log(format!("枚举串口失败: {err}"), LogLevel::Warn);
            }
        }
    }

    fn toggle_serial(&mut self) {
        if self.serial.is_open() {
            self.serial.close();
            self.serial.push_line("[Wild IDE] 串口已关闭");
            self.serial_was_open = false;
            self.reopen_at = None;
            self.status_info("串口已关闭");
        } else {
            self.open_serial();
        }
    }

    fn open_serial(&mut self) -> bool {
        if self.cfg.port.trim().is_empty() {
            self.status_error("请先选择串口");
            return false;
        }
        self.reopen_at = None;
        self.reopen_attempts = 0;
        let port = self.cfg.port.clone();
        let baud = self.cfg.baud;
        match self.serial.open(&port, baud, self.tx.clone()) {
            Ok(()) => {
                self.serial
                    .push_line(&format!("[Wild IDE] 串口已连接：{port} @ {baud}"));
                self.bottom_tab = BottomTab::Serial;
                self.status_success(format!("串口 {port} 已连接"));
                true
            }
            Err(message) => {
                self.status_error(message.clone());
                self.push_log(message, LogLevel::Error);
                false
            }
        }
    }

    fn schedule_serial_reopen(&mut self) {
        self.reopen_attempts = 0;
        self.reopen_at = Some(Instant::now() + toolchain::wait_after_upload());
        self.status_info("等待串口重新连接…");
    }

    fn send_serial(&mut self) {
        if !self.serial.is_open() {
            return;
        }
        let text = std::mem::take(&mut self.serial_input);
        match self.serial.send_line(&text) {
            Ok(()) => {
                self.status_success("已发送");
            }
            Err(message) => {
                self.status_error(message.clone());
                self.push_log(message, LogLevel::Error);
            }
        }
    }

    fn reconnect_serial_if_open(&mut self) {
        if self.serial.is_open() {
            self.open_serial();
        }
    }

    // ----------------------------------------------------------------- files

    fn save_sketch(&mut self, announce: bool) -> bool {
        self.save_sketch_to(self.cfg.sketch_path(), announce)
    }

    fn save_sketch_to(&mut self, path: PathBuf, announce: bool) -> bool {
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                self.status_error(format!("创建目录失败: {err}"));
                return false;
            }
        }
        match fs::write(&path, &self.code) {
            Ok(()) => {
                if self.cfg.sketch_path() != path {
                    self.cfg.sketch = path.to_string_lossy().to_string();
                    self.config_dirty = true;
                }
                self.last_saved = self.code.clone();
                self.last_autosave = Instant::now();
                if announce {
                    self.status_success(format!("已保存 {}", self.cfg.sketch_name()));
                    self.push_log(format!("保存草稿：{}", path.display()), LogLevel::Info);
                }
                true
            }
            Err(err) => {
                self.status_error(format!("保存失败: {err}"));
                self.push_log(
                    format!("保存 {} 失败: {err}", path.display()),
                    LogLevel::Error,
                );
                false
            }
        }
    }

    fn save_sketch_as(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Arduino Sketch", &["ino"])
            .set_file_name(self.cfg.sketch_name());
        let sketch_dir = self.cfg.sketch_dir();
        let default_dir = if sketch_folder_mismatch(&self.cfg.sketch_path()) {
            sketch_dir.clone()
        } else {
            sketch_dir
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or(sketch_dir)
        };
        dialog = dialog.set_directory(default_dir);
        let Some(path) = dialog.save_file() else {
            return;
        };
        let Some(path) = normalize_ino_path(path) else {
            self.status_error("请使用 .ino 扩展名");
            return;
        };
        let path = save_as_target(path);
        if self.save_sketch_to(path.clone(), false) {
            self.status_success(format!("已另存为 {}", self.cfg.sketch_name()));
            self.push_log(format!("另存为：{}", path.display()), LogLevel::Info);
            self.warn_sketch_folder_mismatch(&path);
        }
    }

    fn open_sketch(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Arduino Sketch", &["ino"])
            .pick_file()
        else {
            return;
        };
        if !is_ino_path(&path) {
            self.status_error("请选择 .ino 文件");
            return;
        }
        if !self.save_sketch(false) {
            return;
        }
        match fs::read_to_string(&path) {
            Ok(text) => {
                self.cfg.sketch = path.to_string_lossy().to_string();
                self.code = text;
                self.last_saved = self.code.clone();
                self.last_autosave = Instant::now();
                self.config_dirty = true;
                self.status_success(format!("已打开 {}", self.cfg.sketch_name()));
                self.push_log(format!("打开草稿：{}", path.display()), LogLevel::Info);
                self.warn_sketch_folder_mismatch(&path);
            }
            Err(err) => {
                self.status_error(format!("无法打开文件: {err}"));
            }
        }
    }

    fn warn_sketch_folder_mismatch(&mut self, path: &Path) {
        if sketch_folder_mismatch(path) {
            self.push_log(
                "提示：Arduino 要求 .ino 文件名与所在文件夹同名。当前路径可能导致编译失败，建议把草稿保存到同名文件夹中。",
                LogLevel::Warn,
            );
        }
    }

    // ------------------------------------------------------------------ logs

    fn push_log(&mut self, text: impl Into<String>, level: LogLevel) {
        if level == LogLevel::Error {
            self.log_has_error = true;
        }
        self.logs.push(LogLine {
            text: text.into(),
            level,
        });
        if self.logs.len() > MAX_LOG_LINES {
            self.logs.drain(..self.logs.len() - MAX_LOG_LINES);
        }
    }

    fn push_output(&mut self, line: String) {
        let level = classify_log(&line);
        self.push_log(line, level);
    }

    fn status_info(&mut self, text: impl Into<String>) {
        self.status = text.into();
        self.status_kind = StatusKind::Info;
    }

    fn status_success(&mut self, text: impl Into<String>) {
        self.status = text.into();
        self.status_kind = StatusKind::Success;
    }

    fn status_error(&mut self, text: impl Into<String>) {
        self.status = text.into();
        self.status_kind = StatusKind::Error;
    }

    fn status_busy(&mut self, text: impl Into<String>) {
        self.status = text.into();
        self.status_kind = StatusKind::Busy;
    }

    // -------------------------------------------------------------------- ui

    fn toolbar_ui(&mut self, ui: &mut Ui, action: &mut Action) {
        let p = palette(self.cfg.theme);
        ui.horizontal(|ui| {
            ui.label(RichText::new(APP_NAME_ZH).size(22.0).strong().color(ACCENT));
            ui.label(RichText::new(APP_NAME).size(11.0).color(p.weak_text));
            ui.add_space(4.0);

            // Board picker: only populate it after esp32:esp32 is installed.
            let boards_ready = self.core_installed && !self.boards.is_empty();
            let board_text = if self.env_checking {
                if self.core_installed {
                    "正在加载开发板…"
                } else {
                    "正在检测 ESP32 支持…"
                }
            } else if !self.core_installed {
                "安装 ESP32 支持后选择"
            } else if boards_ready {
                boards::board_name(&self.boards, &self.cfg.board)
            } else {
                "未找到开发板"
            }
            .to_owned();
            let mut selected_board = None;
            ui.add_enabled_ui(boards_ready, |ui| {
                egui::ComboBox::from_id_salt("board_combo")
                    .width(205.0)
                    .selected_text(RichText::new(board_text).size(13.0))
                    .show_ui(ui, |ui| {
                        ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
                            for board in &self.boards {
                                let selected = board.fqbn == self.cfg.board;
                                if ui
                                    .selectable_label(selected, &board.name)
                                    .on_hover_text(&board.fqbn)
                                    .clicked()
                                {
                                    selected_board = Some(board.fqbn.clone());
                                    ui.close_menu();
                                }
                            }
                        });
                    })
                    .response
                    .on_hover_text(if boards_ready {
                        "开发板"
                    } else {
                        "安装 ESP32 开发支持后可用"
                    });
            });
            if ui
                .add_enabled(
                    boards_ready,
                    Button::new(RichText::new("↻")).min_size(egui::vec2(26.0, 26.0)),
                )
                .on_hover_text(if boards_ready {
                    "刷新开发板列表"
                } else {
                    "安装 ESP32 开发支持后可用"
                })
                .clicked()
            {
                *action = Action::RefreshBoards;
            }
            if let Some(fqbn) = selected_board {
                self.cfg.board = fqbn;
                self.config_dirty = true;
            }

            // Port picker
            let port_text = if self.cfg.port.is_empty() {
                "选择串口".to_owned()
            } else {
                self.cfg.port.clone()
            };
            let mut selected_port = None;
            egui::ComboBox::from_id_salt("port_combo")
                .width(100.0)
                .selected_text(RichText::new(port_text).size(13.0))
                .show_ui(ui, |ui| {
                    if self.ports.is_empty() {
                        ui.weak("未发现串口");
                    }
                    for port in &self.ports {
                        if ui.selectable_label(*port == self.cfg.port, port).clicked() {
                            selected_port = Some(port.clone());
                            ui.close_menu();
                        }
                    }
                })
                .response
                .on_hover_text("串口");
            if ui
                .add(Button::new(RichText::new("↻")).min_size(egui::vec2(26.0, 26.0)))
                .on_hover_text("刷新串口")
                .clicked()
            {
                *action = Action::RefreshPorts;
            }
            if let Some(port) = selected_port {
                if port != self.cfg.port {
                    self.cfg.port = port;
                    self.config_dirty = true;
                    self.reconnect_serial_if_open();
                }
            }

            ui.add_space(8.0);
            let busy = self.runner.is_some();
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        Button::new(RichText::new("⚙").size(16.0)).min_size(egui::vec2(32.0, 28.0)),
                    )
                    .on_hover_text("设置")
                    .clicked()
                {
                    *action = Action::Settings;
                }

                if let Some(runner) = &self.runner {
                    if ui
                        .add(
                            Button::new(RichText::new("停止").strong())
                                .fill(ERROR)
                                .stroke(Stroke::NONE)
                                .min_size(egui::vec2(60.0, 28.0)),
                        )
                        .clicked()
                    {
                        *action = Action::StopTask;
                    }
                    ui.add(egui::Spinner::new().size(15.0));
                    ui.label(
                        RichText::new(format!(
                            "{} {}s",
                            runner.kind.label(),
                            runner.elapsed_seconds()
                        ))
                        .size(12.5)
                        .color(p.weak_text),
                    );
                } else {
                    if ui
                        .add_enabled(
                            !busy,
                            Button::new(
                                RichText::new("上传")
                                    .strong()
                                    .color(Color32::from_rgb(10, 22, 12)),
                            )
                            .fill(ACCENT)
                            .stroke(Stroke::NONE)
                            .min_size(egui::vec2(62.0, 28.0)),
                        )
                        .on_hover_text("编译并上传 (F8)")
                        .clicked()
                    {
                        *action = Action::Upload;
                    }
                    if ui
                        .add_enabled(
                            !busy,
                            Button::new(RichText::new("编译").strong())
                                .min_size(egui::vec2(62.0, 28.0)),
                        )
                        .on_hover_text("编译 (F7)")
                        .clicked()
                    {
                        *action = Action::Compile;
                    }
                }

                let serial_open = self.serial.is_open();
                let serial_text = if serial_open {
                    format!("● {}", self.cfg.port)
                } else {
                    "打开串口".to_owned()
                };
                if ui
                    .add_enabled(
                        !busy || serial_open,
                        Button::new(RichText::new(serial_text).size(13.0).color(if serial_open {
                            OK
                        } else {
                            p.text
                        }))
                        .fill(if serial_open { p.faint_bg } else { p.bar_bg })
                        .min_size(egui::vec2(88.0, 28.0)),
                    )
                    .on_hover_text(if serial_open {
                        "关闭串口"
                    } else {
                        "打开串口并查看输出"
                    })
                    .clicked()
                {
                    *action = Action::ToggleSerial;
                }
            });
        });
    }

    fn banner_visible(&self) -> bool {
        let installing = self
            .runner
            .as_ref()
            .map(|runner| runner.kind)
            .is_some_and(|kind| matches!(kind, TaskKind::CoreInstall | TaskKind::CliInstall));
        installing || self.toolchain.cli.is_none() || !self.core_installed
    }

    fn banner_ui(&mut self, ui: &mut Ui, action: &mut Action) {
        let p = palette(self.cfg.theme);
        let installing = self
            .runner
            .as_ref()
            .map(|runner| runner.kind)
            .filter(|kind| matches!(kind, TaskKind::CoreInstall | TaskKind::CliInstall));
        let bg = if p.is_dark {
            Color32::from_rgb(0x2b, 0x24, 0x17)
        } else {
            Color32::from_rgb(0xff, 0xf7, 0xe3)
        };

        if !self.banner_visible() {
            return;
        }

        Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x5b, 0x47, 0x23)))
            .inner_margin(Margin::symmetric(14.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if let Some(kind) = installing {
                        ui.add(egui::Spinner::new().size(14.0));
                        let text = match kind {
                            TaskKind::CliInstall => "正在下载并安装 Arduino CLI…",
                            TaskKind::CoreInstall => {
                                "正在安装 ESP32 开发支持，首次安装需要下载较多文件…"
                            }
                            _ => "",
                        };
                        ui.label(RichText::new(text).color(WARN));
                        return;
                    }

                    if self.toolchain.cli.is_none() {
                        ui.label(RichText::new("⚡").color(WARN));
                        ui.label(
                            RichText::new("首次使用需要 Arduino CLI（用于编译和上传）")
                                .color(p.text),
                        );
                        if ui
                            .add(
                                Button::new(
                                    RichText::new("下载并安装")
                                        .strong()
                                        .color(Color32::from_rgb(10, 22, 12)),
                                )
                                .fill(ACCENT)
                                .stroke(Stroke::NONE),
                            )
                            .clicked()
                        {
                            *action = Action::InstallCli;
                        }
                        if ui.button("选择本机 CLI…").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.cfg.cli_path = path.to_string_lossy().to_string();
                                self.settings_cli = self.cfg.cli_path.clone();
                                self.apply_cli_path();
                            }
                        }
                        return;
                    }

                    if self.env_checking {
                        ui.add(egui::Spinner::new().size(14.0));
                        ui.label(RichText::new("正在检测 ESP32 开发支持…").color(p.text));
                        return;
                    }

                    ui.label(RichText::new("⚡").color(WARN));
                    ui.label(RichText::new("尚未安装 ESP32 开发支持（esp32:esp32）").color(p.text));
                    if ui
                        .add(
                            Button::new(
                                RichText::new("安装 ESP32 支持")
                                    .strong()
                                    .color(Color32::from_rgb(10, 22, 12)),
                            )
                            .fill(ACCENT)
                            .stroke(Stroke::NONE),
                        )
                        .on_hover_text(if self.cfg.use_china_mirror {
                            "通过官方 Jihulab 中国镜像安装 esp32:esp32 的 -cn 版本"
                        } else {
                            "运行 arduino-cli core install esp32:esp32"
                        })
                        .clicked()
                    {
                        *action = Action::InstallCore;
                    }
                    if ui.button("重新检测").clicked() {
                        *action = Action::RefreshBoards;
                    }
                });
            });
    }

    fn console_ui(&mut self, ui: &mut Ui, action: &mut Action) {
        let p = palette(self.cfg.theme);
        ui.horizontal(|ui| {
            let output_label = if self.log_has_error {
                "编译输出 ·"
            } else {
                "编译输出"
            };
            if tab_button(ui, output_label, self.bottom_tab == BottomTab::Output, &p) {
                self.bottom_tab = BottomTab::Output;
            }
            if tab_button(ui, "串口监视器", self.bottom_tab == BottomTab::Serial, &p) {
                self.bottom_tab = BottomTab::Serial;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                match self.bottom_tab {
                    BottomTab::Serial => {
                        if ui
                            .add(Button::new("清空").min_size(egui::vec2(48.0, 24.0)))
                            .clicked()
                        {
                            *action = Action::ClearSerial;
                        }
                    }
                    BottomTab::Output => {
                        if ui
                            .add(Button::new("清空").min_size(egui::vec2(48.0, 24.0)))
                            .clicked()
                        {
                            *action = Action::ClearOutput;
                        }
                        if ui
                            .add(Button::new("复制").min_size(egui::vec2(48.0, 24.0)))
                            .clicked()
                        {
                            *action = Action::CopyOutput;
                        }
                    }
                }
            });
        });

        ui.add_space(4.0);
        match self.bottom_tab {
            BottomTab::Serial => self.serial_view(ui, action),
            BottomTab::Output => self.output_view(ui, &p),
        }

        // Resizable panels store the content rectangle as their next height.
        // If the content does not fill the panel, egui shrinks the panel a
        // little every frame, which makes the splitter snap back. Fill the
        // remaining space so the dragged height stays put.
        let full_rect = ui.max_rect();
        ui.expand_to_include_rect(full_rect);
    }

    fn serial_view(&mut self, ui: &mut Ui, action: &mut Action) {
        let p = palette(self.cfg.theme);
        let serial_open = self.serial.is_open();

        ui.horizontal(|ui| {
            ui.label(RichText::new("波特率").size(12.0).color(p.weak_text));
            let mut baud = self.cfg.baud;
            egui::ComboBox::from_id_salt("baud_combo")
                .width(88.0)
                .selected_text(baud.to_string())
                .show_ui(ui, |ui| {
                    for rate in BAUD_RATES {
                        ui.selectable_value(&mut baud, *rate, rate.to_string());
                    }
                });
            if baud != self.cfg.baud {
                self.cfg.baud = baud;
                self.config_dirty = true;
                self.reconnect_serial_if_open();
            }

            ui.checkbox(&mut self.cfg.auto_scroll, "自动滚动");
            if serial_open {
                ui.label(
                    RichText::new(format!(
                        "● {} · {} 字节",
                        self.serial.port_name.as_deref().unwrap_or(""),
                        self.serial.total_bytes
                    ))
                    .size(11.5)
                    .color(OK),
                );
            }
        });
        ui.add_space(4.0);

        let height = (ui.available_height() - 38.0).max(60.0);
        ScrollArea::vertical()
            .stick_to_bottom(self.cfg.auto_scroll)
            .auto_shrink([false, false])
            .max_height(height)
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Wrap);
                if self.serial.buffer.is_empty() {
                    ui.add_space(12.0);
                    let message = if serial_open {
                        "等待串口数据…"
                    } else if self.cfg.port.is_empty() {
                        "未发现串口，请连接开发板后刷新"
                    } else {
                        "串口未打开"
                    };
                    ui.label(RichText::new(message).color(p.weak_text));
                } else {
                    ui.add(
                        Label::new(
                            RichText::new(self.serial.buffer.as_str())
                                .font(egui::FontId::monospace(12.5))
                                .color(p.text),
                        )
                        .selectable(true),
                    );
                }
            });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let send_width = (ui.available_width() - 96.0).max(120.0);
            let response = ui.add_enabled(
                serial_open,
                TextEdit::singleline(&mut self.serial_input)
                    .hint_text("输入内容，回车发送（CRLF）")
                    .desired_width(send_width),
            );
            if response.lost_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter))
                && serial_open
            {
                *action = Action::SendSerial;
            }
            if ui
                .add_enabled(
                    serial_open,
                    Button::new("发送").min_size(egui::vec2(54.0, 26.0)),
                )
                .clicked()
            {
                *action = Action::SendSerial;
            }
            ui.label(RichText::new("CRLF").size(10.5).color(p.weak_text));
        });
    }

    fn output_view(&mut self, ui: &mut Ui, p: &theme::Palette) {
        let height = ui.available_height().max(60.0);
        ScrollArea::vertical()
            .stick_to_bottom(self.cfg.auto_scroll)
            .auto_shrink([false, false])
            .max_height(height)
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Wrap);
                if self.logs.is_empty() {
                    ui.add_space(12.0);
                    ui.label(RichText::new("编译与上传的输出会显示在这里").color(p.weak_text));
                    return;
                }
                for line in &self.logs {
                    let color = match line.level {
                        LogLevel::Error => ERROR,
                        LogLevel::Warn => WARN,
                        LogLevel::Success => OK,
                        LogLevel::Command => ACCENT,
                        LogLevel::Info => p.text,
                    };
                    ui.add(
                        Label::new(
                            RichText::new(&line.text)
                                .font(egui::FontId::monospace(12.5))
                                .color(color),
                        )
                        .selectable(true),
                    );
                }
                if let Some(runner) = &self.runner {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().size(12.0));
                        ui.label(
                            RichText::new(format!(
                                "{}中… {}s",
                                runner.kind.label(),
                                runner.elapsed_seconds()
                            ))
                            .size(11.5)
                            .color(p.weak_text),
                        );
                    });
                }
            });
    }

    fn status_bar_ui(&mut self, ui: &mut Ui) {
        let p = palette(self.cfg.theme);
        let color = match self.status_kind {
            StatusKind::Info => p.weak_text,
            StatusKind::Success => OK,
            StatusKind::Error => ERROR,
            StatusKind::Busy => ACCENT,
        };
        ui.horizontal(|ui| {
            if self.status_kind == StatusKind::Busy {
                ui.add(egui::Spinner::new().size(11.0));
            } else {
                ui.label(RichText::new("●").size(10.0).color(color));
            }
            ui.label(RichText::new(&self.status).size(11.5).color(color));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let core = if self.core_installed {
                    "ESP32 就绪"
                } else {
                    "ESP32 未安装"
                };
                let cli = match &self.cli_version {
                    Some(version) => format!("Arduino CLI {version}"),
                    None => "Arduino CLI 未安装".to_owned(),
                };
                ui.label(
                    RichText::new(format!("{cli}   ·   {core}   ·   v{APP_VERSION}"))
                        .size(11.0)
                        .color(p.weak_text),
                );
            });
        });
    }

    fn editor_ui(&mut self, ui: &mut Ui, action: &mut Action) {
        let p = palette(self.cfg.theme);
        let dirty = self.code != self.last_saved;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(self.cfg.sketch_name())
                    .size(13.5)
                    .strong()
                    .color(p.text),
            );
            if dirty {
                ui.label(RichText::new("● 修改中").size(10.5).color(WARN));
            } else {
                ui.label(RichText::new("已自动保存").size(10.5).color(p.weak_text));
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(util::short_path(&self.cfg.sketch_dir(), 52))
                    .size(10.5)
                    .color(p.weak_text),
            )
            .on_hover_text(self.cfg.sketch.clone());

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(
                        self.runner.is_none(),
                        Button::new(RichText::new("保存").size(12.0))
                            .min_size(egui::vec2(48.0, 24.0)),
                    )
                    .on_hover_text("保存到当前 .ino 文件 (Ctrl+S)")
                    .clicked()
                {
                    *action = Action::Save;
                }
                if ui
                    .add_enabled(
                        self.runner.is_none(),
                        Button::new(RichText::new("另存为").size(12.0))
                            .min_size(egui::vec2(58.0, 24.0)),
                    )
                    .on_hover_text("另存为新的 .ino 文件 (Ctrl+Shift+S)")
                    .clicked()
                {
                    *action = Action::SaveAs;
                }
                if ui
                    .add_enabled(
                        self.runner.is_none(),
                        Button::new(RichText::new("打开文件").size(12.0))
                            .min_size(egui::vec2(64.0, 24.0)),
                    )
                    .on_hover_text("打开 .ino 文件 (Ctrl+O)")
                    .clicked()
                {
                    *action = Action::Open;
                }
                ui.label(
                    RichText::new(format!("{} 行", self.code.matches('\n').count() + 1))
                        .size(10.5)
                        .color(p.weak_text),
                );
            });
        });
        ui.add_space(6.0);
        editor::show(ui, &mut self.code, self.cfg.font_size, self.cfg.theme);
    }

    fn settings_ui(&mut self, ui: &mut Ui) {
        let p = palette(self.cfg.theme);

        ui.label(RichText::new("外观").strong().color(p.text));
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label("主题");
            let mut theme = self.cfg.theme;
            egui::ComboBox::from_id_salt("theme_combo")
                .width(100.0)
                .selected_text(theme.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut theme, ThemeChoice::Dark, ThemeChoice::Dark.label());
                    ui.selectable_value(&mut theme, ThemeChoice::Light, ThemeChoice::Light.label());
                });
            if theme != self.cfg.theme {
                self.cfg.theme = theme;
                self.config_dirty = true;
                theme::apply(ui.ctx(), self.cfg.theme, self.cfg.font_size);
            }
        });
        ui.horizontal(|ui| {
            ui.label("编辑器字号");
            let mut size = self.cfg.font_size;
            if ui
                .add(
                    egui::Slider::new(&mut size, 11.0..=24.0)
                        .step_by(0.5)
                        .suffix(" px"),
                )
                .changed()
            {
                self.cfg.font_size = size;
                self.config_dirty = true;
                theme::apply(ui.ctx(), self.cfg.theme, self.cfg.font_size);
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.label(RichText::new("Arduino CLI").strong().color(p.text));
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut self.settings_cli)
                    .hint_text(if self.toolchain.cli.is_some() {
                        self.toolchain.display_path()
                    } else {
                        "留空则自动检测 PATH".to_owned()
                    })
                    .desired_width(310.0),
            );
            if ui.button("浏览…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.settings_cli = path.to_string_lossy().to_string();
                    self.apply_cli_path();
                }
            }
            if ui.button("应用").clicked() {
                self.apply_cli_path();
            }
        });
        let cli_status = match (&self.toolchain.cli, &self.cli_version) {
            (Some(_), Some(version)) => format!("已找到 Arduino CLI {version}"),
            (Some(_), None) => "已找到 Arduino CLI".to_owned(),
            (None, _) => "未找到 Arduino CLI".to_owned(),
        };
        ui.label(
            RichText::new(cli_status)
                .size(11.5)
                .color(if self.toolchain.cli.is_some() {
                    OK
                } else {
                    ERROR
                }),
        );
        if let Some(error) = &self.settings_error {
            ui.label(RichText::new(error).size(11.5).color(ERROR));
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.runner.is_none(),
                    Button::new(RichText::new("下载并安装 Arduino CLI").size(12.0)),
                )
                .clicked()
            {
                self.start_cli_install();
            }
            if ui.button("重新检测").clicked() {
                self.cli_version = self.toolchain.version.clone();
                self.refresh_env();
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.label(RichText::new("ESP32 开发支持").strong().color(p.text));
        ui.label(
            RichText::new(if self.core_installed {
                "esp32:esp32 已安装，支持绝大多数 ESP32 开发板。"
            } else {
                "未安装 esp32:esp32。安装后会自动加载完整 ESP32 板卡列表。"
            })
            .size(11.5)
            .color(if self.core_installed { OK } else { WARN }),
        );
        if ui
            .checkbox(
                &mut self.cfg.use_china_mirror,
                "使用 Espressif 中国镜像（Jihulab）",
            )
            .on_hover_text(
                "通过官方 Jihulab 索引安装 -cn 版本，core 和工具链由 dl.espressif.cn 加速",
            )
            .changed()
        {
            self.config_dirty = true;
        }
        ui.label(
            RichText::new("中国镜像不可用时可在上方关闭，改用官方 GitHub 源。")
                .size(11.0)
                .color(p.weak_text),
        );
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.toolchain.cli.is_some() && self.runner.is_none(),
                    Button::new(
                        RichText::new(if self.core_installed {
                            "重新安装 / 更新"
                        } else {
                            "安装 ESP32 支持"
                        })
                        .size(12.0),
                    ),
                )
                .on_hover_text(if self.cfg.use_china_mirror {
                    "通过 Jihulab 中国镜像安装 -cn 版本"
                } else {
                    "通过官方源安装"
                })
                .clicked()
            {
                self.start_core_install();
            }
            if ui
                .add_enabled(
                    self.core_installed && self.runner.is_none(),
                    Button::new(RichText::new("刷新板卡列表").size(12.0)),
                )
                .on_hover_text(if self.core_installed {
                    "重新从 Arduino CLI 读取板卡列表"
                } else {
                    "安装 ESP32 开发支持后可用"
                })
                .clicked()
            {
                self.refresh_env();
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.label(RichText::new("草稿").strong().color(p.text));
        ui.label(
            RichText::new(self.cfg.sketch.clone())
                .size(11.0)
                .color(p.weak_text),
        );
        ui.horizontal(|ui| {
            if ui.button("打开草稿文件夹").clicked() {
                util::open_path(&self.cfg.sketch_dir());
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.label(RichText::new("快捷键").strong().color(p.text));
        ui.label(
            RichText::new("F7 编译   F8 上传\nCtrl+S 保存   Ctrl+Shift+S 另存为   Ctrl+O 打开")
                .size(11.5)
                .color(p.weak_text),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("Wild IDE 野IDE v{APP_VERSION}"))
                    .size(11.0)
                    .color(p.weak_text),
            );
            ui.label(
                RichText::new("Rust · egui · Arduino CLI")
                    .size(11.0)
                    .color(p.weak_text),
            );
        });
    }

    fn apply_cli_path(&mut self) {
        self.cfg.cli_path = self.settings_cli.trim().to_owned();
        self.toolchain = Toolchain::detect(&self.cfg.cli_path);
        self.cli_version = self.toolchain.version.clone();
        self.core_installed = false;
        self.config_dirty = true;
        if self.toolchain.cli.is_some() {
            self.settings_error = None;
            self.status_success(format!(
                "已连接 Arduino CLI {}",
                self.toolchain.version.as_deref().unwrap_or("")
            ));
            self.refresh_env();
        } else {
            let message = "未找到可用的 arduino-cli，请检查路径".to_owned();
            self.settings_error = Some(message.clone());
            self.status_error(message);
        }
    }
}

impl eframe::App for WildApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events();
        self.tick(ctx);

        let mut action = Action::None;
        self.handle_shortcuts(ctx, &mut action);

        let p = palette(self.cfg.theme);

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(26.0)
            .frame(
                Frame::none()
                    .fill(p.bar_bg)
                    .inner_margin(Margin::symmetric(12.0, 4.0)),
            )
            .show(ctx, |ui| self.status_bar_ui(ui));

        egui::TopBottomPanel::top("toolbar")
            .exact_height(54.0)
            .frame(
                Frame::none()
                    .fill(p.bar_bg)
                    .inner_margin(Margin::symmetric(12.0, 8.0)),
            )
            .show(ctx, |ui| self.toolbar_ui(ui, &mut action));

        if self.banner_visible() {
            egui::TopBottomPanel::top("banner")
                .frame(Frame::none())
                .show(ctx, |ui| self.banner_ui(ui, &mut action));
        }

        egui::TopBottomPanel::bottom("console")
            .resizable(true)
            .default_height(245.0)
            .min_height(130.0)
            .frame(
                Frame::none()
                    .fill(p.panel_bg)
                    .stroke(Stroke::new(1.0_f32, p.border))
                    .inner_margin(Margin::symmetric(10.0, 8.0)),
            )
            .show(ctx, |ui| self.console_ui(ui, &mut action));

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(p.panel_bg)
                    .inner_margin(Margin::symmetric(10.0, 8.0)),
            )
            .show(ctx, |ui| self.editor_ui(ui, &mut action));

        if self.show_settings {
            let mut open = self.show_settings;
            egui::Window::new("设置")
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .default_width(520.0)
                .show(ctx, |ui| self.settings_ui(ui));
            self.show_settings = open;
        }

        self.handle_action(action, ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_sketch(false);
        let _ = self.cfg.save();
        self.serial.close();
    }
}

// ------------------------------------------------------------------ helpers

fn tab_button(ui: &mut Ui, text: &str, active: bool, p: &theme::Palette) -> bool {
    let color = if active { ACCENT } else { p.weak_text };
    let response = ui.add(
        Button::new(RichText::new(text).size(13.0).strong().color(color))
            .fill(if active { p.faint_bg } else { p.bar_bg })
            .rounding(Rounding::same(6.0))
            .min_size(egui::vec2(0.0, 26.0)),
    );
    response.clicked()
}

fn classify_log(line: &str) -> LogLevel {
    let lower = line.to_lowercase();
    if lower.contains("error:") || lower.contains("失败") || lower.contains("cannot find") {
        LogLevel::Error
    } else if lower.contains("warning:") || lower.contains("警告") {
        LogLevel::Warn
    } else if line.starts_with('✓') {
        LogLevel::Success
    } else if line.starts_with('▶') {
        LogLevel::Command
    } else {
        LogLevel::Info
    }
}

/// Common Windows/esptool messages when the CDC port cannot be opened.
fn is_upload_port_error(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("could not open com")
        || lower.contains("port is busy")
        || lower.contains("cannot configure port")
        || lower.contains("permissionerror")
        || lower.contains("device attached to the system is not functioning")
        || lower.contains("failed to connect")
        || lower.contains("failed uploading")
}

fn natural_port_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    fn split(value: &str) -> (&str, u32, &str) {
        let prefix_len = value
            .char_indices()
            .find(|(_, ch)| ch.is_ascii_digit())
            .map(|(index, _)| index)
            .unwrap_or(value.len());
        let digits_len = value[prefix_len..]
            .char_indices()
            .find(|(_, ch)| !ch.is_ascii_digit())
            .map(|(index, _)| index)
            .unwrap_or(value.len() - prefix_len);
        let prefix = &value[..prefix_len];
        let digits = if digits_len > 0 {
            value[prefix_len..prefix_len + digits_len]
                .parse::<u32>()
                .unwrap_or(0)
        } else {
            0
        };
        let suffix = &value[prefix_len + digits_len..];
        (prefix, digits, suffix)
    }
    let (ap, an, asuffix) = split(a);
    let (bp, bn, bsuffix) = split(b);
    ap.cmp(bp).then(an.cmp(&bn)).then(asuffix.cmp(bsuffix))
}

fn is_ino_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("ino"))
        .unwrap_or(false)
}

/// Add the Arduino extension when the save dialog omitted it, and reject
/// unrelated extensions so the editor never silently saves a `.txt` as a sketch.
fn normalize_ino_path(mut path: PathBuf) -> Option<PathBuf> {
    match path.extension().and_then(|ext| ext.to_str()) {
        None => {
            path.set_extension("ino");
            Some(path)
        }
        Some(ext) if ext.eq_ignore_ascii_case("ino") => {
            if ext != "ino" {
                path.set_extension("ino");
            }
            Some(path)
        }
        Some(_) => None,
    }
}

fn sketch_folder_mismatch(path: &Path) -> bool {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let folder = path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    !stem.is_empty() && !folder.is_empty() && !stem.eq_ignore_ascii_case(folder)
}

/// A new Save As target gets its own Arduino sketch folder. Existing files
/// are left exactly where the user put them.
fn save_as_target(path: PathBuf) -> PathBuf {
    if path.exists() || !sketch_folder_mismatch(&path) {
        return path;
    }
    let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
        return path;
    };
    let Some(parent) = path.parent() else {
        return path;
    };
    parent.join(stem).join(format!("{stem}.ino"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_sort_naturally() {
        let mut ports = vec!["COM10", "COM2", "COM1", "COM20"];
        ports.sort_by(|a, b| natural_port_cmp(a, b));
        assert_eq!(ports, vec!["COM1", "COM2", "COM10", "COM20"]);
    }

    #[test]
    fn log_levels() {
        assert_eq!(classify_log("error: foo"), LogLevel::Error);
        assert_eq!(classify_log("warning: bar"), LogLevel::Warn);
        assert_eq!(classify_log("✓ done"), LogLevel::Success);
        assert_eq!(classify_log("▶ start"), LogLevel::Command);
        assert_eq!(classify_log("Sketch uses 123 bytes"), LogLevel::Info);
    }

    #[test]
    fn detects_windows_upload_port_errors() {
        assert!(is_upload_port_error(
            "A fatal error occurred: Could not open COM4, the port is busy or doesn't exist."
        ));
        assert!(is_upload_port_error(
            "(Cannot configure port, something went wrong. Original message: PermissionError(13, 'A device attached to the system is not functioning.', None, 31))"
        ));
        assert!(!is_upload_port_error("Sketch uses 306036 bytes"));
    }

    #[test]
    fn ino_paths_are_normalized() {
        assert_eq!(
            normalize_ino_path(PathBuf::from("Blink")),
            Some(PathBuf::from("Blink.ino"))
        );
        let upper = normalize_ino_path(PathBuf::from("Blink.INO")).unwrap();
        assert_eq!(upper.extension().and_then(|ext| ext.to_str()), Some("ino"));
        assert!(normalize_ino_path(PathBuf::from("Blink.txt")).is_none());
        assert!(is_ino_path(Path::new("Blink.ino")));
        assert!(is_ino_path(Path::new("Blink.INO")));
        assert!(!is_ino_path(Path::new("Blink.txt")));
    }

    #[test]
    fn detects_sketch_folder_mismatch() {
        assert!(!sketch_folder_mismatch(Path::new(
            "C:/sketches/Blink/Blink.ino"
        )));
        assert!(sketch_folder_mismatch(Path::new("C:/sketches/Blink.ino")));
    }

    #[test]
    fn save_as_creates_a_matching_folder() {
        let base =
            std::env::temp_dir().join(format!("wild-ide-save-as-test-{}", std::process::id()));
        let source = base.join("Blink.ino");
        let expected = base.join("Blink").join("Blink.ino");
        assert_eq!(save_as_target(source), expected);

        let valid = base.join("Blink").join("Blink.ino");
        assert_eq!(save_as_target(valid.clone()), valid);
    }
}
