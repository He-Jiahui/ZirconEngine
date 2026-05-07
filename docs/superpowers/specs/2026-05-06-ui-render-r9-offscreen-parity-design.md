# UI Render R9 Offscreen Parity Design

## Summary

R9 defines the next renderer acceptance slice after the R8 shared parity seam. The slice is docs and harness design only: it specifies how runtime, editor, and future offscreen render paths will prove they consumed the same neutral UI render contract before comparing backend pixels.

The design keeps source edits out of active runtime/editor/text/layout sessions. It does not change the WGPU renderer, editor native painter, text layout, Material assets, input ownership, or debug reflector implementation.

## Goals

- Establish a clear parity-first golden test strategy for shared UI render fixtures.
- Use `UiRendererParitySnapshot` as the first comparison authority for paint order, batch order, clip identity, resource identity, text render mode, opacity, and summary counts.
- Define how future offscreen/pixel golden checks layer on top of semantic parity instead of replacing it.
- Keep ownership in `zircon_runtime_interface::ui::surface::render` for neutral contracts, with runtime/editor as later consumers.
- Produce an implementation plan that can be executed after active source-lane overlap is clear.

## Non-Goals

- Do not implement runtime WGPU consumption of `UiPaintElement` or `UiBatchPlan` in R9 design work.
- Do not implement editor painter parity comparison in R9 design work.
- Do not add live Widget Reflector panels, GPU overdraw passes, or renderer instrumentation.
- Do not require pixel-perfect output before semantic parity packets match.
- Do not declare workspace-wide validation or UI cutover completion.

## Current Baseline

R1-R8 already added the shared render contract base in `zircon_runtime_interface::ui::surface::render`:

- `UiPaintElement` and `UiPaintPayload` derive richer paint rows from legacy `UiRenderCommand` values.
- `UiBrushPayload`, `UiRenderResourceKey`, and related resource DTOs preserve material, image, icon, atlas, UV, revision, and fallback identity.
- `UiBatchPlan`, `UiBatchKey`, and `UiBatchSplitReason` provide stable batch grouping and split explanations.
- `UiRenderCachePlan` records shared cache/debug status without forcing runtime GPU cache reuse.
- `UiTextPaint`, `UiShapedText`, and glyph/decorator DTOs provide shared text draw-truth slots.
- `UiRenderVisualizerSnapshot` exports replay-friendly render debug rows and overlays.
- `UiRendererParitySnapshot` exports canonical paint and batch rows for runtime/editor comparison.

R8 validation was scoped to `zircon_runtime_interface` and passed focused render contract tests. Runtime WGPU and editor painter consumers remain future work.

## Architecture

R9 keeps the authority split explicit:

- `zircon_runtime_interface` owns neutral render contracts and the canonical parity packet.
- `zircon_runtime` will later provide a runtime adapter that reports what the WGPU path actually planned or drew.
- `zircon_editor` will later provide an editor adapter that reports what the native painter actually planned or drew.
- Offscreen/golden tooling will compare packets first, then optional RGBA output.

The harness should be built around adapters rather than direct renderer coupling. A future adapter should accept a fixture or render extract and emit a normalized parity artifact. Pixel output is a secondary artifact with an explicit backend label.

## Data Flow

The intended R9 acceptance data flow is:

1. A fixture produces a `UiRenderExtract`.
2. The shared contract path derives `UiPaintElement` rows from `UiRenderList::to_paint_elements()`.
3. The shared contract path derives `UiBatchPlan::from_paint_elements(...)`.
4. The shared contract path derives `UiRenderDebugSnapshot` and `UiRendererParitySnapshot`.
5. Future runtime/editor adapters emit backend-observed parity snapshots for the same fixture.
6. The parity harness compares semantic packets and reports structured diffs.
7. Only after semantic parity passes does the harness compare optional offscreen RGBA output.

This ordering prevents a pixel mismatch from hiding a lower shared contract drift, such as a changed clip key, resource revision, text backend, or batch split.

## Fixture Tiers

R9 should define fixtures in tiers so failures localize to one shared capability:

