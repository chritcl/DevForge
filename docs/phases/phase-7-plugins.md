# 阶段七：插件、模型与高级运行时

## 目标

开放可控扩展能力，而不破坏核心安全边界。

## 功能范围

### 模型增强

- Gemini
- LM Studio
- Reranker
- 模型路由
- 显式回退链
- Provider 熔断
- Token 和成本中心
- 模型性能对比

### WASM 插件

先完成：

```text
Plugin Manifest
安装与卸载
权限展示
WASM Runtime
资源限制
插件存储
HTTP Host Capability
Connector Plugin
Parser Plugin
Read-only Tool Plugin
```

### 插件 SDK

提供：

- WIT 接口
- 示例插件
- Rust SDK
- TypeScript 或其他语言绑定
- 本地测试工具
- 插件签名工具

### 高级调度

- 电池模式
- 计量网络
- CPU 与内存压力
- GPU 任务
- 模型下载
- 任务依赖可视化
- Dead Letter 管理

## 阶段退出条件

- 插件无法直接读取宿主文件系统。
- 插件无法获得凭据明文。
- 插件权限增加需要重新审批。
- 插件故障不会导致主应用崩溃。
- Provider 回退不会跨越用户隐私策略。
- 插件 SDK 有至少两个真实示例。
