# Runtime MSDF/MTSDF Dynamic Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete Text05 SM-M2 with pure-Rust dynamic MSDF/MTSDF generation, unified RGBA atlas consumption, explicit GPU decode modes, and a real framebuffer acceptance proof.

**Architecture:** `text/sdf/` becomes the renderer-neutral owner for modes, bake parameters, glyph data, fdsm generation, typed errors, preprocessing, and decode reference math. The existing unified glyph atlas remains the only page/residency/upload owner; the UI renderer consumes shared glyph products and carries an explicit flat decode mode to WGSL without changing shaping or layout.

**Tech Stack:** Rust, `fdsm 0.8`, `fdsm-ttf-parser 0.2`, `nalgebra 0.34`, `ttf-parser 0.25`, `image 0.25`, WGPU 29, WGSL, managed Windows Cargo validation.

---

## Dependency order

1. Shared definitions and deterministic CPU reference math.
2. Font outline import and dynamic MSDF/MTSDF generation.
3. Unified atlas key/storage/bake integration.
4. Vertex and WGSL decode integration.
5. Real WGPU framebuffer acceptance, docs, and status records.

No higher layer may introduce a private generator or decode rule if a lower shared layer is incomplete.

## Milestone SM2-M1: shared contracts and fdsm generator

**Goal:** Produce deterministic, typed MSDF/MTSDF glyph bytes from a shared font face without depending on the UI renderer.

**In-scope behaviors:** normalized 48/8 parameters; SDF/MSDF/MTSDF mode identity; RGBA packing; MSDF median decode; MTSDF alpha true-distance decode; missing-outline/invalid-bounds errors; deterministic edge-coloring seed; vertical output orientation.

**Dependencies:** Text01 face bytes and Text02 shaped glyph ids are already established; `ttf-parser` face indices remain authoritative.

### Implementation slice SM2-M1-S1: dependency and declaration surface

**Files:**

- Modify `zircon_runtime/Cargo.toml`
- Modify `zircon_runtime/src/text/mod.rs`
- Create `zircon_runtime/src/text/sdf/mod.rs`
- Create `zircon_runtime/src/text/sdf/mode.rs`
- Create `zircon_runtime/src/text/sdf/params.rs`
- Create `zircon_runtime/src/text/sdf/glyph_data.rs`
- Create `zircon_runtime/src/text/sdf/generation_error.rs`
- Create `zircon_runtime/src/text/sdf/tests.rs`

- [x] Add optional dependencies `fdsm = "0.8.0"`, `fdsm-ttf-parser = "0.2.0"`, and the required `nalgebra = "0.34.1"` transform type; enable them from the existing `text` feature.
- [x] Add wiring-only `pub(crate) mod sdf;` to `text/mod.rs`.
- [x] Define `SdfMode::{Sdf, Msdf, Mtsdf}` with `channel_count()`, `atlas_format()`, and a stable `u32` shader discriminant.
- [x] Define normalized `SdfBakeParams { mode, bake_em_px, spread_px_milli }`; default is `Sdf/48/8000`, and `screen_px_range(display_px)` remains a single shared formula.
- [x] Define `SdfGlyphData { size, bitmap_left, bitmap_bottom, advance, ascent, pixels, channels, spread_px, mode }` with byte-length validation.
- [x] Define typed `SdfGlyphGenerationError` variants for invalid face, missing glyph outline, empty bounds, invalid dimensions, and invalid output length.
- [x] Add declaration-level tests for normalization, atlas selection, shader discriminants, and output byte-length validation. Tests are authored now and executed only in SM2-M1-T.
- [x] Append one SM2-M1-S1 output row to the Text05 numbered archive immediately after the slice is complete.

The declaration contract must expose this shape:

