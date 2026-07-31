---
related_code:
  - zircon_runtime/src/ui/tests/asset_schema_migration.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - six schema migration semantic tests reviewed
  - one cross-file source-level RED to GREEN single-parse routing guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, source-size scale counters and F4 migration trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset schema migration测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`asset_schema_migration.rs`原始257行/6测试；加入1项跨文件源码性能守卫后为269行/7测试。范围覆盖current/older/future/below-minimum tree、flat node table与legacy source-template fixture迁移语义。

## PERF-MVP-310：source按shape重复TOML decode

原`migrate_toml_str`先把源解析为`toml::Value`判型，再用完整源解析header，随后tree/flat/source-template各自再次`toml::from_str`；常见tree一次load执行3次syntax decode。新增守卫先确认RED，再让routing把同一个owned Value移交给tree/flat/source-template typed deserializer，仅clone小`[asset]` Value取得header；旧的未使用flat-string helper删除，守卫转GREEN。第57组局部优化保持version policy、migration steps、tree authority和错误优先级。

flat materialization本身仍从node table重建recursive tree，deep path/child查找和最终authority validation另有全图工作；source fixture conversion也保留转换后的validation。这些共享single-parse arena/index目标继续归PERF-MVP-310/311和EditorUI05。

## 验收要求

对1 KiB/1 MiB/10 MiB current-tree、flat与source-template输入记录syntax parse count、serde value visits、temporary bytes、tree build visits、RSS及migration p50/p95。每次`migrate_toml_str` syntax decode=1；header只复制`[asset]`子值；各source kind输出与错误顺序保持一致。当前源码7项Cargo、规模counter与F4旧资产打开/保存重开trace完成前，本文件留在`pending.md`。
