---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: editor-m1-plugin-provider-lookup
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/02
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions.rs
  - zircon_runtime/src/builtin/runtime_modules/sound.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::manager::minimal_host_contract::optional_features::editor_manager_plugin_status_lists_owner_optional_feature_dependencies -- --exact --test-threads=1
---

# Frameworks 02：Editor M1 插件 provider 失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 Windows 全量失败聚类与 V2 公共契约闭环测试阶段
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 交接原因：最低共享故障位于 Frameworks 02 所有的 runtime plugin catalog feature-definition/provider identity 构造，不属于 Editor M1 调用点。

## 失败现象与复现证据

上层精确用例 `editor_manager_plugin_status_lists_owner_optional_feature_dependencies` 为 0/1，报告 `sound.timeline_animation_track` 未由 plugin catalog 声明。完成后的显式选择按 `sound.timeline_animation_track@sound_timeline_animation_track` 查询，但 catalog 中只有错误的 `sound.timeline_animation_track@sound` 定义键。

复现命令：

```text
cargo test -p zircon_editor --lib --locked tests::host::manager::minimal_host_contract::optional_features::editor_manager_plugin_status_lists_owner_optional_feature_dependencies -- --exact --test-threads=1
```

## 最低共享层根因

`package_feature_definitions(...)` 对普通 owner package 忽略 `PluginFeatureBundleManifest.provider_package_id`，错误地用 owner package 构造定义键。lookup 对显式 provider 正确拒绝 fallback，因此错误定义无法匹配完成后的选择并产生 false unknown-feature。

## 架构修复验收

- 增加普通 owner package 的显式外部 provider 与 owner-provider 两类 plugin catalog 下层回归。
- 修复 definition provider-key 的唯一构造所有权，使定义和完成后的选择使用同一 provider identity。
- 精确 Editor 用例通过，并重新运行 Editor M1 声明的完整门禁。

## 禁止临时方案

- 禁止增加忽略 provider 的 fallback、别名键、重复 catalog truth 或旧 catalog 兼容路径。
- 禁止在 Editor 调用点增加 `sound.timeline_animation_track` 特例或削弱 unknown-feature 断言。

## 修复结果与回传

- 状态：`open / 待修复`。
- 当前不声明 Frameworks 02 或 Editor M1 对此门禁通过。
- 修复验收后，修复者必须更新本文件、移动到 `docs/plans/zircon_editor/editor/01/`，并重命名为 `fixed-{resolved_at}-editor-m1-plugin-provider-lookup.md`；Frameworks 02 仅保留相对链接和已修复摘要。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 / Editor M1 | Definition provider-key 最低层修复 | `下层已修-上层仍未通过` | 2026-07-11 | 当前 `package_feature_definitions(...)` 已保存普通 owner package 的显式 `provider_package_id`，并新增 external-provider/owner-provider 两个下层测试；重新链接的 2928-test editor binary 不再报告“feature 未声明”，证明 definition lookup 首因已被移除。 |
| M3 / Editor M1 | External provider selection 依赖启用闭环 | `实现已落地-待当前源码Cargo验证` | 2026-07-11 | 同一重新链接 binary 先以 `missing plugins: sound_timeline_animation_track` 证明第二层缺口。`enable_feature_dependency_tree(...)` 现按 catalog 完成结果，在声明 dependencies 后显式启用不同于 owner 的 `provider_package_id` selection，并把它准确计入 `enabled_dependency_plugins`；没有削弱 `feature_status(...)` 的 provider 检查。1404 行 `minimal_host_contract.rs` 将核心合同与 optional-feature 三项测试拆入 `minimal_host_contract/{core_contract,optional_features}.rs`，父 owner 降至 1087 行；新断言要求 `sound`、`animation`、`sound_timeline_animation_track` 均被启用。scoped rustfmt 与 diff-check 通过；当前源码 exact 尚未重建执行。 |
