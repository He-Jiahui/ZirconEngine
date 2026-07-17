---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-plugin-discovery-recursive-rescan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/collect_manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
tests:
  - unchanged plugin discovery generation-cache test
  - manifest add/change/remove invalidation test
  - symlink-junction cycle and depth-bound test
---

# Plugins01：native plugin discovery 重复递归扫描整棵 root

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP native plugin loader/discovery 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：discovery cache、watcher generation、产品 load manifest 与 live-host refresh 必须共用一套插件目录 authority，不能由 editor status 本地 memoize。

## 失败现象与复现证据

`NativePluginLoader::discover` 每次从 root 递归 `read_dir`，对每个 entry 调用 path metadata 并收集
全部 `plugin.toml`。扫描没有显式 symlink/junction cycle、max depth 或 canonical visited-root policy，
也没有 unchanged generation cache。原加载阶段还会 clone 整个 discovered candidate Vec（含 package
manifests）；该局部冗余复制已由性能审计 Session 用 `mem::take` 消除，但文件系统发现 authority 仍未收敛。

Editor export registration/status 与 live-host loading/refresh 均能调用 discovery。产品已有显式
`plugins/native_plugins.toml`，但通用递归路径仍可能在重复状态刷新中发生。

## 最低共享层根因

插件目录缺少“canonical root + discovery generation + manifest fingerprint”的共享 authority；扫描、
解析、加载选择与 editor 状态查询各自从文件系统重新发现事实。

## 架构修复验收

- 产品启动优先消费 export-time `native_plugins.toml`，不递归扫描任意 root。
- Editor/dev discovery 以 canonical root、file identity/visited set、symlink/junction policy和最大深度有界；
  watcher 或显式 refresh 增 generation，只重读新增/修改/删除 manifest。
- load 阶段继续保持已落地的 candidate ownership take/restore，不回退为深 clone discovered reports。
- unchanged 1k/10k tree refresh 的 enumerate/stat/read/parse count 为 0；单文件变更精确为 O(1)+受影响依赖，
  顺序与 duplicate-package diagnostics 保持确定。

## 禁止临时方案

- 不得只在 editor status 层缓存一个 `NativePluginLoadReport`，让 live host 继续独立重扫。
- 不得盲目并行执行动态库 entry callbacks；并行仅用于安全的 filesystem/stat/parse 阶段且要有预算。

## 修复结果与回传

Open state: `待 Plugins01 建立 manifest generation cache 与产品显式 load-manifest authority`。
