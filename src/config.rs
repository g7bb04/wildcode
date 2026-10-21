use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "Wild IDE";
pub const APP_NAME_ZH: &str = "野IDE";
pub const DEFAULT_SKETCH_NAME: &str = "WildSketch";

pub const DEFAULT_CODE: &str = r#"// Wild IDE · 野IDE
// 选择开发板和串口，然后点击「编译」或「上传」。

void setup() {
  Serial.begin(115200);
  pinMode(LED_BUILTIN, OUTPUT);
  Serial.println("Hello, Wild IDE!");
}

void loop() {
  digitalWrite(LED_BUILTIN, HIGH);
  delay(500);
  digitalWrite(LED_BUILTIN, LOW);
  delay(500);
}
"#;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    #[default]
    Dark,
    Light,
}

impl ThemeChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "深色",
            Self::Light => "浅色",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Fully qualified board name, e.g. esp32:esp32:esp32.
    pub board: String,
    /// Serial port name, e.g. COM3.
    pub port: String,
    pub baud: u32,
    /// Absolute path to the .ino file.
    pub sketch: String,
    /// Optional user supplied Arduino CLI path.
    pub cli_path: String,
    pub font_size: f32,
    pub theme: ThemeChoice,
    pub auto_scroll: bool,
    /// Download ESP32 cores and tools through Espressif's China mirror.
    #[serde(default = "default_true")]
    pub use_china_mirror: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            board: "esp32:esp32:esp32".to_owned(),
            port: String::new(),
            baud: 115_200,
            sketch: default_sketch_path().to_string_lossy().to_string(),
            cli_path: String::new(),
            font_size: 14.5,
            theme: ThemeChoice::Dark,
            auto_scroll: true,
            use_china_mirror: true,
        }
    }
}

impl Config {
    pub fn data_dir() -> PathBuf {
        dirs::data_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("WildIDE")
    }

    pub fn tools_dir() -> PathBuf {
        Self::data_dir().join("tools")
    }

    /// Stable build directory. Keeping it outside the sketch folder avoids
    /// polluting the user's project and lets `arduino-cli upload --input-dir`
    /// reuse the exact binaries produced by the compile step.
    pub fn build_dir() -> PathBuf {
        Self::data_dir().join("build")
    }

    pub fn config_path() -> PathBuf {
        Self::data_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let mut cfg = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Self>(&text).ok())
            .unwrap_or_default();

        if cfg.baud == 0 {
            cfg.baud = 115_200;
        }
        if cfg.font_size < 10.0 || cfg.font_size > 32.0 {
            cfg.font_size = 14.5;
        }
        if cfg.board.trim().is_empty() {
            cfg.board = "esp32:esp32:esp32".to_owned();
        }
        if cfg.sketch.trim().is_empty() {
            cfg.sketch = default_sketch_path().to_string_lossy().to_string();
        }
        cfg
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        fs::write(path, text)
    }

    pub fn sketch_path(&self) -> PathBuf {
        PathBuf::from(&self.sketch)
    }

    pub fn sketch_dir(&self) -> PathBuf {
        self.sketch_path()
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(Self::data_dir)
    }

    pub fn sketch_name(&self) -> String {
        self.sketch_path()
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "WildSketch.ino".to_owned())
    }

    /// Make sure the configured sketch exists. Returns true when a new file
    /// was created (the caller should then load the template).
    pub fn ensure_sketch(&self) -> io::Result<bool> {
        let path = self.sketch_path();
        if path.exists() {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, DEFAULT_CODE)?;
        Ok(true)
    }
}

pub fn default_sketch_path() -> PathBuf {
    Config::data_dir()
        .join("sketch")
        .join(DEFAULT_SKETCH_NAME)
        .join(format!("{DEFAULT_SKETCH_NAME}.ino"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_usable() {
        let cfg = Config::default();
        assert_eq!(cfg.board, "esp32:esp32:esp32");
        assert_eq!(cfg.baud, 115_200);
        assert!(cfg.sketch.ends_with("WildSketch.ino"));
        assert!(cfg.font_size >= 10.0);
        assert!(cfg.use_china_mirror);
    }

    #[test]
    fn config_roundtrips() {
        let cfg = Config::default();
        let text = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&text).unwrap();
        assert_eq!(back.board, cfg.board);
        assert_eq!(back.sketch, cfg.sketch);
        assert_eq!(back.use_china_mirror, cfg.use_china_mirror);
    }

    #[test]
    fn old_config_enables_china_mirror_by_default() {
        let back: Config = serde_json::from_str(r#"{"board":"esp32:esp32:esp32"}"#).unwrap();
        assert!(back.use_china_mirror);
        assert_eq!(back.baud, 115_200);
    }

    #[test]
    fn build_dir_lives_under_app_data() {
        assert!(Config::build_dir().ends_with("WildIDE/build"));
    }
}
