# 安全边界

## 设计原则

1. 安全约束在 Rust 层强制执行，不依赖 Prompt 或 UI 隐藏
2. 默认拒绝，显式允许
3. 所有副作用操作必须经过策略检查和审计
4. 敏感数据不以明文存储或传输

## 路径安全（PathGuard）

所有路径操作必须经过 PathGuard 处理：

```text
输入逻辑路径
   ↓
拒绝空路径和控制字符
   ↓
与工作区根目录拼接
   ↓
规范化路径
   ↓
解析符号链接与 Junction
   ↓
确认最终路径位于允许根目录
   ↓
检查敏感规则
   ↓
执行操作
```

### 防止的攻击向量

- `../` 路径穿越
- 符号链接跳出工作区
- Windows Junction 跳出工作区
- UNC 网络路径（`\\server\share`）
- 设备路径（`\\?\C:\`）
- 大小写差异绕过
- 8.3 短路径绕过
- Alternate Data Streams（`C:\file.txt:secret`）
- Windows 保留名（CON、PRN、NUL、COM1）

### 默认拒绝路径

- 用户 SSH 目录（`~/.ssh`）
- 云厂商凭据目录（`~/.aws`、`~/.azure`）
- 浏览器配置目录
- 系统目录
- DevForge 自身凭据目录
- 工作区外目录

## 敏感文件识别

默认不允许 AI 读取：

```text
.env
*.pem
*.key
id_rsa
id_ed25519
credentials.json
secrets.*
*.pfx
*.p12
```

用户可以把文件加入工作区知识库，但凭据类型文件仍应先脱敏。

## 凭据管理

- 敏感凭据不存 SQLite 明文
- Windows 优先使用系统凭据管理器（Windows Credential Manager）
- 跨平台通过 `SecretStore` Trait 抽象
- SQLite 只保存 `credential_ref`（凭据引用），不保存 Token 明文
- 凭据读取需要经过权限检查和审计

## 云模型数据过滤

调用云模型前必须经过：

```text
上下文候选
   ↓
权限过滤
   ↓
敏感信息扫描
   ↓
Token 预算裁剪
   ↓
用户策略检查
   ↓
模型请求
```

### 敏感信息扫描规则

- API Key 和 Token
- 数据库连接字符串
- SSH 私钥内容
- 环境变量中的敏感值
- 用户主目录路径
- 远程仓库 URL 中的凭据

### 云端披露清单

系统应支持在请求发出前查看"本次将发送给云模型的内容"：

```text
本次模型：Claude / GPT / Gemini
工作区：电商平台
将发送：
├─ 4 个 Rust 文件片段
├─ 1 个 Markdown 文档段落
├─ 2 条 Git Commit
└─ 当前问题

已移除：
├─ .env
├─ API Key
├─ 数据库密码
└─ 用户主目录

预计输入：18,240 tokens
```

用户可以选择：继续发送、删除某条证据、切换本地模型、仅发送摘要、取消请求。

## Tauri Capability 边界

React 前端不直接获得通用 Shell 权限，也不直接调用任意文件系统接口。

### 建议的 Capability 划分

```text
main-window
├─ 允许基础事件
├─ 允许窗口操作
├─ 允许文件选择对话框
└─ 允许 DevForge 自定义 Commands

review-window
├─ 允许读取待审批内容
├─ 允许提交审批决定
└─ 不允许创建其他窗口

floating-view
├─ 只读状态
└─ 不允许执行任务
```

### 禁止的配置

不要给所有窗口配置：

```text
windows: ["*"]
shell:allow-execute
fs:allow-write
```

## 命令执行安全

### 命令模型

默认使用 `program + args[]`，而不是 `shell -c "一整段字符串"`。这样可以减少 Shell 转义差异、命令拼接、管道注入和 `&&` 隐藏额外操作。

### 风险等级

| 等级 | 示例 | 行为 |
|------|------|------|
| Level 0 只读 | 搜索代码、读取文件 | 默认自动允许 |
| Level 1 低风险写入 | 修改普通源码、创建测试 | 需要用户批准 ChangeSet |
| Level 2 普通执行 | cargo check、pnpm lint | 默认需要批准 |
| Level 3 高风险 | 安装依赖、删除文件、创建 Commit | 必须单独明确批准 |
| Level 4 危险操作 | 修改 Git 历史、访问工作区外目录 | 第一版默认拒绝 |
| Level 5 禁止操作 | 上传凭据、读取私钥、绕过审批 | 直接拒绝 |

### 高风险命令模式

```text
rm -rf
Remove-Item -Recurse
del /s
git reset --hard
git clean -fd
git push --force
DROP DATABASE
curl 上传
管理员权限提升
```

### 环境变量策略

命令默认不会继承所有环境变量：

- **基础环境**：PATH、TEMP、SYSTEMROOT 等
- **工作区环境**：用户明确配置的变量
- **敏感环境**：API Token、云凭据等，默认不传递

### 进程监管

Windows 通过 Job Object 管理同一任务产生的一组进程，支持整体终止。

### 真实边界声明

第一版的本地命令执行属于**受控执行与进程隔离，不是强安全沙箱**。普通本地进程仍可能：

- 读取当前用户可访问的其他文件
- 访问网络
- 调用其他系统程序
- 使用系统凭据

UI 不能误导用户显示"完全安全沙箱"，应显示"本地受控执行"。

## Prompt Injection 防护

代码注释、README、Issue、网页快照和第三方文档都属于不可信内容。

### 防护措施

- **指令与数据隔离**：所有检索内容放在明确的数据边界中
- **内容扫描**：检测忽略系统指令、请求读取密钥、请求上传文件、伪造 Tool Call 等
- **来源风险等级**：本地代码 Medium、GitHub Issue High、外部网页 High
- **权限在模型外执行**：即使模型被诱导，仍然不能绕过 Rust Policy Engine

## 审计要求

需要审计的操作包括：

- 云模型请求
- 文件内容发送
- 文件修改
- 命令执行
- Git 写操作
- 凭据读取
- Connector 同步
- 用户批准与拒绝
- 回滚操作

审计记录默认不能由普通 UI 操作删除。技术日志被清理后，关键审计事件仍然存在。

## 日志脱敏

日志记录前通过 `RedactionLayer` 处理：

- API Key、Bearer Token、Cookie
- Authorization Header
- 数据库连接字符串
- SSH 私钥内容
- 环境变量中的敏感值
- 用户主目录路径
- 远程仓库 URL 中的凭据
- Prompt 中的敏感代码

AI 请求默认只记录输入 Token、上下文项数量、内容哈希、模型、耗时，不记录完整 Prompt 和完整项目代码。
