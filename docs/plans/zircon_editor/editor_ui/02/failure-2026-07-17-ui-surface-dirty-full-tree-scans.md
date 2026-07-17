---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ui-surface-dirty-full-tree-scans
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/tree
tests:
  - 10k-node one-dirty-leaf visit-count test
  - dirty aggregate domain union test
  - failed rebuild preserves dirty-set retry test
---

# Editor UI 02：UiSurface dirty rebuild 前后全树扫描

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 runtime UI surface rebuild 与 editor retained-host call chain 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 交接原因：dirty aggregate/set 必须与 layout mutation、state flags 和错误重试合同一起设计，不能由 editor host 上层缓存绕过。

## 失败现象与复现证据

`UiSurface::rebuild_dirty` 先用 `dirty_flags()` fold 全部 nodes，再用 `dirty_node_count()` 再扫全部
nodes。即使 incremental layout 只访问一个 dirty subtree，成功后 `clear_dirty_flags()` 仍第三次
遍历全部 nodes。任意 hover/input/render/layout 失效因而有至少 3N 的 metadata scan。

## 最低共享层根因

dirty authority 只分散存储在 node flags；surface 没有 aggregate domain mask、dirty node id set/count
或 rebuild generation。调用端只能每次重新发现全树状态。

## 架构修复验收

- 所有 mutation/state-flag 路径增量维护 surface dirty mask 与 dirty node ids/count；重复标脏不重复计数。
- rebuild 成功后只清登记/访问节点；失败时保留完整 dirty state 供重试，显式 clear 仍有确定语义。
- 10k nodes/1 dirty leaf 的 dirty discovery/clear visit count 接近 dirty set 大小，idle no-dirty 为 O(1)。
- layout/hit/render/input/style/text/visible-range union 与既有 incremental tests 全部保持。

## 禁止临时方案

- 不得删除 dirty diagnostics 或只缓存 `dirty.any` 而让 count/domain 失真。
- 不得把三次扫描移到 worker 后继续与主线程 mutation 竞争同一 tree。

## 修复结果与回传

Open state: `待 Editor UI 02 建立 surface dirty aggregate 与 dirty-node authority`。
