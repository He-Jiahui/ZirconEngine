---
related_code:
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/lib.rs
  - tools/cargo-zircon/src/plugin
  - tools/cargo-zircon/src/product_receipt_cli
implementation_files:
  - tools/cargo-zircon/src
plan_sources:
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
tests:
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/plugin
  - tools/cargo-zircon/src/build
doc_type: cli-reference
---

# `cargo-zircon` CLI 与库 API

`cargo-zircon` 是 Cargo 子命令式工具：入口先剥离可选的 `zircon` 前缀，再路由 `plugin`、`validation-batch` 和 `product-receipt`。二进制 `main::run` 不属于库 API；可复用逻辑在 `cargo_zircon` library crate 的 plugin/build 模块。

## 路由与退出码

| 路由 | 成功 | 失败/漂移 |
| --- | --- | --- |
| `cargo zircon plugin ...` | 0 | 参数 2；诊断 4；manifest drift 3 |
| `cargo-zircon validation-batch <script>` | 透传脚本码 | 参数错误 2 |
| `cargo-zircon product-receipt ...` | 0 | 参数/执行错误通常 2 |

`validation-batch` 只接受文件名以 `zircon-validation-` 开头、扩展名为 `.ps1` 的单个脚本。若设置 `CARGO_TARGET_DIR`，工具把它投影到 `validation-batch` 子目录，避免污染默认 target。

## 插件命令

```powershell
cargo zircon plugin new weather --kind system
cargo zircon plugin new importer_png --kind importer --native
cargo zircon plugin sync-manifest [plugin-id] --root E:\Git\ZirconEngine
cargo zircon plugin check-manifest [plugin-id] --root E:\Git\ZirconEngine
cargo zircon plugin check --root E:\Git\ZirconEngine --artifact-root E:\artifacts
cargo zircon plugin validate zircon_plugins/weather
cargo zircon plugin validate zircon_plugins/weather --artifact E:\artifacts\weather.dll
```

`--kind` 只接受 `importer`、`system`、`editor`。`new` 的 id 必须通过包名校验；脚手架失败会回滚已创建目录、Cargo member、catalog 和 feature wiring。`--native` 额外生成 dist member，但 native 包仍需要明确 runtime/editor owner。

## 库 API

```rust
use cargo_zircon::plugin::scaffold::{
    scaffold_plugin, NewPluginOptions, PluginKind,
};

let report = scaffold_plugin(&NewPluginOptions {
    repo_root: repo,
    id: "weather",
    kind: PluginKind::System,
    native: false,
})?;
assert!(!report.created_paths.is_empty());
```

manifest 同步使用 `synchronize_workspace_manifests(root, selector, mode)`；`SyncMode::Write` 写入投影，`SyncMode::Check` 只检查，返回 `SyncOutcome::Drift` 时 CLI 退出 3。workspace 检查使用 `check_plugin_workspace_with_artifact_root`，单包校验使用 `validate_plugin_manifest`，可选动态库校验使用 `validate_native_artifact`。

## Manifest 单一来源

Rust `declare_plugin!` 是 ID、目标、平台、capability、maturity、packaging 和 entry 的权威来源。带 `@generated from Rust PluginDeclaration` 的 `plugin.toml` 不应手改。推荐顺序是：改 Rust 声明 -> sync -> check -> validate -> 构建 native artifact。

```text
declare_plugin!
  -> syn projection
  -> plugin.toml
  -> workspace/catalog wiring
  -> linked/native validation
```

## 产品 Receipt 子命令

```powershell
cargo zircon product-receipt build --request request.json --output draft.json
cargo zircon product-receipt build-batch --request batch.json --output draft-batch.json
cargo zircon product-receipt issue --closure closure.json --private-key signer.pk8 `
  --signer-id ci --created-utc 2026-09-09T00:00:00Z --output receipt.json
cargo zircon product-receipt verify --receipt receipt.json `
  --trust-registry trust.json --artifact-root artifacts
```

draft、draft-batch、receipt 和 receipt-batch 的 verify 必须明确 trust registry 与 artifact root。签名命令不应从 shell 历史读取私钥内容；只传受保护文件路径。

## CI 推荐 lane

```powershell
cargo zircon plugin check --root $repo --artifact-root $artifacts
cargo zircon plugin check-manifest --root $repo
cargo zircon plugin validate $plugin --artifact $artifact
cargo zircon validation-batch tools/zircon-validation-runtime74-batch.ps1
cargo zircon product-receipt verify --receipt $receipt --trust-registry $trust --artifact-root $artifacts
```

遇到 drift 先执行 sync 并 review diff；不要在 CI 中自动覆盖未审阅的 Rust 声明。构建与验证应使用 coordinator 规定的目标目录和固定 toolchain。

## 诊断格式

插件检查诊断包含 `code`、`message`、`hint`。脚本可依赖 code 做分类，但不要依赖完整自然语言。参数解析错误输出 usage 并返回 2；unknown plugin kind 会明确提示三个可用值。

## 与 Unreal/Godot 的对照

Unreal AutomationTool 将项目构建、cook、package 和 deploy 统一进命令行；Godot 的 editor/import 工具更接近按项目执行。`cargo-zircon` 只扩展 Cargo 不具备的插件清单、native artifact 和 receipt 协议，把通用 Rust 构建仍交给 Cargo。

## 常见恢复

1. drift 退出 3：运行 sync，检查生成 diff，再重跑 check。
2. 诊断退出 4：按 code/hint 修复 owner、manifest、artifact root。
3. unknown kind：使用 importer/system/editor，不猜测别名。
4. validation-batch 拒绝脚本：重命名为 `zircon-validation-*.ps1` 并只传一个参数。
5. receipt verify 失败：先核对摘要，再核对信任 registry 和物料化路径。

## 相关页面

- [`cargo-zircon` 总览](../cargo-zircon.md)
- [发布 Receipt](packages-receipts.md)
- [插件清单与 ABI](../../plugins/index.md)
