---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-26-export-stage-diagnostic-hash-merge.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/938-editor09-export-wizard-session-hash-index.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
tests:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model/stage_diagnostics_tests.rs
---

# Editor939 Editor09 Export Stage Diagnostic Hash Merge

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Diagnostic merge | 阶段诊断先建立 borrowed `HashSet`，再单次扫描 execution diagnostics；保留 progress 优先、首次出现顺序和跨/列表内去重语义。 | 行为测试覆盖既有顺序、重复项与 append 顺序；source contract 禁止恢复 `diagnostics.contains` 重复扫描。 |
| 性能门禁 | 4,096 条既有与 4,096 条 execution diagnostics 从二次比较降为索引构建加 hash probe。 | ignored marker `EDITOR09_EXPORT_STAGE_DIAGNOSTIC_HASH_MERGE_BENCH_V1` 要求 P95 至少降低 75%；managed Editor09 Release 回执仍待定。 |

## 本地验证

- Editor09 Python 合同批量保持 `68/68`。
- Rustfmt、行为/source contract 已随 Editor09 包测试纳入；未修改 tooling。
