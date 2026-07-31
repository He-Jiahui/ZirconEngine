---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-component-catalog-deep-clone
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/component/catalog/registry.rs
  - zircon_runtime/src/ui/component/catalog/palette_view.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_editor/src/ui/asset_editor/palette
tests:
  - shared registry pointer identity test
  - 10k descriptor lookup and palette rebuild clone counter
  - registry revision and custom registration compatibility test
---

# Runtime UI component catalog每次读取深clone

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/component/catalog` 32/32与产品调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 交接原因：catalog ownership、custom descriptor合并和palette revision缓存属于EditorUI06组件库合同。

## 失败现象与复现证据

PERF-MVP-264：两个`OnceLock`只避免重建，却因owned工厂在每次调用深clone完整registry。asset native-slot查询和showcase每事件原先直接命中。本轮已增加shared accessor并切换三条热调用，compiler default及组合registry仍复制。

## 最低共享层根因

API没有区分immutable built-in catalog view与需要custom registration的owned registry，所有consumer只能选择昂贵clone。

## 架构修复验收

- built-in catalog以`Arc`/static generation handle发布；只读lookup clone bytes=0。
- custom overlay registry只存delta并借用base，或在明确builder边界一次materialize；revision由base+delta决定。
- palette entries按registry revision+host capability缓存，稳定revision不重建/排序；输出ownership清晰。
- 1/100/10k lookups/events/compiler instances记录registry/descriptor/String/Vec clone、palette rebuild和CPU p95；注册覆盖与current-source Cargo通过。

## 禁止临时方案

- 不得泄漏新registry获得`'static`，也不得建立无失效约束的第二全局catalog。
- 不得删除owned API而破坏custom registration；必须显式区分read-only与builder路径。

## 修复结果与回传

Open state: `三条热调用已共享；等待EditorUI06回传compiler/custom overlay ownership、palette revision cache与规模counter`。
