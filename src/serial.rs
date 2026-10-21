use crate::events::{BgEvent, EventSender};
use crate::util::truncate_front;
use serialport::SerialPort;
use std::io::{ErrorKind, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const MAX_BUFFER_BYTES: usize = 200_000;

pub struct SerialManager {
    writer: Option<Box<dyn SerialPort>>,
    reader: Option<JoinHandle<()>>,
    stop: Option<Arc<AtomicBool>>,
    pending: Vec<u8>,
    pub port_name: Option<String>,
    pub baud: u32,
    pub buffer: String,
    pub total_bytes: u64,
}

impl Default for SerialManager {
    fn default() -> Self {
        Self {
            writer: None,
            reader: None,
            stop: None,
            pending: Vec::new(),
            port_name: None,
            baud: 115_200,
            buffer: String::new(),
            total_bytes: 0,
        }
    }
}

impl SerialManager {
    pub fn is_open(&self) -> bool {
        self.writer.is_some()
    }

    pub fn open(&mut self, port_name: &str, baud: u32, tx: EventSender) -> Result<(), String> {
        self.close();

        let port = serialport::new(port_name, baud)
            .timeout(Duration::from_millis(60))
            .open()
            .map_err(|err| format!("无法打开串口 {port_name}: {err}"))?;
        let mut reader = port
            .try_clone()
            .map_err(|err| format!("无法读取串口 {port_name}: {err}"))?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let thread_name = port_name.to_owned();
        let handle = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                match reader.read(&mut buf) {
                    Ok(0) => thread::sleep(Duration::from_millis(10)),
                    Ok(count) => {
                        if tx.send(BgEvent::Serial(buf[..count].to_vec())).is_err() {
                            break;
                        }
                    }
                    Err(err) if err.kind() == ErrorKind::TimedOut => continue,
                    Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                    Err(err) => {
                        if !stop_flag.load(Ordering::Relaxed) {
                            let _ = tx.send(BgEvent::SerialError(format!(
                                "串口 {thread_name} 连接中断: {err}"
                            )));
                        }
                        break;
                    }
                }
            }
        });

        self.writer = Some(port);
        self.reader = Some(handle);
        self.stop = Some(stop);
        self.port_name = Some(port_name.to_owned());
        self.baud = baud;
        self.pending.clear();
        Ok(())
    }

    pub fn close(&mut self) {
        if let Some(stop) = self.stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        // Deassert the control lines before dropping the handle. Some native
        // USB CDC boards (ESP32-S2/S3) keep the old driver state until DTR/RTS
        // are released, which can make the next upload fail with a busy port.
        if let Some(port) = self.writer.as_mut() {
            let _ = port.write_data_terminal_ready(false);
            let _ = port.write_request_to_send(false);
            let _ = port.flush();
        }
        self.writer.take();
        if let Some(handle) = self.reader.take() {
            let _ = handle.join();
        }
        self.port_name = None;
        self.pending.clear();
    }

    pub fn send_line(&mut self, line: &str) -> Result<(), String> {
        let Some(port) = self.writer.as_mut() else {
            return Err("串口未打开".to_owned());
        };
        port.write_all(line.as_bytes())
            .and_then(|_| port.write_all(b"\r\n"))
            .and_then(|_| port.flush())
            .map_err(|err| format!("发送失败: {err}"))
    }

    pub fn push_line(&mut self, text: &str) {
        self.buffer.push_str(text);
        self.buffer.push('\n');
        truncate_front(&mut self.buffer, MAX_BUFFER_BYTES);
    }

    pub fn append(&mut self, bytes: &[u8]) {
        self.total_bytes = self.total_bytes.saturating_add(bytes.len() as u64);
        self.pending.extend_from_slice(bytes);

        loop {
            if std::str::from_utf8(&self.pending).is_ok() {
                let text = std::mem::take(&mut self.pending);
                if let Ok(text) = String::from_utf8(text) {
                    self.buffer.push_str(&text);
                }
                break;
            }

            let (valid, error_len) = match std::str::from_utf8(&self.pending) {
                Ok(_) => break,
                Err(err) => (err.valid_up_to(), err.error_len()),
            };

            if valid > 0 {
                let prefix = self.pending[..valid].to_vec();
                self.pending.drain(..valid);
                if let Ok(text) = std::str::from_utf8(&prefix) {
                    self.buffer.push_str(text);
                }
            }

            match error_len {
                Some(len) => {
                    self.pending.drain(..len.min(self.pending.len()));
                    self.buffer.push('\u{FFFD}');
                }
                None => break,
            }
        }

        truncate_front(&mut self.buffer, MAX_BUFFER_BYTES);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.pending.clear();
        self.total_bytes = 0;
    }
}

impl Drop for SerialManager {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_split_utf8() {
        let mut manager = SerialManager::default();
        let bytes = "温度: 25°C\n".as_bytes();
        for chunk in bytes.chunks(2) {
            manager.append(chunk);
        }
        assert_eq!(manager.buffer, "温度: 25°C\n");
        assert_eq!(manager.total_bytes, bytes.len() as u64);
    }

    #[test]
    fn replaces_invalid_bytes() {
        let mut manager = SerialManager::default();
        manager.append(b"ok\xFFdone");
        assert_eq!(manager.buffer, "ok\u{FFFD}done");
    }
}
