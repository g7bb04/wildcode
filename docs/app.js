/*
 * Wild IDE website
 * Set GITHUB_URL after the GitHub repository exists to show the footer link.
 */
const GITHUB_URL = "https://github.com/g7bb04/wildcode";

const I18N = {
  zh: {
    "meta.title": "Wild IDE · 野IDE — 极简 Arduino / ESP32 IDE",
    "meta.description": "Wild IDE 是一个用 Rust 写的极简 Arduino / ESP32 IDE。没有项目树、插件市场或 AI，打开、写代码、编译上传。",

    "skip": "跳到主要内容",
    "nav.features": "功能",
    "nav.start": "开始",
    "nav.philosophy": "理念",
    "nav.history": "历史",
    "nav.download": "下载",

    "hero.eyebrow": "0.1.0 早期测试版 · Windows x64",
    "hero.title": "把 Arduino 写代码，<br><span class='accent'>做得更简单。</span>",
    "hero.lede": "一个用 Rust 写的极简 Arduino / ESP32 IDE。没有项目树、没有插件市场、没有 AI——打开软件、打开串口、写代码、编译上传。",
    "hero.download": "下载 Windows 测试版",
    "hero.start": "看看怎么用",
    "hero.meta1": "Windows 10 / 11 · 64 位",
    "hero.meta2": "免安装 · 约 2.7 MB",
    "hero.meta3": "MIT License",

    "mock.openfile": "打开文件",
    "mock.save": "保存",
    "mock.saveas": "另存为",
    "mock.compile": "编译",
    "mock.open": "打开串口",
    "mock.upload": "上传",
    "mock.output": "编译输出",
    "mock.serial": "串口监视器",
    "mock.clear": "清空",
    "mock.status": "就绪 · Arduino CLI 1.5.1 · ESP32 3.3.10-cn · v0.1.0",

    "features.eyebrow": "WHY WILD",
    "features.title": "少一点工具，<br><span class='accent'>多一点创造。</span>",
    "features.lede": "每一个功能都围绕同一个目标：让你从打开程序到看到开发板运行，中间少几步。",
    "features.f1.title": "单窗口工作流",
    "features.f1.body": "编辑器、串口监视器、编译输出在同一屏。拖动分隔线，把空间留给正在做的事。",
    "features.f2.title": "ESP32 开箱即用",
    "features.f2.body": "一键下载 Arduino CLI，一键安装 ESP32 开发支持。安装完成后，完整板卡列表自动出现。",
    "features.f3.title": "为中国网络设计",
    "features.f3.body": "默认使用 Espressif 中国镜像。CLI 走 Arduino 官方 CDN，ESP32 core 和工具链从 dl.espressif.cn 下载。",
    "features.f4.title": "上传更可靠",
    "features.f4.body": "上传前释放串口，失败自动重试；针对原生 USB 开发板补充重连流程和中文提示。",
    "features.f5.title": "顺手的串口监视器",
    "features.f5.body": "波特率切换、自动滚动、UTF-8 增量解码，中文不会被切成乱码；回车发送，自动追加 CRLF。",
    "features.f6.title": "纯本地，无账号",
    "features.f6.body": "没有 AI、没有遥测、没有插件市场。代码、配置和串口数据都留在你的电脑上。",

    "start.eyebrow": "QUICK START",
    "start.title": "三步，<br><span class='accent'>开始写代码。</span>",
    "start.lede": "第一次启动需要联网安装工具链，之后就可以离线编辑。整个过程都有中文提示。",
    "start.s1.title": "下载并运行",
    "start.s1.body": "解压后双击 wild-ide.exe。免安装；配置和草稿保存在 %APPDATA%\\WildIDE。",
    "start.s2.title": "安装 ESP32 支持",
    "start.s2.body": "按顶部提示下载 Arduino CLI，再安装 ESP32 开发支持。中国大陆网络默认使用镜像。",
    "start.s3.title": "编译，然后上传",
    "start.s3.body": "选择开发板和串口。F7 编译，F8 编译并上传。打开串口查看开发板输出。",
    "start.shortcuts": "快捷键",
    "start.k1": "编译",
    "start.k2": "上传",
    "start.k3": "保存",
    "start.k6": "另存为",
    "start.k4": "打开",
    "start.k5": "发送",
    "start.note": "还没有安装 ESP32 支持时，开发板下拉框只显示提示；安装完成后会自动读取完整板卡列表。",

    "philosophy.eyebrow": "PHILOSOPHY",
    "philosophy.title": "工具应该<br><span class='accent'>退到工作流后面。</span>",
    "philosophy.quote": "“一个顺手的工具，应该让你忘记它的存在。”",
    "philosophy.p1": "Wild IDE 从一个很具体的挫败感开始：只想让一块 ESP32 闪个灯，却要先穿过项目树、插件、配置和一堆不认识的概念。",
    "philosophy.p2": "所以我们把界面压到最少：一个编辑器、一个串口、一个输出面板。没有 AI 补全，没有账号，没有插件市场。不是反对这些功能，而是这个工具不需要它们。",
    "philosophy.p3": "它首先为中文用户和中国大陆网络设计。默认镜像、明确的中文提示、能复现的上传流程，比多十个功能更重要。",
    "philosophy.pr1.title": "少即是多",
    "philosophy.pr1.body": "每个界面元素都要证明自己值得占据空间。",
    "philosophy.pr2.title": "本地优先",
    "philosophy.pr2.body": "代码、配置、串口数据都不离开你的电脑。",
    "philosophy.pr3.title": "默认正确",
    "philosophy.pr3.body": "为中国网络、原生 USB 板和真实失败路径做默认处理。",

    "history.eyebrow": "PROJECT NOTES",
    "history.title": "一条还很短的<br><span class='accent'>时间线。</span>",
    "history.lede": "Wild IDE 是一个很新的项目。这里记录的不是版本号，而是几次关键的取舍。",
    "history.t1.date": "2026.10.20",
    "history.t1.title": "起点",
    "history.t1.body": "第一个 commit。目标很简单：做一个打开就能写 ESP32 的桌面工具。",
    "history.t2.date": "2026.10.21",
    "history.t2.title": "第一版",
    "history.t2.body": "单窗口编辑器、C++ 高亮、串口监视器、编译输出和可拖动面板。",
    "history.t3.date": "2026.10.21",
    "history.t3.title": "速度优化",
    "history.t3.body": "Arduino CLI 优先官方 CDN；ESP32 安装读取 Jihulab 中国索引，并显式使用 -cn 版本。",
    "history.t4.date": "2026.10.21",
    "history.t4.title": "把上传做稳",
    "history.t4.body": "编译与上传拆开，上传前释放串口，失败自动重试；补上原生 USB 开发板的重连提示。",
    "history.t5.date": "现在",
    "history.t5.title": "0.1.0 早期测试版",
    "history.t5.body": "第一个面向早期用户的 Windows 测试包。下一步是收集真实开发板上的反馈。",
    "history.note": "时间线来自 git 提交记录和项目笔记；没有把未发生的计划写成历史。",

    "download.eyebrow": "DOWNLOAD",
    "download.title": "下载 <span class='accent-dark'>Windows 测试版</span>",
    "download.lede": "Windows 10 / 11 64 位免安装测试包。macOS / Linux 版本尚未提供。",
    "download.button": "前往 Gitee 下载",
    "download.unsigned": "未签名测试版",
    "download.file": "文件",
    "download.sha": "SHA256",
    "download.copy": "复制",
    "download.copied": "已复制",
    "download.warning": "测试版没有代码签名，Windows SmartScreen 可能提示风险。选择「更多信息」→「仍要运行」。",
    "download.checksum": "在 Gitee 查看发布页",

    "faq.eyebrow": "BEFORE YOU GO",
    "faq.title": "几个常见问题。",
    "faq.q1": "为什么目前只有 Windows？",
    "faq.a1": "第一个测试包只构建了 Windows x64。Wild IDE 使用 Rust + egui，本身可以跨平台；是否提供 macOS / Linux 版本，取决于早期反馈。",
    "faq.q2": "为什么只支持 ESP32？",
    "faq.a2": "当前版本专注 ESP32 Arduino 体系。内部保留了通用 FQBN 处理，但还没有为其他平台准备板卡列表。",
    "faq.q3": "上传失败怎么办？",
    "faq.a3": "先关闭其他串口工具和 Wild IDE 的串口，拔插开发板；原生 USB 板可以按一下 RST。仍失败时按住 BOOT 再点上传，看到 Connecting... 后松开。",
    "faq.q4": "以后会加入 AI 吗？",
    "faq.a4": "不会。Wild IDE 的目标是简单、本地、可预测。没有账号、没有遥测，也没有插件市场。",

    "footer.github": "GitHub",
    "footer.docs": "文档",
    "footer.top": "回到顶部",
    "footer.fine": "© 2026 Wild IDE contributors. 本地优先，没有 AI，没有账号。"
  },
  en: {
    "meta.title": "Wild IDE — a tiny Arduino / ESP32 IDE",
    "meta.description": "Wild IDE is a tiny Arduino / ESP32 IDE written in Rust. No project tree, plugin marketplace, or AI. Open it, write code, compile and upload.",

    "skip": "Skip to content",
    "nav.features": "Features",
    "nav.start": "Get started",
    "nav.philosophy": "Philosophy",
    "nav.history": "History",
    "nav.download": "Download",

    "hero.eyebrow": "0.1.0 early test build · Windows x64",
    "hero.title": "Write Arduino code.<br><span class='accent'>With less in the way.</span>",
    "hero.lede": "A tiny Arduino / ESP32 IDE written in Rust. No project tree, no plugin marketplace, no AI — open it, open the serial port, write code, compile and upload.",
    "hero.download": "Download Windows test build",
    "hero.start": "See how it works",
    "hero.meta1": "Windows 10 / 11 · 64-bit",
    "hero.meta2": "Portable · about 2.7 MB",
    "hero.meta3": "MIT License",

    "mock.openfile": "Open file",
    "mock.save": "Save",
    "mock.saveas": "Save as",
    "mock.compile": "Compile",
    "mock.open": "Open serial",
    "mock.upload": "Upload",
    "mock.output": "Build output",
    "mock.serial": "Serial monitor",
    "mock.clear": "Clear",
    "mock.status": "Ready · Arduino CLI 1.5.1 · ESP32 3.3.10-cn · v0.1.0",

    "features.eyebrow": "WHY WILD",
    "features.title": "Fewer tools.<br><span class='accent'>More making.</span>",
    "features.lede": "Every feature points at the same goal: fewer steps between opening the app and seeing your board run.",
    "features.f1.title": "Single-window workflow",
    "features.f1.body": "Editor, serial monitor, and build output share one screen. Drag the divider to give space to whatever you are doing.",
    "features.f2.title": "ESP32 out of the box",
    "features.f2.body": "One click downloads Arduino CLI, one click installs ESP32 support. The full board list appears when it is ready.",
    "features.f3.title": "Built for China networks",
    "features.f3.body": "The Espressif China mirror is on by default. CLI uses Arduino's official CDN; ESP32 core and tools download from dl.espressif.cn.",
    "features.f4.title": "Uploads that recover",
    "features.f4.body": "The serial port is released before upload, failures retry automatically, and native-USB boards get a reconnect flow with clear hints.",
    "features.f5.title": "A serial monitor that feels right",
    "features.f5.body": "Baud selection, auto-scroll, incremental UTF-8 decoding so Chinese text is never split into garbage, and Enter-to-send with CRLF.",
    "features.f6.title": "Local by default",
    "features.f6.body": "No AI, no telemetry, no plugin marketplace. Code, settings, and serial data stay on your machine.",

    "start.eyebrow": "QUICK START",
    "start.title": "Three steps.<br><span class='accent'>Start writing code.</span>",
    "start.lede": "The first launch needs network access to install the toolchain. After that, editing works offline. Every step has clear Chinese prompts.",
    "start.s1.title": "Download and run",
    "start.s1.body": "Unzip and double-click wild-ide.exe. No installer; settings and drafts live in %APPDATA%\\WildIDE.",
    "start.s2.title": "Install ESP32 support",
    "start.s2.body": "Follow the banner to download Arduino CLI, then install ESP32 support. Mainland China networks use the mirror by default.",
    "start.s3.title": "Compile, then upload",
    "start.s3.body": "Pick your board and port. F7 compiles, F8 compiles and uploads. Open the serial monitor to see the output.",
    "start.shortcuts": "Shortcuts",
    "start.k1": "Compile",
    "start.k2": "Upload",
    "start.k3": "Save",
    "start.k6": "Save as",
    "start.k4": "Open",
    "start.k5": "Send",
    "start.note": "Until ESP32 support is installed, the board picker shows a hint instead of a list. Once installed, the full board list loads automatically.",

    "philosophy.eyebrow": "PHILOSOPHY",
    "philosophy.title": "The tool should<br><span class='accent'>get out of the way.</span>",
    "philosophy.quote": "“A good tool should make you forget it is there.”",
    "philosophy.p1": "Wild IDE started from a very specific frustration: you just want to blink an LED on an ESP32, but first you have to navigate a project tree, plugins, settings, and a pile of unfamiliar concepts.",
    "philosophy.p2": "So we reduced the interface to the essentials: an editor, a serial port, and an output panel. No AI completion, no account, no plugin marketplace. Not because those things are bad, but because this tool does not need them.",
    "philosophy.p3": "It is designed first for Chinese-speaking users and mainland China networks. A working mirror, clear Chinese messages, and a reproducible upload flow matter more than ten extra features.",
    "philosophy.pr1.title": "Less is more",
    "philosophy.pr1.body": "Every element in the interface has to earn its space.",
    "philosophy.pr2.title": "Local first",
    "philosophy.pr2.body": "Code, settings, and serial data never leave your machine.",
    "philosophy.pr3.title": "Correct by default",
    "philosophy.pr3.body": "Defaults handle China networks, native-USB boards, and real failure paths.",

    "history.eyebrow": "PROJECT NOTES",
    "history.title": "A very short<br><span class='accent'>timeline.</span>",
    "history.lede": "Wild IDE is a new project. This is not a list of version numbers; it is a record of a few key trade-offs.",
    "history.t1.date": "2026.10.20",
    "history.t1.title": "The start",
    "history.t1.body": "The first commit. The goal was simple: a desktop tool that opens straight into an ESP32 sketch.",
    "history.t2.date": "2026.10.21",
    "history.t2.title": "First build",
    "history.t2.body": "Single-window editor, C++ highlighting, serial monitor, build output, and a resizable bottom panel.",
    "history.t3.date": "2026.10.21",
    "history.t3.title": "Speed optimization",
    "history.t3.body": "Arduino CLI prefers the official CDN; ESP32 install reads the Jihulab China index and explicitly uses the -cn version.",
    "history.t4.date": "2026.10.21",
    "history.t4.title": "Making uploads reliable",
    "history.t4.body": "Compile and upload became separate steps, the serial port is released first, failures retry once, and native-USB reconnect hints were added.",
    "history.t5.date": "Now",
    "history.t5.title": "0.1.0 early test build",
    "history.t5.body": "The first Windows test package for early users. The next step is collecting feedback from real boards.",
    "history.note": "The timeline comes from git history and project notes. Plans that have not happened are not written as history.",

    "download.eyebrow": "DOWNLOAD",
    "download.title": "Download the <span class='accent-dark'>Windows test build</span>",
    "download.lede": "A portable test package for Windows 10 / 11 64-bit. macOS and Linux builds are not available yet.",
    "download.button": "Download on Gitee",
    "download.unsigned": "Unsigned test build",
    "download.file": "File",
    "download.sha": "SHA256",
    "download.copy": "Copy",
    "download.copied": "Copied",
    "download.warning": "The test build is unsigned, so Windows SmartScreen may warn you. Choose “More info” → “Run anyway”.",
    "download.checksum": "View the release on Gitee",

    "faq.eyebrow": "BEFORE YOU GO",
    "faq.title": "A few common questions.",
    "faq.q1": "Why Windows only for now?",
    "faq.a1": "The first test package is Windows x64 only. Wild IDE is built with Rust + egui and can be cross-platform; macOS and Linux builds depend on early feedback.",
    "faq.q2": "Why ESP32 only?",
    "faq.a2": "This version focuses on the ESP32 Arduino ecosystem. Generic FQBN handling exists internally, but other platform board lists are not ready yet.",
    "faq.q3": "What if upload fails?",
    "faq.a3": "Close other serial tools and Wild IDE's serial port, then replug the board. On native-USB boards, press RST once. If it still fails, hold BOOT while clicking Upload and release it when you see Connecting...",
    "faq.q4": "Will you add AI later?",
    "faq.a4": "No. Wild IDE is meant to be simple, local, and predictable. No account, no telemetry, no plugin marketplace.",

    "footer.github": "GitHub",
    "footer.docs": "Docs",
    "footer.top": "Back to top",
    "footer.fine": "© 2026 Wild IDE contributors. Local-first. No AI, no account."
  }
};

