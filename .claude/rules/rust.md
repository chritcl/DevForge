---
paths:
  - "crates/**/*.rs"
  - "apps/desktop/src-tauri/**/*.rs"
---

# Rust 代码规则

## Unsafe Rust

- Safe Rust 是默认选择。
- Domain、Application、Storage、Search、AI、Connector 和面向前端的 crate 必须使用 `#![forbid(unsafe_code)]`，除非有已批准的 ADR 另行说明。
- `unsafe` 代码只允许在窄范围的 Platform、FFI 或性能关键适配器模块中。
- 不得仅为避免克隆、借用复杂性或普通边界检查而引入 `unsafe`。
- 每个 `unsafe` 块必须：
  - 包含尽可能少的操作；
  - 在紧前方有 `// SAFETY:` 注释；
  - 说明所依赖的每个不变量；
  - 说明谁建立每个不变量；
  - 解释不变量在块的持续时间内为何仍然有效。
- `unsafe` 模块必须向应用的其余部分暴露安全 API。
- 不允许裸指针、unsafe 类型或 FFI 跨越 Application 或 Domain 边界。
- 启用 `unsafe_op_in_unsafe_fn`。
- `unsafe fn` 内部必须对每个 unsafe 操作使用显式 `unsafe {}` 块。
- 公共 unsafe API 需要 Rustdoc `# Safety` 章节。
- 修改 unsafe 代码需要聚焦测试和专项安全审查。
- 不得抑制 unsafe 相关 lint，除非记录了该 lint 在此位置不正确的原因。

## 宏使用

- 优先使用函数、Trait、泛型、enum、builder 和 derive 实现，避免自定义宏。
- 不得使用宏隐藏业务逻辑、状态转换、权限检查、数据库事务、异步控制流或错误处理。
- 不得创建自定义 DSL，除非普通 Rust 语法会使调用方明显更不清晰。
- 声明式宏可接受的场景：
  - 重复声明；
  - 静态查找结构；
  - 编译时测试用例生成；
  - 无法合理用函数或泛型表达的 API。
- 过程宏需要明确的架构审批，且必须位于专用 proc-macro crate 中。
- 宏输入语法应类似于宏产生的 Rust 语法。
- 导出的宏必须使用 `$crate` 引用定义 crate。
- 宏不得：执行隐藏 I/O、意外读取环境变量、创建隐藏全局状态、静默吞掉错误、生成未明确请求的公共项。
- 复杂宏需要展开测试和编译失败测试，推荐使用 `trybuild`。
- 添加宏之前，必须记录为什么函数、Trait、derive 或 builder 不够用。

## 错误处理

- 跨 crate、Trait、IPC 或 Application 边界的错误使用 `thiserror`。
- `anyhow` 只允许用于二进制入口、启动编排、迁移工具和诊断程序。
- 公共 Trait 和 Application API 不得返回 `anyhow::Error`。
- Domain 错误必须描述业务失败，不得暴露 SQL、HTTP、Tauri、操作系统或 Provider 特定的错误类型。
- 基础设施错误可以包装原始 source error。
- 在跨越抽象边界时添加上下文，而不是在每个 `?` 处添加。
- 不得将错误转换为字符串，直到最终展示或日志边界。
- 不得通过字符串匹配判断错误类型。
- 当 typed error 可以表达契约时，不在 Application API 中使用 `Box<dyn Error>`。
- 错误变体必须可操作且语义上可区分。
- 可重试、用户可修正、安全拒绝和永久失败必须保持可区分。
- 不得在每一层都记录并返回同一个错误。在任务、Command、IPC 或进程边界记录一次。

## 异步与并发

- 不得在 runtime、scheduler、supervisor 或明确批准的并发模块之外调用 `tokio::spawn`。
- 每个长期任务必须有：稳定的任务标识符、所有者、取消支持、超时行为、结构化进度、定义的关闭策略。
- 优先使用结构化任务所有权，而非 detached task。
- 不得在 `.await` 期间持有 mutex 或读写锁守卫。
- 优先使用消息传递，而非共享可变状态。
- Channel 必须有界，除非 ADR 批准了无界 Channel。
- 每个高吞吐量流必须定义背压行为。
- CPU 密集型解析、哈希、压缩、Git 和同步文件系统操作不得阻塞 Tokio worker 线程，必须使用 `spawn_blocking` 或专用 worker pool。
- 每个外部操作需要显式超时：HTTP 请求、模型请求、连接器同步、进程执行、LSP 初始化、插件调用。
- 取消必须从父任务传播到子任务。
- 被取消的副作用操作必须报告其完成状态：已完成、部分完成、未开始。
- 避免 `Arc<Mutex<T>>` 作为默认架构，引入共享可变状态前必须说明原因。
- 不得仅为未来灵活性而使用 async trait，当所有当前实现都是同步的时。
- 不得重试有副作用的操作，除非有显式幂等键或记录的恢复策略。

## API 设计

- 在实际可行的情况下，将无效状态建模为不可表示的类型。
- 对标识符、路径、修订、哈希和审批 token 使用 newtype。
- 不得在 Application 边界上将不相关的标识符作为裸 `String`、`Uuid` 或整数值传递。
- 优先使用 enum 而非布尔参数组合，避免多布尔参数函数。
- 当函数需要多个可选设置时，使用 builder 或配置结构体。
- 优先借用而非克隆，但不得仅为消除廉价克隆而引入复杂的生命周期设计。
- 不得在 Domain 或 Application API 中暴露基础设施类型。
- 可见性尽可能窄：私有 → `pub(super)` → `pub(crate)` → `pub`。
- 避免只有一个实现的 Trait 抽象，除非边界用于架构分离、测试、平台替换或 Provider 替换。
- 不得在至少两个具体用例展示共享行为之前创建泛型抽象。
- 对 Domain 状态机优先使用穷举匹配，不得仅为消除编译器错误而添加通配符匹配臂。
- 不得仅为理论性能而引入生命周期、泛型、Trait 或零拷贝抽象；降低可维护性的复杂性需要性能分析证据。

## Rust 文档

- 公共 Application 契约需要 Rustdoc。
- 公共可失败函数记录有意义的 `# Errors` 条件。
- 可能故意 panic 的函数记录 `# Panics`。
- Unsafe 函数和 Trait 记录 `# Safety`。
- 文档解释不变量和行为，不逐行解释实现。
- Rustdoc 示例在实际可行时必须可编译，使用 `Result` 和 `?`，不使用 `unwrap()`。

## Panic 策略

- Panic 保留给违反的内部不变量和程序员错误。
- 用户输入、文件内容、网络响应、模型输出、插件输出和数据库内容不得被假定为有效。
- 不得在生产路径中使用 `panic!`、`todo!`、`unimplemented!`、`unreachable!`、`unwrap()` 或 `expect()`，除非有明确的不变量证明。
- `unreachable!()` 不是正确建模状态的替代品。
- 测试中可在断言上下文中使用 `unwrap()` 提高可读性。

## Rust 测试默认

- 新行为需要聚焦测试。
- 每个 bug 修复始于修复前会失败的回归测试。
- Domain 状态转换需要显式转换测试。
- 优先在私有实现旁写单元测试，对公共契约写集成测试。
