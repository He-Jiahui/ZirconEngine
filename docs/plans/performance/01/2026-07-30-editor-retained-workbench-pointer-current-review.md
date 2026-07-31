---
related_code:
  - zircon_editor/src/ui/retained_host/app/workbench_pointer.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - inline tests: 0
  - rustfmt check: blocked by pre-existing import-order drift in 5 externally modified files
  - current-source managed Windows Cargo pending
  - click/snapshot/model/lock counter trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained workbench-pointer当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/workbench_pointer.rs`与`workbench_pointer/**`当前源 **7/7** 个Rust文件、**259** 行、**0** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`9b99ee640b54df482efad05a59181dc81aa7957579c59d6fc27dc1db13ed73c1`。其中5个event adapter文件含外部未提交内容，本轮只读纳入current-source审查，未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| module roots | 2/2 | 6 | 0 | document/floating/shell chrome出口 |
| document/floating | 2/2 | 104 | 0 | tab activate/close与floating header focus |
| activity/drawer/page | 3/3 | 149 | 0 | rail/drawer/page/overflow click dispatch |

## 发现

- **正向边界**：adapter不重建pointer layout，直接消费已提交bridge route；负tab index在dispatch前拒绝；无route时多数路径不产生layout effects。点击路径没有本地循环、队列或显式I/O，错误String只在错误边界格式化。
- 所有入口先调用`use_committed_pointer_layout()`，当前唯一作用是重复发布未变化invalidation diagnostics；它已由PERF-MVP-601登记，本模块的每click计数必须包含该写，不能把薄adapter视为零成本。
- **PERF-MVP-105 / floating header单击构建完整model**：命中shell bridge得到window id后，`dispatch_builtin_floating_window_focus()`仍调用完整`chrome_snapshot()`、project command context、持有`runtime.commands().lock()`并构造`WorkbenchViewModel`，只为线性查一个floating window并解析instance id。direct header路径不检查`last_focused_callback_window`；现有`dispatch_builtin_floating_window_focus_for_source()`虽有same-window guard，但这里未使用，重复点击已聚焦header仍支付全链并dispatch layout。
- **PERF-MVP-105 / floating document tab额外chrome**：activate与close在shared pointer dispatch返回`Ok`后无条件调用`note_focused_floating_window_surface(surface_key)`。非`main` surface会再次构造完整chrome，线性扫描`workbench.floating_windows`并clone window id，即使pointer route为`None`。该记录应直接来自committed surface→window identity，不再建snapshot。
- drawer toggle不构造workbench model，但会读取owned current layout、clone active drawer row，并可能连续dispatch activate + reopen两个layout command；其正确性需要保留，本模块只要求stable identity/row窄读取和一次typed transaction，不用局部降频隐藏成本。
- **PERF-MVP-147 / overflow读一位深clone全presentation**：host-page overflow click为了计算`open = !current`调用`get_host_presentation()`，复制完整结构树及可能的viewport RGBA，只读取`host_page_overflow_menu_state.open`。随后setter又借用host state写回3字段。最小止损是host context窄getter/CAS toggle；最终应由EditorUI01 interaction generation持有，禁止保留第二presentation cache。
- 文档tab、drawer、page route dispatch本身使用stable bridge route，是正确方向；但source-window记录、focus resolution、overflow状态仍绕回wide snapshot。由于这些状态属于EditorUI08/01 generation authority，当前设计批准未完成前未做局部Rust修改。

## 参考与目标

- Godot `dev/godot/scene/gui/tab_bar.cpp:299-313,821-850`从命中的tab index直接更新`current`并在相同current时短路结构变化；不会为单击重建整个workbench model。Zircon需保留binding/layout命令语义，但identity应来自committed tab/window generation。
- Bevy `dev/bevy/crates/bevy_ui/src/focus.rs:25-32`把interaction作为entity component并推荐`Changed<Interaction>`消费。Zircon可采用自己的typed route，但focus/overflow变更应按stable identity与changed state传播，而不是读取全presentation。

EditorUI08在presentation/layout generation提交时发布`surface_key -> floating window/active instance`稳定索引和focused window identity；header/tab click只查该索引，same-focused为no-op，commands锁不跨model build。EditorUI01让overflow state成为interaction generation中的窄字段，提供read/toggle API；点击不调用`get_host_presentation()`。所有入口仅在invalidation counter generation改变时发布diagnostics。

## 动态验收

按windows/tabs `1/4/16/1K`、clicks `1/1K/1M`、target `main/floating/already-focused/miss/overflow`与layout `stable/change/close/reopen`记录chrome/editor/model/context build、commands lock wait/hold、floating scans/id clone、full presentation/RGBA copied bytes、layout transactions、diagnostics write、UI p50/p95与RSS。

验收要求：same-focused header的chrome/context/model/lock/layout dispatch=0；floating tab activate/close的focus-note chrome build/scan=0；overflow full presentation clone/RGBA bytes=0；stable click diagnostics write=0；一次drawer action为一个typed transaction且视觉/active/collapse/reopen顺序等价；main/floating focus、tab close、binding fallback、overflow paint和错误语义不变。managed Cargo、规模counter、F4交互trace与independent review完成前保留在`pending.md`，不进入`review.md`。
