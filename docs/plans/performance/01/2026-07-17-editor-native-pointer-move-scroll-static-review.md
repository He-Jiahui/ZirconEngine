---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/input.rs
tests:
  - retained native pointer move/scroll contracts
  - current-source Windows Cargo pending
  - 1000-event hit/redraw/clone counter trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor native pointer move/scroll逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`native_pointer/move_dispatch.rs` + 子文件共 **16** 个Rust文件、**302** 行；`native_pointer/scroll_dispatch.rs` + 子文件共 **18** 个、**321** 行。合计 **34/34** 个文件、**623** 行已逐文件阅读，并核对event-loop调用、pane/workbench route与现有行为测试。当前源Cargo和1,000事件counter trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Captured resize/tab drag在普通hover前处理；menu popup优先于pane/base Workbench，pane move按asset/hierarchy/welcome/template/viewport target派发，viewport input不重绘旧image。Menu move会比较前后state，未变时idle；pane hover redraw按old/new interaction damage计算。Scroll对真实menu、asset、hierarchy、welcome、console、inspector和asset-details回调保持局部damage，viewport scroll保持idle等待renderer更新。

## 热点与计划

PERF-MVP-171：所有未capture的move和所有scroll入口先`get_host_presentation()`，深clone完整dock/pane/template与可能的viewport RGBA payload，结构修复仍由PERF-MVP-147的immutable generation handle负责。局部还有额外放大：`dispatch_pointer_move_body`先调用Workbench route只接受popup；普通Workbench hit被丢弃，pane未命中后又调用同一route。当前route最终进入PERF-MVP-146的full hit-surface build/scan，因此普通hover每event支付两次。

Scroll路径在route到`TemplateNode`、viewport toolbar、UI asset、Other或任何未处理target时，没有callback或state mutation，却仍无条件返回整个pane region damage。Menu scroll同样没有像move一样比较before/after，达到scroll clamp或不可滚动menu时仍重绘。这些是可直接删除的冗余：每move缓存一次Workbench hit并保持popup→pane→base优先；passive/unhandled scroll返回idle；handled menu只有state改变才返回damage。Slint每个mouse event执行一次tree dispatch，以`EventAccepted`/`EventIgnored`决定后续行为；wheel仅在accepted且可能改变布局时补一次move，提供了明确副作用边界。

Asset route里的mode/list-kind和hover hit字符串clone仍属于PERF-MVP-118/170的generation-owned route payload，不在本地再建缓存。Scroll使用`IdleHover` scenario会混淆滚轮与hover测量，动态trace应拆分scenario，但在没有独立scroll KPI前不作为代码热修。

## 动态验收

对1,000次稳定Workbench hover记录presentation clone、Workbench hit-test、pane hit、route allocation和redraw；hit-test必须不超过1/event，最终generation实现后full presentation clone为0。对passive/unhandled、已clamp menu和真实menu/asset/hierarchy/detail/viewport各运行1,000次scroll，记录callback/state change/redraw/damage；无处理或无变化时callback/redraw为0，handled change每event至多一次局部damage，viewport继续等待新image。保持popup优先、pane遮挡、capture、old/new hover damage、scroll clamp、route和像素等价。
