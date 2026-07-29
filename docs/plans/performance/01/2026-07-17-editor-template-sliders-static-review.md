---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_slider_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_slider_geometry
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders_tests
  - zircon_runtime/src/ui/surface/render/sliders.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - slider identity/metrics/geometry/style/range/steps/pixel tests
  - current-source Windows Cargo running
  - editor/runtime oversized tick-count regression pending
  - 1/100/10000 slider theme/metrics/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template sliders逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_slider_geometry*`、`template_sliders*`与tests共 **35/35** 个editor Rust文件、**1,350** 行已逐文件阅读。另对`zircon_runtime/src/ui/surface/render/sliders.rs`的tick解析/command loop做了聚焦跨层核对，但该runtime文件尚未完成逐行验收，不计入35文件。当前源Cargo、局部预算修复与规模trace未完成，因此仍留在`pending.md`。

## P0：外部tick数量可制造无界主线程command storm

Editor的`slider_tick_count`把`layout_third_cell_offset_x`直接round/cast为`usize`，`push_slider_ticks`随后执行`0..tick_count`并逐项push quad。Runtime UI renderer的`tick_count`和`push_tick_commands`具有同样逻辑。输入`f32::MAX`可饱和为极大`usize`，即使普通数千级输入也会让一个node在主线程生成数千commands；两条路径都没有pixel密度上限或硬预算。

PERF-MVP-196要求在`zircon_runtime_interface::ui::surface::render`合同层声明唯一共享`MAX_UI_SLIDER_TICK_COUNT`，editor/runtime解析与最终command loop均防御性clamp；最终有效tick还不得超过track可分辨的pixel columns。该值是跨crate UI command预算，按constant placement ladder不应复制成两个私有magic number。先补0/1/2/normal/超大/非有限值与command-count测试，再实现硬界限。

## P0：同一slider重复完整metrics projection与文字分配

Slider context已正确只解析一次style、percent和基本identity，但没有携带metrics。普通带label/value slider在value rect、track、label、rail、thumb和value中约7次调用`workbench_slider_metrics()`；range+ticks+双thumb路径可达约12次。每次都取得全局metrics锁并重派生21字段结构。Label无条件`to_owned`，value无声明时`format!`，range minimum也每paint`format!`。

PERF-MVP-197把现有`SliderCommandParts`升级为changed-node `SliderPaintSpec`：只取一次theme/metrics/text snapshot，携带borrowed/shared label/value、resolved geometry、tick positions与style。Stable generation由PERF-MVP-178复用compiled segment；percent交互只patch fill/thumb/value相关小段，不重建label与静态track。

## 动态验收

预算验收要求两条renderer对任何输入的tick commands均<=共享上限且<=可分辨columns，正常2/5/常用step语义与pixels不变。规模验收在1/100/10,000 normal/range/steps、label/no-label、hover/focus/press/disabled sliders上记录theme/metrics acquisition、label/value allocation/format bytes、static/dynamic segment builds与commands。Changed node theme/metrics各<=1；stable generation静态计数=0；percent-only更新只重建fill/thumb/value片段。
