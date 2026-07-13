---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: editor-m1-plugin-provider-lookup
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/02
resolved_at: 2026-07-11
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/package_feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_manifest/completion.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/features.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/native_plugins.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/optional_features.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked tests::host::manager::minimal_host_contract::optional_features::editor_manager_plugin_status_lists_owner_optional_feature_dependencies -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::host::manager::minimal_host_contract::native_plugins::native_aware_catalog_enables_external_feature_extension_provider -- --exact --test-threads=1 --nocapture
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

- 根因：普通 owner package 的 feature definition 丢失显式 `provider_package_id`，随后 completed project manifest 又没有把外部 provider 投影为 canonical plugin selection；native provider 路径还错误依赖只含普通 registrations 的 manifest 视图。
- 架构修复：definition key、completed selection、dependency enablement 与 native-aware lookup 统一使用准确的 `feature_id@provider_package_id` 身份；删除 feature-id fallback 和重复 provider 投影，不在 Editor 调用点增加特例。
- 验证：2026-07-11 使用 11:19 且晚于相关源码的 Editor 2949-test binary，builtin external-provider 与 native FeatureExtension 两个 fully-qualified exact 均为 1 passed / 0 failed（各 0.07s）。
- 回传：本 provider lookup 故障已修复并回迁 Editor 01；Editor M1 可继续其余完整门禁，但本记录不据此声明 Editor M1 全量通过。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 / Editor M1 | Definition provider-key 最低层修复 | `下层已修-上层仍未通过` | 2026-07-11 | 当前 `package_feature_definitions(...)` 已保存普通 owner package 的显式 `provider_package_id`，并新增 external-provider/owner-provider 两个下层测试；重新链接的 2928-test editor binary 不再报告“feature 未声明”，证明 definition lookup 首因已被移除。 |
| M3 / Editor M1 | External provider selection 依赖启用闭环 | `已修复-上层精确回归通过` | 2026-07-11 | 05:57 的中间 binary 执行 exact 仍报 `plugin sound_timeline_animation_track is not registered in builtin plugin catalogs`，据此补齐第二层：`complete_project_feature_selections(...)` 把外部 provider 以 disabled `ProjectPluginSelection` 投影进 completed manifest；`enable_feature_dependency_tree(...)` 再显式启用不同于 owner 的 `provider_package_id` selection，并准确计入 `enabled_dependency_plugins`，没有削弱 `feature_status(...)`。当前 06:17 binary 执行 fully-qualified exact 为 1 passed / 0 failed（0.06s），新断言确认 `sound`、`animation`、`sound_timeline_animation_track` 均启用。1404 行测试 owner 已拆为 1087 行父 owner 与 `minimal_host_contract/{core_contract,optional_features}.rs`；scoped rustfmt/diff-check 通过，未增加 provider fallback、别名或旧 catalog 兼容路径。 |
| M3 / Editor M1 | 架构审查硬化：单一投影、exact lookup、native provider | `实现完成-待当前源码Cargo验证` | 2026-07-11 | 独立审查确认并修复三处未闭环：删除先执行且硬编码 `LibraryEmbed` 的 `project_manifest/feature_provider_selections.rs`，external provider 仅由 `complete_project_feature_selections(...)` 投影；删除 feature-id 全 catalog 唯一回退，lookup 只接受准确 `feature_id@provider_package_id`；native-aware catalog 同时合并 native package 与 feature registrations，使 `FeatureExtension` 不再丢失。新增 NativeDynamic packaging、无 provider fallback、native external feature-extension 三类回归。`minimal_host_contract.rs` 进一步把 native discovery/export 迁至 `native_plugins.rs`，parent/native/core/optional 为 490/619/46/258 行。rustfmt、diff-check、退役符号扫描通过；完整 Cargo 仍在执行旧 06:17 binary，故本行不冒充当前源码通过。 |
| M3 / Editor M1 | Native FeatureExtension canonical lookup follow-up | `已修复-当前源码精确回归通过` | 2026-07-11 | 当前源码首轮 native FeatureExtension exact 完成 15m27s 编译后为 0/1（0.04s），报 `feature native_owner.timeline is not registered under plugin native_owner`；Editor 因此改为通过 `RuntimePluginCatalog::feature_manifest_for_selection(...)` 按 completed selection 的准确 provider identity 查询 canonical definition map，不再遍历只含普通 registrations 的 `package_manifests().optional_features`。第二轮编译 13m38s 后仍为 0/1（0.05s），实际 `enabled_dependency_plugins=["native_owner"]`，缺少 `native_owner_extension`；继续向下确认 definition 的 external provider identity 没有投影回返回 manifest。canonical 查询出口补回该准确 provider 后，第三轮当前源码编译 16m49s，native exact 为 1/1（0.13s）；同一 binary 的 builtin provider exact 亦为 1/1（0.06s）。回归断言 provider 被显式启用、报告包含 `native_owner_extension`、packaging 为 `NativeDynamic` 且 runtime crate 准确；无旧投影、feature-id fallback 或 Editor 特例。 |
| M3 / Editor M1 | Fixed handoff 独立复验 | `已修复-复验通过` | 2026-07-11 | 使用 06:17 且晚于相关源码的 Editor test binary 执行 fully-qualified exact，结果为 1 passed / 0 failed / 2927 filtered out，耗时 0.12s。 |
| M3 / Editor M1 | 2026-07-12 canonical fixed handoff 末次复验 | `已修复-builtin与native provider 2/2通过` | 2026-07-12 | Windows 托管 Editor binary（21:47）执行 `editor_manager_plugin_status_lists_owner_optional_feature_dependencies` 与 `native_aware_catalog_enables_external_feature_extension_provider` 两个 fully-qualified exact，均为 1/1（0.16s、0.09s，3,107 filtered）。definition、completed selection、dependency enablement 与 native-aware lookup 继续只接受准确 `feature_id@provider_package_id`；未恢复 feature-id fallback、别名键、重复 catalog truth 或 Editor 特例。 |
| M3 / Editor M1 | 2026-07-13 current Editor binary 独立复验 | `已修复-current-builtin与native-provider-2-of-2-green` | 2026-07-13 | 复用当天 15 时后生成且晚于全部 provider 相关源码的 Editor test binary，直接执行两个 fully-qualified exact：builtin external-provider 为 1 passed / 0 failed / 3,148 filtered（0.31s），native FeatureExtension external-provider 为 1 passed / 0 failed / 3,148 filtered（0.07s）。当前 `definition_for_selection(...)` 仍只以 completed selection 的准确 `feature_id@provider_package_id` 键查找，缺省 provider 不回退到唯一外部定义；Editor 仍通过 canonical `feature_manifest_for_selection(...)` 取 definition，不含 feature-id fallback、alias、重复投影或调用点特例。 |
| M3 / Editor M1 | Frameworks platform owner hard-cut 后复验 | `已修复-current-builtin与native-provider-2-of-2-green` | 2026-07-13 | Frameworks05 将 `RuntimeTargetMode` 硬切到 `core::framework::platform` 后，当前 Editor M1 native-provider 测试仍有 11 处旧 `builtin::RuntimeTargetMode`，使完整 lib-test 在编译阶段 E0433。测试消费端统一改用 canonical platform owner，没有恢复 builtin re-export、alias 或 compat shim。Windows 受管当前源码 `zircon_editor --lib --no-run` exit 0；3153-test 新二进制执行 native external FeatureExtension 与 builtin unique-provider 两个 fully-qualified exact 均为 `1 passed / 0 failed`（0.04s、0.03s）。同一二进制继续通过 Blend Space `13/0/1` 和 ZUI governance `74/74`。 |
