---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/geometry/source_metrics.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/geometry/source_metrics.rs
  - zircon_runtime/src/ui/text/hit_test.rs
plan_sources:
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-15 complete runtime text and layout architecture with real framebuffer proof
  - dev/slint/internal/core/textlayout.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
tests:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map_tests.rs
  - zircon_runtime/src/ui/text/geometry/tests/mixed_bidi.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
doc_type: module-detail
---

# Runtime Text Source/Visual Geometry

## Purpose

Text selection, caret placement, IME rectangles, pointer hit testing and renderer decorations all need the same answer to one question: where does a logical source-byte boundary land in a post-layout visual line? `UiTextLineSourceMap` is the neutral authority for that answer. It is constructed only from `UiResolvedTextLine.runs`, after wrapping, UAX#9 visual ordering, ellipsis and rich-run projection have completed.

The owner lives in `zircon_runtime_interface` because the input is a neutral resolved-layout DTO and the output is renderer- and input-backend-independent geometry. `zircon_runtime` consumes it; it does not reconstruct BiDi policy, mirror tables or source clusters.

## Source Map Model

Each visual grapheme cluster records three facts:

- the authored `source_range`;
- the emitted `visual_range` in the resolved line text;
- the resolved LTR or RTL direction from the UAX#9 line owner.

Logical start/end edges are direction-sensitive. An RTL cluster's logical start is its visual trailing edge, while its logical end is its visual leading edge. This permits the same logical byte offset at an LTR/RTL boundary to produce two valid visual carets: `Upstream` attaches to the preceding logical cluster and `Downstream` attaches to the following logical cluster.

For a source range, the map selects intersecting grapheme clusters in visual order and merges only adjacent visual ranges. Mixed-BiDi selections can therefore produce multiple rectangles; no renderer-local range sorting or descending-range merge is allowed.

Non-isomorphic runs such as generated ellipsis or rich inline placeholders keep a conservative whole-run source range for every emitted grapheme. They never invent byte precision that the resolved DTO does not contain.

## Geometry Consumers

`zircon_runtime/src/ui/text/geometry.rs` uses the map for caret and source-range frames. Horizontal text projects the map's resolved advance onto x; `VerticalRl` projects the identical advance onto y and emits a horizontal caret bar or full-column selection frame. The `source_metrics` child is deliberately narrower: it may replace cached advances only for a source-isomorphic, non-ellipsized, non-tab, horizontal LTR line. BiDi and vertical paths remain on the resolved layout data.

`zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs` uses the same map for selection, composition underline and caret paint decorations. `UiRenderCommand::text_paint()` therefore emits exactly the geometry used by the runtime's input/IME path instead of maintaining a second source-offset algorithm.

`zircon_runtime/src/ui/text/hit_test.rs` performs the inverse operation. It chooses a visual grapheme boundary from the resolved advances, then asks the map for the source offset and affinity at that edge. The deleted `hit_test/visual_source.rs` and monolithic interface `text_geometry.rs` are not retained as aliases or fallback paths.

## Reference Alignment

Slint keeps `text_byte_offset` on positioned glyphs and derives cursor/selection geometry from the same layout traversal. Fyrox keeps formatted-line positions and source indices inside its formatted-text owner. Zircon follows that ownership principle but extends it with explicit visual ranges, per-cluster BiDi direction, caret affinity and VerticalRl axis projection.

## Edge Cases

- Soft-wrap boundaries choose the first matching line for `Upstream` and the last matching line for `Downstream`.
- Partial byte offsets inside a grapheme snap outward according to affinity; range geometry selects the whole intersecting cluster.
- Invalid or negative advances are sanitized to zero. When per-grapheme advances are absent, the map uses the resolved line's measured main-axis extent proportionally.
- Direct source remeasurement is rejected for tabs, justify, ellipsis, non-LTR direction and VerticalRl so it cannot overwrite layout-stage advances.

## Test Coverage

The source-map leaf tests lock distinct visual carets at an LTR/RTL boundary, discontiguous mixed-BiDi selection spans and RTL visual-edge round trips. Runtime geometry tests lock the resulting caret and range rectangles. Hit-test tests lock both leading and trailing RTL edge affinity. The Windows product exporter adds an editable mixed Hebrew/LTR row with real selection, caret and composition decorations and verifies those frames reach WGPU framebuffer pixels.

Cargo and product-export results are recorded only after the Text03 milestone testing stage runs; this document does not treat its design assertions as acceptance evidence.
