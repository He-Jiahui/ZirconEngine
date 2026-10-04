---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/text/mod.rs
  - zircon_runtime/src/core/framework/text/direction.rs
  - zircon_runtime/src/core/framework/text/font_face_handle.rs
  - zircon_runtime/src/core/framework/text/font_request.rs
  - zircon_runtime/src/core/framework/text/glyph.rs
  - zircon_runtime/src/core/framework/text/layout_error.rs
  - zircon_runtime/src/core/framework/text/layout_metrics.rs
  - zircon_runtime/src/core/framework/text/layout_service.rs
  - zircon_runtime/src/core/framework/text/open_type_feature.rs
  - zircon_runtime/src/core/framework/text/render_mode.rs
  - zircon_runtime/src/core/framework/text/shape_request.rs
  - zircon_runtime/src/core/framework/text/shape_result.rs
  - zircon_runtime/src/core/framework/text/shape_run.rs
  - zircon_runtime/src/core/framework/text/writing_mode.rs
  - zircon_runtime/src/text/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/text/direction.rs
  - zircon_runtime/src/core/framework/text/font_face_handle.rs
  - zircon_runtime/src/core/framework/text/font_request.rs
  - zircon_runtime/src/core/framework/text/glyph.rs
  - zircon_runtime/src/core/framework/text/layout_error.rs
  - zircon_runtime/src/core/framework/text/layout_metrics.rs
  - zircon_runtime/src/core/framework/text/layout_service.rs
  - zircon_runtime/src/core/framework/text/open_type_feature.rs
  - zircon_runtime/src/core/framework/text/render_mode.rs
  - zircon_runtime/src/core/framework/text/shape_request.rs
  - zircon_runtime/src/core/framework/text/shape_result.rs
  - zircon_runtime/src/core/framework/text/shape_run.rs
  - zircon_runtime/src/core/framework/text/writing_mode.rs
plan_sources:
  - user: 2026-07-14 implement the approved runtime architecture plan and update milestone status
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/05/baselines/2026-07-10-contract-signatures.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
tests:
  - zircon_runtime/src/core/framework/text/tests.rs
  - tools/tests/test_frameworks_05_text_boundary.py
doc_type: module-detail
---

# Runtime Text Framework Contracts

`TextLayoutService` is the only production shaping contract. The runtime text
owner provides `SharedTextLayoutService`; UI and graphics both depend on the
neutral `TextShapeRequest` / `TextShapeResult` DTOs. Detailed backend request
types remain private to `zircon_runtime::text` and are not a second public API.

## Purpose

`zircon_runtime::core::framework::text` is the neutral contract owner for text shaping and layout. It lets UI, graphics, scene text, editor hosts, and later plugins describe text work without importing a neighboring subsystem implementation.

The module is intentionally implementation-free. It does not own font discovery, shaping backends, line breaking algorithms, glyph rasterization, atlas allocation, GPU upload, UI layout state, or render submission. Those behaviors belong to the runtime text implementation domain and its consumers.

The implementation domain is now physically owned by `zircon_runtime::text`. The retired `core/framework/render/text` and `graphics/text` directories are deleted, with no compatibility re-export or forwarding module.

## Related Files

The root `mod.rs` is structural wiring only. Each contract concept has a narrow declaration file:

- `direction.rs`, `writing_mode.rs`, and `render_mode.rs` define portable policy enums.
- `font_request.rs` describes logical family, asset, size, weight, stretch, italic, and render-mode intent without exposing a font database.
- `font_face_handle.rs` carries typed index+generation identity for a resolved face without exposing a font database slot implementation.
- `shape_request.rs` combines source text, language, direction, writing mode, font intent, and borrowed OpenType feature settings.
- `open_type_feature.rs` defines the serializable feature tag/value DTO without exposing backend shaping types.
- `glyph.rs`, `shape_run.rs`, `layout_metrics.rs`, and `shape_result.rs` describe backend-neutral results.
- `layout_service.rs` defines the consumer-facing service trait.
- `layout_error.rs` preserves typed failures at the contract boundary.

## Behavior Model

`TextLayoutService` exposes three operations from the approved Frameworks05 signature baseline:

1. `resolve_render_mode` maps `Auto` or an explicit request to the backend mode selected by the implementation.
2. `resolve_direction` resolves automatic direction while preserving an explicit left-to-right or right-to-left request.
3. `shape` transforms a `TextShapeRequest` into a `TextShapeResult` or a typed `TextLayoutError`.

`TextShapeResult` returns one or more logical runs, aggregate layout metrics, and the resolved direction. A `TextShapeRun` owns its source range, direction, and glyph placements. Each `TextGlyph` carries source mapping, advance, offset, and optional face/instance handles. `TextFontFaceHandle` is a registry slot plus the shared font-database generation; reload invalidates old handles, and stale projection cannot roll the registry back to an earlier generation.

