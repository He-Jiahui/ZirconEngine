---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/input.rs
tests:
  - retained native pointer button and capture-release tests
  - current-source Windows Cargo pending
  - 1000-event snapshot, route, callback, and redraw counter trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor native pointer button dispatch逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`native_pointer/button_dispatch/` directory tree 共 **104/104** 个Rust文件、**2,000** 行已逐文件阅读，并核对press/release、capture、close prompt、popup/menu/chrome、Workbench、pane callback、text focus与damage顺序。当前源Cargo与产品交互计数trace未完成，因此仍留在`pending.md`。

## 已有正确边界

主入口显式区分press/release和primary/secondary；active resize/tab-drag capture可在release时先完成；close prompt、overflow popup、menu、Workbench popup、top-level chrome、Workbench node与pane route保留确定优先级。Viewport button转发不因输入本身主动重绘，而是等待后续图像；passive pane的press在没有text-focus damage时可返回idle。没有发现递归dispatch、同步文件I/O、线程创建或无界队列。

## 热点与计划

PERF-MVP-176：`button_dispatch_input`在每个button press/release开始时先调用`get_host_presentation()`，深clone完整host snapshot，之后才判断button id，也早于active capture release。未支持的button和已被resize/tab-drag capture消费的release因此仍支付完整dock/pane/template/RGBA snapshot成本。未捕获事件再按Workbench hit、pane hit顺序查询；pane callback接口只返回`bool`或`()`，无法表达实际state/damage，最终fallback在release无条件重绘整pane frame，pressed callback通常请求frame update或full-frame fallback，即使callback未改变可见状态。Viewport toolbar还把稳定control id转成新的`String`只为damage lookup。

局部先把button-id判断与capture release移到presentation读取之前，并让toolbar damage借用route id。最终由EditorUI08提供PERF-MVP-147的immutable presentation generation handle；EditorUI01的单次hit结果应携带typed `Ignored/Handled { damage, frame_update }`，popup/chrome/Workbench/pane只消费同一generation route，不用void callback后的保守全pane重绘。Pressed visual、text-focus clear、menu/popup state和capture边沿必须显式合并damage，不能通过取消release redraw破坏反馈。Slint的input dispatch以`Accepted/Ignored`结果传播是否消费事件，是这里需要的最低契约。

## 动态验收

对unsupported button、captured resize release、captured tab-drag release、passive pane、unchanged callback、Workbench node、menu/popup、viewport与toolbar各运行1,000次press/release，记录full presentation clone/RGBA copied bytes、hit-test、callback、damage area、frame update和full-frame次数。unsupported与captured release的presentation clone=0；每个未捕获event的共享route/hit build最多1；ignored/unchanged callback redraw=0；handled event damage最多合并一次。保持capture final point/drop、pressed/released视觉、focus、secondary menu、route priority、callback order、viewport输入和pixel parity。