```rust
pub(crate) enum SdfMode { Sdf, Msdf, Mtsdf }

pub(crate) struct SdfBakeParams {
    pub(crate) mode: SdfMode,
    pub(crate) bake_em_px: u32,
    pub(crate) spread_px_milli: u32,
}

pub(crate) struct SdfGlyphData {
    pub(crate) size: UVec2,
    pub(crate) pixels: Vec<u8>,
    pub(crate) channels: u8,
    pub(crate) spread_px: f32,
    pub(crate) mode: SdfMode,
    // metrics remain in bake-pixel units
}
```

### Implementation slice SM2-M1-S2: reference decode math

**Files:**

- Create `zircon_runtime/src/text/sdf/decode.rs`
- Create `zircon_runtime/src/text/sdf/tests/decode.rs`
- Modify `zircon_runtime/src/text/sdf/mod.rs`

- [x] Implement stable `median3`, normalized MSDF signed-distance decode, MTSDF true-distance decode, and coverage helpers using the same equations intended for WGSL.
- [x] Add reference-table tests named `text_msdf_median_decode_matches_msdfgen` and `text_msdf_mtsdf_true_distance_decode_uses_alpha`.
- [x] Cover equal channels, every channel ordering, values around 0.5, and minimum screen pixel range.
- [x] Append one SM2-M1-S2 output row to the Text05 numbered archive.

### Implementation slice SM2-M1-S3: fdsm outline generation

**Files:**

- Create `zircon_runtime/src/text/sdf/fdsm_gen.rs`
- Create `zircon_runtime/src/text/sdf/geometry_preprocess.rs`
- Create `zircon_runtime/src/text/sdf/tests/fdsm_gen.rs`
- Modify `zircon_runtime/src/text/sdf/mod.rs`

- [x] Parse `ttf_parser::Face`, resolve the requested `GlyphId`, and import the outline through `fdsm_ttf_parser::load_shape_from_face`.
- [x] Calculate the transform from face units to a 48px em bake with 8px margin, flip the font-up y axis into texture-down orientation, and reject zero/overflow dimensions before allocation.
- [x] Validate contours in `geometry_preprocess.rs`; reject empty segments and preserve a narrow seam for later overlap removal without adding a no-op public strategy object.
- [x] Use a stable seed derived from glyph id and face index for `Shape::edge_coloring_simple`, then prepare the colored shape.
- [x] For MSDF, generate RGB, correct sign/error, and pack RGBA with alpha 255. For MTSDF, generate RGBA, correct sign/error, and preserve the true-distance alpha channel.
- [x] Return `SdfGlyphData` with metrics scaled to bake pixels and exact channel/storage metadata.
- [x] Add tests using `assets/fonts/FiraSans-Regular.ttf`: deterministic repeated generation; non-empty `A`/`M`; typed missing-outline failure; MTSDF alpha monotonicity near a detected edge; and `text_msdf_preserves_sharp_corners` comparing a scaled corner sample against the existing single-channel SDF reference.
- [x] Append one SM2-M1-S3 output row to the Text05 numbered archive.

### Testing stage SM2-M1-T

- [x] Run scoped `rustfmt --check` and `git diff --check` for the new subtree and manifest.
- [x] Acquire a coordinator-managed Windows test target below the approved D/E/F roots.
- [x] Run `cargo check -p zircon_runtime --lib --no-default-features --features graphics --locked --jobs 1 --target-dir <managed-target>`; `graphics` enables `text` and is required for `text/sdf` to enter the crate.
- [x] Run exact filters `text_msdf_median_decode_matches_msdfgen`, `text_msdf_mtsdf_true_distance`, `text_msdf_preserves_sharp_corners`, and the shared SDF parameter tests.
- [ ] On failure, fix the lowest shared layer in order: parameter normalization → outline import/transform → fdsm generation/correction → byte packing → test metric. Do not patch renderer code during this stage.
- [x] Update `docs/zircon_runtime/text/sdf.md` with machine-readable headers, contracts, errors, dependency rationale, and current test evidence.
- [x] Append one SM2-M1-T output row. Promote only when every declared behavior has direct evidence.

