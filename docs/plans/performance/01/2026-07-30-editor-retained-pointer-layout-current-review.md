---
related_code:
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/pointer_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs
  - zircon_editor/src/ui/retained_host/asset_pointer
  - zircon_editor/src/ui/retained_host/hierarchy_pointer
  - zircon_editor/src/ui/retained_host/menu_pointer
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - inline tests: 2
  - rustfmt check: passed 11/11
  - scoped whitespace check: passed
  - current-source managed Windows Cargo pending
  - pointer projection/build/write counter trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained pointer-layout当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/pointer_layout.rs`与`pointer_layout/**`当前源 **11/11** 个Rust文件、**511** 行、**2** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`5c5f31b18442f76fb33012272d87e2065a99c71ef431cf7c11c7c3ff763b7aa8`。范围内无未提交源码差异，11/11 rustfmt通过，本轮未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| module/host context | 2/2 | 17 | 0 | 子模块出口与pane context获取 |
| asset surfaces | 4/4 | 192 | 2 | activity/browser snapshot、tree/content/reference/used-by layout与UI writeback |
| detail/hierarchy | 2/2 | 115 | 0 | console/inspector/details与scene row pointer layout |
| menu/shell/welcome | 3/3 | 187 | 0 | menu、activity/page/tab/drawer与Welcome Recent同步 |

## 发现

- **正向边界**：pointer callback不再构造完整editor snapshot；asset pointer复用slow-path提交的workspace snapshot，hierarchy callback复用`Arc<[SceneEntry]>`；size未变化的高频asset/hierarchy callback有guard。各bridge在接收新layout后先做`layout/state` equality，完全相同时不会执行`rebuild_surface()`。2条源码测试锁定asset snapshot与size guard。
- **PERF-MVP-106 / equality发生得太晚**：每个非paint-only slow recompute无条件调用menu、welcome、hierarchy、3个detail和activity/browser asset pointer同步，随后viewport recompute再调用activity rail、host page、document tabs与drawer header。bridge equality只能避免最终surface rebuild，无法撤销此前完整owned layout构造、clone、format和comparison成本。
- `sync_asset_pointer_layouts()`对activity/browser各执行一次`Arc::new(snapshot.clone())`，并分别构造tree、content、references、used-by四个owned layout；即稳定slow path仍有2份workspace snapshot clone与8次列表投影。下游若相等只丢弃这些新值。该链补强PERF-MVP-102/106/109，不新建任务号。
- `sync_hierarchy_pointer_layout()`先用`Arc::from(scene_entries)`复制完整scene-entry slice，再为每行`entry.id.to_string()`形成node-id Vec；稳定代也支付O(N) copy/format后才由bridge equality拒绝。它与scroll时全surface rebuild共同归EditorUI01 visible-row authority。
- menu先把新owned layout保存在`self.menu_pointer_layout`，随后再`clone()`一份给bridge比较；Welcome Recent每次sync重新收集owned project paths。更窄的click路径还会先构造完整`runtime.chrome_snapshot()`再调用全Welcome layout sync，只为处理一次点击，补强PERF-MVP-117。
- **至少11次同值host写 + 14个空调用**：一次`sync_recompute_pointer_surfaces()`完成后，asset state有8个setter、hierarchy有2个、menu有1个无条件`RefCell::borrow_mut`赋值，即使值不变；另外asset references/used-by 8个、detail 3个、welcome 3个setter当前为空函数，仍被每次调用。没有changed guard。最终generation owner应让stable slow path完全跳过，不以逐setter微缓存长期保留双写。
- detail layout规模小且bridge equality有效；单独删除14个空调用只能减少常数调用，不能关闭snapshot/layout O(N)与11次写。本轮因pointer state/generation迁移需要EditorUI01/08批准，未做局部Rust修改。

## 参考与目标

- Slint `dev/slint/internal/core/model/repeater.rs:447-522,601-607`按`row_changed/added/removed`标记具体instance dirty，并用generation发布变化；不是先重建全部owned rows再做深比较。
- Godot `dev/godot/scene/gui/item_list.cpp:1471-1523`先定义visible frame并定位`first_item_visible`，绘制从可见项开始。Zircon input与paint应共享stable row identity/visible range，不能各自建立不一致缓存。

EditorUI08让pointer projection成为layout/chrome/model generation DAG的consumer：captured generations与sizes未变时，整个`sync_recompute_pointer_surfaces`和viewport shell pointer build均不调用；真实变化时只向EditorUI01提交typed changed rows/sizes。EditorUI01拥有唯一row identity、visible range、hit grid和route handle；scroll只更新transform/visible cells，bridge不接收先全量构造的owned layout。host interaction state按changed fields一次patch，稳定generation的setter/`RefCell` write=0。

## 动态验收

按rows `1/100/1K/10K`、assets/scenes/recent/menu `stable/1% changed/full replace`、pane sizes `same/resize`、invalidations `paint-only/presentation/layout`与storm `1/1K/100K`记录workspace/scene snapshot clones、layout builds/bytes、String format、bridge equality/rebuild、host setter/no-op call/`RefCell` write、active nodes/hit cells、UI p50/p95和RSS。

验收要求：stable generation的snapshot clone、row visit/String format、layout build、bridge sync、setter/no-op call与state write均为0；1%变化工作随delta+visible rows而非总N；scroll active nodes/targets/path受viewport+overscan约束；resize只重建受影响surface一次；route/hover/scroll/clamp/dock/floating/Welcome open-remove parity不变。managed Cargo、规模counter、F4 input trace与independent review完成前保留在`pending.md`，不进入`review.md`。
