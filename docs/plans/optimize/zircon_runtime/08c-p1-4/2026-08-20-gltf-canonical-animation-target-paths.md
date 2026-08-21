# Runtime08C glTF Canonical Animation Target Paths

- Date: 2026-08-20
- Owner: `optimize-runtime08c-gltf-target-path-r1-01a00797-20260820`
- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-4
- Execution plan: `docs/plans/optimize/zircon_runtime/08c-p1-4-gltf-target-path.md`
- Status: implementation complete; combined managed validation pending

## Problem

The builtin glTF importer wrote a clip track's `target_id` as one leaf bone
name. The animation evaluator treats any explicit `target_id` as a complete
canonical skeleton path, so an imported non-root track such as `Node1:Body`
could not resolve the skeleton slot whose path was
`Node0:Root/Node1:Body`. The explicit ID also disabled the evaluator's legacy
unique-leaf fallback.

## Change

- The importer derives node-to-target mappings from the exact generated or
  referenced `AnimationSkeletonAsset` bones and `parent_index` values.
- Skin clips use the selected skin's joint-to-bone order. Animation-only clips
  use the same required-node order that builds their synthetic skeleton.
- A three-state path index resolves every parent chain once, supports parents
  stored after children, and validates cycles, parent indices, canonical names,
  duplicate paths, duplicate node mappings, and node bounds.
- A channel whose node is absent from its selected skeleton now fails import
  with its `AnimationN` and `NodeN` identity instead of producing a clip that
  silently fails during frame evaluation.
- Target-path construction lives in a dedicated importer submodule; the main
  glTF animation importer remains below 750 lines.

## Performance and Failure-Boundary Evidence

| Operation | Before | After |
|---|---|---|
| Resolve imported child track | frame-time compiled lookup misses | import-time canonical path resolves directly |
| Parent-chain indexing | no usable path was generated | one three-state index pass plus required path bytes |
| Missing skeleton node | invalid clip reaches runtime | typed import failure before asset publication |
| Runtime compatibility fallback | unavailable for explicit leaf ID | unnecessary; canonical ID is authoritative |

This correctness slice does not claim a timing speedup. Its performance effect
is removal of the frame-time unresolved-track path and prevention of repeated
runtime failure. Release P50/P95 data for the accompanying Runtime08C batch is
provided by the event-candidate heap gate in the same serialized validation
batch.

## Acceptance

- `importer_emits_synthetic_skeleton_for_node_animation_without_skin` requires
  `Node0:Root/Node1:Body` for the imported child track.
- `target_paths_follow_skeleton_parents_even_when_parent_bones_come_later`
  covers skin-style joint order and the node mapping.
- `target_paths_reject_cycles_and_nodes_outside_the_document` locks the typed
  import boundary.
- Existing one-bone skin import keeps the root target `Node0:TriangleNode`.
- Runtime and neutral asset documentation now state that an explicit clip
  target is a complete canonical skeleton path and never falls back to a leaf;
  the importer and compiled evaluator owners are cross-linked.
- The current Runtime08C/45/48/49 validator runs eight logical tasks in twelve
  Cargo groups and pins Rust 1.94.1, the lockfile, and one Cargo worker.
  Validator SHA-256:
  `A2C1864BDCA19026FD02493EC066031AF95CE6A050E59A608859C64FBC9E0943`.
- Exact-file Rust 1.94.1 rustfmt and scoped `git diff --check`: passed.
- Cargo regressions: pending the serialized multi-task coordinator batch; no
  direct or competing Cargo process was started.

## Remaining Plan Work

This closes only the concrete P1-4 leaf-vs-path mismatch. Duplicate-leaf and
multi-root product fixtures, skin joint subsets, animation-only hierarchy
coverage beyond the current fixture, rename/reimport receipts, the canonical
plugin-vs-builtin glTF owner, and cooked dense target slots remain open.
