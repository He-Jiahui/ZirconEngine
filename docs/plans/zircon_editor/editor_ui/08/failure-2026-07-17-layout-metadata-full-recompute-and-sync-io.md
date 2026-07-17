---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: layout-metadata-full-recompute-and-sync-io
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/compute.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/retained_host/drawer_resize.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
tests:
  - focus/dock/resize metadata visited-node-count regression
  - rapid page-switch persistence debounce/crash-recovery test
  - 10/100/1000 views/windows layout scaling benchmark
  - 1000-event same/changed-point drawer resize coalescing and final-point parity
---

# EditorUI08：layout metadata 全量重算与主线程同步持久化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`ui/host` layout/workspace/window/persistence 逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：layout mutation delta、metadata indexes、window projection 与 persistence cadence 需要一个 EditorUI08 owner。

## 失败现象与复现证据

几乎每个 `LayoutCommand`（包括 focus）都会调用 `recompute_session_metadata`：全布局 collect instance hosts、retain/更新全部 open views、clone 全 view instances 重建 `EditorWindowRegistry`、重建 drawers、retain animation/UI asset sessions，再让 `WindowHostManager` 对 tracked×floating windows 互扫。`current_layout/current_view_instances` 又返回深 clone，reflection 同步重复消费。

Workbench 层已把 repeat resize/drawer/page/focus 修成准确 `changed=false`，canonical `activity_windows()` 改为 borrowed `Cow`，registry/geometry indexes 也不再复制完整 rows/descriptor ids。这能阻止部分伪 dirty，但真实 focus/dock/resize 后仍没有 typed affected ids，geometry 仍按每次调用重建 maps/frames，不能视为架构验收。

切换 main page 还在事件调用链上同步 `save_page_layout`：clone 全 layout、load/deserialize 全 store、serialize/write 全 store，然后 restore page 又 load/deserialize，并多次 full metadata recompute。该路径是 UI 交互，不应把 config I/O 与全图派生重建放在同一主线程事务里。

Retained-host root审查确认 drawer resize group也放大同一事务。left/right结束时分别对 top/bottom slot dispatch `SetDrawerExtent`；每个 command 都独立取得 session lock、apply、`sync_legacy_drawers_from_active_activity_window`并执行完整 `recompute_session_metadata`。因此一次 resize可完成两次全图metadata和两次event/publish，而且第二个失败时第一个已经提交。

Native drag/resize 21文件审查补充PERF-MVP-172：相同resize point仍写state、深clone完整presentation只取center-band frame、同步callback重写相同preferred并mark layout dirty。Pending redraw虽合并最终recompute，却没有消除raw event上的宽clone与mutation。EditorUI08应把latest point/preferred建成transient resize generation，同一redraw drain只commit一次；release必须flush最终point后再进入typed drawer batch。

## 最低共享层根因

`LayoutManager` 只返回 changed bool/粗 diff，不能描述受影响 view/window/drawer；派生 metadata 没有增量索引，persistence 也没有 debounce/transaction generation，因此所有变更只能全量扫描并同步写回。

## 架构修复验收

- layout apply 返回 typed delta（placements/focus/windows/drawers/pages），metadata owners 只更新受影响 ids；普通 focus 不全图扫描/clone。
- geometry cache key 至少包含 layout、shell size/scale、descriptor 与 transient resize generations；未变 key recompute count=0。
- page-switch persistence 快照在锁内 O(1) 获取 immutable generation，I/O 在有界 worker/debounce 上完成；退出/显式保存有 flush 与失败恢复。
- window sync 使用 set-difference/index，不做 tracked×floating 互扫；snapshot consumers 共享 layout/view generation。
- 10/100/1000 views/windows 下 focus visited nodes 近 O(1)，dock/close 近 affected subtree；rapid switch 主线程无同步 config I/O，p95/queue/flush 可观测。
- drawer group使用 typed batch/transaction，一锁一次 apply、一次 metadata delta与一次 publish；left/right两 slot原子成功或失败回滚，bottom与单 slot语义保持。
- 相同resize point完全idle；changed storm以latest-wins transient generation合并，同一redraw drain layout commit≤1，release最终extent严格对应最后point。

## 禁止临时方案

- 不得只延长 debounce 而继续复制/序列化全 store 每次变更。
- 不得让后台持久化借用可变 session 或乱序覆盖新 generation。
- 不得为 window/drawer/view 分别全树扫描同一 layout。

## 修复结果与回传

Open state: `待 EditorUI08 实现 typed layout delta、incremental metadata 与 generation-safe async persistence，并回传规模/交互/恢复证据`。
