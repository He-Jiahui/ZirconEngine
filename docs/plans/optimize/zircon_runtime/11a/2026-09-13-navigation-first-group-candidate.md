---
title: Runtime navigation first-group candidate streaming
category: zircon_runtime
report_id: Runtime11A-navigation-first-group-candidate-2026-09-13
date: 2026-09-13
supersedes: docs/plans/astra/features/runtime/666-navigation-group-candidate-single-pass.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
followed_by: docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-candidate-bucket-reuse.md
---

# Runtime11A navigation first-group candidate streaming

## Scope

The navigation rebuild already visited retained nodes once, but it still
materialized a temporary `BTreeMap<UiNavigationGroupId, Vec<UiNodeId>>` to find
the first directional target for each group. Every candidate cloned the owned
group key and the temporary vectors were sorted before their first element was
copied into the retained map.

## Implementation

The primary node stream now maintains `first_candidate_by_group` directly. A
candidate compares against the current best with the existing deterministic
`compare_tab_nodes` ordering; only a newly observed group clones its key for
insertion, while an existing group updates its scalar node ID in place. The
temporary candidate map, per-group sort, and second traversal are removed.
Modal/spatial/tab candidate vectors and all manual-group query semantics remain
unchanged.

## Complexity and allocation boundary

For `F` focus candidates and `G` groups, first-target derivation is now one
stream with `O(F log G)` retained-map probes (plus the existing node-order
comparison lookups) and no temporary candidate vectors. The old per-candidate
group-key clones and the extra `O(F log F)` per-group sort are removed. The
retained `BTreeMap` still owns one key per live group and is cleared/rebuilt
with the existing index lifecycle; this slice does not alter the public
navigation contract or candidate ordering vectors.

## Validation

- TDD RED: the navigation source contracts rejected the temporary
  `group_candidates` map and required direct first-candidate updates.
- GREEN: the Runtime navigation contract passes `9/9`; the existing Rust source
  regression now checks the direct in-stream update and one-node traversal, and
  `rustfmt --emit stdout` parses the production module.
- The latest same-source batched Runtime/Editor UI performance-contract invocation loaded
  172 modules and passed `772/772` tests in `1.571s`; this is local source
  evidence only.
- Current source hashes:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_runtime/src/ui/surface/navigation_index.rs` | `35FC741B592771C75AFA72B712F582266D24DA5FB66E6B7823D6CF5A58BE93AA` |
  | `zircon_runtime/src/ui/surface/navigation_index/tests.rs` | `FCC2E23B2CA6DF58B84C76DF65DDCC6FCA7D3A263CC9CDA948D39BD9BCEA0228` |
  | `tools/tests/test_runtime_ui_navigation_index_performance_contract.py` | `E6869E3D55DC334426A5069E2C395F596F88AB3CB671F09A8F04EBD130FCE966` |

Managed Cargo/Windows Release and product navigation allocation/latency
evidence remain pending under the existing external dirty-worktree gate.

## Acceptance boundary

Keep `validation_status: managed_validation_pending` until the next batched
managed Runtime/Editor run verifies compilation, candidate-order parity,
allocation behavior, and navigation p50/p95/p99. No coordinator request or
status query was made for this slice.
