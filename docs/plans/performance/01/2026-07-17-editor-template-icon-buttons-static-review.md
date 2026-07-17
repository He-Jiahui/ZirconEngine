---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - icon-button identity/context/geometry/style/command/pixel tests
  - current-source Windows Cargo running
  - 1/100/10000 icon-button classification/theme/resource/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template icon buttons逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_icon_buttons*`与`template_icon_buttons_tests/**`共 **18/18** 个Rust文件、**988** 行已逐文件阅读。覆盖identity、toolbar/panel/rail context、geometry/metrics、surface/content/style、glyph command以及完整state/style/geometry/asset/pixel tests。当前源Cargo、规模计数与产品像素trace未完成，因此仍留在`pending.md`。

## 结论与已有责任项

入口正确地只解析一次context与style并在surface/content之间共享，普通无背景/无border surface也会早退；这组代码没有新增独立根因。稳定帧仍会重复执行component-family/visual-language识别、最多8个control-id前后缀context判断、全局style/theme解析、一次glyph metrics读取（pressed时再读一次host metrics），随后走图标资源解析或manual fallback command构建。

这些成本分别回链PERF-MVP-178（stable compiled segment与typed role）、PERF-MVP-179（manual glyph command amplification）、PERF-MVP-181（paint-time visual resource lookup/copy）、PERF-MVP-182/183（单一theme snapshot与typed visual role）。Changed icon button只允许一次identity/context/style/theme/metrics与resource-handle解析；stable generation这些计数和surface/glyph command build均为0。不得为icon button另建脱离presentation/theme/resource generation的局部缓存。

## 动态验收

在1/100/10,000 toolbar/panel/rail/tab-close、normal/hover/pressed/focused/disabled/selected、resolved/missing icon上记录identity/context probes、theme/metrics reads、resource hit/miss/raster/copy、fallback segments与commands。保持现有18文件覆盖的context、declared style、state priority、pressed offset、glyph sizing/order、asset color与GPU/Softbuffer pixels parity。
