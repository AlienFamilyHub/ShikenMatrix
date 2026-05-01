# ShikenMatrix

<div align="center">

![License](https://img.shields.io/badge/license-AGPL%20v3.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey.svg)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-yellow.svg)
![Solid.js](https://img.shields.io/badge/Solid.js-1.9-blue.svg)

**一个现代化的跨平台桌面窗口与媒体信息展示工具**

</div>

## 📖 简介

ShikenMatrix 是一个基于 **Tauri** 构建的跨平台桌面应用程序，旨在提供优雅的窗口管理和媒体信息展示体验。它利用前端 Web 技术（**Solid.js**）结合高性能的 **Rust** 后端，能够实时显示当前前台窗口的详细信息以及正在播放的媒体内容。

本项目是 [**Kizuna**](https://github.com/AlienFamilyHub/Kizuna) 项目的继任者，在经历原生 UI 版本的迭代后，迎来了基于 Tauri 的全新架构进化。借助于 Tauri 2.0，不仅保持了原有的高性能系统级监控能力，还大大提升了 UI 开发的灵活性与跨平台一致性。

本项目采用 **GNU Affero General Public License v3.0** 开源协议。

---

## ✨ 特性

### 🖥️ 智能窗口信息展示

- **实时监控**：自动获取并显示当前活动窗口的标题、图标和进程信息
- **应用图标提取**：动态加载并显示前台应用的图标
- **窗口标题跟踪**：实时更新窗口标题变化
- **智能缓存**：LRU 缓存机制，最多缓存 20 个应用图标（~7 MB）

### 🎵 媒体播放集成

- **媒体元数据**：支持显示当前播放的音乐/视频标题、艺术家、专辑信息
- **专辑封面展示**：自动获取并显示高质量专辑封面
- **播放状态同步**：实时同步播放/暂停状态，仅在播放时显示媒体信息
- **跨应用支持**：支持系统级媒体控制
  - **macOS**: 使用 MediaRemote 框架（基于 [MediaRemote-rs](https://github.com/TNXG/MediaRemote-rs)）
  - **Windows**: 使用 System Media Transport Controls (SMTC)

### 🎨 现代化 Web UI 设计

- **极致性能**：前端采用极速响应的 Solid.js 与 Vite 构建
- **玻璃拟态风格**：保留精美的透明/模糊视觉效果，完美融入不同操作系统的桌面环境
- **自适应主题**：自动适配系统亮色/暗色模式，提供一致的视觉体验
- **流畅动画**：原生级前端过渡动画与交互响应

### 🌐 跨平台支持

- **macOS**：✅ 完整支持
- **Windows**：✅ 完整支持
- **架构设计**：采用严谨的平台抽象层（`platform` 模块），将 OS 底层 API 与业务逻辑解耦，未来可轻松扩展至 Linux 等其他系统。

---

## 🏗️ 架构设计

### Tauri 混合架构

```
┌─────────────────────────────────────────┐
│         Web UI Layer (Tauri WebView)    │
│  ┌─────────────────────────────────┐    │
│  │   Solid.js + TypeScript + Vite  │    │
│  └─────────────────────────────────┘    │
└──────────────┬──────────────────────────┘
               │ Tauri IPC Bridge
┌──────────────┴──────────────────────────┐
│         Rust Backend (Tauri Core)      │
│  ┌────────────────────────────────┐    │
│  │  Reporter (tokio async)        │    │
│  │  - WebSocket 上报 (tokio-tungstenite)│
│  │  - 数据去重与缓存              │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Platform Layer                │    │
│  │  - macOS: objc2, MediaRemote   │    │
│  │  - Windows: windows-rs, SMTC   │    │
│  └────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 环境要求

#### 通用要求

- **Node.js**: v18 或更高版本
- **包管理器**: pnpm (推荐)
- **Rust**: 1.70 或更高版本（通过 [rustup](https://rustup.rs/) 安装）

#### macOS 特有依赖

- **macOS**: 11.0 Big Sur 或更高版本
- **Xcode / Command Line Tools**: 构建原生扩展所需

#### Windows 特有依赖

- **Windows 10**: 版本 1809 或更高版本
- **Visual Studio**: 含 C++ 桌面开发工作负载的构建工具
- **Windows SDK**: 最新版本

### 编译与运行

1. **克隆项目并安装前端依赖**

```bash
git clone https://github.com/AlienFamilyHub/ShikenMatrix.git
cd ShikenMatrix

# 安装 Node 依赖
pnpm install
```

2. **启动开发环境**

```bash
# 这将会同时启动 Vite 开发服务器和 Tauri 的 Rust 后端
pnpm tauri dev
```

3. **构建生产版本**

```bash
pnpm tauri build
```

（构建产物将输出在 `src-tauri/target/release` 目录下，并打包为各平台的安装程序，如 `.dmg`/`.app` 或 `.msi` 等）

### 权限配置

#### macOS

首次运行时，应用会请求**辅助功能权限**，这是获取前台窗口信息所必需的：

1. 打开 **系统设置** > **隐私与安全性** > **辅助功能**
2. 找到 `ShikenMatrix` 并勾选启用
3. 重启应用

> 如果构建的本地区域应用出现不受信任的提示，可在终端执行：
>
> ```bash
> xattr -d com.apple.quarantine /Applications/ShikenMatrix.app
> ```

#### Windows

Windows 系统的窗口信息访问对权限管理相对宽松，通常无需额外配置即可正常使用。

---

## 📂 项目结构

```
ShikenMatrix/
├── src/                          # 前端 Web UI 代码 (Solid.js)
│   ├── App.tsx                   # 主视图组件
│   ├── index.tsx                 # 前端入口
│   ├── App.css                   # 全局样式
│   └── ...
│
├── src-tauri/                    # Tauri Rust 后端代码
│   ├── src/
│   │   ├── platform/             # 平台原生 API 抽象层
│   │   │   ├── mod.rs
│   │   │   ├── macos/            # macOS 窗口与媒体监控 (Accessibility / MediaRemote)
│   │   │   └── windows/          # Windows 窗口与媒体监控 (Win32 API / SMTC)
│   │   ├── services/             # 业务核心服务
│   │   │   ├── reporter/         # 状态监听与 WebSocket 上报
│   │   │   └── config.rs         # 配置管理层
│   │   ├── main.rs               # Tauri 启动入口
│   │   └── lib.rs
│   ├── Cargo.toml                # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 项目配置
│
├── package.json                  # 前端依赖配置
├── vite.config.ts                # Vite 构建构建
└── README.md                     # 项目说明文档
```

---

## 🛠️ 技术栈

**前端技术**

- **Solid.js** - 高性能声明式 JavaScript UI 库
- **Vite** - 下一代前端构建工具
- **TypeScript** - 类型安全的 JavaScript
- **@iconify** - 统一的 SVG 图标解决方案

**后端技术 (Tauri + Rust)**

- **Tauri 2.0** - 高性能跨平台应用端框架
- **tokio** / **tokio-tungstenite** - 异步运行时与 WebSocket 支持
- **serde** & **serde_json** - 数据序列化
- **image** - 图像处理（如图标与封面提取）

**平台集成**

- **macOS**:
  - `objc2` - 现代系统 Objective-C 绑定
  - `core-foundation` - 系统底层框架调用
  - `MediaRemote-rs` - 苹果私有媒体控制框架接口
- **Windows**:
  - `windows-rs` - 微软官方 Windows API Rust 绑定 (Win32 窗口 / 媒体控件)

---

## ❓ 常见问题

### 💻 macOS 相关

#### Q: 为什么 macOS 上无法获取窗口信息？

**A**: 请确保已授予应用辅助功能权限。前往 **系统设置** > **隐私与安全性** > **辅助功能**，勾选 `ShikenMatrix`。

### 💻 Windows 相关

#### Q: 为什么网易云音乐等特定应用的媒体信息不显示？

**A**: 部分国内音乐播放器（如网易云音乐客户端等）没有按照微软官方标准接入 Windows System Media Transport Controls (SMTC)。对于网易云音乐，可以尝试安装 `BetterNCM` 并搭配 `InfinityLink` 插件来修复上报系统。

#### Q: 媒体信息不显示怎么办？

**A**: 媒体信息仅在有音乐/视频正在播放且提供系统级状态时生效（如 Apple Music、Spotify、Chrome 等）。如果音乐处于完全停止或关闭状态，窗口可能会选择隐藏媒体部分显示。

---

## 🔍 技术实现细节

### MediaRemote-rs：绕过 macOS 15.4+ 权限验证

本项目使用的 [MediaRemote-rs](https://github.com/TNXG/MediaRemote-rs) 参考了 [mediaremote-adapter](https://github.com/ungive/mediaremote-adapter) 的设计思路，通过巧妙的三层架构绕过了 macOS 15.4+ 引入的 MediaRemote entitlement 验证。

根据 [@My-Iris](https://github.com/Mx-Iris) 的发现，系统 Perl 二进制文件 `/usr/bin/perl` （Bundle ID `com.apple.perl`）因白名单策略被系统信任访问相关底层原生接口。因此我们的解决方案中通过将核心 MediaRemote 绑定编译为不受签名的动态库，并在沙盒中调用，以实现无需授权提取系统多媒体控制器信息并反向注入回调。

---

## 📄 许可证

本项目基于 [GNU Affero General Public License v3.0](LICENSE.md) 开源。

---

## 🙏 致谢

- [Tauri](https://tauri.app/) - 用于构建更小、更快、更安全的桌面应用程序。
- [Solid.js](https://www.solidjs.com/) - 编译型响应式 UI 库。
- [tokio](https://tokio.rs/) - 异步运行时
- [objc2](https://github.com/madsmtm/objc2) & [windows-rs](https://github.com/microsoft/windows-rs)
- [mediaremote-adapter](https://github.com/ungive/mediaremote-adapter)及[MediaRemote-rs](https://github.com/TNXG/MediaRemote-rs)
