---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - notification row state tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - 1/100/10000 notification unread/visible/clone/text/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template notification center逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_notification_center*`共 **11/11** 个Rust文件、**501** 行已逐文件阅读。覆盖center identity/open gate、header/unread/empty text、fixed row layout、panel/row commands、severity/state style及row style tests。Current-source baseline的本组测试未失败，但没有notification规模/产品trace，故仍留在`pending.md`。

## P0：消息风暴下双全量clone、晚裁剪和无可见范围

Header每paint调用`notification_rows`，对全部structured options执行owned `row_data` clone以统计unread；随后row loop再次clone全部options。Row painter才执行`intersect`，因此offscreen notifications仍被clone，且循环次数随完整历史线性增长。每个可见option已经是owned clone，title与description又各复制String。当前没有固定容量、visible range、unread generation或overflow/eviction证据。

PERF-MVP-208要求EditorLayout09的数据owner发布immutable notification generation、unread count和有界保留/overflow策略；更新同一generation只做一次。EditorUI08消费该metadata，按固定row stride直接计算visible+overscan range，在`row_data`前裁剪，并为changed visible row编译shared title/description commands。不得在painter建立第二个独立通知真相或每帧扫描历史来算header。

## 动态验收

在0/1/100/10,000 notifications、1,000-message burst、30秒stable open/closed center及scroll top/middle/end记录retained count、generation/unread builds、row_data clones、visited/offscreen rows、String bytes、commands、frame p95与memory。每message amortized O(1)或有界batch，retention有明确上限/overflow语义；同generation unread scan=0，stable open build=0，closed全部paint work=0；changed open visited=visible+overscan且offscreen clone=0。保持unread/header、severity/state、ordering、scroll、focus、clip和pixels parity。