**Exit evidence:** current-source compile plus all exact CPU contracts green; no renderer files changed; all new owners below structure budgets.

## Milestone SM2-M2: atlas and bake integration

**Goal:** Make the shared glyph atlas and font bake path produce and upload SDF/MSDF/MTSDF data without a renderer-private parameter owner.

**In-scope behaviors:** mode in atlas cache identity; R8 SDF versus RGBA MSDF/MTSDF storage; page byte stride; dirty upload; per-glyph generation failure reporting; existing native fallback preservation.

**Dependencies:** SM2-M1 fully accepted.

### Implementation slice SM2-M2-S1: hard-cut bake parameters and atlas key

**Files:**

- Modify `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs`
- Modify `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs`
- Modify imports under `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/`
- Delete `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_params.rs`
- Modify `zircon_runtime/src/graphics/scene/scene_renderer/ui/mod.rs`

- [x] Replace all renderer-local `SdfBakeParams/SdfBakeMode` imports with `graphics::text::sdf` types.
- [x] Include `SdfMode` in `SdfAtlasGlyphKey` through `SdfBakeParams`; preserve layout identity independence.
- [x] Route `SdfMode::Sdf` to `GlyphAtlasFormat::Sdf` and both multi-channel modes to `GlyphAtlasFormat::Msdf`.
- [x] Delete the renderer-local parameter module and prove no compatibility re-export remains.
- [x] Add atlas tests showing identical glyph ids in SDF/MSDF/MTSDF modes do not alias and use the correct page storage.
- [x] Append one SM2-M2-S1 output row.

### Implementation slice SM2-M2-S2: shared generator consumption

**Files:**

- Refactor `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs` into folder-backed owners if it would exceed the repository budget.
- Create or modify `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/distance_field.rs`
- Modify `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs`
- Modify related tests under `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/tests.rs`

- [x] Keep face resolution and metrics scaling in the current font bake owner, but delegate pixel generation by mode to `graphics::text::sdf`.
- [x] Allocate atlas layer byte lengths from each page's `GlyphAtlasStorageFormat`, not an SDF-only one-byte assumption.
- [x] Pack SDF page layers as R8 and MSDF/MTSDF layers as RGBA8; retain page-indexed dirty writes.
- [x] Add typed generation failures to the existing per-glyph bake report and feed them into the normal native overlay fallback span.
- [x] Add focused tests for mixed SDF/MSDF page byte lengths, dirty upload row strides, deterministic cache reuse, and one failed multi-channel glyph beside successful glyphs.
- [x] Append one SM2-M2-S2 output row.

### Testing stage SM2-M2-T

- [x] Run scoped formatting/diff checks and structure budgets.
- [x] Run a managed Windows `cargo check -p zircon_runtime --lib --no-default-features --features target-client --locked --jobs 1`.
- [ ] Run exact atlas/bake/upload filters for SDF, MSDF, MTSDF, multipage, dirty upload, eviction invalidation, and fallback spans.
- [ ] Re-run the accepted SM-M5 Native/SDF layout identity filters to prove raster expansion did not enter the layout key.
- [x] Debug bottom-up: shared mode/format mapping → page stride/upload → generator → fallback mapping → renderer preparation.
- [x] Update `docs/zircon_runtime/text/sdf.md` and the existing text atlas module docs.
- [ ] Append one SM2-M2-T output row and promote only with all exact filters green.

**Exit evidence:** shared generator bytes reach unified atlas plans and upload reports; renderer-local params are absent; no WGPU shader change yet.

## Milestone SM2-M3: GPU decode and product framebuffer

**Goal:** Render dynamic MSDF/MTSDF glyphs through WGPU with explicit decode semantics and prove visible sharp glyph pixels.

**In-scope behaviors:** decode-mode vertex propagation; median RGB fill; MTSDF alpha availability; texture/storage compatibility; large sharp Latin and mixed CJK product proof; target hygiene.

**Dependencies:** SM2-M2 fully accepted and current renderer WGPU baseline green.

