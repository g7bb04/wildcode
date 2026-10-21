use std::path::Path;
use std::process::Command;

pub fn open_path(path: &Path) {
    #[cfg(windows)]
    {
        let mut command = Command::new("explorer.exe");
        command.arg(path);
        let _ = command.spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
}

pub fn truncate_front(text: &mut String, max_len: usize) {
    if text.len() <= max_len {
        return;
    }
    let mut cut = text.len() - max_len;
    while cut < text.len() && !text.is_char_boundary(cut) {
        cut += 1;
    }
    text.drain(..cut);
}

pub fn short_path(path: &Path, max_chars: usize) -> String {
    let text = path.to_string_lossy().to_string();
    let count = text.chars().count();
    if count <= max_chars {
        return text;
    }
    let tail: String = text
        .chars()
        .skip(count.saturating_sub(max_chars.saturating_sub(1)))
        .collect();
    format!("…{tail}")
}
