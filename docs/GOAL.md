# DevForge v0.1 Goal

## 产品目标

交付一个真正可使用、可验证的 v0.1 本地代码知识工作台。

## 用户必须能够

1. 创建、编辑、归档、恢复和删除工作区。
2. 添加多个本地 Git 仓库或普通目录。
3. 安全扫描目录，文件系统根路径必须由后端根据 Source 获取，不能由前端指定。
4. 浏览任意深度的文件树。
5. 使用只读查看器打开代码、文本和 Markdown。
6. 对工作区全部数据源建立持久化全文索引。
7. 执行关键词搜索，并从结果跳转到正确文件和位置。
8. 重启后恢复工作区、数据源、标签和索引。
9. 移除 Source 或工作区时只删除 DevForge 元数据，不删除本地文件。
10. 在 Windows Tauri 应用中完成核心流程验证。

## 非目标

本版本不实现：

- 向量搜索 / Embedding / 语义搜索
- AI 问答 / RAG / Agent
- 代码修改 / Shell 执行
- GitHub/GitLab Connector
- 插件系统 / 团队协作
- 云同步 / 自动更新 / 代码签名

## 文档权威顺序

```
docs/GOAL.md
docs/product/**
docs/architecture/**
docs/adr/**
docs/phases/**
docs/STATE.json
docs/implementation-status.md
```

文档冲突时根据本 Goal 裁决。

## 实施阶段

| 阶段 | 名称 | 状态 |
|------|------|------|
| Phase A | 基线收敛 | DONE |
| Phase B | 文件查看 | DONE |
| Phase C | 基础全文索引 | DONE |
| Phase D | 关键词搜索 | DONE |
| Phase E | v0.1 收口 | DONE |

## 阶段退出条件

### Phase A：基线收敛

- 修复所有 P0/P1 阻塞问题
- 文档与代码一致
- 所有现有测试通过

### Phase B：文件查看

- Monaco 只读代码查看器（语法高亮）
- Markdown 安全渲染（禁止脚本和不安全 HTML）
- 敏感和二进制文件不读取正文
- 大文件限制保留

### Phase C：基础全文索引

- 首次全量索引
- 新增/修改/删除文件的增量更新
- Source 删除和 Workspace 删除的级联清理
- 重启恢复索引状态
- 索引状态和错误查询

### Phase D：关键词搜索

- 搜索全部 Source
- 返回文件名、路径、摘要、位置和相关度
- 点击结果打开文件并跳转到命中位置
- 空查询不搜索
- 索引未完成时显示明确状态
- 删除文件或 Source 后不出现幽灵结果

### Phase E：v0.1 收口

完成真实端到端流程验证：

```
创建工作区
→ 添加两个 Source
→ 扫描和索引
→ 浏览深层文件
→ 查看代码和 Markdown
→ 搜索路径及正文
→ 跳转到结果
→ 重启恢复
→ 移除 Source
→ 标签和搜索结果同步消失
→ 删除工作区
→ 本地文件仍存在
```