### Implementation slice SM2-M3-S1: vertex and WGSL decode

**Files:**

- Modify `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs`
- Replace `zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl` with the plan-owned `zr_text_sdf.wgsl` name and update the include site.
- Modify renderer shader/vertex tests.

- [x] Add `decode_mode: u32` as a flat vertex attribute derived from `SdfMode::shader_discriminant()`.
- [x] Implement WGSL `median3`, mode-selected distance sampling, and existing `sdf_coverage` consumption. MTSDF fill uses the RGB median; alpha is carried as the true-distance value for SM-M4 rather than discarded by the contract.
- [x] Keep SDF page sampling valid by ensuring the renderer binds storage-compatible texture arrays per distance-field page family; do not reinterpret R8 as RGBA.
- [x] Parse WGSL in tests and assert the old single `.r` unconditional decode and old shader filename are absent.
- [x] Add vertex tests proving SDF/MSDF/MTSDF mode propagation for horizontal and VerticalRl glyphs.
- [x] Append one SM2-M3-S1 output row.

### Implementation slice SM2-M3-S2: raster policy and product source

**Files:**

- Modify `zircon_runtime/src/text/raster/policy.rs`
- Modify `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs`
- Modify folder-backed proof command/assertion owners under `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/`
- Create acceptance PNG only during SM2-M3-T under `docs/tests/runtime/text/`

- [x] Make explicit `GlyphAtlasFormat::Msdf` select `SdfMode::Msdf`; requests needing true-distance effects select MTSDF only when SM-M4 effect data is present, while plain large text remains SDF unless explicitly requested.
- [x] Add a product command that renders a large sharp `A/M/W` sample and a mixed `MSDF 尖角` sample through the real MSDF path.
- [x] Add framebuffer assertions requiring non-background pixels, bounded glyph boxes, and sharper corner occupancy than the side-by-side SDF comparison. Assertions inspect pixels; no policy label image is accepted as proof.
- [x] Keep the product root below 800 lines by extending the existing proof command/assertion folders.
- [x] Append one SM2-M3-S2 output row before testing; the PNG is still absent at this point.

### Testing stage SM2-M3-T

- [ ] Run scoped formatting/diff checks, conflict-marker/trailing-whitespace scans, shader parsing tests, and file budgets.
- [x] Build the exact target-client product integration on a coordinator-managed Windows GPU target.
- [x] Run `export_runtime_multilingual_text_product_framebuffer_png --ignored --nocapture --test-threads=1` from the current-source executable.
- [x] Inspect the original-resolution PNG and record dimensions, byte length, color count, SHA256, and MSDF/SDF changed-pixel bounds.
- [x] Scan repository `target` plus approved D/E/F cargo target roots for same-name PNG copies; the count must be zero.
- [ ] Re-run SM2-M1 CPU decode/generator tests, SM2-M2 atlas/upload tests, SM-M5 layout parity, and existing SDF VerticalRl product assertions upward.
- [x] Update `docs/zircon_runtime/text/sdf.md`, Text05 numbered output records, and the active Session note with exact evidence.
- [x] Run the plan-output and failure-handoff audits, recording only unrelated existing violations rather than modifying foreign plans.
- [ ] Append one SM2-M3-T output row and close SM-M2 only when all CPU, atlas, shader, WGPU, visual, and target-hygiene gates pass.

