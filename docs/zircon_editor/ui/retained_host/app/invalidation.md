---
related_code:
  - zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask/requirements.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask/summary.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/reasons.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/requests.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/tests.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style.rs
  - zircon_editor/src/ui/retained_host/event_bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/effects.rs
  - zircon_editor/src/ui/retained_host/drawer_resize.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
implementation_files:
  - zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask/requirements.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/mask/summary.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/reasons.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/requests.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/tests.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events/style.rs
  - zircon_editor/src/ui/retained_host/event_bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/effects.rs
  - zircon_editor/src/ui/retained_host/drawer_resize.rs
plan_sources:
  - user: 2026-05-12 continue closing global dirty-domain incremental refresh gaps for the UI v2 cutover
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/tests/host/retained_asset_refresh/catalog_refresh.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/host/retained_event_bridge/asset_refresh_effects.rs
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
  - 2026-05-13 dirty-preview-validation: cargo test -p zircon_editor --lib asset_preview_refresh_effect_is_paint_only_without_backend_sync --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 dirty-preview-validation: cargo test -p zircon_editor --lib asset_details_refresh_effect_does_not_require_backend_sync --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 dirty-preview-validation: cargo test -p zircon_editor --lib retained_event_effects_route_dirty_domains_through_invalidation_mask --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 backend-preview-dirty-domain-validation: cargo test -p zircon_editor --lib retained_asset_refresh --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 5 tests)
  - 2026-05-13 backend-preview-dirty-domain-validation: cargo test -p zircon_editor --lib asset_preview_refresh_effect_is_paint_only_without_backend_sync --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 backend-preview-dirty-domain-validation: cargo test -p zircon_editor --lib retained_event_effects_route_dirty_domains_through_invalidation_mask --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-05-13 viewport-render-domain-validation: cargo test -p zircon_editor --lib retained_event_bridge --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 8 tests)
  - 2026-05-13 viewport-render-domain-validation: cargo test -p zircon_editor --lib retained_callback_dispatch::viewport --jobs 1 --target-dir target\codex-ui-v2-guard (passed after adding structural viewport render domain, 11 tests)
  - 2026-05-13 viewport-render-domain-validation: cargo test -p zircon_editor --lib retained_event_effects_route_dirty_domains_through_invalidation_mask --jobs 1 --target-dir target\codex-ui-v2-guard (passed, 1 test)
  - 2026-06-18 editor-ui-architecture-validation: cargo fmt -p zircon_editor (passed)
  - 2026-06-18 editor-ui-architecture-validation: cargo fmt -p zircon_editor --check (passed)
  - 2026-06-18 editor-ui-architecture-validation: app invalidation mask ownership scan (passed)
  - 2026-06-19 editor-ui-architecture-validation: app invalidation mask requirement/summary ownership scan (passed)
  - 2026-06-18 editor-ui-architecture-validation: scoped git diff --check (passed with CRLF warnings only)
  - 2026-06-18 editor-ui-architecture-validation: cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (passed with existing warnings)
  - 2026-06-19 editor-ui-architecture-validation: app invalidation root/tests ownership scan (passed)
  - 2026-06-19 editor-ui-architecture-validation: app invalidation root requests/reasons/diagnostics ownership scan (passed)
doc_type: module-detail
---

# Retained Host Invalidation

The retained editor host uses `HostInvalidationRoot` as the owner of refresh reasons. Callers outside `host_lifecycle.rs` should not write the legacy dirty booleans directly. They report a dirty domain through lifecycle helpers such as `mark_presentation_dirty`, `mark_layout_dirty`, `mark_render_and_presentation_dirty`, `record_paint_only_invalidation`, or the lower-level `invalidate_host` entrypoint.

The legacy booleans still exist as a short-term bridge for the current retained host loop, but they are derived from `HostInvalidationMask` inside `invalidate_host`. This keeps slow-path rebuild diagnostics, render-path diagnostics, paint-only fast paths, and future dirty-domain scheduling on the same source of truth.

## Mask Ownership

`app/invalidation/mask.rs` is the structural dirty-domain mask entry. It owns `HostInvalidationMask`, the dirty-domain bit definitions, bitwise union/intersection mutation helpers, and dirty-flag conversion. `mask/requirements.rs` owns recompute requirement predicates. `mask/summary.rs` owns human-readable summaries. This keeps the domain vocabulary independent from root scheduling counters while separating scheduling predicates from diagnostic formatting.

`app/invalidation.rs` is the structural entry for retained-host invalidation. It re-exports `HostInvalidationMask` and the app-internal `HostInvalidationRoot` while keeping mask vocabulary, root scheduling state, and tests in separate children.

`app/invalidation/root.rs` owns the `HostInvalidationRoot` state shape and child module boundary. `root/requests.rs` owns initial full-rebuild invalidation and request counters, `root/reasons.rs` owns recompute reason draining plus slow/render rebuild counters, and `root/diagnostics.rs` owns stats summaries and diagnostics snapshots. The root subtree consumes the mask API but does not redefine dirty-domain constants or presentation/layout/render predicate logic.

