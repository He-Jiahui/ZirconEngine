---
related_code:
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_layout.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/build_dispatcher.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/precision_candidates_from_layout.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/handle_candidate.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/scene_gizmo_candidate.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidate.rs
  - zircon_editor/src/scene/viewport/pointer/precision/candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/precision/precision_candidate.rs
  - zircon_editor/src/scene/viewport/pointer/precision/precision_candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/precision/precision_shape.rs
  - zircon_editor/src/scene/viewport/pointer/tests.rs
  - zircon_runtime/src/core/framework/picking/mod.rs
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/core/framework/picking/hit_target.rs
implementation_files:
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/build_dispatcher.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/precision_candidates_from_layout.rs
  - zircon_editor/src/scene/viewport/pointer/precision/candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/precision/precision_candidate.rs
  - zircon_editor/src/scene/viewport/pointer/precision/precision_candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/tests.rs
plan_sources:
  - .codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_editor/src/scene/viewport/pointer/tests.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1
doc_type: module-detail
---

# Scene Viewport Pointer Routing

## Purpose

`zircon_editor::scene::viewport::pointer` adapts editor viewport overlay hit candidates into the runtime picking contract. The editor still owns the Slint/UI pointer event bridge, authoring-only overlay layout, selection commands, undo grouping, and viewport tool UX. Runtime picking owns the shared target vocabulary and category ordering through `HitTarget`, `PointerHits`, and `hovered_hits_for_pointer`.

This is the M2 editor-adapter step in the Bevy completion plan. It does not yet replace the whole editor selection or transform-handle flow, but it removes the first authoritative editor-only route resolver from the pointer dispatcher.

## Data Flow

`ViewportPointerLayout` is rebuilt from editor overlays: transform handles, scene gizmos, and coarse renderable candidates. The candidate builders still perform editor-local screen-space projection because they consume authoring overlay DTOs and current editor camera state.

`rebuild_surface` maps those precision candidates into the retained UI surface so the existing UI pointer dispatcher can provide the stacked nodes under a cursor. On move/down/up/scroll, `build_dispatcher` now calls `runtime_picking_adapter::resolve_runtime_route`.

The adapter converts every stacked precision candidate that scores at the cursor into one runtime `PointerHits` output for the editor overlay backend. Each hit uses `ViewportPointerRoute::target()` to produce the runtime `HitTarget`. `hovered_hits_for_pointer` then selects the top target using the same runtime ordering that runtime, gizmo, and remote tooling will consume. Grouping all overlay candidates into one backend output keeps the debug/report contract honest: overlapping editor hits are multiple raw hits from one editor overlay backend, not multiple fake backend outputs.

The precision score DTO now carries only screen-space distance and projected depth. Candidate category priority is intentionally not stored there anymore: UI z-index may still decide which retained UI nodes appear in the cursor stack, but authoritative handle/gizmo/renderable ordering is the runtime `HitTarget` priority inside `sorted_hits_for_pointer`.

`ViewportOverlayPointerRouter::debug_feed_at` exposes the same adapter as a read-only devtools path. It asks the retained UI surface for the current hit stack at a point, locks the shared candidate map, and returns `PickingDebugFeed` built from the runtime `PickingPipelineReport`. The normal `ViewportPointerDispatch` also carries this feed after move/down dispatch, so overlay/devtools panels can display raw hit counts, hovered hit counts, and top/blocking runtime targets without reimplementing the editor route resolver.

The normal dispatch result also carries `PointerInput` through `ViewportPointerDispatch::runtime_input`. The conversion is intentionally thin: UI move/down/up/scroll events become runtime pointer actions with the editor viewport pointer id and a stable viewport handle. Release and scroll now use the same runtime resolver as hover and press, so route/debug consumers see one runtime picking view for the full UI pointer event vocabulary currently exposed by `UiPointerEventKind`. The current UI event payload only stores the absolute cursor, so move deltas are emitted as `Vec2::ZERO` until a later stateful input collector owns previous-position tracking.

The router's production callers still expose the existing move/down entry points used by viewport selection. Up/scroll convenience entry points are test-only because no production viewport controller calls them yet; production Up/Scroll route support comes from `build_dispatcher` registering those `UiPointerEventKind`s and from `runtime_pointer_input_for_event` mapping them into runtime DTOs.

## Boundary