**Exit evidence:** accepted real framebuffer in `docs/tests/runtime/text`, current-source exact WGPU pass, CPU sharp-corner/true-distance evidence, and zero target copies.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| SM2-M1 | S1 dependency and declaration surface | implemented-testing-pending | 2026-07-13 | `text/sdf/{mode,params,glyph_data,generation_error}.rs`; optional fdsm dependencies wired; declaration tests authored; scoped rustfmt/diff-check passed。 |
| SM2-M1 | S2 reference decode math | implemented-testing-pending | 2026-07-13 | `text/sdf/decode.rs` and folder-backed decode tests cover median channel ordering, MTSDF alpha, 0.5 boundary, and minimum pixel range; scoped rustfmt/diff-check passed。 |
| SM2-M1 | S3 fdsm outline generation | implemented-testing-pending | 2026-07-13 | `fdsm_gen.rs` imports Fira glyph outlines, applies deterministic 48/8 transform/edge coloring, sign/error correction, vertical flip, typed failures, and SDF/MSDF/MTSDF packing; generator tests authored; scoped rustfmt/diff-check passed。 |
| SM2-M1 | T current-source acceptance | passed | 2026-07-13 | Managed Windows `cargo check -p zircon_runtime --lib --no-default-features --features graphics --locked --jobs 1` passed in 12m23s. Current-source lib-test built in 34m05s; shared params/data 4/4, decode 3/3, fdsm generation 4/4 passed (6173 total tests filtered). The first `features=text` run was retained only as a diagnostic because crate-level `text/sdf` is compiled behind `feature=graphics`; acceptance uses the corrected graphics-enabled gate. |
| SM2-M2 | S1 shared params and mode-keyed atlas hard cut | implemented-testing-pending | 2026-07-13 | Deleted renderer-local `sdf_params.rs` and all imports/registration; all consumers use `graphics::text::sdf::SdfBakeParams`. Atlas allocation now partitions shelf allocators/page reservation by `SdfMode::atlas_format()`, preserving distinct SDF/MSDF/MTSDF keys while sharing MSDF/MTSDF RGBA page families. Added focused identity/storage test; scoped rustfmt/diff-check and zero legacy-symbol scan passed. |
| SM2-M2 | S2 shared generator/stride/fallback integration | implemented-testing-pending | 2026-07-13 | UI font bake delegates pixels to shared `text/sdf`, emits R8 and RGBA page byte spans, mode-keyed cache reuse, typed generation failures, format-aware dirty upload commands, and normal native-overlay fallback. Focused tests are authored; production `graphics` and `target-client` checks pass, while the monolithic lib-test gate is currently blocked by 25 unrelated runtime-plugin test API-drift errors. |
| SM2-M3 | S1 explicit GPU decode and storage families | implemented-testing-pending | 2026-07-13 | Vertex carries flat `decode_mode`; `zr_text_sdf.wgsl` selects R8 SDF or RGBA MSDF sampling, decodes median RGB, and preserves MTSDF alpha true distance. Renderer owns separate storage-compatible texture arrays through folder-backed `sdf_render/atlas_resources.rs`. Current-source target-client check and real WGPU shader/pipeline creation pass. |
| SM2-M3 | S2 explicit UI mode and product source | implemented-testing-pending | 2026-07-13 | `UiTextRenderMode::{Msdf,Mtsdf}` routes through the shared layout/shaper but selects raster-only `SdfMode`; raster policy exposes explicit MSDF and true-distance MTSDF selection. Product proof adds side-by-side `A/M/W · SDF/MSDF 尖角` real framebuffer regions while keeping the root at 800 lines. |
| SM2-M3 | T real WGPU product proof | product-passed-unit-regression-pending | 2026-07-13 | Exact ignored product exporter passed 1/1 after a 32m30s current-source build and 312.49s runtime; the stricter high-contrast A-apex occupancy rerun also passed in coordinator job `174af981a240444c8474eb7f03594769`. PNG is 1080×1690, 315799 bytes, 2444 colors, SHA256 `05BED61944F35380A0967A9A3D04DFC0B1F5413D240A2C5627A4E3D4294FB448`; original review confirms visible SDF/MSDF rows and existing multilingual/RTL/VerticalRl/table content; same-name target scan 0. Scoped format/diff/owner budgets pass. Plan-output audit exits 0; failure audit exits 0 while reporting unrelated pre-existing editor-plan failure schema/backlink diagnostics, which remain with their owners. Monolithic unit filters remain blocked by unrelated runtime-plugin test drift, so SM2-M2/M3 are not promoted yet. |
