# Runtime MSDF/MTSDF Dynamic Pipeline Design

## Goal

Complete Text05 SM-M2 with a renderer-neutral dynamic MSDF/MTSDF data path that preserves sharp glyph corners, shares the unified glyph atlas, and supplies the UI renderer with explicit decode semantics. The work must converge the existing SDF implementation toward `graphics/text/sdf/`; it must not add a second layout, a renderer-local font policy, or an external C++ runtime dependency.

## Current evidence

- `GlyphAtlasFormat::Msdf` already owns RGBA8 storage and multi-channel sampling semantics, but no producer or GPU consumer uses it.
- `scene_renderer/ui/sdf_params.rs`, `sdf_font_bake.rs`, and `sdf_text.wgsl` still form an SDF-only renderer-local island.
- `fdsm 0.8.0` provides pure-Rust `generate_msdf`, `generate_mtsdf`, sign correction, and error correction; `fdsm-ttf-parser 0.2.0` imports `ttf-parser 0.25.1` outlines, matching the workspace version.
- The accepted SM-M5 proof establishes that raster mode cannot affect shaping or layout. SM-M2 therefore begins only after resolved glyph identity and placement.

## Considered approaches

### A. Shared Rust generator and explicit glyph mode — selected

Add optional `fdsm`, `fdsm-ttf-parser`, and the matching `nalgebra` transform dependency to the existing `text` feature. Establish `graphics/text/sdf/` as the canonical owner for bake parameters, glyph output, fdsm outline conversion, edge coloring, sign/error correction, and CPU decode helpers. MSDF and MTSDF both use RGBA8 atlas pages; MSDF writes RGB plus an opaque alpha, while MTSDF writes RGB plus the true signed distance in alpha. Each slot/vertex carries an explicit mode so the shader never infers semantics from pixel values.

This fits the plan, keeps the dependency pure Rust, and supports a hard cut from renderer-local SDF ownership.

### B. Extend `scene_renderer/ui/sdf_*` directly — rejected

This is locally smaller but deepens the exact mixed-responsibility owner drift called out by the structure convention and review findings. It also makes offline bake and non-UI/3D text depend on a UI renderer module.

### C. Bind Godot/msdfgen C++ or invoke an external executable — rejected

This has strong reference fidelity but introduces native build/runtime coupling, platform-specific failure modes, and an unnecessary FFI boundary. Godot remains the semantic reference for defaults and acceptance, not a runtime dependency.

## Architecture

`graphics/text/sdf/` owns:

- `params.rs`: public-to-crate `SdfMode::{Sdf, Msdf, Mtsdf}` and normalized `SdfBakeParams`; SM-M2 uses the plan-approved 48px bake em and 8px spread from one owner.
- `glyph.rs`: `SdfGlyphData` with size, bearings/advance metadata, RGBA-or-R8 pixels, channel count, mode, and spread.
- `fdsm_gen.rs`: `ttf-parser::Face + GlyphId` to prepared fdsm shape, transform, edge coloring, MSDF/MTSDF generation, sign correction, error correction, vertical flip, and deterministic RGBA packing.
- `decode.rs`: median-of-three and true-distance decode helpers used by unit tests and mirrored by WGSL.
- `geometry_preprocess.rs`: a narrow preprocessing seam. SM-M2 starts with contour validation/orientation inputs supported by fdsm; overlap removal remains explicit future work rather than hidden behavior.

The existing `graphics/text/atlas` remains the sole page/residency/upload owner. `GlyphAtlasFormat::Msdf` continues to identify RGBA distance-field pages. MTSDF is a glyph decode mode within those pages, not a second page format, because both modes have identical storage and upload requirements.

The UI renderer migrates in two steps:

1. SDF/MSDF/MTSDF glyph bake selection consumes the shared generator and shared `SdfBakeParams`.
2. Vertex data adds a flat decode mode. WGSL reads `.r` for SDF, `median(r,g,b)` for MSDF, and the median for fill plus `.a` as true distance for MTSDF effects. This milestone only closes fill rendering; outline/glow consumption of true distance remains SM-M4.

## Data flow

1. Shaping produces face id, glyph id, advances, source ranges, and frames once.
2. Raster policy selects `SdfMode` without changing layout identity.
3. The atlas key includes normalized bake parameters and mode.
4. The generator resolves face bytes through the shared `FontDatabase`, parses the requested face index, imports the glyph outline, transforms it into bake pixels, colors/prepares edges, and emits deterministic bytes.
5. The shared atlas allocates an R8 SDF page or RGBA8 MSDF page and uploads dirty rectangles through the existing upload owner.
6. Vertex planning projects the same resolved frames and carries page index, screen pixel range, and decode mode.
7. WGSL decodes the sample and applies the existing antialiasing coverage function.

## Failure handling

- Missing face bytes, invalid face index, missing glyph outline, empty outline, invalid bounds, or generation failure return typed `SdfGlyphGenerationError`; they do not panic.
- The caller records the failure on the existing per-glyph fallback path. No synthetic outline, tofu-only special case, or layout retry is introduced.
- Oversized/page-limit failures remain atlas allocation failures and are not conflated with outline generation errors.
- An fdsm generation failure may fall back to the already supported native overlay for that glyph; it must not silently switch the whole paragraph to a different layout.

## Validation

The milestone testing stage must prove, on Windows-managed targets:

- `text_msdf_median_decode_matches_msdfgen`: deterministic reference samples for median decode.
- `text_msdf_mtsdf_true_distance_channel`: MTSDF alpha is signed-distance monotonic across an edge and differs from RGB median where expected.
- `text_msdf_preserves_sharp_corners`: a sharp glyph such as `A` or `M` has lower corner error than a single-channel SDF at a scaled sample.
- Atlas format/storage tests show SDF R8 and MSDF/MTSDF RGBA separation with shared residency/upload owners.
- WGSL parsing and renderer tests prove mode propagation and median decode.
- A real WGPU framebuffer renders at least one large sharp Latin glyph and one CJK/mixed sample through the MSDF path. The PNG is written only to `docs/tests/runtime/text`; no policy/strategy text screenshot is accepted.

## Structure constraints

- New production owners are folder-backed and remain below the repository file budgets.
- Renderer roots stay wiring-only; generation, decoding, and parameters do not live in `scene_renderer/ui`.
- No compatibility re-export keeps the renderer-local parameter owner alive after migration.
- The plan output record goes only to the numbered Text05 archive.

## Deferred scope

- Offline `.zsdf` serialization/loading is SM-M3.
- Outline, shadow, glow, decoration geometry, and MTSDF true-distance effects are SM-M4.
- Arbitrary transformed/3D screen-pixel-range derivation is accepted with SM-M2 GPU coverage but can be extended with the 3D text owner when that consumer exists.
