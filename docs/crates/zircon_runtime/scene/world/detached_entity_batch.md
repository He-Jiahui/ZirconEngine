---
doc_type: module-detail
related_code:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/prepared.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/preparation_owner.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/root_normalization.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_parent_validation.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/tests.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/tests/derived_state/subtree_cycle_walk.rs
  - zircon_runtime/src/scene/world/error.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/tests/editing/detached_entity_batch.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-08-23-editor-delete-subtree-all-cameras-invariant.md
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-detached-restore-union-cycle-rejection.md
tests:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_tests.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/restore_cycle_profile.rs
  - cargo test -p zircon_runtime --lib detached_entity_batch --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib deleting --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib prepared_camera_subtree_managed_scale_fixture --locked --jobs 1 -- --ignored --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib deleting_all_cameras_after_capture_managed_scale_fixture --locked --jobs 1 -- --ignored --nocapture --test-threads=1
---

# Prepared Subtree Detach

`World::prepare_entity_subtrees` validates and normalizes an entity-root union and
returns a move-only `PreparedEntitySubtrees`. It captures the affected entity and
camera counts, the indexed total camera count, and the observable World generation.
The preparation walks indexed descendants and validates affected storage rows; it
does not clone World state or materialize `NodeRecord` snapshots. Pending query
mutation notifications are applied before the generation and topology are read;
this preserves the effective generation already exposed for those mutations. An invalidated
hierarchy index is rebuilt through the existing hierarchy owner before traversal.

After pending query notifications are applied, preparation validates the parent
chains reachable from the requested roots before collapsing ancestor-covered roots
or rebuilding the hierarchy index. An existing cycle produces
`SceneError::HierarchyParentChainCycle { start, repeated }`; a cyclic request cannot
be accepted as an empty preparation or partially delete its valid roots. The check
does not repair hierarchy rows. It memoizes only the reachable chain union, reading
each parent link once per batch; stable-order sorting and indexed subtree traversal
retain their existing costs. Missing raw ancestors keep their existing terminal-chain
behavior. Empty requests, missing requested entities and exhausted generations retain
their existing typed errors and precedence.

Deferred component notifications can inspect a raw hierarchy cycle before this parent-chain check.
The subtree walkers visit their starting entity once and stop at its returning edge, allowing
the typed parent-chain error to surface without repairing corrupt hierarchy rows.

`World::remove_prepared_entity_subtrees` consumes that exact preparation. It rejects
a different World identity or a changed generation with typed `SceneError` variants
before removing any row. Callers must prepare again after a stale rejection. World
moves preserve the private identity, while construction, cloning and deserialization
allocate distinct identities. A saturated generation cannot issue a preparation.
The identity is runtime-only and does not affect persistence or World equality.

`remove_entity_recursive` and `remove_entity_subtrees` use the same prepare/commit
path. Commit and restore retain the existing move-only row, change-tick, dynamic
component and observer ownership semantics. Dropping a valid preparation does not
publish lifecycle events or advance the World generation. Rejected preparations
only record the existing rejected-preflight diagnostic.

`World::restore_detached_entity_batch` completes its existing identity, storage,
batch-parent existence, and active-camera checks before validating the prospective
union of detached Hierarchy rows and live parent rows. A cycle that contains a
restored row returns `SceneError::HierarchyCycle { child, parent }`; a reached
cycle entirely in live rows returns `HierarchyParentChainCycle { start, repeated }`.
The check follows only parent chains reachable from the batch, shares completed
ancestors across its entries, and preserves the existing terminal behavior for a
raw missing live ancestor. It does not flush pending query mutations or rebuild
the hierarchy index. A rejected restore changes only the rejected-preflight
diagnostic and returns the unchanged move-only batch through
`DetachedEntityBatchRestoreError::into_parts`, so the caller can repair the live
parent chain and retry without losing stored rows, ticks, or observers.

Editor deletion requires `affected_camera_count < world_camera_count` before
commit. Apply and redo perform this check on a fresh preparation inside the same
World write callback, so a capture-time camera count cannot authorize a later
invalid deletion. The command retains a batch only after a successful detach.

The focused regressions cover normalized roots, stale hierarchy and unflushed
component mutations, cross-World misuse, World moves, repeated preparation and
generation exhaustion. Managed scale fixtures cover 2/128 cameras with 100k
unrelated entities and the existing 1/1k/100k detach/restore profiling gate.
Cycle regressions also cover raw and directly deserialized graphs, covered roots,
mixed valid/cyclic requests, the existing deferred-notification boundary, unrelated
cycles, and preservation of owned rows and ticks. The paired Release preparation
profile measures 2/128 requested roots with 100k unrelated entities, including shared
deep ancestors and densely covered roots; it reports parent reads and raw latency samples.
Restore regressions cover union cycles, reachable and unrelated live cycles,
unflushed query mutations, error precedence, and repair with the returned batch.
The paired Release restore profile compares the complete former and current
preflight on valid forests, then measures actual restore calls with 2/128 roots,
shared deep ancestors, and 100k unrelated rows; it reports raw samples and
p50/p95/p99 without an added absolute performance threshold.
Behavior and product acceptance remain pending until matching managed tickets pass.
