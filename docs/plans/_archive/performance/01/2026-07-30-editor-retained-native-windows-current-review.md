---
related_code:
  - zircon_editor/src/ui/retained_host/app/native_windows.rs
  - zircon_editor/src/ui/retained_host/app/native_windows
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
tests:
  - inline tests: 0
  - rustfmt check: blocked by pre-existing import-order drift in 1 externally modified root file
  - current-source managed Windows Cargo pending
  - per-window generation/apply/OS-call counter trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained native-windows当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/native_windows.rs`与`native_windows/**`当前源 **4/4** 个Rust文件、**153** 行、**0** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`d1aa4b60b4dfe01e986827c06a5d6429007dd4bed29b0fd1e151783f6b5e8bfa`。root文件含外部未提交import顺序差异，本轮只读纳入current-source审查，未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| root/target | 2/2 | 45 | 0 | model+projection到native target Vec |
| presenter store | 1/1 | 69 | 0 | create/hide/apply与window map |
| native presentation | 1/1 | 39 | 0 | native ids/title/bounds与OS position/size |

## 发现

- **正向边界**：只有projection明确`native_host_present`且有surface tree id的floating window进入target；stale window从map移除并hide；new window只创建/接线/show一次。空target有早退，避免准备native pane payload。
- target collection每slow path扫描全部floating model rows，并clone window id、title和surface tree id形成新Vec。没有target generation或borrowed view；稳定window集合仍全量构造后才进入store。
- **PERF-MVP-106 / 集合分配与双查找**：`sync_targets`每次clone全部window id到新BTreeSet，再clone stale id到Vec；每target先`contains_key`再`get`做两次树查找。与后续presentation相比这是次要常数，但1K windows下仍应由generation/delta消除，而不是单独换HashSet掩盖全量apply。
- store只持`BTreeMap<id, UiHostWindow>`，没有last-applied target/presentation generation；因此每个existing target无条件执行apply，即使title/bounds/tree/model/chrome/panes均未变。
- **每window每slow path至少三次完整presentation事务**：apply闭包先调用`apply_presentation()`重建并提交完整workbench；再调用toolbar attachment，重新`get/set_host_presentation()`；最后`configure_native_floating_window_presentation()`第三次clone/replace完整presentation以写native ids/title/bounds。三次之间没有single artifact/patch合并。
- native configure随后无条件调用OS `set_position`与`set_size`，即使bounds未变。窗口系统调用、DPI/resize callback和潜在重布局可被稳定slow invalidation重复触发；store没有applied bounds或actual-window comparison。
- create/hide/show和OS geometry调用当前在UI/main路径串行执行。window生命周期应继续由主线程owner执行，但projection/pane artifact可锁外准备；同generation不得重复apply或排无界worker任务。
- `window()`返回strong clone供明确consumer使用，是生命周期语义而非本模块热点；不把Arc/Rc handle clone误报为深presentation copy。

## 参考与目标

- Bevy `dev/bevy/crates/bevy_winit/src/system.rs:313-333,367-405`只查询`Changed<Window>`，并用`CachedWindow`逐字段比较title、position和resolution；position还与实际outer position比较后才调用winit。Zircon应保留自己的native presenter contract，但stable generation的OS调用必须为0。
- Godot/Bevy都要求窗口创建与平台调用留在event-loop owner；Zircon不应为“异步”把native window对象移到任意worker，而应把immutable presentation build与主线程changed apply分开。

EditorUI08让store entry持`last_target_generation/last_presentation_generation/applied bounds-title-tree`；target collection消费floating projection changed rows。每window只在对应generation变化时构造一次shared pane/presentation artifact，并用一次scoped patch同时写toolbar/native fields；OS position/size只在normalized applied bounds变化且与实际状态不同时调用。created/stale ids由ordered delta维护，稳定sync不分配BTreeSet/Vec。

## 动态验收

按windows `0/1/4/16/1K`、state `stable/create/close/title/bounds/tree/pane/toolbar`、invalidations `paint-only/presentation/layout`与storm `1/1K/100K`记录target row visit/clone、BTreeSet/stale Vec bytes、map lookups、pane payload builds、full presentation get/set/clone bytes、per-window apply、OS position/size/show/hide、callback count、UI p50/p95与RSS。

验收要求：stable generation的target rebuild/set allocation、presentation build/get/set/clone、window apply与OS calls均为0；每changed window每generation presentation build/patch=1；bounds不变OS call=0；create/show和stale hide各一次；1% change工作随delta而非总window数；title/bounds/tree/pane/toolbar、close policy、z/focus、DPI/resize和F4 pixels等价。managed Cargo、规模counter、native-window产品trace与independent review完成前保留在`pending.md`，不进入`review.md`。
