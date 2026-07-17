---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: timeline-tick-generation-unbounded
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/07
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_timeline_strip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_timeline_strip/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_timeline_strip/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_timeline_strip/keys.rs
---

# Timeline tick generation unbounded

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_timeline_strip*` 11/11 个 Rust 文件
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：直接终止/数量修复由performance会话落地；timeline generation、预格式化ticks/labels与scrub invalidation属于domain editor长期所有权。

## 失败现象与复现证据

`timeline_ticks`无上限执行`time += interval`并增长Vec；极小interval导致巨量主线程工作和内存，浮点addition不再前进时永久循环。Surface/text每paint各调用一次并分配两份tick Vec，text随后逐tick format。Playhead scrub还重建全部static ticks/text/keys。

## 最低共享层根因

Domain projection没有发布bounded immutable timeline generation、预格式化ticks/labels与static/dynamic invalidation；painter自行从duration/interval推导并拥有列表。

## 架构修复验收

- 立即使用integer-indexed、plot pixel columns+hard cap约束的单次tick list，invalid/subnormal输入必定有界终止。
- Editor07发布typed timeline generation与预格式化tick/track labels；scrub只更新current-time generation。
- EditorUI08拆static surface/ticks/text/keys与dynamic playhead/selection segments。
- 1/100/10,000 keys、300 stable frames、1,000 scrubs报告tick/format/build/alloc/command/CPU p95。
- 保持endpoint、footer percent、progress、selected key、ordering、clip与Softbuffer/RenderDoc pixels等价。

## 禁止临时方案

- 不得只在while内部加入任意迭代次数而保留双Vec和累计浮点漂移。
- 不得在painter建立第二份无domain generation约束的timeline cache。
- 不得让playhead变化继续重建/format static ticks和labels。

## 修复结果与回传

Open state: `待performance会话回传直接bounded tick修复；待Editor07回传timeline generation与scrub/static-dynamic counters`。