`app/invalidation/root/tests.rs` owns the root scheduling regressions. Keeping those tests under the root child lets them inspect root-private counters while the public module entry remains structural.

## Event Effects

`UiHostEventEffects` now carries a `dirty_domains: HostInvalidationMask` field. Event translation uses `request_presentation`, `request_layout`, `request_render_and_presentation`, and `request_paint_only` so callback results keep the exact domain that changed. The old `presentation_dirty`, `layout_dirty`, and `render_dirty` fields remain as compatibility mirrors for existing assertions and call sites, but host lifecycle consumes event effects through `effects.dirty_domains()` before touching the bridge booleans.

Callback effect fan-in also preserves the mask: common merge code and drawer-resize aggregation merge `source.dirty_domains()` instead of OR-ing the legacy fields. This is the next step toward global dirty-domain incremental refresh because every retained callback can now report presentation/layout/render work in the same format used by `HostInvalidationRoot`.

`AssetPreviewRefreshRequested` is a paint-only event-effect route. It asks the editor asset manager to refresh visible previews but does not mark `presentation_dirty` by itself, so standalone preview-refresh requests are counted as `PAINT_ONLY` and stay eligible for the fast path. Asset details refresh remains presentation-domain because it mutates the selected asset details snapshot consumed by retained pane projection.

The backend asset refresh plan now keeps the same distinction after the preview generator finishes. `EditorAssetChangeKind::PreviewChanged` synchronizes the editor catalog snapshot so the runtime sees the latest stable preview record, but it marks only `PAINT_ONLY` and queues a host redraw over the shell frame. Catalog/reference/resource/default-scene changes still mark `PRESENTATION_DATA` or `RENDER` because those events can change pane structure, selected details, resource handles, or viewport content.

`RenderChanged` now means the render domain only. Call sites that also change retained chrome or reflection data must emit `PresentationChanged` or `ReflectionChanged` explicitly. Viewport pointer execution uses that split for high-frequency input: idle pointer moves and press/release bookkeeping can return no dirty domain, camera scroll/orbit/pan can return render-only, and gizmo transform or hover feedback can still opt into presentation/reflection when the visible chrome or inspector data actually changes.

## Presentation Domain

UI Asset Editor actions now route successful model, source, palette-drag, and component-adapter changes through `mark_presentation_dirty()`. This records the `PRESENTATION_DATA` domain before setting the bridge boolean, so the next `recompute_if_dirty()` can attribute the work correctly instead of seeing an anonymous `presentation_dirty = true`.

This matters for the v2 UI cutover because component events can produce small props/state/projection patches. Those patches should first be counted as presentation-domain invalidations; later work can narrow them further into style/text/render domains without losing provenance.

## Guard Coverage

`host_dirty_flags_route_through_invalidation_root_outside_lifecycle_owner` scans retained-host app sources and rejects direct writes to `presentation_dirty`, `layout_dirty`, `window_metrics_dirty`, and `render_dirty` outside `host_lifecycle.rs`. `host_lifecycle.rs` remains the bridge owner that translates masks into the old booleans until the old flags can be removed entirely.

`retained_event_effects_route_dirty_domains_through_invalidation_mask` verifies the callback/event layer uses the event-effect mask for host invalidation. It also rejects new dirty flag OR-assignment or direct true-assignment in retained-host modules outside the bridge owner files.

## Validation Notes

The 2026-06-18 mask split reduced `invalidation.rs` from 287 lines to 174 lines. `invalidation/mask.rs` is 115 lines and owns dirty-domain constants, bit operations, dirty-flag conversion, recompute predicates, and summary formatting.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app invalidation mask ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 root/tests split reduced `invalidation.rs` from 174 lines to 4 lines. `invalidation/root.rs` is 103 lines and owns invalidation scheduling state, counters, recompute reason consumption, rebuild counters, summary formatting, and diagnostics snapshots. `invalidation/root/tests.rs` is 67 lines and owns the root scheduling regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app invalidation root/tests ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). The first cargo check exposed app-sibling visibility after the root moved; the root type and methods were kept app-internal with `pub(in crate::ui::retained_host::app)`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 mask requirement/summary split reduced `invalidation/mask.rs` from 132 lines to a 68-line structural dirty-domain entry. `mask/requirements.rs` is 34 lines and owns layout/presentation/render/window-metrics/hit-test/host-recompute predicates. `mask/summary.rs` is 39 lines and owns dirty-domain summary labels.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app invalidation mask requirement/summary ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. Focused `cargo check` was not rerun for this slice because an independent `zircon_runtime` Cargo test process was active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 root requests/reasons/diagnostics split reduced `invalidation/root.rs` from 103 lines to a 22-line retained state entry. `root/requests.rs` is 49 lines and owns invalidation request accounting, `root/reasons.rs` is 31 lines and owns recompute reason consumption plus rebuild counters, and `root/diagnostics.rs` is 30 lines and owns stats summary/diagnostic snapshot construction.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app invalidation root requests/reasons/diagnostics ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
