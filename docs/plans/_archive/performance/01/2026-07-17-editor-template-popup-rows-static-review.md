---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_row_adornments.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_row_adornments
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - popup option/menu state/style/flag/adornment/geometry/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - clipped-row and 1/100/10000 popup clone/flag/theme/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template popup rows逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_popup_row_adornments*`、`template_popup_rows*`与tests共 **45/45** 个Rust文件、**1,624** 行已逐文件阅读。覆盖menu/option row iteration、state/style/surface/text/shortcut/separator、flag/adornment parsing、manual glyph geometry及state/style/adornment/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，clipped-row与规模trace尚未完成，因此仍留在`pending.md`。

## P0：裁剪判断过晚和重复flag/adornment解析

Menu与option循环对全部row先执行owned `row_data` clone、row style与content projection，leaf函数才用`intersect`判断不可见。长popup即使只有少量可见行，也为offscreen行支付clone、flag/style和geometry成本。Menu adornment每行计算两次：一次决定label reserve，一次实际paint；每次从raw重复扫描submenu/icon flags，`icon=` value分配String后lowercase，default label也分配lowercase。Label/shortcut再分别复制String，可见普通行约3–4次metrics投影。

手工adornment本身发出2–5个quad且folder/save style会再读palette。PERF-MVP-205先把row frame与clip判定移动到`row_data`/style之前、只计算一次typed adornment，并用allocation-free ASCII case matching解析flags/icon/default label。最终changed visible row构建`PopupRowPaintSpec`，包含typed state/adornment、单一theme/metrics/style snapshot、shared text/shortcut和geometry/commands；PERF-MVP-177统一visible range，PERF-MVP-178保留compiled popup segment。

## 动态验收

在1/100/10,000 menu/options、5/20/1000 rows、visible+overscan/offscreen及hover/press/focus/selected/checked/loading/disabled/danger/submenu状态记录visited/row_data clone、flag scans、lowercase/String bytes、selector/theme/metrics、label/shortcut/build和adornment/total commands。局部修复后offscreen row_data/style/flag/adornment/build=0，visible adornment classify=1且临时String/lowercase=0；stable generation全部=0。保持row order/separator、popup bounds、state priority、danger/shortcut/adornment identity、clip/z和GPU/Softbuffer pixels parity。
