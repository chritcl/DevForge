# 阶段五：受控 Agent 与变更中心

## 目标

让 AI 从"回答问题"升级为"可以提出并应用受控代码修改"。

## 功能顺序

不要直接实现完整 Agent，建议拆成四个子阶段。

### 5A：Plan 模式

实现：

- 开发计划
- 影响模块
- 预计文件
- 测试计划
- ChangeBudget
- 计划批准

此时仍不写文件。

### 5B：Patch 生成

实现：

- Change Session
- Unified Diff
- 文件级审批
- Patch 基线哈希
- Diff Review 页面
- Patch 导出

此时可以只生成 Patch，不自动应用。

### 5C：隔离应用

实现：

- Git Worktree
- 非 Git 临时副本
- 原子文件写入
- 多文件 Apply Journal
- 冲突处理
- Session Rollback

### 5D：命令执行

实现：

- 结构化命令
- Policy Engine
- 命令审批
- Windows Job Object
- 实时日志
- 超时与取消
- 验证流水线
- Git Commit 审批

## 交付结果

用户可以：

```text
提出修改需求
→ 查看计划
→ 审批计划
→ 查看 Diff
→ 审批文件
→ 在隔离 Worktree 应用
→ 审批测试命令
→ 查看结果
→ 创建本地 Commit
```

## 阶段退出条件

- AI 无法绕过审批修改文件
- Approval Hash 参数变化后会失效
- 工作区外路径操作全部被拦截
- Patch 冲突不会静默覆盖文件
- 命令取消可以终止进程树
- Session Worktree 不污染用户当前分支
- 崩溃后可以识别部分应用状态
- 回滚测试全部通过
- 安全测试直接调用 Rust API 仍然无法绕过策略
