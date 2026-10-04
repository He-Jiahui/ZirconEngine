---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/91/2026-09-21-shader-dependency-direct-append.md
related_records:
  - docs/plans/astra/features/runtime/866-resolved-dependency-output-capacity.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs
tests:
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies/optimization_batch_runtime867_shader_dependency_direct_append_tests.rs
  - tools/tests/test_runtime867_shader_dependency_direct_append_performance_contract.py
---

# Runtime867 Shader Dependency Direct Append

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime91 shader dependency publication | Append uniquely owned provider locators directly into the retained dependency vector, reserving the authored import bound and removing the full temporary locator vector while preserving provider order, ID deduplication, ambiguous-provider rejection, and the metadata/runtime ownership boundary. | Intentional RED `1/5` → GREEN `5/5`; lower metadata-duplicate/order regression and ignored `RUNTIME867_SHADER_DEPENDENCY_DIRECT_APPEND_BENCH_V1` are wired. The 4,096-provider model changes temporary locator slots `4096→0`; the combined Runtime866–868/glTF batch passes `19/19`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs` | `3C037FFADB7F7A94F8AC2B578FD66EC5444831C54FDE388974692A5926823B87` |
| `zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies/optimization_batch_runtime867_shader_dependency_direct_append_tests.rs` | `EBA53B5EAA8FEEE7EB934B59B81A3BA96EFF3BC9EB583CC25DAB428364AFA9DC` |
| `tools/tests/test_runtime867_shader_dependency_direct_append_performance_contract.py` | `5FF0B49E047EE2C556A3EC6DD68171C2970D62D02DE319F546D4FD324F5E6F68` |

## Managed gate

Keep Runtime867 pending until the combined Runtime866–868 Windows lane supplies
Runtime compilation, lower/ignored Release execution, allocator evidence, and
project-import product p50/p95/p99 percentiles.
