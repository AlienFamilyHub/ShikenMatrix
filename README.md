# Kizuna

Kizuna 是一个基于 Tauri 的桌面应用程序，使用 Rust + Vue 3 开发。该应用程序可以监视当前前台活跃窗口的信息、系统的SMTC信息、窗口图标，并将上传到服务器。

实现方式照抄自 [TNXG/ProcessReporterWingo](https://github.com/TNXG/ProcessReporterWingo)，或许说就是其的Rust实现（附带了ui）


## Now

- [ ] 全新设计的UI
- [ ] 针对MacOS的更深度的适配
- [ ] 尝试接入AI实现自动调整动态简介

## 特性

- 实时监控当前前台活跃窗口的信息
- 系统的媒体信息
- 窗口图标
- 自动上传到服务器
- 可自定义上传间隔
- 可自定义软件名替换规则

## 技术栈

- **前端框架**: Vue 3 + TypeScript
- **构建工具**: Vite / Cargo
- **桌面应用框架**: Tauri
- **状态管理**: Pinia
- **UI 框架**: TailwindCSS + DaisyUI + Shadcn/Vue
- **动画**: GSAP
- **包管理器**: pnpm
- **代码规范**: ESLint + TypeScript

## 项目结构

```
src/
├── assets/      # 静态资源文件
├── components/  # 可复用组件
├── layouts/     # 布局组件
├── lib/         # 工具库
├── stores/      # Pinia 状态管理
├── views/       # 页面视图
└── public/      # 公共资源
```


## 技术实现

关于 Kizuna 的实现原理与设计思路，可参考 [DeepWiki AlienFamilyHub/Kizuna](https://deepwiki.com/AlienFamilyHub/Kizuna) 提供的相关资料。

请注意：DeepWiki内容由 AI 生成，可能并非基于最新信息。在阅读和使用时，请务必自行甄别其准确性与时效性。

尤其需要指出的是，当前本仓库正在进行大版本迭代，代码结构与核心逻辑存在较大变动，DeepWiki文档内容可能存在滞后甚至与实际实现不符的情况，请以实际代码为准。


## 开发环境要求

- Node.js (推荐最新 LTS 版本)
- pnpm 10.10.0 或更高版本
- Rust (用于 Tauri)

## 快速开始

1. 安装依赖：

```bash
pnpm install
```

2. 启动开发服务器：

```bash
# 仅前端开发
pnpm dev

# Tauri 开发
pnpm tauri dev
```

3. 构建应用：

```bash
pnpm build
```

## 主要依赖

### 生产依赖
- Vue 3 - 渐进式 JavaScript 框架
- Vue Router - 官方路由管理器
- Pinia - Vue 状态管理库
- TailwindCSS - 实用优先的 CSS 框架
- DaisyUI - TailwindCSS 组件库
- GSAP - 专业级动画库
- Tauri - 桌面应用程序框架

### 开发依赖
- TypeScript - JavaScript 的超集
- Vite - 现代前端构建工具
- ESLint - 代码质量工具
- UnoCSS - 原子化 CSS 引擎（格式化Tailwindcss用的工具）

## 脚本命令

- `pnpm dev` - 启动开发服务器
- `pnpm build` - 构建生产版本
- `pnpm preview` - 预览生产构建
- `pnpm tauri` - Tauri 相关命令

## 使用说明

1. **配置文件**：
   - 编辑 `config.yml` 文件，设置服务器端点和令牌。

```yaml
server_config:
  endpoint: apiurl # https://api.example.com/api/v2/fn/ps/update
  token: apikey # 设置的key
  report_time: 5 # 上报时间间隔，单位秒
rules: # 软件名的替换规则
  - match_application: WeChat
    replace:
      application: 微信
      description: 一个小而美的办公软件
  - match_application: QQ
    replace:
      application: QQ
      description: 一个多功能的通讯软件
  - match_application: Netease Cloud Music
    replace:
      application: 网易云音乐
      description: 一个音乐播放和分享的平台
```

2. **日志查看**：

   - 日志文件存储在 `logs` 目录下，每天生成一个日志文件。

3. **图标转换**：
   - 应用会获取当前窗口的图标，但是暂且未实现上传逻辑

## 其他问题

### Q：网易云音乐不能上报

A：网易云音乐不按照微软官方的媒体渠道上报媒体信息（即 Windows system media Transport control 集成）

`从 Windows 10 版本 1607 开始，默认情况下，使用 MediaPlayer 类或 AudioGraph 类播放媒体的 UWP 应用会自动与 SMTC 集成。 只需实例化 MediaPlayer 的新实例，并将 MediaSource、MediaPlaybackItem 或 MediaPlaybackList 分配给玩家的 Source 属性，然后用户将在 SMTC 中看到你的应用名称，并且可以使用 SMTC 控件播放、暂停和在播放列表中移动。  -- Windows文档`

这时需要其他方法来使本程序的media上报结构生效

- 通过插件使其通过SMTC上报信息
  - 网易云音乐：[MicroCBer/BetterNCM](https://github.com/MicroCBer/BetterNCM) 和 [BetterNCM/InfinityLink](https://github.com/BetterNCM/InfinityLink) 搭配使用
- Pr Welcome

## 推荐的IDE设置

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 联系我们

- 个人博客: [tnxgmoe.com](https://tnxgmoe.com/about-me#:re:%E8%81%94%E7%B3%BB%E6%96%B9%E5%BC%8F)

2025 © TNXG 本项目遵循 AGPL 3.0 license 开源

## 贡献指南

欢迎提交 issue 和 PR，参与项目共建。

## 许可证

本项目基于 [LICENSE.md](./LICENSE.md) 开源。
