# ShikenMatrix

<div align="center">

![License](https://img.shields.io/badge/license-AGPL%20v3.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Android-lightgrey.svg)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-yellow.svg)
![Solid.js](https://img.shields.io/badge/Solid.js-1.9-blue.svg)
![Vue](https://img.shields.io/badge/Vue-3%20Vapor-42b883.svg)

**跨平台活动监控与状态展示系统**

</div>

## 📖 简介

ShikenMatrix 是一个跨平台活动监控系统，支持桌面端窗口/媒体信息采集、移动端设备状态采集，通过中心服务器汇聚数据并向上游转发，同时提供 Web 管理面板和公开状态展示页。

本项目是 [**Kizuna**](https://github.com/AlienFamilyHub/Kizuna) 项目的继任者，在 Tauri 2.0 架构基础之上扩展为 **多端协同** 的完整监控解决方案。

本项目采用 **GNU Affero General Public License v3.0** 开源协议。

---

## ✨ 特性

### 🖥️ 桌面端 (`apps/desktop`)

- **实时窗口监控**：自动获取当前活动窗口的标题、图标、进程名、PID
- **媒体播放集成**：支持显示播放中的音乐/视频信息（标题、艺术家、专辑、封面、播放进度）
  - **macOS**：MediaRemote 框架（[MediaRemote-rs](https://github.com/TNXG/MediaRemote-rs)）
  - **Windows**：System Media Transport Controls (SMTC)
  - **Linux**：D-Bus MPRIS，支持播放中/暂停状态、多艺术家和本地/HTTP(S) 封面
- **WebSocket 上报**：实时将窗口和媒体数据上报至 ShikenMatrix Server
- **智能去重与缓存**：Hash 去重 + LRU 图标/封面缓存
- **系统托盘**：最小化到托盘、关闭行为可选（隐藏/退出）
- **自适应主题**：自动适配系统亮色/暗色模式

### 📱 移动端 (`apps/android`)

- **前台应用检测**：通过 UsageStatsManager 获取当前前台 App 包名与名称
- **媒体播放监控**：通过 MediaSession 获取播放中的标题、艺术家、专辑、进度
- **设备状态采集**：电池电量/充电状态、网络类型（WiFi/蜂窝/VPN）、粗略位置
- **后台保活**：前台 Service + 开机自启，持续上报
- **可选 Root 增强**：通过 `su` 获取更强的前台应用检测能力 / 没啥用

### 🖧 服务端 (`apps/server`)

- **多客户端接入**：同时支持 Desktop Reporter 和 Mobile 客户端 WebSocket 连接
- **上游转发**：将汇聚的数据实时转发到外部服务
  - **Native WebSocket**：转发到任意 WebSocket 服务器
  - **Mix-Space**：HTTP POST 到兼容 Mix-Space 协议的服务
- **S3 上传**：应用图标 + 媒体封面上传至 S3 兼容存储，AWS SigV4 签名
- **API Key 管理**：为客户端签发/撤销认证密钥
- **SQLite 持久化**：活动记录、运行状态、配置持久存储
- **JWT 认证**：管理面板 API 的登录鉴权

### 🎛️ Web 管理面板 (`apps/panel`)

- **实时仪表盘**：Desktop / Android 双设备视图，SSE 实时推送连接状态、前台活动、媒体播放与设备信息
- **客户端管理**：创建/吊销客户端 API Key，一次性展示新密钥
- **上游配置**：可视化配置转发协议、S3 存储与接入策略
- **公开状态页** (`/`)：游客可见的当前窗口/媒体/活跃状态
- **Vue 3 Vapor**：编译期生成原生 DOM 操作，无虚拟 DOM diff

### 🎨 技术亮点

- **Monorepo 架构**：Turborepo + pnpm workspaces，统一脚本与依赖管理
- **Vue 3 Vapor + Solid.js**：panel 使用 Vapor 模式编译期渲染，desktop 使用 Solid.js 细粒度响应式
- **Rust 后端**：server 和 desktop 后端均使用 Rust，零成本抽象与内存安全
- **原生 CSS 设计令牌**：panel 以 CSS 变量构建明暗主题与组件样式，无运行时样式引擎，图标由 unplugin-icons 内联为静态 SVG

---

## 🏗️ 架构

```
┌──────────────────────────┐  ┌──────────────────────────────┐
│  apps/desktop            │  │  apps/android                │
│  (Tauri 2.0 + Solid.js)  │  │  (Kotlin/Compose)            │
│                          │  │                              │
│  窗口监控 / 媒体采集       │  │  前台App / 媒体 / 电池       │
│  Monitor → Reporter      │  │  网络 / 位置                 │
│         │                │  │         │                    │
└─────────┼────────────────┘  └─────────┼────────────────────┘
          │ WebSocket                   │ WebSocket
          │ /reporter?key=              │ /mobile
          └──────────┬──────────────────┘
                     ▼
┌────────────────────────────────────────────────────────────────┐
│  apps/server (Rust/Axum) :4317                                 │
│                                                                │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌─────────────┐  │
│  │ Reporter │  │  Mobile   │  │  REST    │  │   Admin     │  │
│  │ Handler  │  │  Handler  │  │  API     │  │   Auth      │  │
│  └────┬─────┘  └─────┬─────┘  └────┬─────┘  └──────┬──────┘  │
│       └──────┬───────┘             │               │         │
│              ▼                     ▼               ▼         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────┐   │
│  │  DashboardState  │  │  SQLite Storage  │  │  JWT Auth │   │
│  └────────┬─────────┘  └──────────────────┘  └───────────┘   │
│           ▼                                                   │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  Upstream Relay (Native WS / Mix-Space HTTP / S3)    │    │
│  └──────────────────────────────────────────────────────┘    │
│           │                                                   │
│  ┌────────▼──────────┐                                        │
│  │  嵌入 Panel 前端    │ ← rust-embed 编译时嵌入 dist/         │
│  └───────────────────┘                                        │
└───────────────────────────────────────────────────────────────┘
                     │ HTTP (REST API + 静态资源)
                     ▼
┌────────────────────────────────────────────────────────────────┐
│  apps/panel (Vue 3 Vapor + Vite)                               │
│                                                                │
│  /            → 实时状态页 (连接状态/前台活动/媒体/设备信息)     │
│  /control     → 控制面板 (上游转发/接入控制/密钥/安全/数据)      │
└────────────────────────────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 环境要求

- **Node.js** ≥ 18 + **pnpm** ≥ 9
- **Rust** ≥ 1.70（[rustup](https://rustup.rs/)）
- **macOS** ≥ 11.0 + Xcode Command Line Tools
- **Windows** ≥ 10 (1809) + Visual Studio Build Tools
- **Linux**（桌面端）：GTK 3、WebKitGTK 4.1 / JavaScriptCoreGTK 4.1、D-Bus 开发库及系统托盘依赖；窗口监控支持 EWMH X11，以及 niri / Sway / Hyprland 原生 Wayland IPC
- **Android**（仅移动端）：Android Studio + Android SDK

### 安装依赖

```bash
git clone https://github.com/AlienFamilyHub/ShikenMatrix.git
cd ShikenMatrix
pnpm install
```

### 启动开发

```bash
# 启动所有应用（Turborepo 并行）
pnpm dev

# 单独启动
pnpm dev --filter=@shikenmatrix/desktop   # 桌面端前端 (Vite :1420)
pnpm dev --filter=@shikenmatrix/panel     # 管理面板 (Vite :4400，/api 代理到 :4317)
pnpm dev --filter=@shikenmatrix/server    # 服务端 (Rust :4317)

# 桌面端 Tauri 完整开发（前端 + Rust 后端）
cd apps/desktop && pnpm dev:tauri
```

### 构建

```bash
# 构建所有应用
pnpm build

# 单独构建
pnpm build --filter=@shikenmatrix/panel     # panel → dist/
pnpm build --filter=@shikenmatrix/server    # server → target/release/
cd apps/desktop && pnpm build:tauri         # desktop → .dmg/.msi
cd apps/android && ./gradlew assembleDebug  # android → APK
```

### 服务端配置

服务端默认监听 `0.0.0.0:4317`，可通过环境变量配置：

```bash
SHIKENMATRIX_SERVER_ADDR=127.0.0.1:8080    # 监听地址
SHIKENMATRIX_DB_PATH=/path/to/data.db      # 数据库路径（默认 shikenmatrix.sqlite3）
RUST_LOG=debug                              # 日志级别
```

首次启动会在控制台输出初始管理员密码。

---

## 📂 项目结构

```
ShikenMatrix/
├── apps/
│   ├── desktop/                     # 桌面端 (Tauri 2.0 + Solid.js)
│   │   ├── src/                     # 前端 Web UI
│   │   │   ├── pages/               # MonitorPage / SettingsPage / AboutPage
│   │   │   ├── components/          # AppHeader / RunControls / RuntimeCards / LogPanel
│   │   │   ├── App.tsx
│   │   │   └── index.tsx
│   │   ├── src-tauri/               # Tauri Rust 后端
│   │   │   └── src/
│   │   │       ├── platform/        # macOS / Windows / Linux 平台抽象
│   │   │       ├── services/        # Monitor + Reporter + Config
│   │   │       └── main.rs
│   │   └── package.json
│   │
│   ├── server/                      # 中心服务端 (Rust/Axum + SQLite)
│   │   ├── src/
│   │   │   ├── main.rs              # 启动入口 + 路由
│   │   │   ├── state.rs             # 全局状态管理
│   │   │   ├── storage.rs           # SQLite 存储层
│   │   │   ├── mobile.rs            # 移动端 WS 处理
│   │   │   ├── reporter/            # 上游转发 (WS / Mix-Space / S3)
│   │   │   └── admin.rs             # 管理 API + JWT 认证
│   │   └── package.json
│   │
│   ├── panel/                       # Web 管理面板 (Vue 3 Vapor + Vite)
│   │   ├── src/
│   │   │   ├── app.vue              # 应用外壳 (顶部导航/主题/路由)
│   │   │   ├── views/               # 实时状态页 + 控制面板
│   │   │   ├── components/          # presence/ control/ ui/ 组件
│   │   │   └── lib/                 # API 客户端 / 格式化 / 路由 / 主题
│   │   └── package.json
│   │
│   └── android/                     # 移动端 (Kotlin/Compose)
│       ├── app/src/main/java/.../
│       │   └── nativebridge/        # DeviceSnapshotCollector / BackgroundReporter
│       └── build.gradle
│
├── packages/                        # 共享包（预留）
├── package.json                     # Monorepo 根配置
├── pnpm-workspace.yaml              # pnpm 工作区
├── turbo.json                       # Turborepo 任务编排
└── README.md
```

---

## 🛠️ 技术栈

| 层 | 技术 |
|---|------|
| 桌面端前端 | Solid.js + TypeScript + Vite |
| 桌面端后端 | Tauri 2.0 + Rust (tokio) |
| 管理面板 | Vue 3 Vapor + TypeScript + Vite + unplugin-icons (MingCute) |
| 服务端 | Rust (Axum) + SQLite (rusqlite) |
| 移动端 | Kotlin + Jetpack Compose + OkHttp |
| 构建工具 | Turborepo + pnpm workspaces |

---

## ❓ 常见问题

### macOS

**Q: 无法获取窗口信息？**
前往 **系统设置 > 隐私与安全性 > 辅助功能**，勾选 `ShikenMatrix`。

**Q: 不受信任的开发者提示？**
```bash
xattr -d com.apple.quarantine /Applications/ShikenMatrix.app
```

### Windows

现在大部分音乐软件都能正常通过 SMTC 上报播放信息，可喜可贺

### Linux

**Q: Wayland 下无法获取窗口？**
Wayland 没有通用的前台窗口查询接口。当前直接连接 `NIRI_SOCKET`、`SWAYSOCK` 或 Hyprland 的 `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`，启动桌面端时需继承对应会话环境。GNOME/KDE 等未支持的 Wayland 合成器会返回明确错误，不回退到只能看到部分窗口的 XWayland。X11 需要窗口管理器支持 EWMH；无活动窗口不代表缺少权限。

**Q: 媒体或图标没有显示？**
媒体采集独立于窗口后端，需要可访问的用户 D-Bus 会话，且播放器已发布 MPRIS 接口。优先选取播放中的播放器，其次是暂停的播放器；封面支持 `file://` 和 HTTP(S)，限制为 8 MiB。窗口图标从 X11 `_NET_WM_ICON` 或 XDG desktop entry / 常用图标主题位置读取；不支持的格式（例如 SVG）会省略。

### Android

**Q: 前台应用检测不到？**
确保已授予 **使用情况访问权限**。如需更强检测，可授予 Root 权限。

**Q: 媒体信息不显示？**
确保已开启 **通知监听器权限**，这是获取 MediaSession 所必需的。

---

## 📄 许可证

[GNU Affero General Public License v3.0](LICENSE.md)

---

## 🙏 致谢

- [Tauri](https://tauri.app/) — 跨平台桌面应用框架
- [Solid.js](https://www.solidjs.com/) — 响应式 UI 库
- [Vue 3 Vapor](https://vuejs.org/) — 管理面板 UI 框架
- [Iconify / MingCute](https://iconify.design/) — 图标数据源
- [Axum](https://github.com/tokio-rs/axum) — Rust Web 框架
- [tokio](https://tokio.rs/) — Rust 异步运行时
- [objc2](https://github.com/madsmtm/objc2) / [windows-rs](https://github.com/microsoft/windows-rs)
- [MediaRemote-rs](https://github.com/TNXG/MediaRemote-rs) — macOS 媒体控制绑定
