---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin
  - zircon_runtime/src/plugin/runtime_plugin/descriptor
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/runtime_plugin/feature_registration_report
  - zircon_runtime/src/plugin/runtime_plugin/registration_report
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/bridge/table.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
reference_sources:
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension_manager.h
tests:
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs::bridge_owner_reload_borrows_registry_exports_without_cloning_the_replacement_batch
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection/tests.rs::feature_capability_projection_borrows_manifest_rows_until_owned_index_insertion
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection/tests.rs::catalog_generation_builds_one_projection_for_all_consumers
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection/tests.rs::feature_resolution_visits_each_feature_and_dependency_once
  - current-source Windows Cargo and F0/F4 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime plugin catalog与registration逐文件性能静态审查（2026-07-22）

## 范围与覆盖

本切片逐文件阅读`runtime_plugin` root **11/11**、`runtime_plugin/**` **2/2**、`descriptor/**` **13/13**、`builtin_catalog/**` **44/44**、`runtime_plugin_catalog/**`生产 **84/84**及`derived_projection/tests.rs`、`feature_registration_report/**` **8/8**、`registration_report/**`生产 **13/13**。合计生产 **175/410**、当前文件 **176/414**；剩余生产235个集中在feature/module/package validation细分目录，另有3个standalone validation tests，继续留在`pending.md`。

这不是动态验收。受管Cargo申请`performance-plugin-catalog-preallocation-20260722`被reservation `bd01cb6fb10240d58471db5eba74e556`（Session `frameworks06-g7-performance-directory-owner-targets-batch37-20260722`）占用，未绕过协调器运行raw Cargo；本切片与GPU无直接执行路径，RenderDoc缺失状态继续由Render17/F2门负责。

## 已确认的性能形状

- `RuntimePluginCatalogProjection`已经按catalog generation构建一次并由`Arc`共享，package/feature/module/provider/capability/bridge索引与O(V+E) ready-set属于正确底座；稳定consumer没有再次重建feature definition map。
- `register`和`register_feature`仍对每一项分别执行全量projection build与全量diagnostics rebuild。一次native discovery或hot reload含N项变更时会形成N次O(P+F+E)，因此PERF-MVP-537要求candidate transaction只发布一代。
- `RuntimePluginCatalog::builtin()`只共享projection `Arc`，返回`Self`仍深cloneregistration、feature report与diagnostics；`package_manifests()`深clone全部manifest。`runtime_extensions_for_project()`又clone/补全project manifest、解析feature dependency并重新合并所有enabled contribution到新registry。PERF-MVP-538要求与PERF-MVP-533共用immutable catalog/project extension generation，禁止给各consumer另建cache。
- feature resolution虽已线性化，但active/pending/status/report仍拥有多份owner/definition/capability String；`FeatureStatus`同时保存有序Vec和membership HashSet的完整String。该剩余所有权放大纳入PERF-MVP-538的compact ID/project plan，不以局部linear scan回退换内存。
- builtin row与feature manifest构造只发生在catalog generation创建阶段；`target_modes.contains`最多三个枚举值，分类阶段的少量`package_id.to_string()`没有证据进入stable frame，不冒充MVP逐帧热点。

## 本轮直接止损

1. **PERF-MVP-535**：`RuntimeExtensionRegistry::interface_exports_owned_by`改为borrowed iterator；bridge restore/reload接收`(&str, &InterfaceExport)`，replacement batch只收集引用并以`iter().copied()`复用于多个owner。最终provider发布仍按affected slot复制必要`Arc`，interface id、完整export和replacement Vec不再重复深clone。
2. **PERF-MVP-536**：feature capability iterator从owned `String`改为`&str`；target projection以borrowed set去重，仅在写入owned index时分配一次，dependency resolution也只在capability首次进入owned available set时分配。

两项均先写源码结构守卫并观察RED，再实现GREEN；scoped `rustfmt --edition 2021`、守卫脚本与`git diff --check`通过。公开顺序、duplicate suppression、bridge slot/generation/report和feature fixed-point结果未改变。

## 参考约束与动态验收

Bevy `App`把plugin identity membership和plugin lifecycle state集中在应用generation，而不是让每个consumer重建插件目录；Godot `GDExtensionManager`以`gdextension_map`持有稳定extension identity并显式执行load/reload/unload状态迁移。这支持Zircon把batch catalog mutation、compiled project selection与extension handles作为同一代immutable状态发布，同时保留现有stable slot、last-good rollback与in-flight `Arc` quiescence。

动态验收须覆盖plugins/features/modules/interfaces/contributions **1/100/10k**、stable/toggle/1% reload与1/8/64 Worlds，记录projection/diagnostic/project-plan/registry build count，rows/edges visited，String/export/manifest/registration clone bytes，alloc/RSS/wall，callback queue delay与old-generation age。PERF-MVP-535/536还需现有bridge lifecycle/stable snapshot、catalog O(V+E)和JSON byte-order回归在current-source binary上通过。Cargo、F0/F4产品trace与上述规模counter完成前，本切片不得进入`review.md`。
