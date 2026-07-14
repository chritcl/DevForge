# 阶段六：GitHub 与 GitLab 连接器

## 目标

将开发过程中的历史讨论和远程协作信息加入知识库。

## 第一批功能

### GitHub

- OAuth 或 Fine-grained Token
- 仓库选择
- Commit
- Pull Request
- Review
- Issue
- Comment
- Release

### GitLab

- GitLab.com
- Self-Managed
- OAuth 或 Access Token
- Commit
- Merge Request
- Note
- Issue
- Wiki
- Pipeline 摘要

### 同步能力

- 增量游标
- 内容哈希
- 本地快照
- 限流
- 认证失效
- 定期完整核对
- 手动同步
- 工作区打开时快速同步

### 搜索整合

支持回答：

```text
为什么这里要兼容旧字段？
这个 Bug 以前是否出现过？
PR 中为什么拒绝了这个方案？
这个函数最近是谁修改的？
```

## 暂不实现

- 远程写入
- 自动评论
- 自动合并
- 自动创建 PR
- 本地直接接收公网 Webhook

## 阶段退出条件

- 增量同步不会重复产生文档。
- 认证失效后不会无限重试。
- 本地快照可以离线搜索。
- GitHub/GitLab 数据能与代码引用关联。
- 远程历史不会覆盖当前工作树事实。
- 删除和权限变化能通过定期核对发现。
