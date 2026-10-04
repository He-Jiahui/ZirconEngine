---
title: Runtime200 action revocation scratch capacity
category: zircon_runtime
report_id: Runtime807-action-revocation-scratch-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime807 · action revocation scratch capacity

## Scope

`RuntimeUiActionRequestQueue::record_result` returns at most one revoked secure
reference for each component-event report, but the revocation vector previously
grew from zero whenever an invalid, undelivered, oversized, or backpressure
event was encountered. This slice keeps the revocation scratch at zero
capacity on the normal no-revocation path and, on the first actual revocation,
reserves the remaining component-event upper bound. Queue limits, secure redaction/revocation,
supersession, ordering, and host-request admission are unchanged.

## Implementation

- `zircon_runtime/src/dynamic_api/session/runtime_ui/action_requests.rs`
  keeps `revoked_secure_values` at zero capacity until a value is actually
  revoked.
- `append_revoked_secure_value` reserves
  `result.component_events.len() - action_index` (saturating) only on the first
  non-empty append, then performs the existing push operation.
- All existing revoke sites route through the helper, including undelivered
  secure events, secure-payload rejection, queue-full rejection, serialization
  failure, and encoded-byte rejection.
- The lower Rust source regression asserts the lazy zero-capacity start,
  remaining-event bound, and first-append reservation. No Cargo command was
  started; the managed Windows batch remains the owner of compile execution.

## TDD and deterministic work model

- The Python source/model contract was run RED against the old zero-capacity
  append shape, then GREEN after the helper and lower regression were added.
- For 4,096 component events, the legacy zero-capacity vector model performs
  geometric growth on an all-revoked path; the lazy bounded path reserves once
  on the first revoke and models zero growth events. With no revoked value, the
  helper returns before reserving, so modeled reserved capacity remains zero.
- This is allocation-shape evidence for the revocation scratch only; it does
  not claim allocator, RSS, queue-age, or product p50/p95/p99 acceptance.

## Validation

- Focused Python source/model contract:
  `tools/tests/test_runtime_ui_action_revoke_capacity_performance_contract.py`
  (`4/4`).
- One merged non-tooling contract invocation loaded `873` files and passed
  `3677/3677` tests with zero failures, errors, or skips in `205.214s`; this is
  local source/model evidence rather than a managed Cargo or product-percentile
  receipt.
- Exact-file `rustfmt --edition 2021 --check` passes for the production source.
- Managed Windows Cargo/Release and product percentile evidence remain pending
  under the shared external-worktree admission gate. No coordinator status was
  polled or monitored.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/action_requests.rs` | `28157E2D6AD6610AE5594E106A897BD9AB6B6A4550F002823F4D36380BC672FB` |
| `tools/tests/test_runtime_ui_action_revoke_capacity_performance_contract.py` | `296E381F1A2FA169354C8919C8B269D5B750026BF85A1C16E36B358717BBF4AD` |

## Remaining parent work

Runtime200 continues to own the broader UI input authority, typed terminal
receipts, queue backpressure, secure host acknowledgement, lifecycle/session
identity, and managed product input latency gates. This narrow scratch-capacity
slice does not close those architectural or managed-performance requirements.
