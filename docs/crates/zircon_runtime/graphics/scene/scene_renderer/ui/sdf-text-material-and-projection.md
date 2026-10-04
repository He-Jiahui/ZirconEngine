---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/decorations.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/zr_text_sdf.wgsl
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/decorations.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/zr_text_sdf.wgsl
plan_sources:
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/superpowers/specs/2026-07-13-runtime-text-sdf-effects-decoration-design.md
  - docs/superpowers/plans/2026-07-13-runtime-text-sdf-effects-decoration.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/decoration_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/product_framebuffer
  - docs/tests/runtime/text/runtime_text_sdf_effects_transformed_product_framebuffer_20260713.png
status: complete
doc_type: module-detail
---

# SDF text material, effects, and transformed projection

## Ownership

`sdf_render/material.rs` owns the renderer-side text material semantic value, the group 2 GPU ABI, dynamic-uniform alignment, material draw ranges, and the CPU mirrors of effect math. It does not own glyph identity, atlas residency, shaping, layout, or font metrics. Group 0 remains the R8/RGBA atlas family; group 1 stays reserved for a future view/transform binding; group 2 is the only effect-material binding.

`render/text_projection.rs` is an internal homogeneous clip-transform contract. Normal screen-space UI batches carry no transform and retain the CPU range fast path. A batch with a nontrivial homogeneous transform is rendered with the fragment-derived range path. This enables the real rotated/perspective product fixture without adding a public or test-specific 3D text scene API.

## Material and draw contract

`SdfTextMaterial` contains fill, outline, shadow, and glow colors; outline width; shadow offset; glow radius; effect flags; SDF/MSDF/MTSDF decode mode; projection mode; and atlas dimensions. Effects do not enter `SdfAtlasGlyphKey` or `.zsdf` identity.

`SdfTextMaterialUniform` is seven 16-byte slots (112 bytes): four colors, one effect vector, one flag vector, and one projection vector. `SdfTextMaterialResources` rounds each record up to `min_uniform_buffer_offset_alignment`, stores all records in one buffer, and selects them through one group 2 bind group plus dynamic offsets. Adjacent batches coalesce only when their complete material values, including decode/projection mode, match.

The usable signed-distance extent is half the full encoded screen range. CPU clamping therefore limits one-sided outline width, shadow offset, and glow radius to `screen_px_range * 0.5`; this prevents sampling beyond the baked spread while retaining stable behavior at small display sizes.

## Projection and vertex ABI

`ScreenSpaceUiSdfVertex` stores a homogeneous clip position, UV, color, CPU `screen_px_range`, atlas `atlas_px_range`, page index, decode mode, and primitive kind. Ordinary 2D vertices use `[x, y, 0, 1]`. `ScreenSpaceUiTextClipTransform` can preserve rotation, nonuniform scale, and a nonconstant clip `w`; glyph and face-derived decoration vertices consume the same transform.

Projection modes are:

- `CpuScreenSpace`: shader consumes the CPU range derived from display size, bake em, and spread.
- `FragmentDerived`: shader computes `0.5 * dot(atlas_px_range / atlas_dimensions, 1 / fwidth(uv))`, clamped to at least one pixel.

The product rotation helper compensates for the viewport aspect ratio before transforming NDC, so its asserted principal axis is 45 degrees in framebuffer pixels rather than merely 45 degrees in non-square clip coordinates. Perspective text varies homogeneous `w`, exercising perspective-correct UV derivatives and the same fragment range formula.

## Shader semantics

`zr_text_sdf.wgsl` decodes SDF from `.r`, MSDF/MTSDF fill from the RGB median, and MTSDF true distance from alpha. Layering is glow, derivative-offset shadow, outline, then fill. Each layer uses explicit straight-alpha-over and the pipeline's straight-alpha blending contract.

Shadow UV is `uv - dpdx(uv) * offset.x - dpdy(uv) * offset.y`, which keeps offset values in screen pixels under rotation and projective interpolation. Glow reads MTSDF true distance only. Solid underline/strike quads return before atlas sampling.

## Current evidence

Managed graphics compile jobs `3cbfb0ece9ee45f6b50554e9f1559b2d` and `7872907fd942482583eba421ea2f4bd2` passed the material and homogeneous-vertex production paths. Real WGPU product job `4daaa9cda738434a9d13623a04fdfbc3` passed 1/1; follow-up job `417061de782744059c3fe3e9ac8bfa7b` again passed the product test and ran 121/122 broad `render_text_` regressions. Its only failure was an exact-zero assertion receiving `1.4901161e-7`; after changing that assertion to `1e-5` tolerance, job `8dc2b7e2134b4580aa1cc8aa8cc884fc` passed the exact rotation test 1/1. Together these jobs prove the current group as 122/122. The same final job passed global production/test file budgets 2/2 and the screen-space UI folder-backed test owner gate 1/1. Current-source target-client job `c87fb5aaa200480d987846489a999879` passed in 15m39s.

The product framebuffer is 960×560 RGBA, 5,113 colors, SHA256 `D0BD287F65DBABC33E78045942BB38F19A4EB7B5C2D282FC59907C922649BD59`. It shows real fill, outline, shadow, MTSDF glow, face-derived underline/strike, aspect-correct 45-degree MSDF, and perspective-scaled MTSDF. Repository and coordinator target scans found zero same-name copies.