const root = document.documentElement;
const langButtons = Array.from(document.querySelectorAll("[data-lang]"));
let currentLang = "zh";

function t(key, lang = currentLang) {
  return I18N[lang] && I18N[lang][key] !== undefined
    ? I18N[lang][key]
    : (I18N.zh[key] !== undefined ? I18N.zh[key] : "");
}

function applyLanguage(lang) {
  currentLang = lang === "en" ? "en" : "zh";
  root.lang = currentLang === "zh" ? "zh-CN" : "en";

  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const value = t(el.dataset.i18n);
    if (value) el.textContent = value;
  });

  document.querySelectorAll("[data-i18n-html]").forEach((el) => {
    const value = t(el.dataset.i18nHtml);
    if (value) el.innerHTML = value;
  });

  document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
    const value = t(el.dataset.i18nAria);
    if (value) el.setAttribute("aria-label", value);
  });

  document.title = t("meta.title");
  const description = document.querySelector('meta[name="description"]');
  if (description) description.setAttribute("content", t("meta.description"));
  const ogTitle = document.querySelector('meta[property="og:title"]');
  if (ogTitle) ogTitle.setAttribute("content", t("meta.title"));
  const ogDescription = document.querySelector('meta[property="og:description"]');
  if (ogDescription) ogDescription.setAttribute("content", t("meta.description"));

  langButtons.forEach((button) => {
    const active = button.dataset.lang === currentLang;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-pressed", active ? "true" : "false");
  });

  try {
    localStorage.setItem("wild-ide-lang", currentLang);
  } catch (_) {
    /* localStorage can be unavailable in private modes. */
  }
}

