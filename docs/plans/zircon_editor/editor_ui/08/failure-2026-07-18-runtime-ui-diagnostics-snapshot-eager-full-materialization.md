---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-diagnostics-snapshot-eager-full-materialization
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/timeline.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
tests:
  - default summary zero-overdraw-grid test
  - timeline frame-and-byte-budget retention test
  - unchanged diagnostic generation zero-rebuild test
---

# Runtime UI诊断快照默认全物化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface diagnostics/reflection/timeline及Runtime Diagnostics consumer审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 联动责任：EditorUI09消费Runtime Diagnostics与Widget Tree Debugger产品模块。
- 交接原因：host diagnostics refresh、snapshot artifact与presentation lifecycle在EditorUI08。

## 失败现象与复现证据

PERF-MVP-280：默认capture构造全部node/property/action/count、command/hit/overdraw/overlay数据；overdraw按窗口网格和command分配，timeline读取深clone全部full snapshots。Runtime Diagnostics pane会构造surface/frame并立即生成debug snapshot。

## 最低共享层根因

诊断契约只有全量owned snapshot，没有轻量summary、section request、generation cache或byte budget；timeline frame-count cap不约束每帧payload大小。

## 架构修复验收

- 默认只产summary；commands/hit/overdraw/reflector/overlay按显式section request与frame work budget采集。
- selected-node属性/action支持按需或delta，unchanged generation rebuild=0。
- timeline存Arc artifact并有entry+byte+age上限/drop stats；UI刷新不clone全部历史，导出流式化。
- 1080p/4K、1/1k/10k nodes/commands、60/600 frames记录cell/temp/retained/returned bytes和CPU p95；显式完整capture parity通过。

## 禁止临时方案

- 不得只增大overdraw cell size掩盖默认无条件采集。
- 不得只限制timeline帧数而忽略单帧snapshot bytes和读取时二次clone。

## 修复结果与回传

Open state: `等待EditorUI08/09回传sectioned diagnostics、bounded timeline和产品刷新证据`。
