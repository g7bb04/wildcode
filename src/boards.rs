/// A board known to `arduino-cli`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub name: String,
    pub fqbn: String,
}

impl Board {
    pub fn new(name: impl Into<String>, fqbn: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fqbn: fqbn.into(),
        }
    }
}

/// Built-in fallback list used only after the ESP32 core is installed but
/// `arduino-cli board listall` fails or returns nothing. Before installation
/// the UI intentionally shows no board list at all.
pub fn fallback_boards() -> Vec<Board> {
    [
        ("ESP32 Dev Module", "esp32:esp32:esp32"),
        ("ESP32-WROOM-DA Module", "esp32:esp32:esp32da"),
        ("ESP32 Wrover Module", "esp32:esp32:esp32wrover"),
        ("ESP32 Wrover Kit", "esp32:esp32:esp32wroverkit"),
        ("ESP32 PICO-D4", "esp32:esp32:pico32"),
        ("ESP32-S2 Dev Module", "esp32:esp32:esp32s2"),
        ("ESP32-S3 Dev Module", "esp32:esp32:esp32s3"),
        ("ESP32-C2 Dev Module", "esp32:esp32:esp32c2"),
        ("ESP32-C3 Dev Module", "esp32:esp32:esp32c3"),
        ("ESP32-C6 Dev Module", "esp32:esp32:esp32c6"),
        ("ESP32-H2 Dev Module", "esp32:esp32:esp32h2"),
        ("AI Thinker ESP32-CAM", "esp32:esp32:esp32cam"),
        ("NodeMCU-32S", "esp32:esp32:nodemcu_32s"),
        ("DOIT ESP32 DEVKIT V1", "esp32:esp32:esp32doit-devkit-v1"),
        ("WEMOS LOLIN32", "esp32:esp32:lolin32"),
        ("WEMOS LOLIN32 Lite", "esp32:esp32:lolin32_lite"),
        ("WEMOS D32", "esp32:esp32:lolin_d32"),
        ("WEMOS D32 Pro", "esp32:esp32:lolin_d32_pro"),
        ("WEMOS LOLIN C3 Mini", "esp32:esp32:lolin_c3_mini"),
        ("WEMOS LOLIN S2 Mini", "esp32:esp32:lolin_s2_mini"),
        ("WEMOS LOLIN S3", "esp32:esp32:lolin_s3"),
        ("WEMOS LOLIN S3 Mini", "esp32:esp32:lolin_s3_mini"),
        ("TTGO LoRa32-OLED", "esp32:esp32:ttgo-lora32"),
        ("TTGO T-Beam", "esp32:esp32:ttgo_t_beam"),
    ]
    .into_iter()
    .map(|(name, fqbn)| Board::new(name, fqbn))
    .collect()
}

pub fn board_name<'a>(boards: &'a [Board], fqbn: &'a str) -> &'a str {
    boards
        .iter()
        .find(|board| board.fqbn == fqbn)
        .map(|board| board.name.as_str())
        .unwrap_or(fqbn)
}
