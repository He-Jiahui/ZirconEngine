# Inspector projection publication authority

Date: 2026-08-28

Status: `design_ready_e6`; current-source production owners are externally dirty, so this slice contains review evidence, a deterministic pressure model, and an implementation/acceptance contract only.

## 1. Conclusion

The current Inspector path is not end-to-end retained for presentation-data refreshes. Runtime virtual-list ownership can bound the physical nodes inside a `UiSurface`, but a normal Editor presentation rebuild still recreates the Inspector snapshot, clones the nested component/property payload into the pane DTO, rebuilds a new Surface, lays it out, builds a host model, and materializes one host node for every plugin property. Ordinary `WindowMetrics` refresh is no longer part of that baseline: current source reuses committed chrome/model/pane/presentation state and applies geometry only unless the committed-stage or geometry publication explicitly falls back.

The fix must be a single `RetainedEditorHost`-owned publication entry. A cache local to `inspector_template_projection` would be a second authority with the wrong lifetime and is forbidden. Stable recompute must reuse the published Surface/frame/model handles; resize must advance only geometry; exact field deltas must patch the generation-owned logical source and affected physical slots.

## 2. Current-source evidence

| Source | Current work |
|---|---|
| `zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs:44` | Recreates `InspectorSnapshot` and all plugin-component/property values during editor snapshot construction. |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs:13` | Clones the Inspector payload again, including every component, property, editor kind, and asset marker. |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_projection.rs:62` | Every conversion runs `project_pane_body -> build_shared_surface -> compute_layout -> build_host_model_with_surface`. |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields.rs:21` | Clones all `plugin_components` into `InspectorVisualFields`. |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields.rs:272` | Iterates every plugin property, allocates control/action identifiers, and creates a host node for every logical property. |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields.rs:386` | Traverses the component list again to compute total panel height. |
| `zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs:422` | Existing pane caches use a compiled-document pointer cast to `usize`; this is not a typed, monotonic document generation. |
| `zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs:14` | `WindowMetrics` takes committed chrome/model/pane/presentation ownership without calling `build_chrome`. |
| `zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs:114` | A successful metrics transaction applies geometry presentation and republishes committed state; only an explicit miss/failure reaches full recompute. |

The existing Runtime virtual slots therefore do not make Editor projection `O(V)`. Before publication, the adapter still performs `O(P)` logical-row materialization, where `P` is the plugin-property count.

## 3. Complexity contract

Let:

- `A` be authored template nodes;
- `P` be logical Inspector properties;
- `V` be materialized physical slots, bounded by viewport plus overscan;
- `R` be stable recomputes;
- `Z` be resize steps;
- `F` be resize steps that explicitly fall back from the committed metrics path;
- `D` be source updates;
- `delta` be fields changed by one update.

Current Inspector structural work is `O((1 + R + D + F) * (A + P))`, plus a second `O((1 + R + D + F) * P)` pane-payload copy. Successful metrics-only resize contributes geometry/presenter work but zero Inspector snapshot or pane-payload materialization. `F <= Z` is measured separately; it must not be silently modeled as `Z`.

The target is:

- initial publication: `O(A + P + V)`;
- stable recompute: `O(1)` generation/key checks, with zero payload materialization and zero Surface/layout/host-model rebuild;
- resize: `O(A_geometry + V)` or less, with zero logical-property materialization and unchanged structure/source generation;
- field update: `O(delta log P + affected_visible_slots)`, with zero full Surface build;
- schema/template/theme/font/DPI change: explicit full fallback with a typed reason.

## 4. Unreal evidence

The local Unreal source separates details-tree refresh from ordinary widget geometry and coalesces expensive refresh requests:

- `dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SDetailsView.cpp:618` calls `ShouldSetNewObjects` before replacing the selected-object set.
- `dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SDetailsViewBase.cpp:94` documents that force refresh is deferred to avoid repeated refreshes locking the editor in one frame.
- `SDetailsViewBase.cpp:1296` installs at most one next-tick refresh timer.
- `SDetailsViewBase.cpp:1430` distinguishes property changes, array-size changes, child rebuilds, and invalid objects; only invalid-object/structural cases escalate to `ForceRefresh`.

Zircon should preserve the same boundary: selection/schema/template changes may rebuild the Inspector structure, while geometry and property-value changes patch the retained publication.

## 5. Authoritative identities

The source owner must publish typed, monotonic generations. Deep equality, payload hashing, and pointer addresses are not acceptable hot-path identities.

```text
InspectorSourceGeneration {
    selection: u64,
    schema: u64,
    values: u64,
}