function setupLanguage() {
  let saved = "zh";
  try {
    const stored = localStorage.getItem("wild-ide-lang");
    if (stored === "zh" || stored === "en") saved = stored;
  } catch (_) {
    /* Ignore. */
  }

  langButtons.forEach((button) => {
    button.addEventListener("click", () => applyLanguage(button.dataset.lang));
  });

  applyLanguage(saved);
}

function setupCopyButtons() {
  document.querySelectorAll("[data-copy]").forEach((button) => {
    button.addEventListener("click", () => {
      const value = button.dataset.copy || "";
      const label = button.querySelector("[data-i18n]");
      const finish = () => {
        if (label) label.textContent = t("download.copied");
        button.classList.add("is-copied");
        window.setTimeout(() => {
          if (label) label.textContent = t(label.dataset.i18n);
          button.classList.remove("is-copied");
        }, 1600);
      };

      if (navigator.clipboard && window.isSecureContext) {
        navigator.clipboard.writeText(value).then(finish).catch(() => {
          window.prompt("SHA256", value);
        });
      } else {
        const textarea = document.createElement("textarea");
        textarea.value = value;
        textarea.setAttribute("readonly", "");
        textarea.style.position = "fixed";
        textarea.style.opacity = "0";
        document.body.appendChild(textarea);
        textarea.select();
        try {
          document.execCommand("copy");
          finish();
        } catch (_) {
          window.prompt("SHA256", value);
        }
        textarea.remove();
      }
    });
  });
}

