# ADR-0001：使用 Tauri 2 + React + Rust 构建桌面应用

## 状态

Accepted

## 背景

DevForge 需要一个跨平台桌面应用，包含代码索引、AI 问答、文件修改审批等复杂功能。需要选择桌面框架、前端框架和后端语言。

## 决策

使用 Tauri 2 作为桌面宿主，React + TypeScript 作为前端，Rust 作为核心引擎。

## 备选方案

- Electron + React + Node.js
- Tauri 2 + React + Rust
- 原生 Qt/GTK + Rust
- 纯 Web 应用 + 后端服务

## 原因

- Tauri 安装包体积远小于 Electron
- Rust 性能适合索引、搜索和进程监管等核心任务
- Tauri 2 支持 Capability 细粒度权限控制
- React 生态成熟，适合复杂 UI
- Rust Core 不依赖 Tauri，可演进为独立 Daemon

## 后果

- 需要维护 Rust 和 TypeScript 两套代码
- Tauri IPC 需要类型同步方案
- Windows 是第一优先平台
- WebView 兼容性需要关注

## 以后何时重新评估

- 当 Tauri 生态出现重大安全或性能问题时
- 当需要支持移动端（iOS/Android）时，可能需要评估 Flutter 或其他方案
- 当团队规模显著增大且需要更统一的技术栈时
- 当 Rust Core 需要独立部署为服务端时，Tauri 层可替换为纯 Web 前端
