---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-instance-upload-hash-membership.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/planning/build_instance_upload_plan.rs
tests:
  - zircon_runtime/src/graphics/visibility/planning/build_instance_upload_plan.rs
---

# Runtime949 · instance-upload hash membership

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b visibility-to-upload planning | Dynamic stable keys and incremental dirty-key admission use `HashSet<u64>` membership while published static, dynamic, and dirty vectors continue to follow BVH/input order. Full rebuild, incremental filtering, and unrelated-key rejection are unchanged. | Current-source Release test binary completed the full focused owner batch: `3 passed; 0 failed`, including order preservation, HashSet source contract, and ignored `RUNTIME09B_INSTANCE_UPLOAD_HASH_MEMBERSHIP_BENCH_V1`. The latest grouped owner marker reports ordered P95 `2,639,000ns` versus hash P95 `1,077,100ns` (`59.18%` reduction), clearing the plan's `40%` reduction requirement. Managed allocation/product evidence remains pending. | implemented_pending_validation |

## Deterministic boundary

The planner constructs dynamic keys in the original BVH order, derives dirty
membership from inserted and updated keys, and filters the original dynamic
vector to publish dirty keys. Hash-set iteration order is never published.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/planning/build_instance_upload_plan.rs` | `26E3BC4E36A96A779BF4E13AC5366EABC542B92565AEB48078D1939868707C9D` |

## Validation handoff

The current-source Release binary at
`F:\\codex-targets\\zircon-engine\\runtime09c-graphics-batch-20260926`
completed the three-owner selector in one invocation. This is local Release
evidence; the final-source grouped managed Runtime/Editor wave (Runtime PTY
`54276`, Editor PTY `15395`) remains the authoritative package validation lane.
No coordinator status was read, and no per-task Cargo run was started.
