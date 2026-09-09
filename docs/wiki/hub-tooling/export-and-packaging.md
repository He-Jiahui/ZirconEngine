---
related_code:
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/projects/install_receipt.rs
  - tools/cargo-zircon/src/build/product_build.rs
  - tools/cargo-zircon/src/build/receipt/mod.rs
  - tools/cargo-zircon/src/build/receipt/product_receipt.rs
implementation_files:
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - tools/cargo-zircon/src/build/product_build
  - tools/cargo-zircon/src/build/receipt
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
tests:
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - tools/cargo-zircon/src/build/receipt
doc_type: workflow-reference
---

# 导出与项目打包

本页区分三种产物：项目包、设备安装包和产品构建 receipt。它们解决的问题不同，不能用“复制目录成功”替代签名或验证。

## 项目包

`ProjectPackageRequest::new` 接受项目名、源根和输出根。`package_project` 创建 `packages/<sanitized-name>-<unix-ms>/project`，复制项目文件并跳过 `.git`、`target`，随后写入 `zircon-package.toml`（源路径、时间、复制数量）。输出根必须在项目根外，任务可通过 `TaskCancellationToken` 中止；取消或失败会删除本次创建的目录。

```rust
use zircon_hub::projects::{package_project, ProjectPackageRequest};
use zircon_hub::state::TaskCancellationToken;

let request = ProjectPackageRequest::new("Demo", project_root, output_root);
let outcome = package_project(&request, &TaskCancellationToken::new(1))?;
```

## 设备安装

`DeviceInstallRequest` 和 `install_package_to_device` 将已生成包物料复制到设备安装目录，并产出 `DeviceInstallReceipt`。receipt 记录每个文件的相对路径、长度、digest 和下载 manifest；重复安装必须通过路径和 digest 校验，不能静默覆盖不匹配的文件。

## 产品构建

`product_build` 将请求解析成 build set，按 target profile 选择 Cargo 包、插件 dist、runtime cdylib 和 editor executable，捕获 stdout/stderr 并写入 draft。构建环境、工具链、profile 和 artifact root 都会进入 canonical input，保证 receipt 可复核。

## Receipt 生命周期

```text
build request
  -> ProductReceiptDraft / DraftBatch
  -> canonical digest + artifact inventory
  -> Ed25519 issue (closure or batch)
  -> trust-registry verification
  -> materialization and platform checks
```

验证器检查：签名者是否在 trust registry、draft digest 是否匹配、artifact 是否存在且 digest 一致、目标平台和 toolchain 是否符合 policy、closure 是否包含所有依赖。验证失败必须停止发布，不得退回到“仅看文件名”的模式。

## Rust API 重点

| 类型/函数 | 责任 |
| --- | --- |
| `ProjectPackageRequest` / `package_project` | 生成可搬运的项目包 |
| `DeviceInstallRequest` / `install_package_to_device` | 安装并写设备 receipt |
| `ProductReceiptDraft` / `ProductReceiptBatch` | 描述待签发物料 |
| `ProductReceiptSigner` / `ProductReceiptVerifier` | Ed25519 签发与 trust registry 验证 |
| `ReceiptMaterialization` | 将已验证 receipt 映射到 artifact root |

这些类型主要供 Hub、CLI 和 CI 使用；Runtime 游戏代码应读取已安装项目/manifest，而不是直接构造 receipt 内部结构。
