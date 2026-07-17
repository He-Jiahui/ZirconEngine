---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: retained-control-index-and-virtual-row-sync
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node
  - zircon_runtime/src/ui/surface
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/menus.rs
  - dev/slint/internal/core/window.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - control-id index insert/detach/reuse/reload parity
  - duplicate control-id and descendant traversal contract
  - 1/100/10000 control lookup visited-node and allocation benchmark
  - virtual-row add/remove/reorder/scroll delta and route parity
  - 1000-key active-popup navigation and outside-dismiss scale/parity matrix
---

# EditorUI01：retained control-id 全树扫描与 virtual-row 同步放大

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/ui/retained_host/callback_dispatch` 135/135 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 交接原因：control identity、pointer route 与 virtual-row identity 必须由共享 `UiSurface`/template generation owner维护。

## 失败现象与复现证据

性能计划逐文件审查 `retained_host/callback_dispatch` 135/135 后，仍找到 10 处直接 `tree.nodes.values().find_map` 和 22 个 control-id lookup 调用。popup/menu 一次 action 会对 open/visible/selected/property 分别扫描 surface；Workbench data sync 对每个 scene/component row 调用多个 control mutation helper，每个 helper又从头查 control id，形成 rows × properties × tree-size 的主线程放大。

本轮已先修复 virtual-row 创建时每行重复查 control id和求 max node id的问题，但 data sync、popup descendants、property edit 和动态 row insert/prune 仍没有共享稳定索引。各 callback 私建缓存会在 template reload、surface rebuild、row detach/reuse 后失效，不能作为正确修复。

`host_contract/surface_hit_test`及直接支撑39文件审查补充pointer consumer：Workbench每次move先扫描bounds并新建/填充/rebuild完整hit surface，之后扫描全部template nodes寻找open popup。性能切片已把node/bounds lookup改为借用，并按Y在O(1)时间定位uniform menu/option candidate（inclusive共享边界最多2行）；稳定路径仍需由同一presentation/surface generation发布持久hit surface与open-popup z stack，不能在hit tester建立第二份失效authority。

PERF-MVP-147进一步要求pointer route不能通过`get_host_presentation()`深clone完整dock/pane/template/RGBA snapshot再查询命中。EditorUI08发布immutable generation handle；本计划的hit/control index必须附着同一handle并完成查询，不能要求consumer重新物化host DTO。

Native keyboard/popup 16文件审查补充PERF-MVP-170：每个Arrow/Home/End/Accept/typeahead事件都深clone完整presentation、倒序clone宽Workbench rows，并为active popup重建全部navigation row DTO；outside press再次全树discover popup。EditorUI01必须在同一surface/presentation generation上提交active-popup stack、navigation rows/current index与top-popup dismiss identity，使按键只改index、typeahead只查已提交rows、outside只查top popup。Slint用dirty-tracked menu shadow tree与window-owned active popup collection证明该状态属于owner，而不是event consumer临时反射。

Native pointer move/scroll 34文件审查补充PERF-MVP-171：普通Workbench hover同一event会先做popup hit并丢弃非popup结果，pane miss后再次做相同base hit；move/scroll入口还深clone完整presentation。EditorUI01的generation-owned hit/control index必须让event只查一次hit，并把同一结果用于popup优先、pane遮挡与base fallback。Passive/unhandled scroll不得仅因route存在就制造damage；handler/state未变应返回ignored/idle。

Native pointer routing 48文件审查补充PERF-MVP-173：floating windows、rail buttons、document/drawer/page tabs和asset panel都用`row_data`逐candidate深clone，且按每event线性扫描；asset panel为1-2个control id重复扫全部wide nodes。局部改borrow后，本计划的generation index仍须覆盖chrome/pane spatial与typed route identity，保留floating reverse-z和既有route priority，不得让每个dispatcher各自缓存。

Native menu geometry 27文件审查补充PERF-MVP-175：稳定popup move/scroll/press重复构造root/nested stack、containment和damage，press还分别反射before/after；各层`row_data` clone selected branch。Menu state/layout generation应原子提交popup frames、row ranges、blocking frame和damage bounds，stable event build=0，path delta只更新changed suffix，并与shared menu pointer bridge保持同一truth。

Native button dispatch 104文件审查补充PERF-MVP-176：每个press/release在unsupported button判断和active capture release之前就深clone完整presentation；未捕获路径再顺序做Workbench/pane hit。Pane callback只返回`bool`或`()`，consumer无法区分ignored、handled-without-visual-change和精确damage，因而release保守重绘整pane、press常升级frame update/full frame。本计划的generation route必须把单次shared hit与typed `Ignored/Handled { damage, frame_update }`结果一路传播，避免consumer重新命中或用无条件damage补偿缺失契约。

Workbench renderer 102文件审查补充PERF-MVP-177：Hierarchy和长menu在clip前遍历全部row，Assets/AssetBrowser的projector、hover与scrollbar对同一node table重复多次全扫。Virtual-row owner必须同时发布paint visible range + overscan与稳定row identity，使input和renderer共用同一行集合；不得只优化pointer surface而让paint继续O(total rows)，也不得由renderer私建不同步cache。

## 最低共享层根因

`UiSurface` tree 有稳定 `UiNodeId`，但没有由同一结构 owner维护的 control-id reverse index；template bridge 只能用字符串从 `nodes.values()` 线性反查。row同步输入也只有整份 projection，没有 changed row/property delta。

Bevy `UiSurface` 以 `EntityHashMap<LayoutNode>` 维护 entity 到 Taffy node 的映射并随 upsert/remove更新；Slint repeater 对 row change/add/remove定点更新实例与 dirty state。Zircon 应在自己的 surface/template generation owner建立同类稳定索引与 delta contract。

## 架构修复验收

- control-id index由 `UiSurface` 或同一 generation owner维护；insert、detach、reparent、pool reuse、template reload和surface replacement后不命中 stale node。
- duplicate control id、root-only lookup、all matching nodes与 descendant traversal语义有显式测试；确定顺序不得依赖 HashMap iteration。
- popup/menu/property/data-sync steady lookup均摊 O(1)；1/100/10k controls 下 visited-node count不随 tree size线性增长。
- virtual rows只 materialize visible range + overscan，row add/remove/reorder以 typed delta更新；1/100/10k rows 不执行 rows × full-tree scan。
- 1k pointer/action/data-sync storm记录 allocation、lookup、layout、hit-grid rebuild和p95；route/action/focus/popup/selection bytes与当前语义等价。
- hit surface与open-popup z stack随surface/presentation generation原子提交；1k moves的bounds/surface build/rebuild=0、无popup全node visited=0，10k uniform popup rows每hit visited≤1，并保持clip、disabled/separator阻断underlay与z-order。
- active-popup navigation rows/current index与同一generation原子提交；1k keyboard/typeahead/outside events的full presentation/node clone与row rebuild=0，dismiss只读top popup，popup/template/row变化后stale identity不可命中。
- 1k stable move的Workbench hit-test≤1/event；passive/unhandled或clamped scroll的callback/redraw=0，handled scroll至多一次局部damage，popup/pane/base优先与viewport等待新image保持等价。
- 1/100/10k floating/tab/rail/asset route的candidate DTO clone=0，最终visited与无关tree size解耦；static route kind/surface id不分配String，z/order/duplicate identity保持确定。
- stable 1k menu events的popup stack/geometry build=0；submenu path delta只重算changed suffix，native/shared menu containment、blocking和damage读取同一generation projection。
- 1k native button events的共享route/hit build≤1/event；ignored/unchanged callback redraw=0，handled damage合并≤1；popup/chrome/Workbench/pane优先级、pressed/released视觉与callback order等价。
- 10k hierarchy/menu/asset rows的paint visited≤visible+overscan，input hit与paint row identity/range一致；scroll/add/remove/reorder后无stale row且pixels等价。

## 禁止临时方案

- 不得在每个 bridge复制一份不可验证失效的 `HashMap<String, UiNodeId>`。
- 不得只把全树扫描放到后台线程；surface mutation与generation commit仍需有界、确定和可观测。
- 不得用仅缓存首个 node破坏 duplicate/descendant语义，或通过关闭动态 rows规避测试。

## 修复结果与回传

Open state: `待 EditorUI01 在 UiSurface/template generation owner实现 control-id reverse index与virtual-row typed delta，并回传规模、失效和完整路由 parity 证据`。
