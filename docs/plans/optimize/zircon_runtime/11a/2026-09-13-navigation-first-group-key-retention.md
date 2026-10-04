---
title: Runtime navigation first-group key retention
category: zircon_runtime
report_id: Runtime11A-navigation-first-group-key-retention-2026-09-13
date: 2026-09-13
related_to:
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-first-group-candidate.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-candidate-bucket-reuse.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A navigation first-group key retention

## Scope

The first-group target map was already reduced in the retained navigation-node
stream, but every rebuild still dropped the `HashMap<UiNavigationGroupId, _>`
entries. Stable groups therefore released and reallocated their owned `String`
keys even when the group remained present in the next frame.

## Implementation

`first_candidate_by_group` now stores a narrow `FirstGroupCandidate` value with
the selected node and a rebuild-local `seen` bit. `clear_for_rebuild` marks
existing values unseen without clearing the map. The hot node stream updates an
existing entry through `get_mut`; only a newly admitted group clones its
`UiNavigationGroupId`. The publish step prunes unseen groups, so removed scopes
cannot be returned by manual group navigation while retained keys and map
capacity survive the next rebuild.

The helper lifecycle is isolated in
`zircon_runtime/src/ui/surface/navigation_index/candidate_buckets.rs`; sorted
candidate vectors remain the ordering authority and manual group-target
semantics are unchanged.

## Complexity and allocation boundary

For `F` focus candidates and `G` previously admitted groups, first-target
selection remains one `O(F)` stream with expected `O(1)` map probes and the
existing node-order comparisons. Reset/prune work is bounded `O(G)`. Stable
groups no longer allocate a replacement `String` key or map entry on each
rebuild; allocation is limited to genuinely new groups or map growth. Stale
groups are removed before queries are published.

## Validation

- TDD RED: the new source contract failed while the map still used a plain
  `BTreeMap<UiNavigationGroupId, UiNodeId>` and `.clear()`.
- GREEN: the Runtime navigation source contract passes `12/12`; the lower Rust
  regression checks `seen` reset/prune behavior, stable key-buffer identity, and
  retained map capacity. Standalone Rust shape checks report
  `RUST_CANDIDATE_KEY_REUSE_SHAPE_OK` and
  `RUST_FINISH_FIRST_GROUP_BORROW_SHAPE_OK`; compiling the actual helper module
  against minimal interface stubs reports `RUST_ACTUAL_CANDIDATE_MODULE_COMPILE_OK`.
- The batched Runtime/Editor UI source-contract invocation covered 21 modules
  and passed `105/105` tests in `25.523s`. This is one batched local run, not a
  per-task validation loop.
- Rustfmt parses the production, helper, and test modules; the helper module is
  rustfmt-clean. Current source hashes:

  | File | SHA-256 |
  | --- | --- |
  | `zircon_runtime/src/ui/surface/navigation_index.rs` | `96562C274D28257338A006F4A9C39733D73CDD7E1D253065FF39E18A36879809` |
  | `zircon_runtime/src/ui/surface/navigation_index/candidate_buckets.rs` | `2CC60F7DF3925A5E7CDA3BF9F789A434F3D0AC2802574EB12FCB7BA772B1E268` |
  | `zircon_runtime/src/ui/surface/navigation_index/tests.rs` | `86306F2873FCF5916E341720DB934E30D5D1D39D56EA284F0FEC6854ED283092` |
  | `tools/tests/test_runtime_ui_navigation_index_performance_contract.py` | `C84E10534D6B75AD122CCA8ED89965E2355621E3755FC1EA8C44316D81D9491C` |

Managed Cargo/Windows Release compilation and product navigation allocation,
CPU, RSS, and p50/p95/p99 evidence remain pending under the existing external
dirty-worktree gate.

## Acceptance boundary

Keep `validation_status: managed_validation_pending` until the next batched
managed Runtime/Editor run verifies compilation, candidate-order parity,
allocation behavior, and navigation latency. No coordinator request or status
query was made for this follow-up; tooling changes remain out of scope.
