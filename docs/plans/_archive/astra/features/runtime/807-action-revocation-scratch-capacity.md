---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-19-action-revocation-scratch-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/dynamic_api/session/runtime_ui/action_requests.rs
tests:
  - tools/tests/test_runtime_ui_action_revoke_capacity_performance_contract.py
---

# Runtime807 · action revocation scratch capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime200 action queue | Keep the revoked secure-reference vector at zero capacity until the first real revoke, then reserve the remaining component-event bound through a shared helper; preserve all queue, redaction, rejection, supersession, and ordering semantics. | TDD source/model contract `4/4`; lower Rust source regression covers lazy start, remaining-event bound, and first-append reservation; merged non-tooling batch `873` files / `3677` tests / `0` failures / `0` errors / `0` skips / `205.214s`; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Complexity boundary

The helper removes geometric growth on a revocation-heavy component-event batch
without allocating this revocation scratch on the normal no-revocation path. Its
bound is one secure reference per remaining event, and the reserve occurs only
when an actual value is appended. It does not change action admission, queue limits, secure payload
redaction, supersession, serialization, public DTOs, or host receipts. The
deterministic 4,096-event model is allocation-shape evidence, not product
latency, allocator, RSS, or p50/p95/p99 acceptance.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/action_requests.rs` | `28157E2D6AD6610AE5594E106A897BD9AB6B6A4550F002823F4D36380BC672FB` |
| `tools/tests/test_runtime_ui_action_revoke_capacity_performance_contract.py` | `296E381F1A2FA169354C8919C8B269D5B750026BF85A1C16E36B358717BBF4AD` |

## Managed gate

No Cargo process was started locally. The shared owner-attributed Windows batch
still has the external dirty `E:\Git\zr_vm` admission blocker; managed compile,
allocation, and input p50/p95/p99 evidence remain open. This record is the
Runtime feature completion list for the slice and is not a product-performance
claim. Tooling production remains deferred and coordinator status is not polled.
