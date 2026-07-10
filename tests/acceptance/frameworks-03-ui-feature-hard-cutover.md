---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs
  - zircon_runtime/src/platform/tests/feature_manifest.rs
  - zircon_runtime/src/platform/tests/app_feature_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules/core_spine.rs
  - tools/dev-module-interactive.ps1
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs
  - zircon_runtime/src/platform/tests/feature_manifest.rs
  - zircon_runtime/src/platform/tests/app_feature_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules/core_spine.rs
  - tools/dev-module-interactive.ps1
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-10 frameworks 基础架构新版硬切换目标
tests:
  - zircon_runtime/src/platform/tests/feature_manifest.rs
  - zircon_runtime/src/platform/tests/app_feature_manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs
doc_type: acceptance-evidence
status: validation_in_progress
---

# Frameworks 03 UI Cargo feature 命名硬切换验收证据

## 范围

本记录覆盖 Frameworks 03 M1 的 `plugin-ui` → `ui`、ZR VM backend → `backend-zr-vm`、`jolt` → `backend-jolt` 三个 feature 命名硬切片。旧名已从 Runtime/App/first-party catalog/plugin runtime Cargo surface、生产 cfg、工具和当前非计划文档删除，不提供 alias。animation/navigation/script/diagnostic_log、framework contracts、server 依赖裁剪、profile feature 单源与 CI 矩阵不在本切片完成声明内。

## TDD 红绿证据

- RED：硬切前执行 `cargo +nightly check --manifest-path zircon_runtime/Cargo.toml -p zircon_runtime --lib --no-default-features --features ui --locked --offline`，Cargo 立即报告 `zircon_runtime` 不含 `ui` feature。
- GREEN：Runtime/App locked offline metadata 显示 `ui` feature 存在并正确转发，feature map 中不存在 `plugin-ui`。
- GREEN：`cargo +nightly check --manifest-path zircon_runtime/Cargo.toml -p zircon_runtime --lib --no-default-features --features core-min,ui --locked --offline --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks03-ui-feature` 在 WSL nightly 通过，耗时 11m56s；420 条 warning 为当前工作区既有 warning。
- RED：旧 surface 下 `--features backend-zr-vm` 被 Cargo 拒绝，证明新 backend 命名不存在。
- GREEN：Runtime/App/first-party catalog/ZR VM language plugin 四个 manifest 的 locked offline metadata 均解析 `backend-zr-vm`，旧 backend feature keys 为 0。实际 Runtime check 进入 native binding build script后，因未提供既有外部前置 `ZR_VM_RUST_BINDING_LIB_DIR` 终止；不声明 backend 编译 pass。
- RED/GREEN：旧 surface 下 `backend-jolt` 被 Cargo 拒绝；硬切后 Runtime/physics plugin metadata 解析新 key 且不含旧 `jolt`。Runtime `core-min,backend-jolt` WSL nightly check 通过（6m10s）。

## 硬切结果

- `zircon_runtime` 与 `zircon_app` 只暴露 `ui`；`target-client` / `target-editor-host` 只组合 `ui`。
- Runtime、App、first-party catalog 与 ZR VM language plugin 统一只暴露 `backend-zr-vm`；生产 cfg/诊断/测试 reason 同步新名。
- Runtime 与 physics plugin 的未来 Jolt slot 统一只暴露 `backend-jolt`；backend option 值仍为 `"jolt"`，它是运行时配置值，不是 Cargo feature alias。
- builtin module manifest/loader、profile bootstrap cfg、feature manifest tests 和开发工具全部迁移到 `ui`。
- 生产代码、工具与当前非计划文档中的旧 feature token 扫描为 0；`plugin-ui-component` 是 UI descriptor role，不是 Cargo feature，保持不变。
- 当前用户文档和命令示例已改为新 feature 名，不再教用户调用已删除接口。

## 验证

- Runtime locked/offline metadata：通过。
- App locked/offline metadata：通过。
- Runtime `core-min,ui` WSL nightly check：通过。
- Windows nightly scoped rustfmt：通过。
- scoped `git diff --check`：通过。

## 当前判定

UI、ZR VM backend 与 Jolt backend feature 命名硬切片完成；ZR VM backend 的 native-link 编译证据仍等待外部 import-library 路径。Frameworks 03 M1 仍处于进行中。后续必须继续补齐域级模块门、server dependency tree 纯度和逐域组合验证，不能把命名 hard-cut metadata 或单域编译通过外推为整个 feature 矩阵完成。
