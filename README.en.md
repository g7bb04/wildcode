# Wild IDE

A tiny Arduino / ESP32 IDE written in Rust. No project tree, no plugin marketplace, no AI —
just open the app, open the serial port, write code, compile and upload. The layout feels like
a serial debugging tool.

[中文](README.md) · [Website](https://g7bb04.github.io/wildcode/) · [GitHub](https://github.com/g7bb04/wildcode) · [Gitee](https://gitee.com/fablabswjtu/wildcode)

## Features

- **Single-window workflow**: code editor, serial monitor, and build output on one screen.
  The bottom panel is resizable.
- **C++ / Arduino syntax highlighting**: a lightweight built-in tokenizer covers keywords,
  types, strings, comments, numbers, preprocessor directives, and common Arduino APIs.
  Line numbers included.
- **ESP32 out of the box**: one-click Arduino CLI download and one-click `esp32:esp32` core install.
- **Full board list**: after the core is installed, boards are read from
  `arduino-cli board listall`; a curated fallback list is shown before that.
- **Compile and upload**: `F7` compiles, `F8` compiles and uploads. The serial port is released
  before upload and reconnected automatically afterwards.
- **Serial monitor**: baud rate selection, incremental UTF-8 decoding (multi-byte characters
  never get split into garbage), auto-scroll, clear, and Enter-to-send with CRLF.
- **Auto-save**: the sketch is saved about 1.2 seconds after you stop typing; settings persist too.
- **File operations**: open an `.ino` file, save to the current file, or save as a new Arduino sketch folder.
- **Dark / light themes** and adjustable editor font size.
- **Local-only**: no account, no telemetry. Network is only needed to install the toolchain and
  to compile for ESP32.

## Quick Start

### 1. Get the app

Download a [release build](https://gitee.com/fablabswjtu/wildcode/releases), or build from source (see below). On Windows, run `wild-ide.exe`.

### 2. First-time setup (once)

On first launch a banner appears at the top:

1. Click **Download and install**. The app prefers Arduino's official CDN
   (`downloads.arduino.cc`) and falls back to GitHub Releases only if the CDN is unavailable.
   Arduino CLI is placed at `%APPDATA%\WildIDE\tools\arduino-cli.exe`.
2. When that finishes, click **Install ESP32 support**. The app runs `core update-index` and
   `core install`. The first install downloads a lot of files and can take a while.
   "Use Espressif China mirror (Jihulab)" is enabled by default: the app reads Espressif's official
   China index, picks the newest `-cn` version (for example `esp32:esp32@3.3.10-cn`), and downloads
   the core and toolchains from `dl.espressif.cn`. The China index usually lags slightly behind
   the global release.

> If GitHub is hard to reach, install Arduino CLI manually (see "Manual toolchain setup").
> You can disable the China mirror under **Settings ⚙ → ESP32 support** to use the official source.

Until ESP32 support is installed, the board picker only shows a hint instead of a board list.
Once installation finishes, the full board list is loaded automatically from Arduino CLI.

### 3. Connect a board

1. Connect your ESP32 board over USB.
2. Pick the board (e.g. `ESP32 Dev Module`) and the port (e.g. `COM3`) in the toolbar.
   The port list refreshes every 5 seconds; you can also click `↻`.
3. Edit the code, press **F7** to compile, or **F8** to compile and upload.
4. Click **Open serial** to watch the board's output. Press Enter in the input box to send a line.

A sample LED-blink sketch is created automatically on first run.

## UI Overview

| Area | Description |
| --- | --- |
| Toolbar | Board picker (available after ESP32 support is installed), port picker, serial toggle, compile, upload, settings |
| Banner | Appears when Arduino CLI or ESP32 support is missing, with install actions |
| Editor | `.ino` sketch with line numbers and highlighting; open / save / save as at top right |
| Bottom panel | "Serial monitor" and "Build output" tabs; drag the top edge to resize |
| Status bar | Current status, Arduino CLI version, ESP32 support state, app version |

## Shortcuts

| Shortcut | Action |
| --- | --- |
| `F7` | Compile |
| `F8` | Compile and upload |
| `Ctrl + S` | Save to the current `.ino` file (`Cmd + S` on macOS) |
| `Ctrl + Shift + S` | Save as a new `.ino` file (`Cmd + Shift + S` on macOS) |
| `Ctrl + O` | Open an `.ino` file (`Cmd + O` on macOS) |
| `Enter` | Send from the serial input (CRLF appended) |
| `Esc` | Close the settings window |

## Data Locations

On Windows everything lives under `%APPDATA%\WildIDE\`:

| Path | Contents |
| --- | --- |
| `config.json` | Board, port, baud rate, font size, theme, CLI path, mirror switch |
| `sketch\WildSketch\WildSketch.ino` | Default sketch (use Open to switch to another `.ino`, or Save As to keep it in your own project folder) |
| `tools\arduino-cli.exe` | Arduino CLI downloaded by the app |

Arduino CLI manages its own core/toolchain data in its default data directory. Wild IDE does not
override it, so it can coexist with Arduino IDE and other CLI setups.

## Manual Toolchain Setup

If automatic download is unavailable:

1. Install Arduino CLI using any of:
   - place `arduino-cli.exe` in `%APPDATA%\WildIDE\tools\`;
   - add it to `PATH`;
   - set its path in **Settings ⚙ → Arduino CLI**.
2. Run the official commands:

   ```bash
   arduino-cli core update-index --additional-urls https://espressif.github.io/arduino-esp32/package_esp32_index.json
   arduino-cli core install esp32:esp32 --additional-urls https://espressif.github.io/arduino-esp32/package_esp32_index.json
   ```

   On mainland China networks, use Espressif's official China index instead (replace the version
   with the newest `-cn` version):

   ```bash
   arduino-cli core update-index --additional-urls https://jihulab.com/esp-mirror/espressif/arduino-esp32/-/raw/gh-pages/package_esp32_index_cn.json
   arduino-cli core install esp32:esp32@3.3.10-cn --additional-urls https://jihulab.com/esp-mirror/espressif/arduino-esp32/-/raw/gh-pages/package_esp32_index_cn.json
   ```

   Specify the `-cn` version explicitly. Arduino's default index also contains `esp32:esp32`, so
   an unversioned `core install esp32:esp32` may select the global package and download from GitHub.

3. Back in Wild IDE, click **Settings → Re-detect** or the board refresh button.

## Mainland China / Mirror

The app is configured for mainland China networks by default:

| Item | Default source | Notes |
| --- | --- | --- |
| Arduino CLI | `downloads.arduino.cc` | Official Arduino CDN; GitHub Releases is only a fallback |
| ESP32 index | `jihulab.com/esp-mirror/...` | China index recommended by Espressif's documentation |
| ESP32 core / tools | `dl.espressif.cn/github_assets/` | The `-cn` versions in the China index point here |

During installation the app reads the China index, finds the newest `-cn` platform version, and runs:

```bash
arduino-cli core update-index --additional-urls <China index URL>
arduino-cli core install esp32:esp32@<version>-cn --additional-urls <China index URL>
```

Specifying the `-cn` version is important. Arduino's default index now also contains
`esp32:esp32`, so an unversioned install may pick the global package and download from GitHub.
With the `-cn` version, the core, compiler, esptool and other tools all come from
`dl.espressif.cn`.

The China index usually lags a few days behind the global release. If the mirror is unavailable,
disable "Use Espressif China mirror (Jihulab)" under **Settings ⚙ → ESP32 support** to use the
official GitHub source.

> Reference: [Espressif Arduino-ESP32 installation guide](https://docs.espressif.com/projects/arduino-esp32/en/latest/installing.html)

## Building From Source

Requirements:

- Rust stable (MSVC target recommended on Windows)
- Windows: Visual Studio Build Tools with the C++ desktop workload
- Linux / macOS: the usual GUI and serial development dependencies

```bash
cargo run                 # run in debug mode
cargo test                # run unit tests
cargo build --release     # release build
```

The binary is written to `target/release/wild-ide.exe` on Windows or `target/release/wild-ide`
elsewhere. The release profile enables LTO, size optimization, codegen-units = 1, and symbol
stripping.

## Project Layout

```
src/
  main.rs       Entry point and window setup
  app.rs        UI layout, state, background task orchestration
  editor.rs     Code editor with line numbers and highlighting
  highlight.rs  C++ / Arduino tokenizer and colors
  arduino.rs    arduino-cli compile / upload wrapper
  toolchain.rs  CLI detection and install, ESP32 core install, board parsing
  boards.rs     Built-in ESP32 fallback board list
  serial.rs     Serial open/read thread, incremental UTF-8 decoding, send
  proc.rs       Streaming child-process output and ANSI cleanup
  events.rs     Background event definitions
  config.rs     Config persistence and default sketch
  theme.rs      Palettes, fonts, egui styling
  runner.rs     Task state and cancellation
  util.rs       Small path/text helpers
```

## Troubleshooting

**No serial ports?**
Install the USB-to-serial driver for your board (CH340, CP210x, FTDI, ...), make sure no other
serial tool holds the port, then click `↻` next to the port picker.

**Arduino CLI download or ESP32 install fails?**
Usually a network issue. Configure a proxy and retry, or follow "Manual toolchain setup".
Progress is shown in the Build output tab.

**Upload fails with `Could not open COMx`, `PermissionError(13)`, or `A device attached to the system is not functioning`?**
This is usually a Windows serial/USB CDC state problem, not a compile or board-selection error. Try:

1. Close Arduino IDE, other serial monitors, and Wild IDE's **Open serial**.
2. Unplug and replug the board. For native-USB boards such as the LOLIN S2 Mini, press `RST` once.
3. Wait for the port to reappear, then click **Upload** again.

If it still fails, hold `BOOT` while clicking Upload, then release it after `Connecting...` appears.
Wild IDE now compiles first, uploads separately, releases the serial port before uploading, and retries once.

**Garbled serial output?**
Switch the baud rate to the value used by `Serial.begin(...)` in your sketch.

**Why only ESP32?**
This version focuses on the ESP32 Arduino ecosystem. Generic FQBN handling exists internally,
but no other platform board lists are provided yet.

## License

MIT — see [LICENSE](LICENSE).
