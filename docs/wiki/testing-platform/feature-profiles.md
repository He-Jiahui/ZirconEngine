---
related_code:
  - zircon_app/Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_app/src/entry/entry_profile.rs
  - zircon_app/src/entry/product_host_config/product_role_descriptor.rs
implementation_files:
  - zircon_app/Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_app/src/entry
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/mvp/index.md
tests:
  - zircon_app/src/entry/tests/profile_bootstrap
  - zircon_app/src/entry/tests/product_host_config.rs
  - .github/workflows/profile-feature-contract.yml
doc_type: configuration-reference
---

# Feature 与 Profile 矩阵

Cargo feature 是编译边界，`EntryProfile`/`RuntimeTargetMode` 是产品角色边界，插件 manifest 的 targets/platforms 是分发边界。三者必须同时满足；只打开一个 feature 不能保证功能可以在目标产品中使用。

## 主要目标

| 目标 | 典型 feature | 允许的内容 |
| --- | --- | --- |
| `target-server` | `zircon_app` server + `zircon_runtime` server | 无窗口/无图形的服务端运行时 |
| `target-client` | client、winit、input、可选 graphics | 客户端窗口、输入和渲染 |
| `target-editor-host` | editor-host、editor、runtime gateway | 编辑器宿主与 authoring 工具 |
| `target-client` + viewer | client + shader/PBR viewer | 图形验证和材质查看器 |

实际可用 feature 以当前 `Cargo.toml` 为准。文档中的 `animation`、`navigation`、`script`、`dynamic-api` 等仍受各自 gate 和插件 capability 影响。

## 组合规则

- server 默认不拉入 winit、UI、文本和图形；共享 runtime contract 必须保持 `no-default-features` 可编译。
- editor-host 需要 runtime 与 editor 两侧的契约，不能只编译 `zircon_editor`。
- dynamic API 需要稳定的 `zircon_runtime_interface` DTO；启用 ABI 不等于加载了全部插件。
- 插件 catalog feature 通过 `zircon_app` 传递；native dist 还要匹配平台和 artifact 文件名。

## 契约矩阵

仓库 `validate-matrix.ps1 -RunProfileFeatureContract` 对以下组合做机器化检查：`zircon_app` 的 server、client-platform、editor-host、shader-pbr-viewer，以及 `zircon_runtime` 的 client/editor-host/server。该矩阵验证的是 feature reachability 和编译契约，不代表所有产品运行时行为都通过。

## Rust 侧读取

```rust
use zircon_app::entry::EntryProfile;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;

let profile = EntryProfile::Editor;
let target = RuntimeTargetMode::ClientRuntime;
// 具体组合仍由 product host config 与 plugin catalog 决定。
```

应用代码应通过 profile/catalog resolver 取得能力报告，不要用 `cfg!(feature = ...)` 作为运行时授权判断。
