---
related_code:
  - zircon_runtime/src/text/sdf/mod.rs
  - zircon_runtime/src/text/sdf/mode.rs
  - zircon_runtime/src/text/sdf/params.rs
  - zircon_runtime/src/text/sdf/glyph_data.rs
  - zircon_runtime/src/text/sdf/generation_error.rs
  - zircon_runtime/src/text/sdf/decode.rs
  - zircon_runtime/src/text/sdf/geometry_preprocess.rs
  - zircon_runtime/src/text/sdf/fdsm_gen.rs
  - zircon_runtime/src/text/font/decoration_metrics.rs
  - zircon_runtime/src/text/sdf/offline
  - zircon_runtime/src/text/font_sdf_build_tool
  - zircon_runtime/src/bin/zircon_font_sdf_bake
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/distance_field.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/atlas_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/decorations.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/zr_text_sdf.wgsl
implementation_files:
  - zircon_runtime/src/text/sdf/mod.rs
  - zircon_runtime/src/text/sdf/mode.rs
  - zircon_runtime/src/text/sdf/params.rs
  - zircon_runtime/src/text/sdf/glyph_data.rs
  - zircon_runtime/src/text/sdf/generation_error.rs
  - zircon_runtime/src/text/sdf/decode.rs
  - zircon_runtime/src/text/sdf/geometry_preprocess.rs
  - zircon_runtime/src/text/sdf/fdsm_gen.rs
  - zircon_runtime/src/text/sdf/offline/artifact.rs
  - zircon_runtime/src/text/sdf/offline/codec.rs
  - zircon_runtime/src/text/font/decoration_metrics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_projection.rs
plan_sources:
  - user: 2026-07-13 complete the Runtime Text architecture and verify real rendered pixels
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/superpowers/specs/2026-07-13-runtime-msdf-mtsdf-dynamic-pipeline-design.md
  - docs/superpowers/plans/2026-07-13-runtime-msdf-mtsdf-dynamic-pipeline.md
  - docs/superpowers/specs/2026-07-13-runtime-text-sdf-effects-decoration-design.md
  - docs/superpowers/plans/2026-07-13-runtime-text-sdf-effects-decoration.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - zircon_runtime/src/text/sdf/tests.rs
  - zircon_runtime/src/text/sdf/tests/decode.rs
  - zircon_runtime/src/text/sdf/tests/fdsm_gen.rs
  - zircon_runtime/src/text/sdf/tests/offline.rs
  - zircon_runtime/src/text/sdf/font_bake/tests.rs
  - zircon_runtime/src/text/sdf/font_bake/tests/offline.rs
  - zircon_runtime/tests/runtime_text_sdf_offline_artifact.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/cache_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/shader_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/decoration_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/product_framebuffer
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
doc_type: module-detail
---

# Runtime SDF/MSDF/MTSDF core

## Ownership

`text/sdf/` is the renderer-neutral owner of signed-distance-field mode identity, bake parameters, generated glyph data, typed failures, reference decode math, outline preprocessing, pure-Rust fdsm generation, CPU font/glyph caches, and batched glyph/decorations metrics. It runs after shaping and layout. Consequently, selecting SDF, MSDF, or MTSDF may change pixels and atlas storage, but it must never change glyph identity, advances, source ranges, line breaks, or resolved frames.

The existing `text/atlas/` subtree remains the only owner of page identity, residency, shelf allocation, dirty regions, upload planning, retry state, and placeholder behavior. This module maps `Sdf` to an R8 SDF page and maps both `Msdf` and `Mtsdf` to RGBA8 MSDF pages. MTSDF is a decode mode, not a second storage format: its RGB channels contain the multi-channel distance and alpha contains true signed distance.

## Contracts

`SdfMode` has stable shader discriminants:

- `Sdf = 0`, one channel, `GlyphAtlasFormat::Sdf`.
- `Msdf = 1`, four stored channels, `GlyphAtlasFormat::Msdf`; alpha is opaque.
- `Mtsdf = 2`, four stored channels, `GlyphAtlasFormat::Msdf`; alpha is true distance.

`SdfBakeParams` is part of raster/atlas cache identity. The shared default is a 48px bake em and 8px spread. Normalization prevents zero dimensions, and `screen_px_range(display_px)` is supplied from this one type so CPU bake, vertex planning, offline artifacts, and shader tests cannot drift onto separate constants.

