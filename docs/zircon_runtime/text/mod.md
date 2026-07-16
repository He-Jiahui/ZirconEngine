---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/model/mod.rs
  - zircon_runtime/src/text/font/mod.rs
  - zircon_runtime/src/text/shaping/mod.rs
  - zircon_runtime/src/text/layout/mod.rs
  - zircon_runtime/src/text/raster/mod.rs
  - zircon_runtime/src/text/atlas/mod.rs
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/parallel/mod.rs
  - zircon_runtime/src/text/rich/mod.rs
  - zircon_runtime/src/text/sdf/mod.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - zircon_runtime/src/text/font/source_manifest.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
implementation_files:
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/model/mod.rs
  - zircon_runtime/src/text/font/mod.rs
  - zircon_runtime/src/text/shaping/mod.rs
  - zircon_runtime/src/text/layout/mod.rs
  - zircon_runtime/src/text/raster/mod.rs
  - zircon_runtime/src/text/atlas/mod.rs
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/parallel/mod.rs
  - zircon_runtime/src/text/rich/mod.rs
  - zircon_runtime/src/text/sdf/mod.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - zircon_runtime/src/text/font/source_manifest.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/render_state.rs
plan_sources:
  - user: 2026-07-11 hard-cut all old runtime architecture to the new plan
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
tests:
  - tools/tests/test_frameworks_05_text_boundary.py
  - zircon_runtime/src/text/cache/tests.rs
  - zircon_runtime/src/text/shaping/tests.rs
  - zircon_runtime/src/text/layout/tests.rs
  - zircon_runtime/src/text/raster/swash/tests.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
doc_type: module-detail
---

# Runtime Text Implementation Domain

The production `SharedTextLayoutService` implements the neutral
`core::framework::text::TextLayoutService` contract. UI measure/layout and
parallel prewarm both enter `SharedTextLayoutSession` and the same canonical
adapter; graphics consumes the same request/result contract and no longer calls
the backend shaper directly.

CPU font discovery/database state, shaping support state, raster workers, bitmap
source/retry caches, native bitmap atlas state, SDF font/glyph/offline caches, and
SDF pixel/metric generation live under this text owner through `TextRenderState`,
`native_bitmap_atlas`, and `sdf/font_bake`. `prepare_sdf_runs_cpu` returns one
batched CPU preparation per run, including glyph metrics, resolved/fallback
advances, and decoration metrics. Graphics owns only atlas planning plus GPU
upload, vertex/material preparation, and draw resources for those Text-produced
results.

## Purpose

`zircon_runtime::text` is the single runtime implementation owner for font discovery, shaping, line layout, rich-text parsing, glyph rasterization, atlas planning, caches, and parallel text work. UI and graphics are consumers: UI owns widget/layout state and hit testing; graphics owns GPU scene submission. Neither domain owns or re-exports the text backend.

This owner replaces both retired paths:

- `zircon_runtime::graphics::text`, which incorrectly placed shared CPU text behavior under a renderer domain;
- `zircon_runtime::core::framework::render::text`, which mixed reusable text records with render contracts.

The migration is a hard cut. There is no module alias, facade re-export, compatibility trait, or forwarding wrapper at either retired path.

## Related Files

`text/mod.rs` wires the domain and exposes the runtime text vocabulary. The implementation is folder-backed by responsibility:

- `model/` owns the existing shaped-run, font descriptor, and rich-layout records used by current UI and renderer consumers;
- `font/` owns database discovery, matching, fallback, composite fonts, face bytes, decoration metrics, and generation tracking;
- `shaping/` owns BiDi/script segmentation, horizontal and vertical shaping, fallback spans, and backend projection;
- `layout/` owns measurement, line breaking, wrapping, justification, kinsoku, tabs, rich layout, and vertical layout;
- `raster/`, `sdf/`, and `atlas/` own glyph pixels, dynamic/offline distance-field generation, CPU font/glyph caches, atlas residency/upload contracts, and renderer-facing draw plans;
- `cache/` and `parallel/` own bounded reuse and task-pool execution;
- `rich/` owns BBCode/HTML subset parsing and decorator registration.

The smaller `core/framework/text` package is a separate neutral service-contract surface. It must remain backend-free and must not absorb these implementation modules.

## Behavior Model

Text work starts with source text plus style, language, writing direction, font request, and render intent. The implementation resolves a font face and generation, segments text into script/BiDi spans, shapes glyph clusters, performs horizontal or vertical layout, and returns source-mapped glyph runs and metrics. Consumers may then use the same result for UI measurement/hit testing or for native/SDF/MSDF atlas preparation.