function setupReveal() {
  const items = Array.from(document.querySelectorAll("[data-reveal]"));
  if (!("IntersectionObserver" in window)) {
    items.forEach((item) => item.classList.add("is-visible"));
    return;
  }

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("is-visible");
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.12, rootMargin: "0px 0px -40px 0px" }
  );

  items.forEach((item) => observer.observe(item));
}

function setupHeader() {
  const header = document.querySelector(".site-header");
  if (!header) return;

  const update = () => header.classList.toggle("is-scrolled", window.scrollY > 8);
  update();
  window.addEventListener("scroll", update, { passive: true });
}

function setupNavHighlight() {
  const links = Array.from(document.querySelectorAll('.site-nav a[href^="#"]'))
    .filter((link) => !link.classList.contains("nav-download"));
  const sections = links
    .map((link) => document.querySelector(link.getAttribute("href")))
    .filter(Boolean);

  if (!sections.length || !("IntersectionObserver" in window)) return;

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        const id = `#${entry.target.id}`;
        links.forEach((link) => link.classList.toggle("is-active", link.getAttribute("href") === id));
      });
    },
    { rootMargin: "-35% 0px -58% 0px", threshold: 0 }
  );

  sections.forEach((section) => observer.observe(section));
}

function setupGithubLink() {
  const link = document.getElementById("github-link");
  if (!link || !GITHUB_URL) return;
  link.href = GITHUB_URL;
  link.hidden = false;
}

setupLanguage();
setupCopyButtons();
setupReveal();
setupHeader();
setupNavHighlight();
setupGithubLink();
