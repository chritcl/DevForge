# Phase 1：Local Workspace 实施路线图

## 概述

Phase 1 实现完整的本地工作区闭环。分为 6 个子计划，按依赖顺序执行。

## 子计划列表

| 编号 | 名称 | 依赖 | 状态 |
|------|------|------|------|
| 01 | Domain 和 Storage | 无 | NOT STARTED |
| 02 | Workspace CRUD | 01 | NOT STARTED |
| 03 | Source 和 PathGuard | 01 | NOT STARTED |
| 04 | 文件发现 | 03 | NOT STARTED |
| 05 | 文件树和查看器 | 02, 04 | NOT STARTED |
| 06 | 会话持久化和 E2E | 05 | NOT STARTED |

## 执行顺序

```
01 Domain 和 Storage
   ↓
02 Workspace CRUD ←→ 03 Source 和 PathGuard（可并行）
   ↓
04 文件发现
   ↓
05 文件树和查看器
   ↓
06 会话持久化和 E2E
```

## 验收标准

完成所有子计划后，执行 Phase 1 最终验收。
