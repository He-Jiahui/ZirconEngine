---
related_code:
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/asset
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler
  - zircon_runtime/src/ui/template/asset/schema
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - thirty-three asset/compiler/migration/style semantic tests reviewed
  - one source-level RED to GREEN in-place reorder guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, authoring scale counters and F4 trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset core测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`tests/asset.rs`与`tests/asset/**`共7/7个tracked Rust文件；原始1,493行/33测试，`style_write_apis.rs`加入1项源码性能守卫后为当前1,513行/34测试。范围覆盖loader/tree/flat/legacy migration、component defaults、import/reference/slot/style compile、stable stylesheet/rule ID及全部authoring write API原子语义。

## PERF-MVP-306/307/311：小fixture不证明compile规模

主compile fixture只有3级节点、2 widgets、1 style；测试还主动clone compiled document再build surface。它锁定layout/render/slot/style parity，但没有prototype frame、selector probes、tree/metadata clone bytes或stage visits。compiled instance/tree/style generation继续归PERF-MVP-306/307/311和EditorUI04/05。

## PERF-MVP-310/311：authoring全量clone/validation

stable ID/write tests每次只有2 sheets/3 rules。原`move_style_rule`纯重排也clone完整stylesheets、重新校验并整体赋值。新增守卫先确认RED，再保留现有ID/selector validation barrier，校验通过后在原document内remove/insert，守卫转GREEN；missing rule/sheet、target index和错误语义不变。第60组局部优化删除reorder的全量stylesheet深复制。

insert/replace/set仍为了atomic failure复制完整sheet/rule graph并重扫所有selectors；style lookup/position/rename均线性扫描，direct mutable API又阻止简单跳过validation。EditorUI05需提供generation style/rule index与transactional patch arena，EditorUI04共享parsed selector；本轮不做破坏API的局部硬切。

## 验收要求

对1/100/10k sheets/rules/nodes/imports和1/100/1k authoring ops记录stylesheet/rule clone bytes、selector parses、ID/index probes、tree/style visits、undo payload bytes与edit/compile p95。move-only clone bytes=0；insert/replace failure保持原子；单generation selector parse<=1且lookup为indexed。当前源码34项Cargo、规模counter与F4 UI asset editor add/replace/move/undo/preview trace完成前，7文件留在`pending.md`。
