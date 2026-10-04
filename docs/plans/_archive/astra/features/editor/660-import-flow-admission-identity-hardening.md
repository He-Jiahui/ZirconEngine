---
related_code:
  - zircon_editor/src/core/asset/import_flow/error.rs
  - zircon_editor/src/core/asset/import_flow/flight.rs
  - zircon_editor/src/core/asset/import_flow/state.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
doc_type: milestone-detail
status: implemented_pending_validation
---

# Import Flow Admission Identity Hardening

The import-flow fast path no longer wraps its mutex-group, flight, UUID-lifecycle, or per-UUID
active-flight identities. A final valid `u64::MAX` identity is admitted once; subsequent requests
return a typed rejection before publishing partial shared state. Active-flight count overflow is
also rejected before insertion.

Coalesced import reasons now use an inline `AtomicU8` bitset rather than a mutex-protected
`BTreeSet`. Snapshot order remains Watch, DigestMismatch, then Manual, and repeated reasons remain
deduplicated. This removes allocation and mutex contention from the common shared-flight reason
path without changing the public result ordering.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor04/import-flow admission | Reject exhausted identities before state publication; use inline atomic reason coalescing | implemented_pending_validation | Scoped Rustfmt and diff checks pass. Local static Editor performance contracts `580/580` and focused import/export contracts `21/21` pass. Coordinator batches `addd4cbc81ac41a593392c3be3b49dd5` and `1969ede38024473f9e960fcb1a55a9e9` were queued without polling. Managed Editor Cargo and release p50/p95 performance evidence are still pending because the external `E:/Git/zr_vm` worktree is dirty. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_editor/src/core/asset/import_flow/error.rs` | `F23B734B38175C686CFD93D51632FADAA64BA8D26FB4F8739310F5C7C185C6FA` |
| `zircon_editor/src/core/asset/import_flow/flight.rs` | `160FA41AA22E91232F848AF0F8E84E72E1212C465C6CAE124FFA06BAD5A82108` |
| `zircon_editor/src/core/asset/import_flow/state.rs` | `702C627585D52529C981FBD239E4F0EEB2DCF39B2AF322EE553E2658CE916A5C` |

The status must not be promoted to performance-qualified until a managed release run records the
relevant p50/p95 evidence against this source snapshot.
