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

Open state: `current source 已具备 pixel-column + hard-cap 的整数索引 tick admission；Editor07 已补齐 immutable timeline generation、预格式化 tick label 与 static/dynamic identity。仍待受管 Rust 验证、EditorUI08 static/dynamic compiled cache、Render13 bounded batch 与 1/100/10,000 keys 的完整规模证据，当前不得 fixed return`。

## 产出记录与时间

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
|---|---|---|---|
| 2026-08-08 | Timeline generation hard cut | 代码完成 / 二次审查 0/0/0 / failure 保持 `open` | 新增 `ui/timeline_strip` 作为唯一领域 owner：`TimelineStripGeneration` 持有不可变 keys、预格式化的 visual-budget tick 内容、static/dynamic generation identity；`TemplatePaneTimelineStripData` 删除 raw duration/current-time/tick-interval/`ModelRc` 字段。投影一次性构造 generation，painter 只消费 typed slices，不再自行格式化或读 ModelRc。审查揭示的跨 reprojection static reuse、key static-vs-selection dynamic identity 与 resize cache 无界问题已前向修复为按 `static_generation + visual_budget` 键控的 16-entry LRU，miss 在锁外构建并二次入库；并补齐 hard cap、非法输入、跨 generation `Arc` 复用与缓存上界回归。静态合同 RED 5 项后 GREEN 5/5；与 sample-grid、preview hit-index 合并为 12/12；Python 语法检查、Rust 逐文件解析、旧 consumer 零命中和 scoped `git diff --check` 已通过。两轮独立复审均为 0/0/0。受管 Cargo、EditorUI08 compiled cache、Render13 bounded batch 与完整规模证据仍待执行，故不关闭 failure。 |
