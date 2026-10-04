---
title: Runtime62 Checked Parent Chain Cycle Rejection
category: zircon_runtime
date: 2026-09-27
session_id: astra-optimize-20260926-batch-a
implementation_status: implemented_pending_validation
validation_status: static_checks_passed_managed_tests_pending
performance_status: release_measurement_pending
related_code:
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/error.rs
tests:
  - zircon_runtime/src/scene/tests/derived_state/checked_parent_chain.rs
  - zircon_runtime/src/scene/world/hierarchy/checked_parent_chain_profile.rs
  - zircon_runtime/src/scene/tests/derived_state/work_counters.rs
  - zircon_runtime/src/scene/tests/derived_state/spawn_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_derived_state.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
---

# Runtime62 checked parent chain cycle rejection

## Scope and contract

This candidate addresses the checked-reparent subpath of RSH-P0-001,
RSH-P1-018 and RSH-G03. Public `get_mut::<Hierarchy>` can close a pending
cycle before the next validity stage. Calling `set_parent_checked` for an
unrelated child under that chain previously never returned. Direct `World`
deserialization also preserves this pending window; `load_project_from_path`
has a separate normalization stage and is not claimed as the same reproduction.

The private ancestry predicate now returns `SceneResult<bool>` and uses
Brent checkpoints. A pre-existing cycle produces the new typed
`HierarchyParentChainCycle { start, repeated }` before any mutation, fact,
generation or derived-state publication. The requested-ancestor comparison
keeps its original priority, so a prospective cycle still returns
`HierarchyCycle`. Missing entity, direct self/missing parent, Static rejection,
valid reparent, no-op and detach preserve their existing routes.

The guard uses constant auxiliary state and linear parent-chain traversal.
It does not allocate a depth-sized visited set or perform a whole-World
validity snapshot. These are source properties, not measured allocation or
latency results. Existing before/after `Reparented` fact publications retain
their old/new ancestry invalidation role and remain unchanged.

The primary engine reference is local Unreal Engine
`dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/SceneComponent.cpp`
(SHA-256 `12d1ef7cd76425b34e9922260228e331bb86d98443265719d30a08958ef17e9e`).
Lines 2366–2371 reject a prospective cycle with `Parent->IsAttachedTo(this)`
before attachment; this supports retaining rejection before mutation. Its
`IsAttachedTo` implementation at lines 2794–2806 uses an ordinary parent loop
under the attachment invariant. Zircon's raw mutation/direct-serde window
requires its own bounded guard; Brent and performance equivalence are not
attributed to Unreal Engine. The reference file is read-only and is not owned
by this candidate.

## Behavior coverage and evidence boundary

Eight real World regressions were authored before changing the production
guard. They cover two-node, three-node, self and tailed cycles; direct serde
restore before first flush; ancestor-hit error priority; valid reparent,
no-op and detach; and the existing rejection variants. Rejected cycle
requests compare the complete serialized World, generation, change tick,
pending derived state, derived diagnostics, binding generations and the live
subscription fact queue. Snapshot reads do not flush or repair the graph.

The old hanging path was not executed in the shared process. Test authoring,
source inspection and rustfmt do not constitute dynamic red/green evidence.
Managed package compilation, focused `checked_parent_chain` and existing
`derived_state::hierarchy_behavior` execution remain pending.

## V2 test-structure correction

Independent review found the existing folder-backed test gate expected zero
inline parent tests and 23 declarations in its five child-owner files. The
parent already contained seven work-counter tests, and those five files
actually contained 33 declarations. V2 moves the complete existing helper/test
block, including its two scale constants, into `work_counters.rs` with unchanged
test bodies, assertions, thresholds and ignore behavior. The parent again has
zero executable tests and keeps the shared helpers used by older children.

The same gate retains its previous ownership assertions and the below-800-line
budget, repairs two stale child-test name needles to their current cached-kind-count
and mutation-index test names, adds both new child owners, and expects their actual 33 + 8 + 7 = 48
test declarations. This count covers only the seven explicitly enumerated
files, not every recursively mounted descendant. Static block identity and
the gate's source assertions are checked; managed Rust execution is pending.
V1 source/evidence artifacts remain frozen; V2 has separate preimages,
checkpoint, inverse checks and a source manifest. Production, cycle behavior
tests and the Release profile are unchanged from V1.

The same source-gate review confirmed the preexisting
`CR-SCENE-TEST-CONTRACT-0001` failure: the spawn-path test read the removed
`components/scene.rs` before reaching its Copy/derive assertions. Its source
path now targets `components/scene/identity.rs`; the BUG marker remains as a
regression note, and the existing module commentary and unrelated foreign
bytes are preserved. The NodeKind, spawn and ordinal assertions remain in
place. An existing `///` comment between the derive attribute and NodeKind
declaration also made the test's old adjacency assumption false. The test
locally excludes only those documentation-comment lines while retaining all
other attributes, declarations and their order, then uses the original full
derive/Copy assertion. The definition owner remains read-only.
The old guard also assumed spawn called the prevalidated publisher directly.
The current path goes through `insert_node_record`, `insert_owned_node_records`,
preparation and commit. The guard now checks those actual method slices,
validation-before-commit order and the final prevalidated call while retaining
default-record construction, Copy, no-clone and ordinal constraints.
Read-only `world/records.rs` reference SHA-256:
`8eb72e723c74f6f82dd17ac2186377919c57688a126e993508db2aae11121560`.
This exact test owner is included in V2; no repository-wide test
cleanup is claimed.

## Release protocol

Ignored test: `runtime62_checked_parent_chain_release_profile`.
Output marker: `RUNTIME62_CHECKED_PARENT_CHAIN_BENCH_V1`.

| Corpus | Timed operation | Protocol |
|---|---|---|
| Valid parent-chain depths 1, 32, 1K and 100K | Actual `World::set_parent_checked` versus the complete frozen preimage operation | Five warmup pairs, 31 alternating sample pairs, raw nanoseconds and p50/p95/p99 for both paths |

The frozen methods differ only in test-local visibility, names and their mutual call. They are
called only on valid forests. Both complete operations include existing
mutation and fact behavior. Persisted-row construction, deserialization,
restoration and equality checks are outside timed intervals; constructing
the 100K chain does not perform successive checked reparent operations.
Full post-round persistent state, generation and derived counters must match.

No numeric speedup or latency ceiling is invented for this safety guard.
The profile must run in the managed Windows Release batch before any timing
claim. Full RSH-G24 visited/written/allocated-byte ceilings, allocator/RSS
evidence, frame/product behavior and cross-engine qualification remain open.
This slice does not close all RSH-G03 walkers, protected mutation G01/G02,
or RSH-P0-001 as a whole; detached-subtree normalization remains separate.

## Independent review

V1 production, behavior and profile review and the V2 incremental test-owner
review passed with no remaining source findings. Managed compilation, behavior
execution and Release measurement remain pending.

## Ownership

Batch Q, Runtime1010. Preimages, foreign/index diffs, tests-first snapshots,
exact leases and plan authorizations use artifact prefix
`2026-09-27-astra-optimize-batch-q-runtime62-checked-parent-chain` under
`.codex/state/session-coordinator/async-validation-batches/`.
The V2 test-structure correction adds the `-v2` artifact suffix and expands
the exact owned scope from nine to twelve paths.
The candidate has no overlap with the inspected O/P frozen input anchors
or Q Editor1020. No Cargo submission, commit, push or external notification
was performed by this implementation lane.
