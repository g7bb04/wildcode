use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    Compile,
    Upload,
    CoreInstall,
    CliInstall,
}

impl TaskKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Compile => "编译",
            Self::Upload => "上传",
            Self::CoreInstall => "安装 ESP32 开发支持",
            Self::CliInstall => "安装 Arduino CLI",
        }
    }
}

pub struct Runner {
    pub id: u64,
    pub kind: TaskKind,
    pub cancel: Arc<AtomicBool>,
    pub started: Instant,
}

impl Runner {
    pub fn new(id: u64, kind: TaskKind, cancel: Arc<AtomicBool>) -> Self {
        Self {
            id,
            kind,
            cancel,
            started: Instant::now(),
        }
    }

    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn elapsed_seconds(&self) -> u64 {
        self.started.elapsed().as_secs()
    }
}