The editor adapter is intentionally thin:

- It may compute screen-space candidate shapes because those shapes come from editor-only overlays.
- It may encode precision score into `HitData.depth` for the temporary screen-space backend.
- It must not define an independent handle/gizmo/renderable category order.
- It may map UI pointer events into runtime `PointerInput`, but it must not synthesize higher-level picking events in editor code.
- Debug/devtools reads must consume `PickingDebugFeed` from the runtime report path rather than reading `ViewportPointerRoute` as the source of truth.
- It must not emit undo, selection, or scene mutations from runtime picking itself.

The next M2 steps should move more of the candidate backend shape into reusable runtime primitives or debug feeds, but editor UX should remain the consumer of the final `ViewportPointerRoute`.

The pointer subsystem root file is intentionally structural. Focused route-adapter tests live in `zircon_editor/src/scene/viewport/pointer/tests.rs`, and that file includes a source guard that keeps `pointer/mod.rs` free of inline behavior while preventing the deleted editor-only `better_score` / `resolve_best_route` resolver path from being reintroduced.

## Validation

`zircon_editor/src/tests/editing/viewport.rs` covers the adapter through the public router behavior:

- handle axis beats overlapping renderable candidate,
- scene gizmo beats overlapping renderable candidate,
- renderable resolves when no overlay hits exist,
- editor routes round-trip to runtime `HitTarget`,
- runtime `HitTarget` priority preserves handle axis, scene gizmo, then renderable order.
- the editor router can expose a runtime `PickingDebugFeed` for overlapping overlay candidates before dispatching a move event.
- the editor dispatch bridge carries runtime `PointerInput` for move, down, release, and scroll-shaped input.
- release and scroll dispatch return the same runtime-picked route and debug feed as move/down for overlapping viewport overlay candidates.
- overlapping editor overlay candidates are reported as one runtime backend output with multiple raw hits.
- the direct runtime resolver tests prove handle and scene gizmo targets beat a renderable even when the renderable is earlier in the UI stack or has a better projected depth.
- the source guard keeps the pointer root structural and keeps the overlay dispatcher routed through `resolve_runtime_route` instead of the removed editor-only resolver.

Fresh validation evidence should be recorded here whenever the viewport pointer adapter changes.

Latest continuation note:

- 2026-05-25: pointer route-adapter focused tests were moved from `zircon_editor/src/scene/viewport/pointer/mod.rs` to `zircon_editor/src/scene/viewport/pointer/tests.rs` so the subsystem root stays structural. The new source guard asserts that `pointer/mod.rs` does not regain inline behavior or inline test bodies, that `better_score` / `resolve_best_route` remain absent from the overlay router path, and that the dispatcher continues to call `resolve_runtime_route`.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/tests.rs` passed on 2026-05-25.
- `git diff --check -- zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/tests.rs docs/zircon_editor/scene/viewport/pointer.md .codex/sessions/20260525-2142-runtime-picking-continuation.md` passed on 2026-05-25 with CRLF warnings only.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260525-2142 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib pointer_module_keeps_runtime_route_adapter_as_authoritative_resolver --locked --color never --jobs 1` passed on 2026-05-25: 1 focused source-guard test passed, 0 failed, 1507 filtered out. Cargo emitted existing warning noise in runtime UI/text/profile helpers and editor sprite-atlas exports.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260525-2142 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1` passed on 2026-05-25 after the target directory was warmed: 83 focused viewport tests passed, 0 failed, 1425 filtered out. Earlier attempts timed out during compilation and did not produce test failures.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-m1-20260525-2142 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` passed on 2026-05-25: 20 focused picking tests passed, 0 failed, 2031 filtered out. Cargo emitted existing warning noise outside the picking framework.

Latest review follow-up note:

- 2026-05-24/25: the editor runtime picking adapter now packs all scored overlay candidates at a cursor into one `PointerHits` output for the editor overlay backend, and the debug-feed regression asserts one backend output with two raw hits for overlapping handle/renderable candidates. `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs` passed. `git diff --check -- zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs docs/zircon_editor/scene/viewport/pointer.md docs/zircon_runtime/core/framework/picking.md .codex/sessions/20260524-1739-review-followup-deferred.md` passed with CRLF warnings only. Focused Cargo validation attempts first timed out during compilation, then hit a transient root lockfile blocker, then compiled but ran 0 tests because `--exact` did not match the fully qualified lib-test name. The warmed lib-test binary was then run directly with the correct substring filter: `& "E:\cargo-targets\zircon-review-followup\debug\deps\zircon_editor-88ef940533c92e4c.exe" overlay_router_debug_feed_reports_runtime_picking_route_at_point --test-threads=1 --nocapture` passed with 1 passed, 0 failed, 1463 filtered out.

