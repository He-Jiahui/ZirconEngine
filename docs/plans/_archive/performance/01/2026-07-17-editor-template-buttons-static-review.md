---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_button_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - button state/style/geometry/glyph/text/pixel tests
  - current-source Windows Cargo running
  - 1/100/10000 button classification/theme/metrics/allocation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template buttons逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_button_glyphs*`、`template_buttons*`与`template_buttons_tests/**`共 **29/29** 个Rust文件、**2,973** 行已逐文件阅读。覆盖button identity/kind、surface/state-layer/indicator、content layout/text/icon/glyph、theme metrics及完整state/style/geometry/pixel tests。当前源Cargo正在运行，规模trace与像素验收未完成，因此仍留在`pending.md`。

## P0：同一按钮重复分类、style与theme projection

`button_kind`先调用`button_key`，把control/text/value/variant/surface/validation经`format!`拼接后`to_ascii_lowercase()`；content的`button_glyph`再次构造同一key。一次paint至少两次format和两次lowercase，随后style selector还独立执行command/tab分类。

Surface的`button_surface_command_style`和content的`button_content_style`分别调用一次完整`button_style`。因此同一按钮的state、declared color、command/tab role与3至8次palette读取被整体重复。Content几何又把`current_host_metrics()`分散到font size、line height、text clip guard、max slot padding、icon size/gap、chevron reserve、trailing inset、pressed offset与radius等helper；带icon/label/chevron的常见按钮可约十次metrics读取，另有一次text-preferences读取。Tab/asset identity在这些helper中也重复分类。

PERF-MVP-190要求入口构建一次`ButtonPaintSpec`：typed kind/glyph/command/tab role、resolved interaction/style、one theme metrics/text snapshot、resolved label/text layout及surface/content/indicator geometry。Surface与content借用同一spec；局部阶段即消除owned key、第二次style selector和重复metrics读取，最终由PERF-MVP-178 compiled segment在stable generation完全复用。真实icon优先asset handle，fallback多segment仍回链PERF-MVP-179/186。

## 动态验收

在1/100/10,000 normal/hover/pressed/focused/disabled、primary/danger/add/dropdown/tab/asset-toolbar按钮上记录key String allocations/bytes、kind/glyph/role probes、style selector calls、palette/metrics/preferences reads、text measurements与commands。Changed button要求owned key=0、classification/style/label/measurement各<=1、theme snapshot<=1；stable generation均为0。保持现有29文件覆盖的state priority、declared style、pressed offset、indicator ordering、icon-label gap、text family/weight/centering、asset glyph与GPU/Softbuffer pixels parity。
