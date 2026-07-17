---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-plugin-catalog-rebuild-and-deep-copy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/native.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/report.rs
tests:
  - editor plugin catalog generation build-count regression
  - plugin-manager recompute package/capability clone-count regression
  - plugin enable-disable/hot-reload ordering parity
---

# Editor12：editor plugin catalog 重建与 owned projection 复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：editor core plugin/catalog 与直接 UI 调用方静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：runtime/editor catalog generation、插件 UI projection 与 hot-reload invalidation 必须由 Editor12 单一管理。

## 失败现象与复现证据

plugin manager 的 `editor_plugin_catalog()` 从 runtime package manifests 重新构造 builtin editor catalog；多个 status/enablement/project/manifest-completion consumer 随后反复调用 `package_manifests()` 与 `capabilities_for_package()`，分别深 clone manifests/capability strings。`from_descriptors` 对每个 descriptor 线性扫描 runtime manifests；`editor_extensions()` 每调用一次都重建 extension/asset registry并 clone 所有 descriptor/contribution。调用链位于插件管理 UI 的 recompute/enablement 路径，当前没有 generation cache 或 build-count 证据。

extension 注册本身也随已注册数量重复工作：每个 plugin 先 clone 完整 `EditorCommandRegistry` 做事务候选，分别五次重扫既有 drawer/menu/component/template/importer ids，重建 available-operation set，并从 builtins 重放全部 prior asset contributions 做冲突验证。批量 bootstrap 因此存在 registrations×existing-catalog 的二次增长候选。

native plugin 管理 UI 的只读 status 路径调用 `NativePluginLoader.load_discovered_all`，会重新递归发现并实际加载 runtime/editor 动态库、执行 descriptor/entry 路径，然后又重建 builtin status 与临时 runtime catalog。native-aware enablement 先 discover 一次取 package，随后 manifest completion 内再次 discover；feature/packaging/target-mode 操作也各自重建 catalog/completed manifest。也就是说，稳定面板刷新和单次 toggle 都可能重复文件系统扫描、manifest parse、foreign library load/callback 与全量 projection。

Retained host 451-file 审查确认这条只读路径位于可见 pane 的 slow recompute：`module_plugins_pane_data` 每次重新解析 project root/`zircon-project.toml`，随后调用 `native_plugin_status_report` 并完整 materialize plugin rows、capabilities/features/diagnostics。visibility gate 只避免隐藏 pane，无法让 unchanged visible pane 命中 generation cache。

## 最低共享层根因

editor plugin catalog 不是由 runtime/editor plugin generation 持有的稳定投影；API 主要返回 owned collections，迫使 UI consumer 重建 catalog 或深 clone manifests/capabilities/extensions。

## 架构修复验收

- runtime/editor plugin generation 只构建一次 immutable ordered editor catalog projection，UI consumers 共享借用/`Arc` rows。
- package id、capability、extension 与 asset contribution 建有序索引；lookup 近 O(1)/O(logN)，不返回全量 owned clone。
- register/enable/disable/hot reload 精确递增 generation；单次 UI recompute build count 为 0（未变）或 1（变更）。
- batch registration 使用一次 staging generation/projection 与一次原子 publish；失败仍不泄漏 commands/views/consumers，不能每 plugin clone 完整 registry。
- 只读 status/report 只消费 live-host/catalog generation，不触发 dynamic library load、entry callback 或文件系统 discovery；explicit refresh 每 generation 至多发现/加载一次。
- 1/100/1000 plugin benchmark 记录 clone bytes、build count 和 recompute p95；manifest/extension/diagnostic 顺序与 lifecycle 语义等价。
- unchanged 可见 Module/Plugin pane 的 project manifest read/parse、discovery/library load/entry callback、catalog/status/row build count 全为 0；显式 refresh 或 lifecycle generation 每代至多一次。

## 禁止临时方案

- 不得在每个 UI panel 各缓存一份 catalog。
- 不得用无序 map 迭代改变 builtin/registration/diagnostic 顺序。
- 不得让借用跨越 catalog generation 更新而悬垂。

## 修复结果与回传

Open state: `待 Editor12 建立 generation-owned immutable catalog projection，并回传插件管理 UI build/clone/scale 证据`。
