---
related_code:
  - zircon_hub/src/service/catalog/mod.rs
  - zircon_hub/src/service/catalog/artifacts.rs
  - zircon_hub/src/service/cloud/mod.rs
  - zircon_hub/src/service/cloud/manifest.rs
  - zircon_hub/src/service/cloud/snapshot.rs
implementation_files:
  - zircon_hub/src/service/catalog
  - zircon_hub/src/service/cloud
plan_sources:
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
tests:
  - zircon_hub/src/service/catalog
  - zircon_hub/src/service/cloud/tests
doc_type: api-reference
---

# Catalog、Cloud Manifest 与快照 API

本页覆盖本地服务中面向 Hub 的 catalog/cloud 公共模型。catalog 负责发布物、查询、授权和许可证；cloud 负责 manifest、blob、snapshot、配额与保留策略。所有入口都在 service authority 内执行，不能从 WebView 直接拼接 HTTP 上传。

## Catalog 模型

```rust
pub struct CatalogPolicy { /* limits and authorization policy */ }
pub struct Publisher { /* publisher authority */ }
pub struct PublishRequest { /* release, artifact, metadata */ }
pub struct Publication { /* published identity and digest */ }
pub struct CatalogQuery { /* filters and pagination */ }
pub struct CatalogPage { /* entries and cursor */ }
pub struct Entitlement { /* principal access */ }
pub struct AcceptLicense { /* artifact/license acceptance */ }

pub fn publish(...) -> Result<Publication, ServiceError>;
pub fn entitlements(...) -> Result<Vec<Entitlement>, ServiceError>;
pub fn authorized_artifact(...) -> Result<..., ServiceError>;
pub fn list(...) -> Result<CatalogPage, ServiceError>;
pub fn accept_license(...) -> Result<..., ServiceError>;
```

实际参数由 `catalog/mod.rs` 定义；这些函数必须经过 principal、scope、license 和 artifact policy 检查。`CatalogPolicy::load(path)` 读取受控策略文件，不能在运行时从客户端字段覆盖额度或允许的 artifact 类型。

## Manifest

```rust
pub const MAX_BLOB_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PROJECT_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
pub const IGNORE_POLICY: &str = "zircon-project-v1";

pub struct Manifest { /* project digest, files, policy */ }
pub struct FileEntry { /* path, size, digest */ }
pub fn digest(value: &str) -> Result<(), ServiceError>;
impl Manifest { pub fn validate(&self) -> Result<(), ServiceError>; }
```

manifest 要求相对、规范化、无 parent traversal 的文件路径；单 blob、单项目和 manifest 都有大小上限。`validate` 必须在存储和快照提交前执行。

## 快照与提交

```rust
pub struct CommitRequest { /* base, manifest, receipt, principal */ }
pub struct Snapshot { /* immutable snapshot identity */ }
pub struct SnapshotReceipt { /* commit evidence */ }
pub enum CommitOutcome { /* committed, conflict, rejected */ }

pub fn authorize_receipt(...) -> Result<SnapshotReceipt, ServiceError>;
pub fn head(...) -> Result<Option<Snapshot>, ServiceError>;
pub fn commit(...) -> Result<CommitOutcome, ServiceError>;
```

提交遵循“授权 receipt -> 读取 head -> 校验 base/manifest -> 原子提交”的顺序。并发 base 不匹配应返回 conflict，由调用者重新读取 head 后决定合并，不能静默覆盖。

```text
principal + entitlement
  -> authorized_artifact
  -> manifest.validate
  -> authorize_receipt
  -> head(base)
  -> commit
  -> SnapshotReceipt
```

## Blob 存储

`BlobStore::load(&CloudConfig)` 打开后端；`upload` 和 `read_blob` 使用 digest/受控路径。单 blob 超过 `MAX_BLOB_BYTES` 必须在读取完整内容前拒绝。下载到 Hub 时使用 `HubContentDownloadManifest`/chunk receipt，断点续传必须再次验证 digest。

## 配额与保留

`usage` 返回 `Usage`；retention 模块提供 `RetentionPolicy`、`RetentionState`、`RetentionRequest`、`RetentionOutcome` 以及 `get`/`update`/`maintain`。GC 只能删除策略允许且没有活动引用的对象；维护报告应保留删除数、跳过数和错误摘要。

## 与 Unreal/Godot 的对照

Unreal 的 derived data/cache 与 artifact service 重视内容哈希；Godot 导入缓存更偏本地项目。Zircon cloud manifest 把项目文件、授权、receipt 和 snapshot 绑定，能够在本地 Hub 与服务端之间重放验证。

## 失败恢复

1. manifest validation：修正路径、大小或 digest 后重新生成 manifest。
2. unauthorized：刷新 account broker entitlement，不绕过服务层。
3. base conflict：读取 `head`，展示差异后显式合并。
4. blob 缺失：根据 digest 重新上传或标记 receipt 不可物料化。
5. quota/retention：查看 Usage 和维护报告，不手删存储目录。

## 相关页面

- [账户与本地服务](account-service.md)
- [项目包与 Receipt](packages-receipts.md)
- [插件清单与分发](../../plugins/index.md)
