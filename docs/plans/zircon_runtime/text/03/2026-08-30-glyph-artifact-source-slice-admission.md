# Runtime Text glyph artifact source-slice admission

## Scope

Close the remaining fail-open path between resolved layout ranges and visual glyph projection.
`visual_projection` consumes UTF-8 slices, but the artifact builders previously checked only numeric
containment. A line or run range that split a scalar could therefore fall back to the whole line/run
range and publish a misleading source map.

## Structural correction

`artifact_line_source_ranges_are_sliceable` is now an artifact-owner admission check. It validates
the line range and every run range against the exact source snapshot and origin before either the
plain or rich artifact builder performs shaping or visual projection. `str::get` is the boundary
authority, so out-of-bounds, reversed, and non-UTF-8 ranges fail closed as `LayoutFailed`.

Zero-width virtual runs remain supported, but their anchors must also be valid UTF-8 boundaries. This
preserves generated ellipsis/replacement markers without permitting an anchor inside a multi-byte
scalar. The rich builder applies the same guard instead of relying on a renderer-local repair.

## Evidence

- Existing line-range UTF-8 rejection remains covered.
- Added regression for a non-empty run range splitting `é`.
- Added regression that rejects an empty anchor inside `é` and accepts the end-of-source anchor.
- `rustfmt --edition 2021 --check` passes for the touched Rust sources.
- Managed Cargo, real WGPU/PNG, profile/RSS/power, and Unreal comparison remain pending; no
  validation image is produced by this source-only change.

Status: `glyph_artifact_source_slice_admission_static_implemented /
virtual_anchor_boundary_preserved / managed_product_validation_pending`.
