---
doc_type: optimization-implementation
status: source_candidate_managed_validation_pending
runtime: Runtime62
gate: RSH-G18
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
tests:
  - equivalent_parent_reparent_preserves_derived_ticks_and_render_fields
  - checked_reparent_render_stage_publishes_changed_world_matrix
---

# Runtime62 RSH-G18: unchanged derived component publication

## Candidate change

`World::propagate_active_state` and `World::propagate_world_matrix` compare the computed value with the stored derived component before calling `replace_derived_component`. A missing or changed component is still inserted/replaced and marks that entity render-dirty; an equal value keeps its existing component and change ticks and does not mark a render-component candidate.

Checked hierarchy changes now only mark the active, transform, and node-cache frontiers. `RenderExtractPrepare` runs whenever any derived frontier is pending, so a direct render-extract stage first propagates those changes and lets changed `WorldMatrix` or `ActiveInHierarchy` components mark their own render candidates. The render component projector recognizes those derived components as sources and does not treat `Hierarchy` as one. The unchecked targeted hierarchy mark and global/repair `mark_all` paths are unchanged.

The equivalent-parent regression attaches a renderable mesh child to one parent, flushes render extraction, then reparents it to a second parent with the same local transform and active state. It checks the child's world transform remains `(5, 0, 0)`, its `WorldMatrix` and `ActiveInHierarchy` values and change ticks remain unchanged, and the render-component artifact retains the same `Arc` and journal generation with the same projected fields. A second behavior regression reparents a renderable child from an x=1 parent to an x=4 parent with local x=2 and invokes the RenderExtract stage directly; it checks the child moves from world x=3 to x=6 and that the render artifact generation and child world-matrix projection update. The assertion remains scoped to the render-component artifact; independent hierarchy or topology publication remains valid.

Value comparison follows `WorldMatrix`/`Mat4` and `ActiveInHierarchy` `PartialEq` semantics. Authored transform writes pass `validate_transform_for_write`; this candidate makes no guarantee for externally corrupted non-finite state (for example, NaN compares unequal to itself). Propagation remains subtree-based. No latency or allocation threshold was measured or defined, and this change does not establish RSH-G19 O(1) reads.

## Source provenance and validation

The frozen Runtime wave manifest remains unchanged. Before this successor edit, its exact source entries were:

- `zircon_runtime/src/scene/world/derived_state.rs`: `c66ef3bc0f40ab1e035723482522d069ea441dcd07aca5587b075af707175b64`
- `zircon_runtime/src/scene/tests/derived_state/projected_reads.rs`: `d6f753072a77986bf870ecc70a394c61dda93865fc752ae25fef67689c2f4ab8`
- `zircon_runtime/src/scene/tests/derived_state/work_counters.rs` remains unchanged: `4a8fb637262a979f865b2bdec44befe9ce23f55c3d50bae816461b0a4beb8678`

The original four-path candidate manifest remains unchanged at `docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-g18-source-manifest.json` (SHA-256 before this successor: `b27232d95cefb162fa920f7a4cb33788621637178f06d59d266945ead0550de9`). Its final hashes before this successor were `derived_state.rs` `f86f2f04b7d6978a00eb2b1a53b5ee0ada083738ac5b99bbf2e101a3f57a4573`, `projected_reads.rs` `adcba07f82f6ee17d7171526cae3872d23d0695017c6cfa98724067db36df846`, this optimization record `3da3842992bb0c9b151ab12f133f0d069df86c000709bc40d3cea641f17bd04c`, and the Astra list `a36416bceecb3adc0d6aaf5ac19fa5636b0372dd5e2ca358b654fee958e26897`. The newly scoped `dirty_state.rs` exact preimage was `131d2fdfbfb5a816b3769b24ec6e9d9098a19b15864726ee866161789401fa77`. Successor postimage hashes for the same four artifacts plus `dirty_state.rs` are sealed in `docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-g18-render-publication-successor-source-manifest.json`.

Pinned `rustup run 1.94.1 rustfmt --edition 2021 --config skip_children=true` formatted the changed `dirty_state.rs`; its matching `--check` passed for `dirty_state.rs`, `derived_state.rs`, and `projected_reads.rs`. Direct final-newline and trailing-whitespace checks passed for all candidate source, record, and manifest paths. Scoped `git diff --check` reported no errors (it does not inspect untracked files). Cargo and managed tests were not run; both behavior regressions and the affected `scene::world::dirty_state::tests` / `scene::tests::derived_state::projected_reads` owners still require grouped managed compilation and execution. RSH-G18 remains open until that evidence is accepted. No performance pass is claimed.

The intended addition to `projected_reads.rs` could not be written: its preimage hash remained `adcba07f82f6ee17d7171526cae3872d23d0695017c6cfa98724067db36df846`, while a single guarded byte-write returned Windows `OSError [Errno 22] Invalid argument` (the standard patch writer also failed). No permission or lock workaround was attempted; the changed-value regression is instead located in the newly touched `dirty_state.rs` unit-test module.

The coordinator registration request `aa81a44ed9cc48c0b9aa33c4b1bd4ec9` was accepted but remained pending on its single receipt reconciliation; no lease or attribution success is claimed. The candidate is retained as an isolated successor for parent-managed grouping.
