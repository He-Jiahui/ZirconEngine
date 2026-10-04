# Layout09 retained UI performance review

## Status

- Result: `static_complete / managed_profile_pending`.
- Review date: 2026-08-11.
- Scope owner: Layout09 owns the editor-side retained hierarchy projection, notification projection, and dirty-refresh application. `zircon_runtime::scene` remains the sole owner of the authoritative scene and inspection artifact.
- Disposition: the notification projection has a source-proven bounded-formatting repair below. No hierarchy data-structure or invalidation-policy optimization is accepted from static review alone; its frame cost must be measured through the existing profiling scopes before changing it.

## Reviewed data flow

| stage | current owner | normal name-change work | explicit recovery work |
|---|---|---|---|
| world to inspection artifact | `zircon_runtime::scene::WorldInspectionArtifact` | publishes the changed row anchor | complete artifact read only for generation/topology recovery |
| editor publication | `SceneInspectionPublication` | reuses the previous `Arc<BTreeSet<EntityId>>` when selection revision is unchanged; publishes one changed anchor | rebuilds the selected set and emits a selection resync only after a selection revision or generation gap |
| latest transport | `EditorMessageBus` / `EditorMessageDelivery` | one retained subscriber, one `SceneInspection` Latest key, shared `Arc` payload | coalesced selection deltas compose from their oldest compatible revision, otherwise force resync |
| retained bridge | `SceneHierarchyFragmentApply` | updates changed row controls plus added/removed selection controls, then commits generation only after refresh succeeds | `resync_scene_hierarchy_at_selection` rebuilds the bridge projection only for topology, filter, generation, or revision recovery |
| surface and host projection | `UiSurface::rebuild_dirty` / `EditorWorkbenchTemplateSurface` | incremental layout/render and changed-node projection patch | topology mismatch is the only normal reason for a full host projection rebuild |

The normal rename path is therefore expected to be `O(delta_rows log N + delta_selection log N)` for the bridge maps and ordered changed-control set, with no `O(S)` selection snapshot when the selection revision is unchanged. A selection change itself is intentionally `O(S log S)` because the authoritative selection owner exposes a snapshot; this is a recovery or selection-update cost, not a row-rename cost. A full hierarchy reflow remains `O(N)` by design and is guarded by generation, topology, filter, and explicit resync conditions.

## Static evidence and current gaps

- `zircon_editor/src/tests/editor_message/refresh.rs` already covers a 10,000-node rename and asserts that runtime hierarchy rows are not fully materialized. It did not cover a 10,000-item stable selection combined with a one-row rename; that regression guard is required before treating the no-snapshot assertion as complete.
- `scene_hierarchy_fragment.rs` exposes `hierarchy_fragment_apply`, patch-row, selection-delta, resync-required, and full-resync counters. `template_surface.rs` and `UiSurface::rebuild_dirty` expose layout, changed-node, projection-patch/full-rebuild, arranged, hit-test, and render counters. These are sufficient to locate a regression without adding speculative instrumentation.
- The remaining measurement candidate is not a proven bottleneck: a sparse bridge update still calls `refresh_after_state_change`. Its surface report can stay incremental, but only a managed editor trace can prove that the changed-node, layout-visit, render-visit, and host-projection-full-rebuild counts remain proportional to the patch.
- Transport still has the general Editor02 global route-lock and unbounded inbox-drain concerns recorded in `2026-07-30-editor-core-editor-message-current-review.md`. The retained hierarchy subscriber itself subscribes only to the Latest `SceneInspection` topic, so that broader issue must not be "optimized" through a hierarchy-specific bypass.

## Reference evidence and design decision

- Unreal: `dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Slate/SRetainerWidget.h` makes retained rendering an explicit invalidation-root capability, while `.../Components/ListView.h` preserves a list-view owner with per-entry initialization. This supports Zircon's retained surface plus explicit row-patch boundary.
- Slint: `dev/slint/internal/core/model.rs` uses `ModelNotify::row_added`, `row_removed`, and `row_changed` for single-row mutation and reserves `reset` for replacement. This supports delta delivery rather than rebuilding an unchanged hierarchy for a name edit.
- Zircon deliberately uses versioned `SceneInspectionMessage` and a generation-checked bridge rather than importing Unreal's widget invalidation root or Slint's model API. That divergence fits the existing retained host and preserves the runtime/editor ownership boundary without a second hierarchy truth.

## Managed profiling plan

Use Windows profiling builds through the repository validator, never a repository-local `target` directory. Capture three repeatable editor runs each for:

1. 10,000 hierarchy rows, no selection, one rename.
2. 10,000 hierarchy rows, 10,000 stable selected entities, one rename.
3. 10,000 hierarchy rows, selection revision change, and one forced Latest gap/resync.

For every run collect p50/p95 of `hierarchy_fragment_projection`, `hierarchy_fragment_apply`, `workbench_surface_compute_layout`, `workbench_surface_extract_frames`, and `workbench_surface_build_host_projection`; also collect patch-row, selection-resync, layout-visited-node, render-visited-node, incremental-projection, and full-projection-rebuild counters. Record CPU and working-set samples with WPR/WPA for the interactive window. The acceptance comparison is: case 2 must remain within the case-1 sparse-path work envelope except for the already-selected overlay lookup; case 3 may pay snapshot/recovery cost but must not silently use a full projection in cases 1 or 2.

No dynamic profile was run during this source slice, so this report records no fabricated timing, allocation, RSS, or counter deltas. Any later performance-affecting production change must append its before/after samples, runner configuration, repeat count, and unchanged correctness gates here or in the owning accepted Layout09 record.

## 2026-08-11 forward repair data

`sync_notification_projection` previously formatted every source notification into an intermediate vector before retaining the first 64 rows. The forward repair first computes the source count from the three slice lengths, then applies `take(MAX_NOTIFICATION_HISTORY)` before entry formatting.

| 1,000-toast burst operation | before | after | dynamic measurement |
|---|---:|---:|---|
| formatted history entries | 1,000 | 64 | pending managed profiling |
| intermediate history strings | 1,000 | 64 | pending managed profiling |
| retained history rows | 64 | 64 | unchanged contract |
| overflow count | 936 | 936 | unchanged contract |

The 936 avoided entry format/allocation operations are a source-derived operation count, not a latency or memory measurement. `wpr.exe` 10.0.26100.8875 is available on the Windows host; an actual trace remains deferred until the repository validator provides a managed profiling build and the interactive editor scenario can be driven reproducibly.