## Design and Rationale

Frameworks05 requires graphics-to-UI and UI-to-graphics text references to reach zero before Frameworks01 can extract `zr_text`. The neutral contract therefore lives beside other framework domains instead of under `framework::render`: shaping and measurement are needed before a renderer is selected, and headless or asset tooling must be able to use them without enabling graphics.

The interface follows Bevy's separate `TextPipeline` resource and Godot's replaceable text backend boundary, translated into the repository vocabulary. Zircon does not use non-network `server` naming and does not expose backend objects or global singleton access through this contract.

The DTO path is batch-oriented. A shape request returns complete runs and metrics instead of resolving a service or dispatching dynamically for every glyph. Font database and atlas handles remain implementation details until a stable resource contract is needed by more than one existing consumer.

## Control Flow

The intended runtime flow is:

1. A UI or scene-text consumer creates a neutral `TextFontRequest` and `TextShapeRequest`.
2. The runtime text implementation resolves mode and direction, validates the request, and invokes its shaping/layout backend.
3. The implementation returns `TextShapeResult` DTOs.
4. UI uses metrics and source ranges for measure, wrapping, selection, and hit testing; graphics consumes glyph placement plus implementation-owned atlas output for rendering.

No consumer should retain a concrete shaping backend or import `graphics::text`/`ui::text` to complete this flow.

## Edge Cases and Constraints

- Font size must be finite and positive; invalid sizes return `TextLayoutError::InvalidFontSize`.
- `TextLayoutError` is non-exhaustive and distinguishes invalid language, unavailable fonts, exhausted fallback, unsupported writing/render modes, shaping failure, layout failure, and backend absence. Consumers must keep a fallback match arm so future backend-neutral failure classes can be added without breaking them.
- Source ranges must remain byte ranges into the original UTF-8 source and preserve cluster coverage for ligatures, combining marks, BiDi, and vertical text.
- `Auto` direction and render mode are requests for implementation resolution, not serialization aliases for a resolved value.
- The contract may use standard library and lightweight serialization types, but it must not import UI, graphics, wgpu, fontdb, glyphon, or a concrete shaping backend.
- Requests and shape results are in-process call DTOs, not persisted or cross-process wire formats. Result structs intentionally do not implement serde; authored direction/writing/render policy enums remain serializable for asset/config use.
- Face lifetime and cache invalidation remain implementation-owned, while `TextFontFaceHandle { index, generation }` gives consumers enough identity to reject a stale cached result after reload.

## Test Coverage

`zircon_runtime/src/core/framework/text/tests.rs` supplies a small recording implementation that verifies mode resolution, automatic direction, result metrics, typed invalid-font-size failure, and generation-sensitive face identity without importing an engine domain.

`tools/tests/test_frameworks_05_text_boundary.py` is the architecture acceptance guard. It reuses the production Rust lexical view and dependency audit so grouped imports, aliases, comments, strings, and `cfg(test)` owners do not bypass or pollute the result. It now also rejects backend shaping in the parallel prewarm pool, Graphics CPU SDF cache/database ownership, and missing layout-fallback projection. These new assertions must pass on current source before M3 acceptance; the previous 12/12 result predates the blind-spot additions.

The Windows managed target-server profile check passed as coordinator job `fc80663e0c454fe5a8e7aaa30e9ac684`, proving the neutral contract compiles without enabling graphics or UI backend features. This is a contract compile gate, not the full M3 behavior or rendering gate.

The fresh Windows managed production compile after canonical prewarm, versioned registry hardening, typed RenderStats projection, and SDF CPU owner migration is coordinator job `4659a570a86d4c73b752dceb53e58eb4`, `released / exit 0`.

The real Windows GPU product gate, coordinator job `5039b9c015114ebb9b03f1fcc009ac81`, passed 1/1 in 499.45 seconds. It produced the sole 1080×2000 framebuffer at `docs/tests/runtime/text/runtime_text_mixed_bidi_source_geometry_product_framebuffer_20260715.png`, SHA-256 `30793B75AC50FD95B558DAFE5B8B98C9DB6C67737FF9B409FF6CCEFE53384D42`; no same-named file exists under the managed Cargo targets.

## Plan Sources

Frameworks05 M3 owns the immediate boundary cut. The M1 contract-signature baseline fixes the public shape. Render14 owns text rendering behavior, caching, and atlas expectations. Frameworks01 M3 will later extract the completed runtime text domain into `zr_text` without reintroducing cross-domain paths.

## Open Issues

- Adapt remaining runtime implementation model records that embed `UiResolvedStyle` toward the smaller neutral `TextFontRequest`/`TextShapeRequest` contract when both existing consumers can use it without losing source mapping or rich layout behavior.
- Extract `zircon_runtime::text` into `zr_text` under Frameworks01 M3 without restoring either retired namespace.