UiTemplateDocumentGeneration(u64)
```

`selection` changes when the inspected entity set changes. `schema` changes when component/property membership, order, editor kind, editability, or customization document changes. `values` changes when one or more existing values change and is accompanied by exact changed field IDs.

`UiTemplateDocumentGeneration` must be owned by the template document catalog and advance on same-ID replacement. It replaces `Arc::as_ptr(... ) as usize` as the cache contract.

Content size is deliberately absent from the structural key. It belongs to the cached Surface geometry state; otherwise every resize step invalidates the structure cache.

## 6. Publication owner

`RetainedEditorHost` owns a bounded `InspectorPaneProjectionCache`, parallel to the existing Console and ModulePlugins caches but not implemented inside the converter.

Each entry contains:

```text
pane_id
template_document_generation
inspector_source_generation
Arc<InspectorLogicalSource>
UiSurface
UiSurfaceFrame / host projection handles
field_id -> logical row index
last_content_size
last_used sequence
```

The cache holds at most eight pane entries and replaces an entry for the same pane ID. Logical properties live in one generation-owned `Arc<[InspectorProperty]>`; entries and snapshots share that allocation. Each Surface contains only `A + V` nodes. It must never retain eight deep copies of all `P` property DTOs.

## 7. Update algorithm

1. Snapshot publication emits typed Inspector generations and an immutable logical source.
2. Pane payload forwards the source handle/generations instead of rebuilding a nested property `Vec`.
3. Converter lookup uses pane ID plus template document generation and Inspector selection/schema generations.
4. Exact key and size hit returns shared published handles; no projection, Surface build, layout, host-model build, or property loop runs.
5. Size-only change reuses the same Surface and logical source, applies incremental geometry, and publishes only changed frame rows.
6. Values-only change uses changed field IDs to update the logical source/index and rebinds only affected visible physical slots.
7. Selection/schema/template/theme/font/DPI mismatch performs one full rebuild and records a typed fallback reason.
8. Multiple refresh requests before publication coalesce into one highest-domain update, following Unreal's deferred-refresh boundary.

## 8. Required counters

- `ui.inspector.projection_cache_hit_count`
- `ui.inspector.projection_cache_miss_count`
- `ui.inspector.surface_build_count`
- `ui.inspector.layout_full_count`
- `ui.inspector.layout_geometry_patch_count`
- `ui.inspector.host_model_full_build_count`
- `ui.inspector.logical_property_materialization_count`
- `ui.inspector.changed_field_count`
- `ui.inspector.physical_slot_rebind_count`
- `ui.inspector.fallback_reason.*`
- `ui.inspector.cache_entry_count`
- `ui.inspector.logical_source_bytes`
- `ui.inspector.surface_node_count`

## 9. Pressure model

Tool: `tools/editor_inspector_projection_pressure.py`

Tests: `tools/tests/test_editor_inspector_projection_pressure.py`

Artifact: `E:\zircon-profiles\editor-inspector-projection-pressure-20260831-current.json`

SHA-256: `AD3DD2799681C9478C122591D2EF7D5E238B2FB0F63AA882E73B6E9B6702C1E2`

Default deterministic scenario: 10,000 plugin properties, 256 authored nodes, 64 physical slots, 1,000 stable presentation recomputes, 200 successful metrics-only resize steps, and 1,000 single-field updates. The model also retains a rejected all-resize-full-rebuild counter so regressions remain visible, but it is not counted as current work.

| Metric | Current model | Target model |
|---|---:|---:|
| Snapshot property materializations | 20,010,000 | 11,000 |
| Pane payload property copies | 20,010,000 | 0 |
| Combined property-record work | 40,020,000 | 11,000 |
| Stable property materializations | 10,000,000 | 0 |
| Metrics fast path / fallback | 200 / 0 | 200 / 0 |
| Resize property materializations | 0 (rejected all-fallback baseline: 2,000,000) | 0 |
| Delta property materializations | 10,000,000 | 1,000 |
| Total Surface builds | 2,001 (rejected all-fallback baseline adds 200) | 1 |
| Cached logical property capacity | repeated DTOs | 10,000 shared once |
| Eight-entry Surface node capacity | unbounded by `V` | 2,560 (`8 * (256 + 64)`) |

The snapshot-property reduction is 1,819.09x and the combined snapshot-plus-pane property-record reduction is 3,638.18x in this operation-count model. These numbers are not elapsed time, allocator bytes, or RSS.

## 10. Acceptance gates

Lower-layer tests must prove:

- a stable generation returns the same logical source and Surface/frame identities;
- size-only update preserves structure/source generations and field identities;
- a one-field delta visits/rebinds only that field when visible, or zero slots when outside the materialized window;
- schema/template replacement performs exactly one typed full fallback;
- LRU eviction never exceeds eight entries and logical source memory is not multiplied by entry count;
- popup, focus, hit-test, edit commit, and IME routes continue to target the same field ID.

Product validation must use current-source managed validation and report CPU/RSS plus p50/p95/p99 for 64 panes, one 10,000-field Inspector, 1,000 stable recomputes, 200 resize steps, and 1,000 one-field updates. Stable and resize phases require Surface build, host-model full build, and logical property materialization counts of zero. No performance claim is accepted from the deterministic model alone.

## 11. Cutover order

1. Add typed template-document and Inspector source generations at their source owners.
2. Publish the immutable logical property source and exact changed field IDs.
3. Add the bounded cache to `RetainedEditorHost` state/startup assembly.
4. Thread the cache through recompute, docked/floating scene conversion, and pane conversion.
5. Convert Inspector projection to reuse/geometry-patch/delta-patch/full-fallback branches.
6. Delete deep property copies from snapshot-to-payload-to-converter boundaries.
7. Add managed lower-layer tests, then Editor product profile and pixel/hit/edit parity.

Current source has external changes in all production owners needed by steps 1-5 except the leaf `inspector_projection.rs`. Implementing only the leaf would create an unused or wrongly scoped cache, so production integration remains intentionally pending until those owners can be changed as one controlled slice.

## 12. Current-source fail-closed revalidation (2026-08-31)

The pressure model now binds the exact five production owners behind the
current baseline: snapshot construction, pane payload construction, Inspector
projection, Inspector field materialization and template document identity.
The binding records current HEAD
`14c89f9776bed828cc85e05e4b9914b3f8d1e784`, dirty paths, file byte lengths,
per-file SHA-256 values and the model SHA-256. Its source guards require the
current full projection/Surface/layout/host-model path, property payload copy,
field loops and pointer-derived document identity; a later production change
cannot silently reuse this baseline artifact.

The focused suite was extended test-first. It first failed because the tool had
no source-binding API; a second RED proved that a rejected source binding still
returned exit code zero. It now passes 8/8 after adding current-source hashing,
fail-closed anchors, nonzero CLI failure, and D:/E:/F: artifact enforcement.
Python bytecode compilation and scoped `git diff --check` pass.

That artifact described the pre-committed-metrics baseline and is superseded by
the current-source revalidation below. The production cache/generation cutover,
managed Rust parity, real allocation/RSS and input-to-present percentiles remain
open.

## 13. Current-source metrics correction and cross-engine binding (2026-09-01)

Artifact: `E:\zircon-profiles\editor-inspector-projection-pressure-20260901-r2.json`

SHA-256: `479E7F2B10CBB5488081583CBCA563074906F0DA156F8017F91FA8551552952C`

Source-set SHA-256: `E1D516EE51F954AA65717180FD794FEAB311B779E61B36DB7830B0F6F4A608E7`

The source binding now covers seven Editor owners plus four local reference
sources. Unreal retains `RootTreeNodes`, avoids replacing an unchanged object
set through `ShouldSetNewObjects`, and refreshes visibility separately. Fyrox
retains property-editor handles in `InspectorContext` and synchronizes them with
property messages. Slint exposes row-level changed/added/removed notifications
for stable repeater models.

The model also binds the current committed `WindowMetrics` transaction. The
default 200 resize steps are 200 fast-path hits and zero fallbacks, so they do
not enter the Inspector snapshot/pane baseline. The remaining presentation and
value-refresh workload performs 20,010,000 snapshot property materializations
and 20,010,000 pane payload property copies. The target retained publication
performs 11,000 property-record updates. The focused suite passes 9/9. Product
allocation, CPU/RSS, percentiles, pixel/hit/edit/IME parity, and runtime fallback
frequency remain open; the operation model does not establish responsiveness.
