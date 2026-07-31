---
related_code:
  - zircon_runtime/src/ui/tests/ecs_projection.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - 6 ECS projection/delta/schedule tests reviewed
  - interaction-only changed-node result semantics present
  - 7 full projection and 12 from-previous helper calls represented
  - full-project-before-delta counters and current-source Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI ECS projection测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/ecs_projection.rs` 1/1个tracked Rust文件、344行、6个测试。测试覆盖node dirty/interaction/render facts、effective disabled、frame/debug ownership、structure+interaction delta、interaction-only分类及schedule/domain impacts。

## PERF-MVP-278：稀疏结果不等于稀疏计算

interaction-only测试最终只返回1个changed node且不要求Layout/Picking，这是正确的schedule语义；但`ui_ecs_projection_delta_from`先重新物化完整current projection，再与previous比较。各component/interaction/render-only ids、schedule impacts与domain impacts helper也经相同入口重复计算。本文件代表7次full projection与12次`*_from(previous)`调用，却没有projected node visits、map build、clone bytes或helper cache计数。

## Frame/debug继续复制同一projection

测试要求`surface_frame`、`debug_snapshot`与即时`ui_ecs_projection()`三份owned值完全相等，证明当前多份projection authority在小树上语义一致；它也固化了每次frame/debug access复制完整nodes/totals/impact Vec的成本。EditorUI08应在rebuild时发布generation-owned immutable projection handle，dirty changed set直接产出delta；helper借用同一delta artifact，不先构造两份full snapshot。

## 验收要求

1/1k/10k nodes、stable/单interaction/单render/structure change连续1k次记录full projection builds、projected/compared nodes、map/Vec/String clone bytes、delta helpers、generation hits及CPU p95。stable access full build/clone=0；interaction-only访问changed set和必要owner path，不随N增长；同generation所有helper共享一次delta。current-source Cargo与workbench ECS schedule trace完成前，本文件留在`pending.md`。
