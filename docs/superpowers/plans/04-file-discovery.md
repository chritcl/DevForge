# 子计划 04：文件发现

## 目标

实现目录扫描、文件类型识别、敏感文件检测和 ignore 规则支持。

## 精确修改文件

### 新增文件

| 文件 | 职责 |
|------|------|
| crates/devforge-domain/src/document.rs | Document 领域模型（如 01 未创建） |
| crates/devforge-domain/src/sensitivity.rs | 敏感文件识别 |
| crates/devforge-application/src/discovery.rs | 文件发现用例 |
| crates/devforge-storage/src/document_repository.rs | Document Repository |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| crates/devforge-storage/src/lib.rs | 添加 document_repository 模块 |
| crates/devforge-application/src/lib.rs | 添加 discovery 模块 |

## 公共接口签名

### 文件发现用例

```rust
pub struct ScanSource {
    source_repo: Arc<dyn SourceRepository>,
    document_repo: Arc<dyn DocumentRepository>,
}

impl ScanSource {
    pub async fn execute(&self, source_id: &SourceId) -> Result<ScanResult, AppError>;
}

pub struct ScanResult {
    pub added: u32,
    pub updated: u32,
    pub removed: u32,
    pub skipped: u32,
}
```

### 敏感文件识别

```rust
pub fn is_sensitive(path: &Path) -> bool;
pub fn is_binary(path: &Path) -> bool;
pub fn identify_document_kind(path: &Path) -> DocumentKind;
```

### Ignore 规则

```rust
pub struct IgnoreRules {
    gitignore: Option<Gitignore>,
    devforgeignore: Option<Gitignore>,
}

impl IgnoreRules {
    pub fn load(root: &Path) -> Self;
    pub fn is_ignored(&self, path: &Path) -> bool;
}
```

## 依赖关系

- 依赖子计划 01（Domain 和 Storage）
- 依赖子计划 03（Source 和 PathGuard）

## 不能并行的任务

- 必须在 03 完成后执行

## 失败测试

### 文件发现测试

1. 扫描空目录返回 0
2. 扫描包含文件的目录
3. 递归扫描子目录
4. 遵守 .gitignore 规则
5. 遵守 .devforgeignore 规则
6. 忽略 .git 目录
7. 识别二进制文件
8. 识别大文件（>100MB）
9. 重复扫描幂等
10. 文件删除后更新状态

### 敏感文件测试

1. .env 识别为敏感
2. .env.production 识别为敏感
3. *.pem 识别为敏感
4. *.key 识别为敏感
5. 普通文件不识别为敏感

## 最小实现步骤

1. 实现 is_sensitive 函数
2. 实现 is_binary 函数
3. 实现 identify_document_kind 函数
4. 实现 IgnoreRules
5. 实现 ScanSource 用例
6. 实现 DocumentRepository
7. 编写测试

## 精确验证命令

```bash
cargo test -p devforge-domain -- sensitivity
cargo test -p devforge-application -- discovery
cargo test -p devforge-storage -- document
```

## 人工验收步骤

1. 创建工作区
2. 添加包含 .gitignore 的 Git 仓库
3. 等待扫描完成
4. 确认 .gitignore 中的文件未出现
5. 确认 .env 文件标记为敏感
6. 确认二进制文件正确识别

## 独立提交信息

```
feat(discovery): 实现文件发现和敏感文件识别

- 添加 is_sensitive、is_binary 函数
- 实现 IgnoreRules 支持 .gitignore
- 实现 ScanSource 用例
- 实现 DocumentRepository
- 编写测试
```

## 回滚和兼容性风险

- 新增模块，不影响现有功能
- 删除 discovery 模块需要同步清理依赖
