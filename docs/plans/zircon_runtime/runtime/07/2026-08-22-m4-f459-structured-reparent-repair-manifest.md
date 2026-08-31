# Runtime07 F459 M4 Structured Reparent Repair Candidate

Plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
Milestone: M4
Status: candidate_pending_managed_validation
Files: ["docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md", "docs/plans/zircon_runtime/runtime/07/2026-08-22-m2-world-derived-state-generation-topology-manifest.md", "docs/plans/zircon_runtime/runtime/07/2026-08-22-m4-f459-structured-reparent-repair-manifest.md", "docs/plans/zircon_runtime/runtime/07/failure-2026-08-22-world-deserialize-node-cache-initializer.md", "zircon_runtime/src/scene/world/bootstrap.rs", "zircon_runtime/src/scene/world/derived_state.rs", "zircon_runtime/src/scene/world/dirty_state.rs", "zircon_runtime/src/scene/world/hierarchy.rs", "zircon_runtime/src/scene/world/hierarchy_topology.rs", "zircon_runtime/src/scene/world/hierarchy_validation.rs", "zircon_runtime/src/scene/world/mod.rs", "zircon_runtime/src/scene/world/typed_api.rs", "zircon_runtime/src/scene/world/typed_api/component_mutation_effects.rs", "zircon_runtime/src/scene/world/world.rs", "zircon_runtime/src/scene/tests/derived_state.rs", "zircon_runtime/src/scene/tests/derived_state/hierarchy_behavior.rs", "zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs"]

## Structural Repair

- The production-file audit found `derived_state.rs` at 820 lines, beyond the 800-line review
  warning in the engine code structure convention. Its hierarchy-validity snapshot and repair
  behavior now has the explicit `hierarchy_validation` owner; `derived_state` retains dispatch,
  projection, and the other derived-state families. This returns the dispatch owner to 706 lines
  without changing the validation contract or public surface.
- M2 independent review found that a valid `set_parent_checked` write still entered the
  generic hierarchy-validity path, which snapshots every parent edge before it can propagate the
  changed subtree. M4 makes the checked write mode explicit through the typed insertion boundary;
  generic `Hierarchy` insertion and `get_mut` remain the conservative, validation-required path.
- The World-owned topology now reads ordered children directly from its stable parent adjacency
  maps. It does not rebuild a world-wide dense child projection after an attached edge changes.
  This preserves one authoritative tree while making normal reparent work proportional to the
  changed edge plus its subtree, matching Unreal scene-component attachment ownership.
- Validation repairs remain global because the raw escape hatch cannot identify the changed edge.
  When validation actually removes an invalid edge, M4 expands active, matrix, NodeCache and
  render frontiers before the next systems run, and marks repaired inspection fields dirty.
- Raw validation first clears direct missing/self parents, then walks the remaining single-parent
  graph with completed paths. It selects one stable edge only from each actual cycle segment, so
  every cycle deterministically loses one edge while descendants that merely lead into the repaired
  root remain attached. The path scratch buffer is reused across starts, avoiding per-root heap
  churn in large raw-validation passes.

## Deterministic Acceptance

- Structured reparent fixtures at 1,000 and 100,000 entities must record zero hierarchy parent
  snapshots, zero validity entities and zero source-topology rebuild entities. The changed two-node
  subtree must record active/matrix visits of `2` and NodeCache rows of `1`.
- A raw cycle repair fixture must verify repaired parents, A/B world matrices, active values and
  cached node parents after the flush; a three-node cycle must retain the non-selected edges. An
  earlier descendant feeding a later self-cycle owner must also remain attached after the owner edge
  is repaired.
- Stable-order fixtures must preserve sibling traversal order after checked detach/reattach and
  removal, and preserve root order across the equivalent ordered-topology transitions.
- Managed Windows validation uses the non-C target pool
  `D:\\cargo-targets\\zircon-engine\\pool\\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
  The production-only `validate-matrix.ps1 -Package zircon_runtime -SkipTest -VerboseOutput`
  run completed successfully as Cargo job `a97a3972585e4baf9736dad06c990105` (13m42s). This
  verifies the production runtime build and the extracted validation module, but does not execute
  behavioral or visit-counter assertions.
- The first isolated Windows validation also exposed the serialized `World` constructor omission
  for `node_cache_rows` and `node_cache_topology_generation`; M4 initializes both runtime-only
  fields, adds a source guard, and adds a restore-then-reparent behavioral regression. That
  regression requires the restored first flush to rebuild the cache before proving its next
  checked reparent updates exactly one active, matrix, and NodeCache row. Its focused test was
  submitted as Cargo job `d8540e5eed3d4f38b1c5010b3993937f`, but its `zircon_runtime` lib-test
  harness compilation failed before the named test ran with 19 unrelated Runtime74 UI errors:
  15 removed `UiAssetLoader::load_str` calls, two untyped `serialized.try_into()` conversions, a
  binding-ownership expected-count scope error, and one stale
  `UiBindingMutationTransaction::commit()` call. The first three are recorded by Runtime74's
  [loader-contract failure](../../../optimize/zircon_runtime/74/failure-2026-08-22-ui-asset-binding-canonical-loader-api-tests.md)
  and [compiled-binding failure](../../../optimize/zircon_runtime/74/failure-2026-08-22-text03-compiled-binding-contract-compile.md).
  The open Runtime07 failure record remains pending its original upward validation; this is not a
  passing result.
