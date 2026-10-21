use crate::boards::Board;
use std::sync::mpsc::Sender;

/// Events produced by background workers and the serial reader.
#[derive(Debug)]
pub enum BgEvent {
    /// One line of output belonging to a task.
    Output { task: u64, line: String },
    /// A task has ended.
    Finished {
        task: u64,
        success: bool,
        code: Option<i32>,
        cancelled: bool,
    },
    /// Raw bytes read from the serial port.
    Serial(Vec<u8>),
    /// The serial reader stopped because of an error.
    SerialError(String),
    /// Result of probing Arduino CLI / the ESP32 core.
    Env(EnvInfo),
}

#[derive(Debug, Clone)]
pub struct EnvInfo {
    pub generation: u64,
    pub cli_version: Option<String>,
    pub core_installed: bool,
    pub boards: Vec<Board>,
}

pub type EventSender = Sender<BgEvent>;
