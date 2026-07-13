---
related_code:
  - zircon_plugins/plugin_sdk/src
  - zircon_plugins/gltf_importer/runtime/src
  - zircon_plugins/gltf_importer/dist/src
  - zircon_plugins/native_dynamic_fixture/native/src
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/plugin/native_plugin_loader
  - tools/audit_plugin_structure.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
  - docs/plans/zircon_runtime/frameworks/development-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/audit_plugin_structure.py --json --repo-root E:\Git\ZirconEngine
---

# Frameworks 04：Plugin DX 当前基线产出记录

> 本文件归档 `04-plugin-dx-and-sdk-toolchain.md` 的当前实现审计与后续里程碑证据；计划定义仍由父计划持有。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M0 当前基线 | M1–M4 计划契约与当前实现逐项对照 | `frameworks_04_m0_current_baseline_audited_m1_m3_partial_m2_missing_m4_core_partial` | 2026-07-13 | 当前源码扫描确认 `declare_plugin!`、统一公开 `PluginLoadError` 与 `cargo zircon plugin` 调用均为 0。M1 已有 `RuntimePluginDeclaration`、manifest builders、`runtime_plugin_exports!` 与 native dist ABI 宏，插件结构审计 exit 0、37/37 manifest 存在、schema violation 0、generated-header violation 0；但 36 份所谓 generated manifest 目前只有 header/parity 守卫，没有计划要求的 Rust 单源生成命令，`native_dynamic_fixture` 仍被 catalog 测试明确锁定为唯一手写 manifest，因此 M1 不完成。M2 的 `cargo-zircon` crate、new/check/validate 命令及 CI 接入不存在。M3 loader 已拆出 manifest/ABI/behavior/registration/live-host 多个内部 typed error owner，但尚无计划规定的统一 `PluginLoadError` 类型树和公开阶段/期望/实际/修复提示合同。M4 已有 fixture `save_state`/`restore_state`/`unload`、live-host snapshot/rollback/bridge lifecycle 与故障回归；源码未发现 native 文件 watcher 或 dev-profile 自动重载入口，且 glTF importer dist 仍是 stateless callbacks，故仅记核心部分落地。此审计不把插件结构审计的“classified-and-clear”外推为 Frameworks 04 M1–M4 完成。 |