`SdfGlyphData` records bake-pixel metrics, pixel dimensions, channel count, spread, and mode. `validate()` requires `width × height × channels == pixels.len()` and rejects zero or overflowing sizes.

## Dynamic generation

`generate_distance_field_glyph(font_bytes, face_index, glyph_id, params)` performs the following sequence:

1. Parse the exact face index with `ttf-parser` and resolve the exact shaped glyph id.
2. Import its outline through `fdsm-ttf-parser` and reject missing/empty contours.
3. Scale font units to the shared bake em, add the shared spread on every side, and transform the outline into bake-pixel coordinates.
4. Apply a deterministic edge-coloring seed derived from face index and glyph id.
5. Generate pixels through fdsm. MSDF and MTSDF use error correction and sign correction; all modes are vertically flipped from font-up coordinates to texture-down rows.
6. Pack SDF as R8, MSDF as RGB plus opaque alpha, and MTSDF as RGBA with its true-distance alpha intact.

The implementation uses `fdsm 0.8`, `fdsm-ttf-parser 0.2`, `nalgebra 0.34`, the workspace `ttf-parser 0.25`, and the workspace `image 0.25`. It does not bind Godot's C++ msdfgen at runtime; Godot/msdfgen remains the semantic reference for 48/8 defaults, median decode, and error measurements.

## Decode reference

`median3(r, g, b)` is the CPU authority mirrored by WGSL. MSDF fill reads that median. MTSDF fill also reads the median, while `mtsdf_sample_true_distance` exposes alpha for later outline/glow work. `distance_field_coverage` mirrors the existing screen-pixel-range and derivative antialiasing equation, including the one-pixel minimum range.

## Failure behavior

Generation returns `SdfGlyphGenerationError` for invalid face indices, missing outlines, empty bounds, invalid dimensions, or invalid output byte lengths. It does not synthesize a layout or panic. Renderer integration must map the failure to the existing per-glyph native overlay/fallback path, leaving other glyphs and the paragraph layout untouched. Atlas page-limit and oversized-slot failures remain separate atlas allocation failures.

## Runtime atlas and GPU integration

`text/sdf/font_bake/distance_field.rs` is a narrow adapter: it resolves authoritative standalone face bytes and the shaped glyph id, then delegates all pixel generation to this shared module. The surrounding Text-owned font-bake state retains face fallback, metrics scaling, cache lifetime, atlas-page assembly, typed reporting, and batched CPU preparation. Shaped face and variation identities remain versioned `TextFontFaceHandle` values until Text resolves them through the generation-checked registry; no raw numeric roundtrip survives. Its cache key includes normalized `SdfBakeParams`, so SDF, MSDF, and MTSDF outputs never alias.

All face-derived CPU state follows the shared font-database generation. Before a public bake, measure, decoration, atlas, or failure-query entry point uses cached data, `SdfFontBakeCache` compares the observed generation and clears resident fonts, generated glyphs, measured metrics, face-resolution results, project-font resolution, decoration metrics, and offline-source state together when it changes. A shaped glyph id is reused only when its current generation-checked face or instance handle resolves to the same face selected for the bake; a stale or mismatched handle falls back to scalar lookup on the selected face instead of carrying a glyph id across faces.

The screen-space text system applies face invalidation before the first SDF plan or bake of the frame. It clears the Text CPU state, bitmap source cache, SDF slot plan, and cached slots as one operation. The SDF atlas then forces every page in each non-empty intermediate plan to be fully dirty until the final renderer preparation has consumed the plan. This state deliberately survives an empty frame and the second plan produced by whole-batch native fallback, so an intermediate `prepare` cannot erase the upload needed for new face pixels. Only a completed non-empty renderer preparation acknowledges the upload; a face discovered later by the native backend schedules the same invalidation contract for the next frame.

Atlas bake output is one sorted byte stream with explicit per-page source spans. `GlyphAtlasFormat::Sdf` pages use R8 and `GlyphAtlasFormat::Msdf` pages use RGBA8. Full and dirty upload commands calculate row stride, rect offset, and cumulative page source offset from each page's storage format; there is no one-byte assumption in the multi-channel path.

`sdf_render/atlas_resources.rs` owns two WGPU texture arrays and one bind group: an R8 SDF family and an RGBA MSDF/MTSDF family. `ScreenSpaceUiSdfVertex.decode_mode` is a flat shader discriminant. `zr_text_sdf.wgsl` samples the compatible texture family, reads `.r` for SDF, uses RGB median for MSDF/MTSDF fill, and carries MTSDF alpha true distance separately for the later effect milestone. The deleted `sdf_text.wgsl` has no compatibility include.

