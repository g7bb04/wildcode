# Wild IDE · 野IDE

一个用 Rust 写的极简 Arduino / ESP32 IDE。没有项目树、没有插件市场、没有 AI——
打开软件、打开串口、写代码、编译上传，界面就像一个顺手的串口调试工具。

[官网](https://g7bb04.github.io/wildcode/) · [English](README.en.md) · [GitHub](https://github.com/g7bb04/wildcode) · [Gitee](https://gitee.com/fablabswjtu/wildcode)

## 特性

- **单窗口工作流**：代码编辑器 + 串口监视器 + 编译输出放在同一屏，底部面板可拖动调整高度。
- **C++ / Arduino 语法高亮**：内置轻量 tokenizer，支持关键字、类型、字符串、注释、数字、预处理指令和常用 Arduino API 高亮，带行号。
- **内置 ESP32 支持**：一键下载 Arduino CLI、一键安装 `esp32:esp32` 开发支持。
- **完整板卡列表**：安装开发支持后，通过 `arduino-cli board listall` 动态读取所有 ESP32 板卡；未安装时提供常用板卡的内置列表。
- **编译与上传**：F7 编译、F8 编译并上传。上传前自动释放串口，上传完成后自动重新连接。
- **串口监视器**：波特率切换、UTF-8 增量解码（不会把中文切成乱码）、自动滚动、清空、回车发送（CRLF）。
- **自动保存**：停止输入约 1.2 秒后自动保存草稿，配置修改也会自动持久化。
- **文件操作**：打开 `.ino` 文件、保存到当前文件，或另存为新的 Arduino 草图文件夹。
- **深色 / 浅色主题**，编辑器字号可调。
- **纯本地**：不需要账号，不联网也能编辑代码；只有安装工具链和编译 ESP32 时需要网络。

## 快速开始

### 1. 获取程序

从 [Release](https://gitee.com/fablabswjtu/wildcode/releases) 下载，或从源码构建（见下文）。Windows 上直接运行 `wild-ide.exe`。

### 2. 首次准备（只需一次）

启动后，顶部会出现提示条：

1. 点击 **「下载并安装」**，程序会优先从 Arduino 官方 CDN
   （`downloads.arduino.cc`）下载 Arduino CLI 到
   `%APPDATA%\WildIDE\tools\arduino-cli.exe`；官方 CDN 不可用时才回退到 GitHub Release。
2. 安装完成后，点击 **「安装 ESP32 支持」**。程序会执行
   `core update-index` 和 `core install`，首次需要下载较多文件，请耐心等待。
   默认开启「使用 Espressif 中国镜像（Jihulab）」：程序会读取 Espressif 官方推荐的中国索引，
   选择最新的 `-cn` 版本（例如 `esp32:esp32@3.3.10-cn`）安装，core 和工具链均从
   `dl.espressif.cn` 下载。中国索引通常比全球版稍晚更新。

> 如果 GitHub 访问困难，也可以手动安装 Arduino CLI（见「手动安装工具链」）。
> 中国镜像可在 **设置 ⚙ → ESP32 开发支持** 中关闭，关闭后使用官方源。

未安装 ESP32 开发支持前，工具栏的开发板下拉框只显示提示，不会列出 ESP32 板卡；
安装完成后会自动从 Arduino CLI 读取并显示完整板卡列表。

### 3. 连接开发板

1. 用 USB 线连接 ESP32 开发板。
2. 在工具栏选择开发板（例如 `ESP32 Dev Module`）和串口（例如 `COM3`）。
   串口列表每 5 秒自动刷新，也可以点 `↻` 手动刷新。
3. 在编辑器中修改代码，按 **F7** 编译，或按 **F8** 编译并上传。
4. 点击 **「打开串口」** 查看开发板输出。串口输入框回车即可发送一行文本。

首次打开会自动创建示例草稿，内容是一个 LED 闪烁程序。

## 界面说明

| 区域 | 说明 |
| --- | --- |
| 顶部工具栏 | 开发板选择（安装 ESP32 支持后可用）、串口选择、打开串口、编译、上传、设置 |
| 提示条 | 未安装 Arduino CLI 或 ESP32 支持时出现，提供一键安装入口 |
| 编辑区 | `.ino` 草稿，带行号和高亮；右上角可打开 / 保存 / 另存为文件 |
| 底部面板 | 「串口监视器」与「编译输出」两个标签页，可拖动上边缘调整高度 |
| 状态栏 | 当前状态、Arduino CLI 版本、ESP32 支持状态、程序版本 |

## 快捷键

| 快捷键 | 功能 |
| --- | --- |
| `F7` | 编译 |
| `F8` | 编译并上传 |
| `Ctrl + S` | 保存到当前 `.ino` 文件（macOS 为 `Cmd + S`） |
| `Ctrl + Shift + S` | 另存为新的 `.ino` 文件（macOS 为 `Cmd + Shift + S`） |
| `Ctrl + O` | 打开 `.ino` 文件（macOS 为 `Cmd + O`） |
| `Enter` | 在串口输入框中发送（自动追加 CRLF） |
| `Esc` | 关闭设置窗口 |

## 数据目录

Windows 下默认位于 `%APPDATA%\WildIDE\`：

| 路径 | 内容 |
| --- | --- |
| `config.json` | 开发板、串口、波特率、字号、主题、CLI 路径、中国镜像开关等设置 |
| `sketch\WildSketch\WildSketch.ino` | 默认草稿（可用「打开文件」切换其他 `.ino`，或用「另存为」保存到自己的项目文件夹） |
| `tools\arduino-cli.exe` | 程序自动下载的 Arduino CLI |

Arduino CLI 自身的开发板数据（core、工具链）由 Arduino CLI 管理，使用其默认数据目录，
Wild IDE 不会覆盖它，因此可以和 Arduino IDE / 其他 CLI 配置共存。

## 手动安装工具链

如果自动下载不可用：

1. 安装 Arduino CLI，任选其一：
   - 把 `arduino-cli.exe` 放到 `%APPDATA%\WildIDE\tools\`；
   - 或加入系统 `PATH`；
   - 或在软件 **设置 ⚙ → Arduino CLI** 中指定可执行文件路径。
2. 打开终端执行。

   官方源：

   ```bash
   arduino-cli core update-index --additional-urls https://espressif.github.io/arduino-esp32/package_esp32_index.json
   arduino-cli core install esp32:esp32 --additional-urls https://espressif.github.io/arduino-esp32/package_esp32_index.json
   ```

   中国大陆网络推荐使用 Espressif 官方中国索引（版本号替换为最新的 `-cn` 版本）：

   ```bash
   arduino-cli core update-index --additional-urls https://jihulab.com/esp-mirror/espressif/arduino-esp32/-/raw/gh-pages/package_esp32_index_cn.json
   arduino-cli core install esp32:esp32@3.3.10-cn --additional-urls https://jihulab.com/esp-mirror/espressif/arduino-esp32/-/raw/gh-pages/package_esp32_index_cn.json
   ```

   注意要显式指定 `-cn` 版本。Arduino 官方默认索引也包含 `esp32:esp32`，
   不指定版本时可能安装成从 GitHub 下载的全球版。

3. 回到 Wild IDE，点击 **设置 → 重新检测** 或工具栏的开发板刷新按钮。

## 中国大陆网络 / 镜像

程序针对中国大陆网络做了以下处理，默认开启，无需额外配置：

| 内容 | 默认下载源 | 说明 |
| --- | --- | --- |
| Arduino CLI | `downloads.arduino.cc` | Arduino 官方 CDN，失败时才回退到 GitHub Release |
| ESP32 索引 | `jihulab.com/esp-mirror/...` | Espressif 官方文档推荐的中国索引 |
| ESP32 core / 工具链 | `dl.espressif.cn/github_assets/` | 中国索引中的 `-cn` 版本指向该镜像 |

安装 ESP32 支持时，程序会先读取中国索引，找到最新的 `-cn` 平台版本，
再让 Arduino CLI 执行：

```bash
arduino-cli core update-index --additional-urls <中国索引 URL>
arduino-cli core install esp32:esp32@<版本>-cn --additional-urls <中国索引 URL>
```

显式指定 `-cn` 版本很重要：Arduino 官方默认索引现在也包含 `esp32:esp32`，
使用不带版本的 `core install esp32:esp32` 可能选中默认索引里的全球版，从而回到 GitHub 下载。
使用 `-cn` 版本后，core、编译器、esptool 等工具都会从 `dl.espressif.cn` 下载。

中国索引通常比全球版晚几天更新。如果镜像不可用，可在
**设置 ⚙ → ESP32 开发支持** 中取消勾选「使用 Espressif 中国镜像（Jihulab）」，
改用官方 GitHub 源。

> 参考：[Espressif Arduino-ESP32 安装文档](https://docs.espressif.com/projects/arduino-esp32/en/latest/installing.html)

## 从源码构建

需要：

- Rust stable（推荐 MSVC target）
- Windows：Visual Studio Build Tools（C++ 桌面开发）
- Linux / macOS：常见的图形与串口开发依赖

```bash
cargo run                 # 调试运行
cargo test                # 运行单元测试
cargo build --release     # 发布构建
```

产物为 `target/release/wild-ide.exe`（Windows）或 `target/release/wild-ide`。

发布配置启用了 LTO、体积优化和符号剥离，程序体积较小。

## 项目结构

```
src/
  main.rs       程序入口与窗口设置
  app.rs        界面布局、状态管理、后台任务调度
  editor.rs     带行号和高亮的代码编辑器
  highlight.rs  C++ / Arduino 词法分析与配色
  arduino.rs    arduino-cli compile / upload 命令封装
  toolchain.rs  CLI 检测、下载安装、ESP32 core 安装、板卡解析
  boards.rs     内置 ESP32 板卡回退列表
  serial.rs     串口打开/读取线程、UTF-8 增量解码、发送
  proc.rs       子进程输出流式读取与 ANSI 清理
  events.rs     后台线程事件定义
  config.rs     配置与默认草稿
  theme.rs      深/浅色配色、字体与 egui 样式
  runner.rs     任务状态与取消
  util.rs       路径与文本小工具
```

## 常见问题

**找不到串口？**
安装开发板的 USB 转串口驱动（CH340、CP210x、FTDI 等），确认没有其他串口工具占用该端口，
然后点击串口旁的 `↻`。

**下载 Arduino CLI 或安装 ESP32 支持失败？**
多为网络问题。可以配置代理后重试，或按「手动安装工具链」操作。安装过程可在「编译输出」中查看。

**上传失败：`Could not open COMx` / `PermissionError(13)` / `A device attached to the system is not functioning`？**
这通常是 Windows 的串口或 USB CDC 状态异常，不是编译或开发板选择错误。请依次尝试：

1. 关闭 Arduino IDE、其他串口监视器，以及 Wild IDE 的「打开串口」。
2. 拔下开发板 USB 后重新插上；LOLIN S2 Mini 等原生 USB 板可以按一下 `RST`。
3. 等端口重新出现后，再点击「上传」。

如果仍失败，按住 `BOOT` 再点「上传」，看到 `Connecting...` 后松开。
Wild IDE 会先编译、再单独上传，上传前释放串口，并自动重试一次。

**串口输出乱码？**
把波特率切换到与代码中 `Serial.begin(...)` 一致的值。

**为什么只有 ESP32？**
当前版本专注 ESP32 Arduino 体系。代码中保留了通用 FQBN 处理，但尚未提供其他平台的板卡列表。

## 许可

MIT，见 [LICENSE](LICENSE)。
