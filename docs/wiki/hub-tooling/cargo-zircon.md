---
related_code:
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/lib.rs
  - tools/cargo-zircon/src/plugin/scaffold/mod.rs
  - tools/cargo-zircon/src/plugin/manifest_sync.rs
  - tools/cargo-zircon/src/plugin/check.rs
  - tools/cargo-zircon/src/product_receipt_cli/mod.rs
implementation_files:
  - tools/cargo-zircon/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
tests:
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/plugin
  - tools/cargo-zircon/src/build
doc_type: cli-reference
---

# `cargo-zircon`

`cargo-zircon` 是 Cargo 子命令式工具。入口先剥离可选的 `zircon` 前缀，再路由到插件、验证批次或产品 receipt。命令失败返回非零 `ExitCode`，诊断包含稳定 code、说明和恢复 hint。

## 命令矩阵

| 命令 | 作用 | 成功/漂移语义 |
| --- | --- | --- |
| `cargo zircon plugin new <id> --kind importer\|system\|editor [--native]` | 生成插件目录、Cargo member、catalog 和 app feature wiring | 生成 `ScaffoldReport`；任何一步失败会回滚已写入文件 |
| `cargo zircon plugin sync-manifest [<id>]` | 从 `declare_plugin!` 投影 `plugin.toml` | 更新后 0；无变化 0 |
| `cargo zircon plugin check-manifest [<id>]` | 只检查 projection 与 manifest 是否漂移 | 漂移返回 3 |
| `cargo zircon plugin check [--root <repo>] [--artifact-root <dir>]` | 扫描全部 manifest、workspace member、catalog wiring、native artifact | 有诊断返回 4 |
| `cargo zircon plugin validate <dir\|plugin.toml> [--artifact <file>]` | 校验单个包及可选动态库 | 有诊断返回 4 |
| `cargo-zircon validation-batch <zircon-validation-*.ps1>` | 启动受管 PowerShell 批次 | 透传脚本 exit code |
| `cargo-zircon product-receipt ...` | 构建、签发、验证产品 receipt | 参数/密码学/物料错误返回 2 或子命令错误码 |

## 插件脚手架

`PluginKind` 只有 `Importer`、`System`、`Editor`。脚手架先读取 workspace 版本和 SDK API 版本，再渲染模板，然后原子地更新：

1. `zircon_plugins/Cargo.toml` member；
2. runtime/editor catalog 的 optional dependency 与 feature；
3. `zircon_app` 对应 feature；
4. `plugin.toml` 的 generated projection。

任意写入失败都会删除新目录并恢复原始 Cargo/catalog 文件。`--native` 额外添加 `dist` member；native 包仍必须有 runtime 或 editor owner。

## Manifest authority

`PluginDeclarationProjection::parse` 使用 `syn` 读取 `declare_plugin!`，投影 id、目标、平台、能力、成熟度、打包方式和 entry。`sync-manifest` 是唯一推荐的生成入口；不要手改带有 `@generated from Rust PluginDeclaration` 的 `plugin.toml`。

```powershell
cargo zircon plugin check --root E:\Git\ZirconEngine
cargo zircon plugin sync-manifest zircon_plugin_rendering
cargo zircon plugin validate zircon_plugins/rendering --artifact E:\artifacts\zircon_plugin_rendering.dll
```

## 产品 receipt

receipt 子命令把构建请求规范化为 `ProductReceiptDraft`，对输入和 artifact 做 SHA-256，使用 Ed25519 signer 产生 closure/batch receipt，再用 trust registry 验证签名、版本、artifact digest 和 materialization。`issue-draft` 只签草稿；`issue` 需要完整 closure。验证必须指定 `--artifact-root`，避免把未审计的任意文件当作发布物。

## 与 Cargo 验证的关系

`validation-batch` 只接受文件名形如 `zircon-validation-*.ps1` 的脚本，并将 `CARGO_TARGET_DIR` 投影到受管 `validation-batch` 子目录；它不绕过 coordinator。日常检查应使用 [Session Coordinator](session-coordinator.md) 说明的 `validate-matrix.ps1`，而不是直接在仓库中创建 `target/`。

## Rust 调用接口

```rust
use cargo_zircon::plugin::scaffold::{scaffold_plugin, NewPluginOptions, PluginKind};

let report = scaffold_plugin(&NewPluginOptions {
    repo_root: repo,
    id: "weather",
    kind: PluginKind::System,
    native: false,
})?;
println!("created {} files", report.created_paths.len());
```

库 crate 暴露 `plugin` 和 `build` 模块；CLI 的 `run`、参数解析和 product receipt 子命令实现属于二进制内部，不应从其他 crate 反向依赖。