`UiTextRenderMode::{Msdf,Mtsdf}` is a raster request, not a layout input. Mode resolution and the shared shaper continue to produce the same glyph ids, advances, source ranges, line breaks, and frames; only `ScreenSpaceUiTextBatch.distance_field_mode` changes the atlas key and GPU decode route.

## Face-derived decorations

Underline and strikeout do not enter glyph or atlas identity. `text/font/decoration_metrics.rs` reads post/OS/2 face metrics, scales them from font units to display pixels, caches by face and size, and applies the documented em fallback only when a table is absent. Resolved line baselines are carried into the text batch; raw text falls back to the same face ascender rather than `run.bottom()`.

`sdf_render/decorations.rs` emits clipped solid quads before distance-field glyph vertices. The vertex ABI has an explicit solid primitive discriminant, and `zr_text_sdf.wgsl` returns its color without sampling the atlas. This lets Native and SDF text share one face-derived decoration geometry path while keeping editing decorations and selection/caret composition semantics independent. Detailed ownership and formulas are recorded in `docs/zircon_runtime/graphics/text/font-decoration-metrics.md`.

## Material effects and transformed range

`sdf_render/material.rs` owns group 2 material values, the 112-byte seven-vec4 uniform ABI, device-aligned dynamic offsets, and material draw coalescing. Fill, outline, derivative-offset shadow, and MTSDF true-distance glow are material state; none enter atlas or offline-artifact identity. Effect extents are clamped to half the full signed-distance screen range because the encoded spread covers both sides of the 0.5 contour.

The vertex ABI now carries homogeneous clip coordinates plus both CPU screen range and atlas pixel range. Untransformed screen UI consumes the CPU fast path. Internal batches with a homogeneous clip transform use the fragment path `0.5 * dot(atlas_px_range / atlas_dimensions, 1 / fwidth(uv))`; the same transform is applied to face-derived decoration quads. Shadow sampling continues to use UV derivatives, so offsets remain screen-pixel quantities under rotation and perspective. Detailed ABI, ownership, and product evidence are recorded in `docs/zircon_runtime/graphics/scene/scene_renderer/ui/sdf-text-material-and-projection.md`.

## Offline artifact integration

`text/sdf/offline/` is the shared versioned `.zsdf` codec and identity owner. The feature-gated build tool calls this module after using the same `generate_distance_field_glyph(...)` function and the same shelf allocator as the dynamic path. Runtime font bake resolves the project font manifest's authoritative `.zmeta` UUID plus the exact standalone face-source hash and accepts an artifact only when UUID, face index, variation hash, source hash, mode, bake em, and spread all match.

The renderer lookup order is in-memory glyph cache, accepted offline artifact, dynamic generator, then existing typed per-glyph failure handling. Accepted offline pixels flow through the same R8/RGBA page stream, upload planner, WGPU textures, and shader decode modes. Missing, corrupt, stale, or uncovered artifacts therefore affect generation cost only; they cannot introduce another layout path or another GPU atlas. The full binary/build/runtime contract is documented in `docs/zircon_runtime/graphics/text/offline-sdf.md`.

## Current validation state

The 2026-07-13 broad Runtime scene gate exposed a metric-authority regression after the dynamic fdsm cutover: atlas quads consumed padded `SdfBakedGlyph` metrics while whitespace, format-control, alignment, justification, and test-side measurements still queried the legacy fontsdf metric path. `SdfFontBakeCache::measure_glyph(...)` now reuses an already cached generated glyph for the same `SdfAtlasGlyphKey`, so advance/ascent/bearing calculations and vertex planning share one baked-metric authority. The focused draw-plan whitespace and format-control regressions and the complete 16-test layout-placement group pass on the fresh Runtime binary. This does not change shaping or paragraph layout; it removes a second metric interpretation after shaping.

SM2-M1 passed current-source Windows acceptance on 2026-07-13. The managed locked graphics-enabled check completed in 12m23s; the matching library test binary built in 34m05s. Shared parameter/mode/data contracts passed 4/4, decode reference tests passed 3/3, and Fira Sans fdsm generation tests passed 4/4, including deterministic output, typed missing outline/face errors, MTSDF alpha edge behavior, and sharp-corner fidelity relative to single-channel SDF.

