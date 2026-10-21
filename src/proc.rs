use crate::events::{BgEvent, EventSender};
use std::io::{BufReader, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Prevent a console window from flashing up for every child process on Windows.
pub fn hide_console(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = command;
    }
}

/// Run a command and stream stdout/stderr into `tx` as individual lines.
/// Returns once the child exits (or is killed through `cancel`).
pub fn run_streaming(
    command: &mut Command,
    tx: &EventSender,
    task: u64,
    cancel: &AtomicBool,
) -> std::io::Result<ExitStatus> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(command);

    let mut child = command.spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let out_handle = stdout.map(|pipe| {
        let tx = tx.clone();
        thread::spawn(move || stream_lines(pipe, tx, task))
    });
    let err_handle = stderr.map(|pipe| {
        let tx = tx.clone();
        thread::spawn(move || stream_lines(pipe, tx, task))
    });

    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(30)),
            Err(err) => {
                let _ = child.kill();
                if let Some(handle) = out_handle {
                    let _ = handle.join();
                }
                if let Some(handle) = err_handle {
                    let _ = handle.join();
                }
                return Err(err);
            }
        }
    };

    // Join the readers so that no output line is lost before Finished is sent.
    if let Some(handle) = out_handle {
        let _ = handle.join();
    }
    if let Some(handle) = err_handle {
        let _ = handle.join();
    }

    Ok(status)
}

fn stream_lines<R: Read>(pipe: R, tx: EventSender, task: u64) {
    let mut reader = BufReader::new(pipe);
    let mut chunk = [0u8; 8192];
    let mut line = Vec::new();

    loop {
        let count = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => count,
            Err(_) => break,
        };
        for &byte in &chunk[..count] {
            if byte == b'\n' || byte == b'\r' {
                emit_line(&mut line, &tx, task);
            } else {
                line.push(byte);
            }
        }
    }
    emit_line(&mut line, &tx, task);
}

fn emit_line(line: &mut Vec<u8>, tx: &EventSender, task: u64) {
    if line.is_empty() {
        return;
    }
    let raw = String::from_utf8_lossy(line);
    let text = strip_ansi(raw.trim_end());
    line.clear();
    if text.trim().is_empty() {
        return;
    }
    let _ = tx.send(BgEvent::Output { task, line: text });
}

pub fn run_quiet(command: &mut Command) -> std::io::Result<std::process::Output> {
    hide_console(command);
    command.stdin(Stdio::null()).output()
}

/// Remove the handful of ANSI escape sequences Arduino CLI may emit.
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for code in chars.by_ref() {
                    if code.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_colors() {
        assert_eq!(strip_ansi("\u{1b}[31merror\u{1b}[0m"), "error");
    }
}
