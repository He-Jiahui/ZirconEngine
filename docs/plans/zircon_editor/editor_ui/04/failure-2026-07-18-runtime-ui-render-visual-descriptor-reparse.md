---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-render-visual-descriptor-reparse
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime/src/ui/tests/material_layout
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/node_visual_data.rs
  - zircon_runtime/src/ui/surface/render/painter_state.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
tests:
  - stable generation zero-TOML-visual-parse test
  - one-percent painter-state delta allocation test
  - theme and hot-reload pixel parity test
---

# Runtime UI render每帧重解析visual/style descriptor

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：render resolve/painter及全部specialized consumers
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md`
- 联动责任：EditorUI05在compile artifact写入typed visual contract；EditorUI08消费generation handle。
- 交接原因：computed style、painter family/state和theme brush ownership属于EditorUI04。

## 失败现象与复现证据

PERF-MVP-289：每node多次TOML查字段并复制font/color/language，suppression与special render重复分类，每原子command再复制style/String。本轮已删除多类lowercase分配并让Button/Dropdown/Segmented局部一次解析。`tests/material_layout/**`的field/value/placeholder/options与icon render断言可作为typed visual/text descriptor cutover parity，但小树测试没有lookup/String clone规模计数。

## 最低共享层根因

compiled UI节点没有typed visual descriptor或interned brush/font handle，render从通用TOML恢复静态契约。

## 架构修复验收

- compile/style generation发布family、behavior/suppression mask、brush/font/text/options handles。
- stable render TOML lookup、classification、constant color/font String alloc=0。
- hover/focus/press只更新compact painter-state delta，不复制静态style。
- 1/100/10k nodes×1% change记录lookups/classifiers/style bytes与CPU p95；theme/hot-reload和全部alias像素/Cargo通过。

## 禁止临时方案

- 不得为每个special renderer建立私有TOML cache。
- 不得把String换成另一个owned DTO而继续每帧全量投影。

## 修复结果与回传

Open state: `等待EditorUI04回传compiled visual descriptor、interned style handles与stable-generation证据`。
