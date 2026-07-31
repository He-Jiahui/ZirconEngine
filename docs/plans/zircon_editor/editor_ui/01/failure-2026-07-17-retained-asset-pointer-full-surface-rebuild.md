---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: retained-asset-pointer-full-surface-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/tree/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/tree/layout.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/handle_scroll.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_scroll.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_handle_move.rs
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/scene/gui/item_list.cpp
  - dev/godot/scene/gui/tree.cpp
tests:
  - 1/100/10000-row asset pointer scroll build/allocation regression
  - visible-window virtualization and route parity matrix
  - move/scroll target clone-count and hit-grid update regression
---

# EditorUI01：retained list/menu pointer 每次滚动重建完整命中树

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/ui/retained_host/asset_pointer` 23/23、`hierarchy_pointer` 20/20、`menu_pointer` 26/26 与 `welcome_recent_pointer` 20/20 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 共同责任：EditorUI08 workbench surface generation、Editor09 asset catalog projection
- 交接原因：命中树、pointer dispatcher、scroll transform、virtualization 与 route identity 属于统一输入/dispatch authority，不能在 app callback 再维护另一套可变列表。

## 失败现象与复现证据

Asset content、reference 与 folder-tree 三种 bridge 在 scroll event 后读取 viewport offset；只要 offset 改变，就调用 `rebuild_surface()`。该函数重新创建完整 `UiSurface`、`UiPointerDispatcher` 与 `BTreeMap<UiNodeId, Target>`，为全部 folder/item/reference 行重新格式化 `UiNodePath`、注册 move/down callback、克隆 UUID/folder id、插入节点并执行 `surface.rebuild()`。三个 ScrollableBox 都显式设置 `virtualization: None`，因此 10k 行滚动每步都是 O(N) node/target/path 构建，而不是可视窗口成本。

move/down dispatch 还通过 `targets.get(...).cloned()` 复制携带 `String` 的 target，再转换为 public route。App 的 move consumer 只需要 hovered row/state，却仍支付 UUID/folder String clone。上层本轮已经消除了每 move 的完整 editor snapshot 与 unchanged-size layout rebuild；因此这个 bridge 重建成为新的最低共享层瓶颈。

Hierarchy bridge 具有相同的根因：`handle_scroll()` 在 offset 变化后调用 `rebuild_surface()`，为所有 scene rows 重新格式化 path、注册回调、clone `node_id` 到 route map 并执行 `surface.rebuild()`；其 ScrollableBox 同样声明 `virtualization: None`。大型场景层级因此把一次滚轮事件放大为 O(scene rows) 的命中树重建，不能单独留在 hierarchy app callback 中修补。

Menu bridge 的 root popup scroll 也在 offset 变化后重建全部 menu buttons、dismiss overlay、root/submenu rows、dispatcher 与 routes，ScrollableBox 同样没有 virtualization。本轮已消除重复菜单树 clone、每行双重 root route-index 扫描、owned route clone 与 already-closed rebuild，但 scroll/full-surface topology 问题仍属于同一输入层 authority。大量 layout presets 或扩展菜单项会直接放大该路径。

Welcome Recent bridge 对每个项目创建 item、open、remove 三类节点；scroll offset 变化后整棵重建，且 Open/Remove route 各 clone 一份 project path，move 命中也会物化 owned path。近期项目通常较少，但它处于最小编辑器的首屏路径，必须复用同一 stable-row/visible-range机制，不能作为“数据量小”例外长期保留。

2026-07-31 current app 4/4与bridge 20/20补证：稳定move在`sync_pane_size`内先投影三项Slint状态，dispatch后又投影三项；action hover仍从route-intent map clone owned project path，即使app只消费state。每click还先构造完整chrome snapshot并clone全部recent paths再做单row hit。PERF-MVP-117因此要求same-hover move为0 path clone/0 UI write，stable click复用generation-owned typed route/path，不得把bridge equality当作上游O(N)工作的门禁。完整证据见`../../../performance/01/2026-07-31-editor-retained-welcome-recent-pointer-current-review.md`。

2026-07-30 app pointer-layout current-source 11/11补充了scroll之外的上游放大：activity/browser在bridge equality前先clone两份workspace snapshot并构造tree/content/references/used-by共8个owned layout；hierarchy先复制完整scene-entry slice并为每row格式化String id；menu保留完整layout后再clone给bridge，Welcome重收集owned paths且click先建完整chrome。即使bridge最终判定相等、surface rebuild=0，row visit/clone/format/layout bytes仍为O(N)。EditorUI08负责stable generation整组短路；本EditorUI01 failure负责changed generation只接typed delta并物化visible rows，二者不得各建全量cache。

2026-07-31 asset-drag payload current 3/3补证：content route已携`item_index`，reference route已携`row_index`，但press均丢弃索引并按UUID线性重扫；每普通left-down还在threshold前构造完整owned payload与status。PERF-MVP-109要求同一row generation authority提供typed drag candidate，down只arm，Begin才O(1)解引用Editor09 metadata并一次物化；不得给drag consumer另建UUID map。完整证据见`../../../performance/01/2026-07-31-editor-retained-asset-drag-payload-current-review.md`。

Slint repeater 只更新 dirty row/range，Godot ItemList/Tree 也以滚动窗口和可见项驱动绘制/命中。Zircon 应保留 typed route 与稳定 node identity，但不应把 scroll transform 表达成全树销毁重建。

## 最低共享层根因

Pointer surface 把数据 projection、scroll transform、hit-grid materialization 与 owned route payload 绑定在一个全量 `rebuild_surface()`；没有稳定 row identity、visible-range virtualization、增量 frame/hit update 或 state-only move dispatch。

## 架构修复验收

- layout generation 改变时冻结 immutable row identities；scroll offset 只更新 viewport transform/visible range 与受影响 hit cells，不重建全部 nodes/dispatcher/targets。
- 只 materialize viewport + overscan rows；1/100/10000 rows 的 active node/target/path 数量受 viewport 行数约束，scroll step build/alloc 与总 N 解耦。
- move-only API 返回 hovered row/state，不 clone UUID/folder String；只有 click/press/drag 需要 owned route 时才物化 stable id。
- content list/grid、folder tree、scene hierarchy、welcome recent open/remove、root/nested menu、known/unknown references、scroll clamp、child-window resize、drag payload、disabled/action rows 与 public route 顺序/identity 等价。
- 1k scroll/move storm 记录 surface rebuild count、node/path/target alloc bytes、hit-grid updates、p95 与 queue age；unchanged move rebuild=0，scroll 不全树重建。
- stable generation在app/bridge两层的workspace/scene clone、row visit/String format、owned layout build与deep equality bytes均为0；1% changed工作随delta+visible rows而非总N，click不为单row命中构造完整chrome/list projection。
- welcome recent same-hover move的owned path clone与Slint setter均为0；stable open/remove click的full chrome snapshot、全path Vec与deep layout compare为0，只有最终action边界物化一个owned path。
- asset click-only down的row scan、UUID compare、full drag payload、metadata/summary/status String bytes均为0；真实drag Begin按generation+slot近O(1)解引用且payload只物化一次，stale generation不可命中。

## 禁止临时方案

- 不得只降低 scroll event 频率而保留 O(N) rebuild。
- 不得在 app、bridge 与 painter 各建一份不同步的 visible-row authority。
- 不得用 hash iteration 改变 row/route deterministic order，或让 stale generation 的 hit target 指向新 catalog row。

## 修复结果与回传

Open state: `待 EditorUI01 建立 stable row identity、visible-range pointer surface 与增量 scroll/hit update，并回传 10k-row storm/route parity 证据`。

2026-07-31 menu app adapter增量证据：`app/menu_pointer.rs`当前1/1、47行、SHA-256 `FC58D55CEA13DCD011B4B9CBF496BA721EC4E8D8E5C72DD8F992E44B790E9F2D`完成静态复读。bridge unchanged layout避免部分rebuild，但move/scroll仍clone完整state，app无条件替换owner并写全部Slint字段；有效scroll/submenu仍可full rebuild。PERF-MVP-112验收补充same hover与zero/clamped scroll的state/path clone、setter、rebuild=0，并要求typed changed/damage返回；managed Cargo、1M input/10k rows、F4与independent review未完成，failure保持open。完整证据见`../../../performance/01/2026-07-31-editor-retained-small-input-adapters-current-review.md`。

2026-07-31 asset reference state增量证据：`app/{asset_surface_pointer_state.rs,reference_drop_payload.rs}`当前2/2、104行完成静态复读。两文件自身是常数state match与最多三槽drop take；高频consumer仍让reference bridge克隆完整state，host对单列表stable event重写8项asset UI字段，有效scroll重建全部rows。PERF-MVP-109补充same hover/zero-clamped scroll的state clone、setter、rebuild=0；drop owner move/probe保持常数。managed Cargo、1M event/10k row、F4与independent review未完成，failure保持open。完整证据见`../../../performance/01/2026-07-31-editor-retained-state-visibility-helpers-current-review.md`。
