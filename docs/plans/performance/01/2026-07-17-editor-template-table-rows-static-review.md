---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - table row identity/style/cell normalization/column allocation/geometry/action asset/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - visible-range and 1/100/10000 row-column allocation/measurement/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template table rows逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_table_rows*`与tests共 **28/28** 个Rust文件、**2,052** 行已逐文件阅读。覆盖table/header/tail identity、state/style/surface、option/archived cell normalization、column allocation/alignment/clipping、action slot/asset/fallback glyph及cell/geometry/style/action/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，visible-range与规模trace尚未完成，因此仍留在`pending.md`。

## P0：rows乘columns放大的重复列布局和文字测量

`push_table_cells`先取一次cell metrics，但每个cell又调用`table_cell_rect`；该函数为每列重新获取cell metrics、action metrics和column metrics，并重新计算整行4列布局。`table_column_metrics`还每次测量Name/Type/Size/Revision四个固定样本。因此一行4个cell会重复4次相同列分配和16次固定样本文字测量，numeric cell再各测一次实际值。cell color逐列完整select style，surface三次、action一次，4列行合计可达约8次selector。

Option cells先取得owned `row_data`后再`to_string`二次复制，随后normalize再次为每cell分配String；archived fallback先收集token Vec再分配最多4段String。Action geometry/slot也多次投影metrics/theme，并在可见action上逐行解析asset；测试已要求settings/more-horizontal真实资产命中且禁止missing-icon。

PERF-MVP-203要求changed row只构建一次`TableRowPaintSpec`：typed kind/state、shared/typed cells、一次column layout、一次theme/metrics/style snapshot、每列geometry/color/text measurement与action asset handle。固定header minimum widths只按font/metrics generation计算一次。PERF-MVP-177提供visible range，PERF-MVP-178保留compiled row segments，PERF-MVP-181拥有action resource generation。

## 动态验收

在1/100/10,000 rows、1/2/4 columns、regular/narrow widths、visible+overscan/offscreen及header/normal/hot/selected/disabled状态记录row/cell visits、column layout、fixed/actual text measurements、row_data/String bytes、selector/theme/metrics/resource/build/commands。changed visible row column layout=1、fixed sample measurement=0（稳定theme）且各projection<=1；stable visible以上及build=0；offscreen全部=0；shipped action fallback=0。保持column drop/minimum/right alignment、normalization、state/action visibility、clip/order和GPU/Softbuffer pixels parity。
