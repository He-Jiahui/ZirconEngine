---
related_code:
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking
  - zircon_editor/src/ui/retained_host/tab_drag/group.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/tab_drop.rs
  - zircon_editor/src/ui/retained_host/drawer_resize.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
tests:
  - inline source guard: 1
  - external retained drawer-resize files/tests inspected: 6/14
  - external retained tab-drag files/tests inspected: 9/37
  - rustfmt check: blocked by pre-existing import-order drift in 1 externally modified file
  - current-source managed Windows Cargo pending
  - drag/resize storm counters and WPR/Tracy trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained workspace-docking当前源码复核（2026-07-31）

## 范围

`zircon_editor/src/ui/retained_host/app/workspace_docking.rs`与`workspace_docking/**`当前源 **6/6** 个Rust文件、**327** 行、**1** 条内联`#[test]`已逐文件阅读；path+raw-content SHA-256为`cdcad19d0a8aab9b53302c6be0e1644b9dde859c452d216fdf511847c44749b6`。`drag_drop/route.rs`只有外部import顺序差异，`drawer_resize/movement.rs`含外部same-preferred早退修复；本轮都按current source只读审查，未修改Rust。另回查tab-drag group、tab-drop dispatch、drawer group dispatch及既有drawer-resize 6文件/14 tests、tab-drag 9文件/37 tests合同。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| pointer entry | 1/1 | 61 | 0 | drag/resize down-move-up分派 |
| drag/drop | 2/2 | 140 | 1 | transient group、drop route与layout dispatch |
| drawer resize | 3/3 | 126 | 0 | capture、transient extent与release commit |

## 发现

- **正向边界**：invalid pointer kind只在错误路径format；drag group相等时不重复`set_drag_state`，且有内联source guard。drawer resize在down时只捕获可见frame/base extent，move只写transient preferred并mark dirty，release才提交持久layout command；current外部修复让相同preferred不写map/不置layout dirty，符合PERF-MVP-172。
- `sync_drag_target_group`每个down/move/up先做一次drag-surface dispatch，再把typed route转成owned String：静态group也`to_string`，floating/edge用`format!`。只有分配group key并读取完整Slint drag-state DTO后才比较same group；稳定drag storm仍有每event String与state-get成本，虽已避免state-set。
- pointer-up先调用`sync_drag_target_group`完成一次hit dispatch，随后`resolve_drag_drop_route_from_pointer`再次`drag_route_at`，同一point重复命中。release还把drag tab id与target group各复制成String。
- **PERF-MVP-603 / drop release全模型重建**：route resolver在检查`target_group.is_empty() && pointer_route.is_none()`的detach快路前，已clone完整`current_layout()`、构建chrome/project context/WorkbenchViewModel并持commands锁。明确detach不消费layout/model；普通drop也应读取同generation committed route/layout/model index，而不是由release事件重建完整workbench。
- collapsed drawer attach先`current_layout()`深clone查询mode，再分两条`AttachView`和`SetDrawerMode(Pinned)`事件。每条都重跑session metadata/window registry、event/journal/invalidation与scene-inspection，且第一条成功第二条失败会留下半提交；应由PERF-MVP-603 typed drop batch一次原子应用。
- left/right resize release仍由`dispatch_resize_to_group`分两条`SetDrawerExtent`事件，补强PERF-MVP-131。更严重的是click/no-move也读取base preferred并照常dispatch；下层即使`changed=false`仍无条件`recompute_session_metadata`并返回layout/presentation/reflection effects，故无移动release也触发两次完整事务。应在capture generation/changed bit处直接no-op，不能只依赖layout manager比较。
- changed resize move仍先更新resize surface、写BTreeMap transient并mark layout dirty；其每redraw drain coalescing、窄frame snapshot与`use_committed_pointer_layout` diagnostics重复写继续归PERF-MVP-172/601，不在本模块另建任务。

## 参考与目标

- Godot `dev/godot/scene/gui/split_container.cpp:1500-1508`在split offset相同直接返回，真实变化才`queue_sort()`；Zircon保留current same-preferred guard，并把no-move release也收敛为0 layout command。
- Godot `dev/godot/editor/docks/editor_dock_manager.cpp:140-183`在drag session缓存稳定dragged dock identity，stop只清session；layout变化在`:185-195`走`save_editor_layout_delayed()`合并。Zircon不复制其全局对象模型，但应让drag session持typed identity与committed route generation，release只提交一次原子事务。

EditorUI08在drag start捕获`DragSession { tab, source, route_generation }`，move返回borrowed/typed `HostShellPointerRoute`并只在identity变化时更新latest target；String只在诊断/持久边界生成。release复用单次hit和generation-owned drop index，detach先早退，attach/split/reopen统一为typed `ApplyTabDrop` batch，一锁一次layout/metadata/event/dirty publish。若route generation漂移则显式重新解析一次或取消，不以旧geometry提交。PERF-MVP-131/172共享同一layout batch基础设施，但保持独立验收。

## 动态验收

按moves `1/1K/1M @125/500/1000Hz`、tabs/windows/nodes `1/100/10K`、route `drawer/document edge/floating/floating edge/detach/invalid`、drawer mode `open/collapsed`、resize `no-move/same-point/changed storm`记录surface dispatch、group String alloc/bytes、drag-state get/set/clone、route hits、layout/chrome/context/model builds、commands lock wait/hold、layout snapshots/events、metadata/window-registry/scene observes、dirty/recompute/redraw与UI p50/p95。

验收要求：same-group move String alloc与drag-state get/set=0或只读typed scalar一次，pointer-up hit dispatch≤1；stable generation drop的layout/chrome/context/model build=0；detach full build/commands lock=0；collapsed attach的layout transaction/metadata/event/publish各=1且失败原子；no-move resize layout command/event/recompute=0，left/right changed resize继续满足PERF-MVP-131各≤1。route priority、anchor/split/order、detach id、focus、cancel、resize min/snap、Cargo、F4 pixels与WPR/Tracy通过前保留在`pending.md`，不进入`review.md`。
