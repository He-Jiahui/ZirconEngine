---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-09-09-manifest-bound-and-root-admission.md
related_code:
  - zircon_runtime/src/asset/project/manifest/load.rs
  - zircon_runtime/src/asset/project/manifest/validation.rs
  - zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs
tests:
  - tools/tests/test_runtime85_project_root_dedup_performance_contract.py
  - tools/tests/test_astra_benchmark_evidence_scope_contract.py
  - zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs
---

# Manifest Bounded Root Admission

Runtime04 now rejects oversized project manifests before TOML/JSON materialization, bounds
file reads to the shared manifest budget, and admits at most the interface-defined number of
asset roots. Root overlap detection uses a borrowed path-boundary index while preserving the
previous input-order error result; UI-root duplicate checks borrow canonical paths instead of
formatting temporary URI strings.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime04/manifest admission | Bound load/read growth and replace quadratic root-overlap checks with capacity-aware borrowed indexes | implemented_pending_validation | Static Runtime/Astra contract batch `7/7` and the latest cross-slice Runtime manifest/Editor export source-contract batch `42/42` pass; scoped `rustfmt --check` and `git diff --check` pass. The release helper is intentionally ignored; managed Runtime Cargo plus release p50/p95/p99 evidence remain pending. |

## Validation boundary

The managed validation batch cannot currently be admitted because the external
`E:/Git/zr_vm` checkout is dirty. This record makes no product-performance, allocation, RSS,
or release-percentile claim. It must remain `implemented_pending_validation` until a managed
release run validates this source snapshot.
