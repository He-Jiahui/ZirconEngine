---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/85/2026-09-21-resolved-dependency-output-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/865-borrowed-handwritten-meta-dependency-index.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs
tests:
  - zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/optimization_batch_runtime866_resolved_dependency_capacity_tests.rs
  - tools/tests/test_runtime866_resolved_dependency_output_capacity_performance_contract.py
---

# Runtime866 Resolved Dependency Output Capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime85 dependency resolution outputs | Reserve the authored dependency bound for ordered resolved IDs and lazily reserve the same bound for diagnostics only on the first missing locator, preserving lookup, first-seen deduplication, order, diagnostic text, and empty/all-success diagnostic zero capacity. | Intentional RED `2/5` → GREEN `5/5`; lower empty/missing-path regression and ignored `RUNTIME866_RESOLVED_DEPENDENCY_OUTPUT_CAPACITY_BENCH_V1` marker are wired. The 4,096-value model changes geometric growth `11→0`; Runtime864/865/866 contracts pass `13/13`. Managed Cargo/Release, allocator, and project-import p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs` | `A1207221C7FB47BA6F8595843484D73CD723C3CC7876C3449537213FF3D03A73` |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/optimization_batch_runtime866_resolved_dependency_capacity_tests.rs` | `A358CA0A29F73A1AEBF7E0BC261634104DEF2E075B143E79D164BEF338A556D5` |
| `tools/tests/test_runtime866_resolved_dependency_output_capacity_performance_contract.py` | `26C5042A0F1A4DDFD4CC52281555F0D648F52A87DB65AB21DA7588C61AE9D347` |

## Managed gate

Runtime866 landed after v8 and has not been submitted alone. Keep it pending
until a later multi-task current-source lane supplies Runtime compilation,
lower/ignored Release execution, allocator evidence, and project-import product
percentiles.