Latest local evidence:

- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/overlay_router/mod.rs` passed on 2026-05-22.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_debug_feed_reports_runtime_picking_route_at_point --locked --color never --jobs 1` passed on 2026-05-22: 1 focused debug-feed test passed, 0 failed, 1422 filtered out.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1` passed on 2026-05-22 after the debug-feed dispatch slice: 79 focused viewport tests passed, 0 failed, 1344 filtered out. Cargo emitted existing warnings in runtime UI text/ECS helpers and editor sprite-atlas exports.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-debug-20260522-0045 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` was attempted on 2026-05-22 but did not enter tests because unrelated active manifest edits in `zircon_app/Cargo.toml`, `zircon_plugins/net/features/content_download/runtime/Cargo.toml`, and `zircon_plugins/sound/runtime/Cargo.toml` made Cargo require a `Cargo.lock` update. The picking lane did not run non-locked validation to avoid writing another session's lockfile changes.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs` passed on 2026-05-22 after adding `PointerInput` dispatch output.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_ --locked --color never --jobs 1` was attempted on 2026-05-22 for the input DTO follow-up but did not enter tests because the same unrelated active manifest edits required a `Cargo.lock` update. The picking lane again left `Cargo.lock` untouched.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs zircon_runtime/src/core/framework/picking/debug_feed.rs zircon_runtime/src/core/framework/picking/mod.rs zircon_runtime/src/tests/picking/mod.rs` passed on 2026-05-22 before the resolver source-guard follow-up.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_ --locked --color never --jobs 1` was retried on 2026-05-22 but still did not enter tests because unrelated active manifest edits required a `Cargo.lock` update.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/overlay_router/build_dispatcher.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/precision/candidate_score.rs zircon_editor/src/scene/viewport/pointer/precision/precision_candidate_score.rs` passed on 2026-05-23 after the up/scroll debug-feed coverage follow-up.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_dispatch_maps_release_and_scroll_through_runtime_pointer_input --locked --color never --jobs 1` was attempted on 2026-05-23 but did not enter tests because unrelated active manifest edits in `zircon_app/Cargo.toml`, `zircon_plugins/net/features/content_download/runtime/Cargo.toml`, and `zircon_plugins/sound/runtime/Cargo.toml` still require a `Cargo.lock` update.
- `Get-ChildItem zircon_editor/src/scene/viewport/pointer -Recurse -File -Include *.rs | Select-String -Pattern 'resolve_best_route|better_score'` returned no source matches on 2026-05-23, confirming the old editor-only resolver names are absent from the pointer source tree.
- `Select-String` source guard for `CandidateScore|priority:` on 2026-05-23 showed `CandidateScore` no longer stores priority; remaining priority matches are candidate z-index inputs, runtime `PickingTargetPriority` constants, and test candidate setup.
- Directly running the warmed editor lib-test binary on 2026-05-25 passed `overlay_router_debug_feed_reports_runtime_picking_route_at_point`: 1 passed, 0 failed, 1463 filtered out.
- Directly running the warmed editor lib-test binary on 2026-05-25 passed `overlay_router_dispatch_maps_release_and_scroll_through_runtime_pointer_input`: 1 passed, 0 failed, 1463 filtered out.
- On 2026-05-25, the direct resolver coverage was extended with `runtime_route_resolution_prefers_scene_gizmo_over_renderable_depth`; Cargo execution for the new test is pending because unrelated shared Cargo jobs were still active.
- `rustfmt --edition 2021 --check` passed on the touched editor pointer and runtime UI helper files on 2026-05-21.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo check -p zircon_editor --lib --locked --color never --jobs 1` passed on 2026-05-21. It emitted existing warning noise in runtime ECS and editor sprite-atlas modules.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1` passed on 2026-05-21: 78 focused viewport tests passed, 0 failed, 1342 filtered out.
- `cargo fmt --all --check` passed on 2026-05-21 after the M2 adapter slice.
