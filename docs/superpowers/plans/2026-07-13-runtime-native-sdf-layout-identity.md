# Runtime Native/SDF Layout Identity Implementation Plan

**Goal:** Make SM-M5 layout identity independent from raster backend selection and extend parity coverage to VerticalRl.

## SM-M5 S0: Red contracts

- [x] Add style-key and full measure-cache Native/SDF equality contracts.
- [x] Run the focused contracts red against the current implementation.

## SM-M5 S1: Hard cut

- [x] Remove `text_render_mode` from `UiTextStyleKey` and its constructor.
- [x] Keep backend selection and render batch routing unchanged.
- [x] Add VerticalRl Native/SDF resolved-frame, source-range, and advance parity.
- [x] Add Latin/CJK/mixed/RTL coverage at the 23.5px/24px raster-policy boundary.

## SM-M5-T: Validation and records

- [x] Run exact formatting/scoped diff checks and structure budgets.
- [x] Run focused cache-identity and horizontal/VerticalRl paragraph parity tests on Windows.
- [x] Update Text05 output records, concise status, and module docs.
- [x] Re-run real framebuffer parity after the external WGPU binding collision is fixed.

## Status

| Slice | Status | Evidence |
|---|---|---|
| SM-M5 S0 | completed-red | Both exact contracts fail on the old key: style-key Native/Sdf mismatch and two layout-cache misses instead of one. |
| SM-M5 S1 | completed | Raster mode removed only from layout identity; backend selection/batch routing remain unchanged. VerticalRl projection parity and the multiscript 23.5px/24px threshold corpus are folder-backed; the exact corpus passes 1/1. |
| SM-M5-T | completed-current-horizontal-pixel-gate | Focused identity/projection gates and the multiscript threshold corpus pass. The rebuilt exact WGPU exporter produced a real 1080×1690 framebuffer under `docs/tests/runtime/text`; Native and SDF mixed-paragraph regions both contain glyph pixels and meet bbox tolerance. MSDF/MTSDF and a separate VerticalRl side-by-side pixel tolerance remain later Text05 scope. |
