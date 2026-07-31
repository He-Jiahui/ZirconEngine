---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: sample-grid-command-amplification
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/07
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sample_grid.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sample_grid/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sample_grid/points.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sample_grid/text.rs
---

# Sample grid command amplification

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_sample_grid*` 11/11 个 Rust 文件
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：sample-grid tick/label/point generation与交互更新边界属于domain editor；EditorUI08/Render13只消费typed generation和提供batch primitive。

## 失败现象与复现证据

默认360×260、x/y各5 ticks的静态grid把每条虚线按3px dash展开为数百HostPaintCommand quad。每个diamond再展开9/13条scanline，selected marker可达约25条；tick和labels每paint重新format/copy。Stable frame和单点drag都重建完整surface/grid/text/points。

## 最低共享层根因

Domain projection没有发布immutable sample-grid generation、预格式化ticks/labels与静态/动态point identity；painter只能重新遍历models并用通用quad表达线段/marker像素片段。

## 架构修复验收

- Editor07发布typed ticks、预格式化labels、points与generation；selection/drag只改变必要dynamic generation。
- EditorUI08按generation编译static surface/grid/text与dynamic selected/drag segments。
- Render13/host dashed-line/marker使用有界batch/geometry command，不按dash或scanline扩高层命令。
- 1/10/100 ticks、1/100/10,000 points、300 stable frames和1,000 drags报告build/format/copy/host-compiled-RHI command/vertex/batch/CPU p95。
- 保持invalid range、selected label clamp、zero axes、ordering、clip与Softbuffer/RenderDoc pixels等价。

## 禁止临时方案

- 不得只增大dash长度或缩小point radius来隐藏命令数。
- 不得在painter建立第二份无generation约束的grid cache。
- 不得让stable frame继续format tick String或让point drag重建static grid。

## 修复结果与回传

Open state: `Editor07 typed generation 与 static/dynamic identity 已于 2026-07-18 实现，静态合同 5/5；仍待受管 Rust 验证、EditorUI08 generation cache/scale counters、Render13 bounded dashed-line/marker batch 及完整规模证据，当前不得 fixed return`。

- 已完成的最低层修复：template attribute projection 构造唯一 `SampleGridGeneration`；tick label 只在 generation 构造时格式化；axis labels/ticks 进入 static content token，point position/label/selection 进入 dynamic content token；range 因同时影响 grid 与 point projection 而进入两个 token。
- 已完成的 hard cut：`TemplatePaneSampleGridData` 删除原始 axis/range/`ModelRc` tick/point 平行字段，host painter 与既有投影/视觉测试只消费 typed generation slices，不保留兼容 accessor。
- 尚未关闭：host 仍以多个 quad 展开 dashed line 和 diamond scanline；generation-aware compiled command cache、batch primitive、300 stable frames/1,000 drags 和 1/100/10,000 规模报告归下游验收，不能用本次静态门替代。
