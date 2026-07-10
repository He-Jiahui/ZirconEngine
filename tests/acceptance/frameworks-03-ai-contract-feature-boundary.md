---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/ai
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_plugins/ai/runtime/Cargo.toml
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/asset/runtime_asset_path/diagnostics_enabled.rs
  - zircon_runtime/src/asset/runtime_asset_path/diagnostics_disabled.rs
  - zircon_plugins/ai/runtime/Cargo.toml
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-10 frameworks 基础架构新版硬切换目标
tests:
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary tools.tests.test_frameworks_03_server_feature_boundary
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features ai-contracts --locked --offline --jobs 1
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features ai-contracts,diagnostic-log --locked --offline --jobs 1
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features target-server --locked --offline --jobs 1
  - cargo +nightly check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --lib --locked --offline --jobs 1
doc_type: acceptance-evidence
status: passed
---

# Frameworks 03 AI contract feature boundary 验收证据

## 范围

本记录只覆盖 Frameworks 03 M1 的 `ai-contracts` 独立门控切片。Net、Physics、Sound 契约域，完整逐域矩阵，Runtime/App 全测试门与 M2 profile/CI 单源仍未完成。

## 红绿前沿

- RED：首次 `ai-contracts` 单开编译在 `asset/runtime_asset_path.rs` 失败，E0432 指向 foundational asset 无条件导入可选 `diagnostic_log`；AI contract 自身尚未产生错误。
- 最低层修复：路径解析业务改为调用 compile-time diagnostics adapter；父模块只在声明处选择 enabled/disabled owner。启用版本保持原 `DiagnosticLogLevel::Verbose`、scope 和消息，禁用版本不引用 diagnostic-log，也没有运行时 fallback 或 feature alias。
- GREEN：AI contract/static 边界守卫 4/4，连同 server feature 守卫共 8/8；scoped nightly rustfmt 与 `git diff --check` 通过。

## Cargo 证据

- Runtime `ai-contracts` 单开：WSL nightly locked/offline check 通过，7m25s。
- Runtime `ai-contracts,diagnostic-log`：WSL nightly locked/offline check 通过，3m47s，覆盖 enabled adapter。
- Runtime `target-server`：WSL nightly locked/offline check 通过，3m47s，证明 Server 不启用 AI 仍可编译。
- AI runtime plugin：WSL nightly locked/offline check 通过，5m17s，证明插件 manifest 的显式 `ai-contracts` 请求可实际编译。

四条子命令日志均以 Cargo `Finished` 结束，位于 `/tmp/frameworks03-{ai-contract,ai-diagnostic,server-no-ai,ai-plugin}-check.log`。串联命令的外层 20 分钟工具上限在末段触发，AI plugin 子进程随后完成；因此不把外层超时写成总链通过，只分别采信四条完整子命令日志。

## 硬切结果

- Runtime/App 提供同名 `ai-contracts`；Client/Editor 预设默认包含，Server 不隐式包含。
- `core/framework::ai` 与 `core/manager` 的 AI trait、service name、holder 和 resolver 同门裁剪。
- AI plugin 直接声明 `zircon_runtime` 的 `ai-contracts` feature，不依赖其他 crate 的 Cargo feature 合并。
- 不提供旧 feature 名、兼容 re-export、placeholder manager 或运行时补偿路径。

## 当前判定

Frameworks 03 M1 AI contract 切片完成。M1 本身仍为进行中，下一步从 Net 的 ZRPack DTO 所有权硬迁移开始，随后完成 Net/Physics/Sound 契约独立门控。
