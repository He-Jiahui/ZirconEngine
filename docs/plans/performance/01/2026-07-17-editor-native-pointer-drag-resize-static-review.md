---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize/**/*.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/input.rs
tests:
  - retained drawer resize and tab drag contracts
  - current-source Windows Cargo pending
  - 1000-event resize coalescing/clone trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor native pointer drag/resize逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`native_pointer/drag_resize.rs` + `drag_resize/**`共 **21/21** 个Rust文件、**398** 行已逐文件阅读，并核对app drawer-resize/drag callbacks与redraw damage。当前源Cargo和1,000事件trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Tab drag在4px阈值前idle，active move只同步pointer/target group且不重绘旧帧；app已在target group未变时跳过state publish。完整chrome/model只在drop release解析一次，不在move循环。Resize/tab drag capture先于普通hover，redraw按center/dock/drag target damage而非无条件full；native redraw scheduling已由PERF-MVP-165合并到pending边沿。

## 热点与计划

PERF-MVP-172：resize move读取当前state后没有相同point no-op。即使x/y完全未变，仍写回state、调用`get_host_presentation()`深clone完整host DTO、同步callback重复计算/写入相同preferred并`mark_layout_dirty()`，最后请求需要frame update的redraw。实际recompute由pending redraw合并，不是每raw event都立即布局，但宽snapshot clone、callback、map mutation和dirty publish仍发生在每个input事件。

`resize_pointer_redraw`从完整presentation只读取一个`host_layout.center_band_frame`。局部修复应让相同point立即idle，并用窄frame snapshot或arm时捕获的damage代替完整presentation clone。架构上，EditorUI08应把latest pointer/preferred作为transient resize generation，同一redraw drain只commit一次layout；release先flush最终point，再用PERF-MVP-131的typed drawer batch持久提交，不能丢最后位置或把layout mutation搬到无界后台任务。

Drag state仅包含共享字符串与坐标，active move的clone为轻量引用计数；floating tab payload的window/tab `row_data`扫描只发生在arm，当前没有证据支持为其新增独立缓存。Drop release的chrome/model build应由最终drop规模trace决定，不混入move gate。

## 动态验收

输入1,000个相同resize points，state write、presentation clone、callback、transient map write、layout dirty和redraw都必须为0。输入1,000个changed points并限制到一个render drain，记录callback、layout commit、presentation clone与native redraw；最终layout commit≤1、full presentation clone=0、最终extent对应latest point。验证down/move/up、left/right/bottom、min/snap、old/new damage、release flush、typed batch/undo/persistence和tab drag/drop视觉等价。