Font identities include generation-bearing records. Cached shaped runs and glyph data must therefore be invalidated when the database generation changes. Graphics preserves `TextFontFaceHandle { index, generation }` in its shaped-glyph projection; only Text resolves that handle through the generation-checked registry. Atlas/SDF keys never extract or reconstruct raw backend IDs. Atlas and raster code consume resolved identities; they do not rediscover fonts per glyph or retain a neighboring UI/graphics manager as an implicit owner.

## Design and Rationale

Text is shared infrastructure, not a graphics submodule. Moving the complete implementation together preserves internal cohesion: shaping, fallback, layout, raster, atlas, and caches can call one another without creating UI↔graphics dependency edges. It also establishes the physical directory shape required for the later Frameworks01 `zr_text` crate extraction without another business-logic rewrite.

The existing model records remain in `text/model` because they are part of the runtime implementation vocabulary and some carry current UI-surface fields. New cross-domain service contracts belong in `core/framework/text`; backend-specific state and rich rendering details do not. This separation avoids pretending that every existing shaped-run record is already a serialization-neutral public contract.

The graphics module identity constant moved to `core/framework/render::GRAPHICS_MODULE_NAME`. Runtime UI can declare its module dependency without importing the graphics implementation domain, while graphics host assembly consumes the same neutral identity.

## Control Flow

1. UI or a scene renderer supplies source/style data to `zircon_runtime::text` shaping/layout entry points.
2. `font` resolves the face and variation instance; `shaping` builds glyph clusters and source ranges.
3. `layout` computes lines, wrapping, tables, vertical columns, metrics, and hit-testable positions.
4. `cache` and `parallel` reuse or batch work at run/paragraph granularity; prewarm misses call the same canonical adapter before entering the shared shaped-run cache.
5. UI consumes metrics and ranges. Graphics consumes resolved runs, calls narrow `TextRenderState` native/SDF preparation methods, uploads Text-produced pixels, and submits GPU draws.
6. Typed layout fallbacks remain visible through `TextLayoutFallbackReport` and are projected into frame `RenderStats`; they are not silently converted into an unobservable empty result.

## Edge Cases and Constraints

- Old `graphics::text` and `framework::render::text` paths are forbidden even in new tests; callers must migrate to `text` or `core/framework/text` according to ownership.
- Production text code must not import `crate::ui::text` or `crate::graphics::text`.
- UI-to-graphics imports are forbidden for text plumbing. Neutral module identity and runtime text APIs are the allowed dependency shape.
- Source byte ranges, grapheme/cluster mapping, BiDi levels, vertical rotation, fallback spans, and variation coordinates must survive the move unchanged.
- Hot paths remain batch-oriented. The hard cut changes ownership and imports, not shaping, raster, cache, or draw behavior.
- `text` is enabled by the `text` feature; graphics depends on that feature, while the neutral framework contract can compile in non-graphics profiles.

## Test Coverage

`tools/tests/test_frameworks_05_text_boundary.py` is the architecture gate. It verifies the neutral framework owner, absence of retired paths and compatibility exports, canonical UI/prewarm entry, RenderStats error projection, zero production UI→graphics/text-path edges, and absence of Graphics-owned CPU font/raster/SDF caches or database closures. The pre-fix gate passed 12/12 but missed prewarm and SDF ownership; those blind spots are now explicit assertions and require a fresh current-source run before M3 acceptance.

The fresh Windows managed production compile after the canonical-prewarm and SDF-owner repair is coordinator job `4659a570a86d4c73b752dceb53e58eb4`, `released / exit 0`. Focused lib tests, the product framebuffer/editor-host smoke, and independent re-review remain milestone-stage gates.

## Plan Sources

Frameworks05 M3 owns this physical boundary cut. Frameworks01 defines the future `zr_text` crate destination, and Render14 owns observable text rendering behavior. The engine structure/review plans require the folder-backed owner and prohibit leaving old facade paths behind.

## Open Issues

- Complete managed focused/full Runtime text tests after the shared Cargo test pool is available.
- Rerun the multilingual product framebuffer and add the still-missing editor-host text screenshot.
- Obtain fresh independent review with Critical/Important 0/0 before coordinator milestone commit.
- Extract this directory into `zr_text` only in Frameworks01 M3; do not create another interim compatibility namespace.
