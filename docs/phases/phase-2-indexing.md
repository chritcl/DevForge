# 阶段二：文档和代码索引

## 目标

建立完整的本地知识索引流水线。

## 功能范围

### 全文索引

- Tantivy 工作区索引
- 路径 Tokenizer
- 代码 Tokenizer
- 标题、路径、符号和正文权重
- 索引 Manifest
- Building 与 Active 版本切换

### Tree-sitter

第一批语言：

```text
TypeScript / JavaScript
Rust
Python
```

提取：

- 函数
- 类
- 接口
- Struct
- Trait
- Import
- Export
- 注释
- 基础调用关系

### Chunk

实现：

- 代码符号 Chunk
- Markdown 标题 Chunk
- 普通段落 Chunk
- 超大结构二次切分
- Token 计数
- 内容哈希

### 增量索引

- 文件监听
- Debounce
- 内容哈希判断
- 单文件索引任务
- 文件删除
- 文件重命名
- Git 分支变化感知
- 崩溃恢复

### 调度器第一版

实现：

- SQLite Job Queue
- 优先级
- 去重
- 取消
- 重试
- 进度
- Lease
- 崩溃恢复

## 交付结果

导入仓库后，用户可以看到：

```text
发现文件
解析代码
构建全文索引
提取符号
索引完成
```

并可以执行基础关键词搜索。

## 阶段退出条件

- 三种语言能够稳定提取主要符号
- 文件修改后可以增量更新
- 文件删除后不会残留搜索结果
- 应用崩溃后索引任务可以恢复
- 当前 Active Index 不会因重建失败损坏
- 中型仓库首次索引性能达到内部目标
- Fixture 索引回归测试稳定
