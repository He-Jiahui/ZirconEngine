---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-native-window-stale-id-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
tests:
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
  - tools/tests/test_editor874_native_window_stale_identity_capacity_performance_contract.py
---

# Editor874 - native-window stale-ID capacity

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Native presenter topology | Lazily reserve the current-window upper bound on the first stale match, preserve BTree retirement order, and keep stable topology at zero capacity. | RED→GREEN `4/4`; lower order/empty semantics; ignored `EDITOR874_NATIVE_WINDOW_STALE_ID_CAPACITY_BENCH_V1`; adjacent native projection batch `38/38`; non-Tooling batch `4204/4204` across `990` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until current-source Windows Release, allocator, marker, and native
topology-sync p50/p95/p99 evidence pass.
