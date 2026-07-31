---
related_code:
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload.rs
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/press.rs
  - zircon_editor/src/ui/retained_host/asset_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-asset-pointer-full-surface-rebuild.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
tests:
  - drag_sources asset-specific tests inspected: 6
  - direct rustfmt check: passed for 3/3 current-source files
  - current-source managed Windows Cargo pending
  - click/drag-arm allocation counters and F4/WPR trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained asset-drag payload current review (2026-07-31)

## Scope

`zircon_editor/src/ui/retained_host/app/asset_drag_payload.rs` and `asset_drag_payload/**` are **3/3** clean Rust files, **129** physical lines, with path+raw-content SHA-256 `5abf6accc6e58f969019fe17cfbb76892290dfb7b707bd846b94fee491eb05ca`. All 3 files pass direct `rustfmt --check`.

The review traced content/reference press and target preparation, both pointer route/target definitions, committed asset snapshot access, the interface `UiDragPayload` / `UiDragSourceMetadata`, and the **6** asset-specific tests in the 895-line `app/tests/drag_sources.rs` suite.

## Findings

- Positive boundary: pointer press reuses the committed `Arc<AssetWorkspaceSnapshot>` and does not build a full editor snapshot. Payload construction occurs only on left-button press, rejects unknown surfaces/lists and unknown project references, and stale active payloads are cleared on errors/release.
- The content route already contains `{ row_index, item_index, asset_uuid }`, and the reference route contains `{ row_index, asset_uuid }`. Both press handlers discard the index and call helpers that linearly scan `visible_assets`, `references`, or `used_by` by UUID. A 10K-row press therefore repeats O(N) work after hit testing already resolved the row.
- Every ordinary left-button down immediately builds the full drag payload before any drag threshold is crossed. Content clones UUID, display name, extension, locator twice, formats kind, and converts source strings. Reference additionally formats `surface_mode.list_kind`, derives/allocates extension, and formats optional kind.
- After constructing the payload, press calls `source_summary()`, which allocates another summary `String`, then formats another status-line `String`. A click that never becomes a drag pays all scan/clone/format/status costs and clears the payload on pointer up.
- The final drag/drop/serde boundary legitimately needs owned data, but pointer down only needs a compact `DragCandidate { asset_generation, row_slot, source_kind }`. PERF-MVP-109's generation-owned row index should validate that slot in O(1), cancel stale generations, and materialize one payload only when the shared drag state crosses Begin/threshold. Editor09 owns immutable asset drag-source metadata; EditorUI01 owns the route/candidate transition. Neither consumer may add a second UUID map.
- `UiDragPayload` currently duplicates locator in `reference` and `source.locator`, and all metadata fields are owned strings. PERF-MVP-109 acceptance should count this ownership; any shared internal descriptor must preserve the public serde/ABI shape at the final boundary rather than changing drop semantics opportunistically.

## Reference and target

Godot `dev/godot/scene/main/viewport.cpp:2056-2099` accumulates mouse motion first and calls `get_drag_data` only after motion exceeds `drag_threshold`; it then publishes one drag-data object and sends Drag Begin. Zircon should follow the same lazy materialization principle while retaining its typed payload and deterministic route generation.

Acceptance should cover assets/references/used-by `1/100/10K`, main/floating Activity/Browser surfaces, click-only, threshold-never-crossed, actual drag, cancel, stale generation, unknown/external reference, and drop. For `1/1K/1M` click-only downs record row visits, UUID comparisons, payload builds, String alloc/clone/format bytes, summary/status writes, and UI p50/p95: scan=0, full payload=0, metadata/string/status bytes=0. Actual drag must resolve near O(1), materialize payload once, keep one authoritative metadata owner internally, and preserve source summary/control/surface/UUID/locator/kind/extension and drop results. Managed Cargo, F4 pointer trace, WPR/Tracy, and independent review remain required before moving this module to `review.md`.
