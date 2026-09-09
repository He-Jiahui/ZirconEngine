---
related_code:
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/projects/install_receipt.rs
  - tools/cargo-zircon/src/product_receipt_cli
  - tools/cargo-zircon/src/build/receipt
implementation_files:
  - zircon_hub/src/projects
  - tools/cargo-zircon/src/build/receipt
plan_sources:
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
tests:
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - tools/cargo-zircon/src/build/receipt
doc_type: api-reference
---

# 项目包、设备安装与产品 Receipt

Hub 的“打包”是可恢复的文件复制动作；`cargo-zircon product-receipt` 是可验证的发布物证明。二者有关联但不是同一个 API：包目录描述复制结果，receipt 描述输入闭包、签名、摘要和物料化结果。

## 项目打包

```rust
pub struct ProjectPackageRequest {
    pub project_name: String,
    pub project_root: PathBuf,
    pub output_root: PathBuf,
    pub created_unix_ms: u64,
}

impl ProjectPackageRequest {
    pub fn new(name: impl Into<String>, project: impl Into<PathBuf>,
               output: impl Into<PathBuf>) -> Self;
}

pub struct ProjectPackageReport {
    pub package_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub files_copied: usize,
}

pub fn package_project(
    request: &ProjectPackageRequest,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<ProjectPackageReport>, HubError>;
```

输出结构固定为 `output_root/packages/<sanitized-name>-<timestamp>/project` 和 `zircon-package.toml`。复制跳过 `.git` 与 `target`；项目根和输出根不能为空，输出根不能位于项目根内部。失败或取消只清理由本次调用创建的目录。

## 设备安装

```rust
pub struct DeviceInstallRequest {
    // fields are source-owned; construct with new
}

impl DeviceInstallRequest {
    pub fn new(package_dir: impl Into<PathBuf>, device_root: impl Into<PathBuf>) -> Self;
}

pub fn install_package_to_device(
    request: &DeviceInstallRequest,
    cancellation: &TaskCancellationToken,
) -> Result<TaskExecutionOutcome<DeviceInstallReport>, HubError>;
```

设备安装必须从 package manifest 和文件 receipt 读取输入，不能直接安装项目源目录。`DeviceInstallReceipt`、`DeviceInstallFileReceipt`、`HubContentDownloadManifest` 和 `HubContentDownloadChunk` 用于记录每个文件的大小、摘要和分块传输事实。

## Receipt 类型

| 类型 | 作用 |
| --- | --- |
| `ProductReceiptDraft` | 未签名的输入闭包草稿 |
| `VerifiedProductReceiptDraftHandoff` | 带 handoff SHA-256 的草稿交换 |
| `ProductReceipt` | 单产品签名证明 |
| `ProductReceiptBatch` | 多产品批次证明 |
| `ProductReceiptClosure` | toolchain、源码和 artifact 闭包 |
| `ProductReceiptError` | 解析、摘要、签名、物料化错误 |

## Rust 验证接口

```rust
let receipt: ProductReceipt = serde_json::from_slice(&bytes)?;
receipt.verify_integrity()?;
receipt.verify_attestation(&trust_registry)?;
receipt.verify_materialization(artifact_root)?;
receipt.verify_attestation_and_materialization(&trust_registry, artifact_root)?;
receipt.write_new_verified(output_path)?;
```

`verify_integrity` 检查结构和摘要；`verify_attestation` 检查 signer 与 trust registry；`verify_materialization` 将 receipt 中的 artifact 相对路径解析到受控根目录。生产流水线通常调用组合方法，再调用 `write_new_verified`。批次类型提供同名批量方法。

## 签名和信任

`Ed25519ProductReceiptSigner::from_pkcs8` 读取 PKCS#8 私钥，`public_key_hex` 返回公开键；`ProductReceiptTrustRegistry::from_json` 读取信任注册表。私钥只应来自受保护的 CI secret，不应放进项目包或 command line 日志。

```text
build request
  -> ProductReceiptDraft
  -> handoff_sha256
  -> signer.issue_verified
  -> trust registry verification
  -> artifact-root materialization check
  -> publish
```

## CLI 形状

```powershell
cargo zircon product-receipt build --request request.json --output draft.json
cargo zircon product-receipt issue-draft --draft draft.json `
  --expected-draft-sha256 <SHA256> --private-key signer.pkcs8 `
  --trust-registry trust.json --signer-id ci --created-utc 2026-09-09T00:00:00Z `
  --output receipt.json
cargo zircon product-receipt verify --receipt receipt.json `
  --trust-registry trust.json --artifact-root artifacts
```

参数缺失、摘要不匹配、签名不受信任或 artifact 不存在都会失败。verify 的 artifact root 必须是明确目录，不能传仓库根或任意用户输入目录。

## 与 Unreal/Godot 的对照

Unreal 的 pak/IoStore pipeline 关注内容分块和安装产物；Zircon 的 package + device receipt 额外记录文件摘要与安装事实。Godot 的 export preset 更偏平台导出配置；Zircon receipt 将 toolchain、artifact 和签名纳入可审计闭包，适合 CI 复现。

## 故障与恢复

1. 输出目录嵌套项目：更换 output root，不删除项目内容。
2. 复制取消：检查 `TaskExecutionOutcome::Cancelled`，清理由本次创建的目录。
3. handoff 摘要不符：重新生成草稿，不能跳过 `expected-draft-sha256`。
4. 签名失败：确认 signer ID 在 registry 中且私钥格式为 PKCS#8。
5. materialization 失败：核对 artifact root 和 receipt 相对路径。

## 发布检查清单

- [ ] 项目包不包含 `.git`、`target` 或 secret。
- [ ] 包 manifest 的 files_copied 与实际内容一致。
- [ ] receipt 完整性、签名和物料化全部验证。
- [ ] trust registry 与 signer 公钥由受控来源提供。
- [ ] 输出、日志和 receipt 被纳入 CI 保存策略。

## 相关页面

- [导出与项目打包](../export-and-packaging.md)
- [`cargo-zircon`](../cargo-zircon.md)
- [资产导入与消费方案](../../recipes/asset-import-and-consume.md)
