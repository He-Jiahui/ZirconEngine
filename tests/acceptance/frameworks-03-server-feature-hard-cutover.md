---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/behavior.rs
  - zircon_runtime/tests/frameworks_03_server_profile.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/groups.rs
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/plugin/extension_registry
  - zircon_runtime/src/plugin/runtime_plugin
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/groups.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-10 frameworks 基础架构新版硬切换目标
tests:
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - tools/tests/test_runtime_domain_dependency_audit.py
  - tools/tests/test_frameworks_05_asset_ui_boundary.py
  - zircon_runtime/tests/frameworks_03_server_profile.rs
doc_type: acceptance-evidence
status: passed
---

# Frameworks 03 target-server feature 硬切换验收证据

## 范围

本记录覆盖 Frameworks 03 M1 的 server 域实现与重依赖裁剪切片。它不把 core/framework 其余契约域、逐域单开矩阵、完整 Runtime/App 测试门或 M2 profile/CI 单源声明为完成。

## 红绿编译前沿

- RED：首次 server check 暴露 70 个错误；根域模块加门后仍有 38 个错误，定位到动态客户端 API、字体/着色器 importer、builtin render assembly 与 plugin graphics/UI extension slots。
- RED：第一轮声明/装配收口后只剩 5 个 plugin manifest/catalog 调用点，证明最低共享边界已经收敛。
- RED：Runtime 通过后，App server check 暴露 plugin group 无条件引用 GraphicsModule/ScriptModule 两处上层越界；App feature/组装守卫先红后绿。
- GREEN：`cargo +nightly check -p zircon_runtime --lib --no-default-features --features target-server --locked` 在 WSL 独立 target 通过，耗时 3m54s。
- GREEN：`cargo +nightly check -p zircon_runtime --lib --locked` 在同一 WSL target 通过，耗时 6m01s，确认默认客户端能力未回归。
- GREEN：`cargo +nightly check -p zircon_app --no-default-features --features target-server --locked` 通过，耗时 6m42s。
- GREEN：`cargo +nightly check -p zircon_app --locked` 通过，耗时 8m05s。首次运行只因 WSL 缺少 `libudev.pc` 停在 libudev-sys build script；将 `libudev-dev` 无特权解包到 `/tmp` 并仅为该命令设置 pkg-config/library path 后，原样重跑通过，未改系统包或仓库。
- RED（后续支持层复查）：`core-min` lib-test 暴露 server 行为测试无条件引用 Script；继续下钻确认 default/client 编译物选择 `ServerRuntime` 时，`core_modules.rs` 仍仅按 compile feature 装入 `ScriptModule`。
- GREEN：graphics/non-graphics 两条 core-module 组装路径均在 target assembly 边界排除 `ServerRuntime` 的 Script，旧 server 测试期望同步删除。默认客户端完整编译环境下的 `frameworks_03_server_profile` integration test 1/1 通过（测试 0.01s，冷构建 26m52s）；同一独立 target 上 `target-server` lib check 通过（8m36s）。

## 硬切结果

- `target-server` 只组合 `core-min`、`diagnostic-log`、`platform-headless`；不组合 graphics、text、ui、animation、navigation、script、dynamic-api。
- 根模块与 prelude 按域 feature 声明；动态客户端 API 是显式组合，不在 server surface。
- 字体解码/metadata/importer 只随 `text` 编译；Naga shader validation/package/GLSL/SPIR-V importer 只随 `graphics` 编译。
- builtin runtime module assembly 在无 graphics 时没有 render 参数或 provider placeholder，在无 script 时不装配 `ScriptModule`；即使二进制已经编译 Script，运行期选择 `ServerRuntime` 也不会装入它。
- plugin registry、ownership、manifest registration 与 catalog merge 的 graphics/UI 扩展槽按同名 feature 一次裁掉，不保留兼容表或运行时 fallback。
- App 暴露同名域转发 feature，target 组合显式组合这些域；Headless/server plugin group 不再引用 GraphicsModule 或 ScriptModule。

## 验证

- server `cargo tree` 对 `wgpu|winit|taffy|glyphon|naga|swash|fontsdf|woff2-patched` 的筛选输出为空。
- Frameworks 03 server 守卫：5/5 通过；Frameworks 03/05 当前自有静态门合计 24/24 通过。
- `cargo +nightly test -p zircon_runtime --test frameworks_03_server_profile --locked --offline --jobs 1 -- --nocapture --test-threads=1`：1/1 通过。
- `cargo +nightly check -p zircon_runtime --lib --no-default-features --features target-server --locked --offline --jobs 1`：通过（8m36s）。
- touched Rust `rustfmt +nightly --check`：通过。
- scoped `git diff --check`：通过（仅行尾转换提示）。

## 当前判定

`target-server` 域实现与重依赖硬裁剪切片完成。Frameworks 03 M1 仍为进行中；下一切片必须继续 core/framework 契约域 feature 与逐域单开验证，不能从本切片外推整个计划完成。
