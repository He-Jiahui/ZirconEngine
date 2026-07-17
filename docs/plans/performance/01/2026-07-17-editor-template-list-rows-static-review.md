---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_row_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_row_glyphs/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - list row identity/style/density/adornment/asset/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - visible-range and 1/100/10000 row selector/resource/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template list rows逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_list_row_glyphs*`、`template_list_rows*`与tests共 **20/20** 个Rust文件、**1,038** 行已逐文件阅读。覆盖list-row identity/state/style、surface/selection indicator/text、adornment kind/asset/fallback geometry及identity/style/density/asset/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，visible-range与规模trace尚未完成，因此仍留在`pending.md`。

## P0：每行重复完整selector与资源解析

一行paint会为background、border、border width、text、adornment color和adornment kind分别执行完整list-row selector，约6次state/theme projection；disabled adornment另取一次row palette。Surface、text和adornment geometry约3次完整metrics投影，label复制一次，且每个adornment都进入视觉资源解析路径。现有测试明确要求shell check/chevron/disabled资产命中且禁止missing-icon，产品路径因此不应常态执行3–4 quad手工fallback。

PERF-MVP-202要求changed row一次构建typed `ListRowPaintSpec`，包含identity/adornment/state、单一style/theme/metrics snapshot、shared label、asset handle和surface/indicator/text/adornment geometry。PERF-MVP-178保存compiled segment，PERF-MVP-177提供与input一致的visible range，PERF-MVP-181拥有资源generation。稳定且offscreen行不得执行selector、label、resource resolve或command build。

## 动态验收

在1/100/10,000 rows、visible+overscan/offscreen、normal/hover/focus/press/selected/checked/disabled/loading状态记录visited rows、selector/theme/metrics acquisition、label bytes、resource resolve/cache/fallback、command build与draw。changed visible row各projection<=1；stable visible行以上及command build=0；offscreen行全部=0；shipped adornment fallback=0。保持density、state priority、selection indicator、asset tint、clip/order和GPU/Softbuffer pixels parity。
