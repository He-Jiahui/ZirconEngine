# Runtime Native/SDF Layout Identity Design

## Goal

Close the lowest SM-M5 parity defect: raster backend selection must not split layout or measurement cache identity.

## Architecture

- `UiResolvedStyle.text_render_mode` remains the requested raster/backend policy and continues to route Auto, Native, and Sdf batches.
- `UiTextStyleKey` describes only inputs that can change shaping, wrapping, source ranges, advances, or resolved frames.
- Native and SDF use the same `UiSharedTextShaper`; therefore switching only `text_render_mode` must keep `UiTextStyleKey`, `UiTextMeasureKey`, layout, bbox, advances, and line breaks identical.
- No compatibility key, renderer-side reconciliation, or duplicated layout is introduced.
- Writing mode, direction, language, font identity/weight/size, line height, tab size, wrap, align, overflow, and rich format remain in the key because they can change resolved layout.

## Contracts

1. Native and SDF styles differing only by render mode produce equal layout style keys.
2. Native and SDF full measure keys for the same text/frame/clip are equal and hit one cached layout entry.
3. Existing horizontal paragraph projection parity remains green.
4. Add the plan-required VerticalRl Native/SDF bbox/advance/source-range parity gate.
5. Cover Latin, CJK, mixed-script, and RTL at 23.5px/24px on the two sides of the default bitmap/SDF policy boundary.
6. Pixel tolerance evidence uses the real product framebuffer after the renderer owner's bind-layout repair; it never substitutes a policy-text screenshot.