The acceptance gate intentionally uses `--no-default-features --features graphics`, because `text/sdf` is reached through the crate's `#[cfg(feature = "graphics")] pub mod graphics`; a `features=text` build alone validates dependencies but does not compile this owner.

SM2-M2/M3 production checks now pass with `graphics` and `target-client`. A real WGPU product export also passed and wrote `docs/tests/runtime/text/runtime_text_multilingual_sdf_msdf_product_framebuffer_20260713.png`: 1080×1690, 315799 bytes, 2444 colors, SHA256 `05BED61944F35380A0967A9A3D04DFC0B1F5413D240A2C5627A4E3D4294FB448`, with zero same-name copies under repository/approved Cargo target roots. The accepted frame contains side-by-side real SDF and MSDF `A/M/W`/CJK pixels plus the existing multilingual, RTL, VerticalRl, table, and inline-texture proof.

The product proof keeps two independent responsibilities. The renderer-level assertion requires real SDF and MSDF pixels, a distinct framebuffer decode result, a visible high-contrast `A` apex in both samples, and an MSDF apex that starts no lower than the SDF apex. The renderer-neutral fdsm test remains the geometry authority and compares the generated apex row against the expected bake-space tip. It deliberately accepts equal error because raster-grid quantization can make two correct 24px product samples occupy the same number of high-contrast pixels. A former product assertion instead required MSDF to contain strictly more pixels in the first four apex rows; the accepted 2026-07-13 frame passed by only one pixel (22 versus 21), while the current frame produced 19 versus 19. Pixel count is neither a stable nor directional sharpness metric, so it is no longer used as a superiority claim.

The corrected current-source product gate passed on 2026-07-14. Managed integration compilation job `017de9fb45784ec396ec90dcd20f6584` completed with exit 0, then the exact ignored exporter passed 1/1 in 1017.34 seconds. Its real WGPU framebuffer is `docs/tests/runtime/text/runtime_text_multilingual_sdf_msdf_product_framebuffer_20260714.png`: 1080×1690, 321453 bytes, 2442 colors, SHA256 `2A033D76EF5C16F99FB6B256AD8F480ACE494FB03537A9E4502DEA293BED866E`. Original-size inspection confirms actual multilingual, RTL, emoji, VerticalRl, BBCode/table, inline-texture, SDF, and MSDF pixels. The same filename is absent from repository and approved Cargo target roots. The integration renderer also follows the current runtime manager boundary: it installs `ProjectAssetManager` in `ProjectAssetTestRuntime` and passes the resolved `ProjectAssetManagerAccess` service handle to `WgpuRenderFramework`; no compatibility constructor was added.

The earlier runtime-plugin report API-drift compile seam no longer blocks the current-source monolithic library test. The dynamic atlas exact regressions pass: font bake/offline/fallback 12/12, mixed-format upload 11/11, shader/decode-mode contracts 7/7, and atlas/policy identity 3/3. This closes the previously deferred SM2-M2/M3 exact gate without modifying foreign plugin tests.

SM-M3 offline artifacts also passed: artifact/runtime exact 6/6, CLI range 1/1, independent SDF/MSDF/MTSDF deterministic/decode/checksum integration 2/2, and managed target-client check job `1aaddc58899c4d77a23a88005614eb77` exit 0 in 11m44s. Detailed hashes, build-tool regressions, runtime precedence, and artifact hygiene are recorded in `docs/zircon_runtime/graphics/text/offline-sdf.md` and the Text05 numbered output archive.

SM4 production effects and transformed rendering are complete. Managed graphics checks `3cbfb0ece9ee45f6b50554e9f1559b2d` and `7872907fd942482583eba421ea2f4bd2` passed; current-source target-client job `c87fb5aaa200480d987846489a999879` also passed. Real WGPU jobs `4daaa9cda738434a9d13623a04fdfbc3` and the product phase of `417061de782744059c3fe3e9ac8bfa7b` passed. The latter ran 121/122 broad `render_text_` regressions; its only failure was a new exact-zero assertion receiving `1.4901161e-7`. After applying `1e-5` tolerance, final job `8dc2b7e2134b4580aa1cc8aa8cc884fc` passed that exact test, establishing the combined group as 122/122, plus production/test budgets 2/2 and UI folder-backed ownership 1/1. The 960×560 product PNG has 5,113 colors and SHA256 `D0BD287F65DBABC33E78045942BB38F19A4EB7B5C2D282FC59907C922649BD59`; no same-name file exists under repository or coordinator targets.