- Tier 1: solid quad, rounded quad, border, opacity, z-order, and simple clip.
- Tier 2: image, icon, vector fallback, atlas page, UV rect, resource revision, and fallback resource.
- Tier 3: text render modes, shaped text rows, line frames, glyph atlas identity, ellipsis, selection, caret, and composition paint slots.
- Tier 4: material brush, material variant, material revision, fallback color, and material resource key.
- Tier 5: mixed command order with overlapping frames, overdraw/debug export, cache status, and batch split reasons.

The first implementation can use hand-authored `UiRenderExtract` fixtures in `zircon_runtime_interface` tests. Later implementations can add `.ui.toml` fixture compilation and backend adapter output when active runtime/editor lanes are available.

## Diff Model

Parity failures should be reported as structured row differences rather than opaque JSON mismatches:

- Paint row differences: `paint_index`, `node_id`, `paint_order`, frame, clip frame, clip key, payload kind, batch key, resource, text render mode, opacity, and debug label.
- Batch row differences: `batch_index`, first paint index, paint count, node ids, batch key, primitive, shader, and resource.
- Stats differences: paint count, batch count, clipped paint count, resource-bound paint count, and text paint count.
- Pixel differences: backend label, fixture name, viewport size, changed pixel count, first mismatch coordinate, and a reason pointing back to semantic parity status.

The harness should fail semantic drift before pixel drift. If semantic parity fails, pixel comparison is either skipped or marked diagnostic-only.

## Error Handling

R9 harness design should treat failures as ownership signals:

- Shared contract failure belongs to `zircon_runtime_interface` fixtures or DTO derivation.
- Runtime-observed parity failure belongs to runtime renderer consumption or runtime resource/text adapters.
- Editor-observed parity failure belongs to editor painter consumption or host asset/text adapters.
- Pixel-only failure after semantic parity passes belongs to backend rasterization, color conversion, font rasterization, sampling, or antialiasing tolerance.

Each failure should include the fixture tier and the owning adapter name. This avoids broad edits in another active session's module just because a high-level screenshot changed.

## Testing Strategy

The first R9 implementation plan should prefer focused checks:

- Add interface-level parity fixture tests that compare expected `UiRendererParitySnapshot` rows for hand-authored extracts.
- Add a reusable parity diff helper in tests or a narrow test-support module only if multiple fixtures need readable diffs.
- Add docs for fixture tiers and exact acceptance commands.
- Defer runtime/editor/offscreen pixel tests until their source lanes are clear and a separate implementation plan is approved.

R9 should not run broad Cargo validation while active sessions own Cargo lanes. Milestone testing should use scoped `zircon_runtime_interface` checks first, with runtime/editor checks added only when their adapters are implemented.

## Documentation Impact

R9 should update:

- `docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md` to record the R9 parity/golden strategy and make clear it is not renderer cleanup completion.
- `docs/zircon_runtime_interface/ui/surface/render.md` to document how `UiRendererParitySnapshot` is intended to feed future golden tests.
- A plan under `docs/superpowers/plans/` with milestones, testing stage, and exact source boundaries.

## Coordination Boundaries

At the time this design was approved, active sessions owned runtime text layout, Canvas/Free slot contracts, Material M3 validation, input owner-safety validation, Asset Browser Material/SVG/FPS, debug-reflector validation, and Cargo/disk hygiene lanes.

R9 implementation must avoid these files unless a fresh coordination scan and an approved plan say otherwise:

- `zircon_runtime/src/graphics/scene/scene_renderer/ui/**`
- `zircon_editor/src/ui/slint_host/host_contract/painter/**`
- `zircon_runtime/src/ui/text/**`
- `zircon_runtime/src/ui/layout/pass/**`
- `zircon_runtime_interface/src/ui/layout/**`
- `zircon_runtime/src/ui/surface/input/**`
- Material `.ui.toml` assets and active editor debug-reflector files

## Acceptance

This design is accepted when:

- The R9 scope is recorded as parity/golden harness design, not renderer implementation.
- The implementation plan names interface-only first steps and later runtime/editor adapter steps separately.
- No source files in active sibling-owned runtime/editor/text/layout lanes are edited during the design step.
- Future acceptance starts with semantic `UiRendererParitySnapshot` equality and only then considers optional pixel golden output.
