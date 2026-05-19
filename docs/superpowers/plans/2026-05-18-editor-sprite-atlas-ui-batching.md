# Editor SpriteAtlas UI Batching Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build editor-generated SpriteAtlas artifacts that can feed retained-host UI images through one atlas texture per resource key, while proving WGPU UI surface batching preserves partial-order correctness for same-resource images.

**Architecture:** `zircon_editor` owns authoring-time atlas generation under the editor asset manager and writes deterministic atlas artifacts into the project library. `zircon_runtime` owns neutral atlas data contracts, asset loading, retained UI surface payload semantics, WGPU UV adjustment, and draw submission. Concrete sprite renderer/product-pipeline edits remain out of scope until coordinated with the active render M7A session.

**Tech Stack:** Rust 2021, Cargo workspace, `image`, `serde`, `toml`, planned workspace `rectangle-pack = "0.4"`, runtime WGPU UI surface, retained-host command stream, project library artifacts.

---

## Repository Cadence And Coordination

- Work directly in the existing `main` checkout. Do not create a worktree, branch, or compatibility path.
- Preserve unrelated dirty files from active sessions. Never revert render, material, input, MUI, Hub, or ZrVM-session changes unless the user explicitly requests it.
- Follow the milestone-first policy: implementation slices may add production code, tests, comments, and docs, but compile/build/unit-test execution belongs to the milestone testing stage unless a scoped `cargo check` is needed to remove malformed Rust before handoff.
- Prefer external target dirs for validation, for example `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'`. Do not commit machine-specific paths.
- Before entering any concrete sprite renderer files, refresh `.codex/sessions/20260508-2100-render-m4-plus-design.md` and coordinate ownership. This plan intentionally keeps sprite renderer integration as a separate coordinated milestone gate.
- Keep `zircon_hub` and Slint untouched. Validation scope for UI remains `zircon_editor` retained host plus `zircon_runtime` WGPU UI surface.
- Do not claim pass/completion without fresh command output. If existing render-lane drift blocks focused runtime tests before these tests run, record that exact blocker and do not mark the milestone promoted.

## Current Baseline

- `zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs` already uses partial-order layers and groups images by `resource_key` through `BTreeMap<String, Vec<ImageVertex>>` inside each independent layer.
- Existing test `batch_plan_groups_images_by_resource_key` proves same-key grouping for disjoint images, but does not prove the key regression cases for same-resource overlapping images, same-resource separated-by-overlap chains, or multiple atlas subregions sharing one texture.
- `zircon_runtime/src/rhi/ui_surface.rs` defines `UiSurfaceImagePayload` with `resource_key`, `width`, `height`, `upload_bytes`, and optional `rgba`; it has no atlas-region or UV-rect field.
- `zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs` defines the editor-side `ChromeImagePayload` with the same payload shape and no atlas-region field.
- `zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs` forwards `ChromeImagePayload` into `UiSurfaceImagePayload` field-for-field.
- `zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs` computes image UVs from the visible clipped rect against the command frame as if the whole source image is sampled.
- `zircon_runtime/src/rhi_wgpu/ui_surface.rs` uploads one texture per `resource_key`, binds that texture once per image draw op, and draws the image vertex range.
- `zircon_runtime_interface/src/resource/marker.rs` currently has no `ResourceKind::SpriteAtlas`, so a standalone runtime asset kind is a public contract change.
- The selected product direction is editor-generated SpriteAtlas artifacts first, not runtime importer output and not a transient renderer cache.

## Reference Evidence

- `dev/bevy/crates/bevy_image/src/texture_atlas.rs` separates the atlas texture from `TextureAtlasLayout`, stores atlas size plus per-region pixel rects, and can derive UV rects from the atlas size.
- `dev/bevy/crates/bevy_image/src/texture_atlas_builder.rs` uses `rectangle-pack = "0.4"`, grows atlas dimensions up to a max size, preserves insertion order for layout indices, copies source image rows into a new atlas image, and returns both texture and layout.
- `dev/bevy/crates/bevy_ui/src/widget/image.rs` lets UI images carry an optional atlas reference and source rect while keeping the sampled texture handle as the binding authority.
- Zircon divergence: because this slice chooses an editor-generated artifact, atlas generation belongs in `zircon_editor` while the serializable atlas layout type belongs in `zircon_runtime` so runtime/UI consumers can read artifacts without depending on editor code.

## Target Architecture

- Runtime asset contract:
  - `TextureAsset` remains the atlas texture payload container for RGBA pixels.
  - New neutral SpriteAtlas data stores atlas texture locator plus named entries with source locator, pixel rect, UV rect, original size, trimmed size if introduced, and padding.
  - The first slice should avoid changing `ResourceKind` unless a project-facing standalone atlas asset becomes necessary. Store the atlas manifest as `DataAsset` or a serializable runtime asset module loaded by editor code first, then promote to `ResourceKind::SpriteAtlas` only if runtime project loading requires direct registry identity.
- Editor artifact ownership:
  - Atlas generation lives under `zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/`.
  - The editor scans selected texture records or a deterministic atlas group, decodes source images with `image`, packs rectangles with `rectangle-pack`, writes an atlas PNG plus manifest TOML/JSON under the project `library`, and records enough diagnostics to make stale or failed atlas generation visible.
  - Existing preview cache patterns under `editor_asset_manager/preview.rs` and `preview_refresh/generate_preview_artifact.rs` are examples for editor-owned artifact output, but SpriteAtlas artifacts must not be thumbnail-only or UI-cache-private.
- Retained UI consumption:
  - Extend `ChromeImagePayload` and `UiSurfaceImagePayload` with an optional atlas UV rect while preserving the existing non-atlas path as the default data shape.
  - The `resource_key` for an atlas-backed image is the atlas texture identity. Different subimages inside the same atlas use the same `resource_key` plus different UV rects, which makes WGPU batching bind one texture and draw all independent subregions together.
  - The editor still includes upload bytes and RGBA bytes only when the atlas texture is newly produced or refreshed; repeated subimages should carry `rgba: None` after cache warm-up.
- WGPU geometry and batching:
  - `geometry.rs` maps clipped command-frame UVs into the atlas UV rect when present, otherwise keeps full-image UV semantics.
  - `batching.rs` still groups by `resource_key`; no atlas-specific grouping branch should be added.
  - `ui_surface.rs` keeps texture upload/cache authority keyed by `resource_key`; atlas subregions must not create separate GPU textures.

## File Map

### Workspace And Dependencies

- Modify `Cargo.toml`:
  - Add `rectangle-pack = "0.4"` under `[workspace.dependencies]` when the editor packer implementation begins.
- Modify `zircon_editor/Cargo.toml`:
  - Add `rectangle-pack.workspace = true` when `sprite_atlas/packer.rs` is added.
- No plugin workspace manifest changes are expected for this plan.

### Runtime Atlas Contract

- Create `zircon_runtime/src/asset/assets/sprite_atlas/mod.rs`:
  - Structural module file exporting the SpriteAtlas layout types and validation helper modules.
- Create `zircon_runtime/src/asset/assets/sprite_atlas/layout.rs`:
  - Owns serializable atlas manifest structures such as `SpriteAtlasAsset`, `SpriteAtlasEntry`, `SpriteAtlasRect`, and `SpriteAtlasPadding`.
- Create `zircon_runtime/src/asset/assets/sprite_atlas/validation.rs`:
  - Owns invariant checks for atlas dimensions, non-empty entries, in-bounds rects, finite UVs, duplicate names, duplicate source URIs, and byte-size overflow protections.
- Modify `zircon_runtime/src/asset/assets/mod.rs`:
  - Add `mod sprite_atlas;` and curated re-exports only. Keep this root file structural.
- Modify `zircon_runtime/src/asset/assets/imported.rs` only if runtime loading must treat atlas manifests as an `ImportedAsset` variant after Milestone 2 evidence shows `DataAsset` is insufficient.
- Modify `zircon_runtime_interface/src/resource/marker.rs` only if the implementation promotes SpriteAtlas to a first-class `ResourceKind`. If this promotion occurs, update all `asset_kind_for_imported_asset` and preview placeholder matches in the same milestone.

### UI Surface Payload And Geometry

- Modify `zircon_runtime/src/rhi/ui_surface.rs`:
  - Add an optional atlas UV rect to `UiSurfaceImagePayload`.
  - Add tests for default/non-atlas stats and atlas payload visibility.
- Modify `zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs`:
  - Store the optional atlas UV rect on `ImageItem` or precomputed vertices.
  - Make `image_vertices` compose visible-frame UVs with the optional atlas rect.
  - Add clipped atlas UV tests.
- Modify `zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs`:
  - Add tests proving same-resource atlas images batch when disjoint and split when overlapping.
  - Avoid atlas-specific branches in production code unless a test exposes a true missing abstraction.
- Modify `zircon_runtime/src/rhi_wgpu/ui_surface.rs`:
  - Confirm atlas-backed images upload/cache/draw using only the atlas texture `resource_key`.
  - Add headless stats tests if needed to prove one texture key drives one image draw op for multiple subregions.

### Editor Atlas Artifact Generation

- Create `zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/mod.rs`:
  - Structural module file for the editor atlas generator.
- Create `zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/config.rs`:
  - Owns `SpriteAtlasBuildConfig` with deterministic defaults: padding, initial size, max size, output stem, and source selection mode.
- Create `zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs`:
  - Wraps `rectangle-pack`, validates source dimensions, grows atlas size, and returns placements without touching project IO.
- Create `zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/artifact.rs`:
  - Writes atlas image plus manifest into the project library and returns artifact locators.
- Create `zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/diagnostics.rs`:
  - Owns success/error summaries such as source count, atlas size, packed area, padding, skipped sources, and failure reason.
- Modify `zircon_editor/src/ui/host/editor_asset_manager/manager/mod.rs`:
  - Add a structural `mod sprite_atlas;` plus curated internal exports only.
- Modify `zircon_editor/src/ui/host/editor_asset_manager/api.rs` or the current manager API file only after inspecting the narrowest existing call surface for editor asset operations. Add a single method such as `generate_sprite_atlas(...)` if there is already an editor-asset manager API pattern for preview refresh.

### Retained Host Atlas Consumption

- Modify `zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs`:
  - Add optional atlas UV rect to `ChromeImagePayload`.
  - Keep existing `push_image(...)` callers compiling by passing `None` where no atlas entry is selected.
  - Add tests that command-stream replay preserves non-atlas pixels and forwards atlas metadata.
- Modify `zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs`:
  - Forward atlas UV rect from `ChromeImagePayload` into `UiSurfaceImagePayload`.
  - Add a presenter unit test that inspects the recorded `UiSurfaceDrawList` and asserts atlas metadata reaches runtime.
- Modify image command producers only where atlas manifests are intentionally consumed:
  - `zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs`
  - `zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs`
  - `zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs`
  - Keep viewport frame images non-atlas unless a concrete atlas manifest owns them; do not force dynamic viewport textures into a static atlas.

### Docs

- Modify `docs/zircon_runtime/rhi/ui_surface.md`:
  - Document atlas UV semantics, `resource_key` as atlas texture identity, and partial-order batching evidence.
- Modify `docs/zircon_editor/ui/retained_host/performance.md`:
  - Document retained-host atlas payload forwarding, expected profile counters, and validation artifacts.
- Create `docs/zircon_runtime/asset/assets/sprite_atlas.md`:
  - Module-detail doc for runtime atlas data model and validation.
- Create `docs/zircon_editor/ui/host/editor_asset_manager/sprite_atlas.md`:
  - Module-detail doc for editor-generated atlas artifacts.
- Every new or updated code-facing doc must have YAML frontmatter with `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`.

## Milestone 1: UI Partial-Order Same-Resource Evidence

- Goal: Prove the current WGPU UI surface planner batches same-resource images only when partial-order dependencies allow it, before adding atlas payload fields.
- In-scope behaviors:
  - Disjoint images with the same `resource_key` in one layer produce one image draw op.
  - Disjoint images with different `resource_key` values produce one draw op per resource key.
  - Overlapping images with the same `resource_key` stay in separate layers and produce separate image draw ops.
  - A chain where a middle overlapping item separates two same-resource images preserves painter order through dependency depth.
  - Solid/text items remain normal material boundaries and do not become atlas-specific branches.
- Dependencies:
  - Existing `draw_items`, `dependency_depths`, and `push_layer_image_draws` behavior in `zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs`.
- Implementation slices:
  - [ ] Add `batch_plan_batches_disjoint_images_with_same_resource_key_into_one_draw` in `zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs` using three non-overlapping image commands with `resource_key = "atlas://editor/icons"`; assert `visible_draw_item_count = 3`, `draw_calls = 1`, `batch_layer_count = 1`, `batch_dependency_count = 0`, and `ImageDraw.vertices.len() = 18`.
  - [ ] Add `batch_plan_splits_overlapping_images_even_with_same_resource_key` using two overlapping images with the same key; assert `visible_draw_item_count = 2`, `draw_calls = 2`, `batch_layer_count = 2`, `batch_dependency_count = 1`, and both image draws use the same key.
  - [ ] Add `batch_plan_preserves_overlap_chain_between_same_resource_images` using image A at x 0..20, solid or image B at x 10..30, and image C at x 24..44 with A and C sharing the same key but B overlapping both; assert three layers and three draw ops.
  - [ ] Add `batch_plan_batches_independent_same_resource_images_around_overlap` using image A at x 0..20, solid B at x 10..30, and image C at x 40..60 with A and C sharing the same key; assert `visible_draw_item_count = 3`, `draw_calls = 2`, `batch_layer_count = 2`, `batch_dependency_count = 1`, and the layer-0 `ImageDraw` contains 12 vertices for A plus C.
  - [ ] Do not change production planner code in this milestone unless a new test exposes an existing incorrect expectation.
  - [ ] Update `docs/zircon_runtime/rhi/ui_surface.md` plan source and tests header entries for the new batching evidence.
- Testing stage: `M1 UI planner evidence gate`.
  - Compile/build command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_runtime --lib --locked --message-format short --color never`.
  - Unit-test command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never`.
  - Focused fallback command if render-lane drift blocks the broader filter before these tests run: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib batch_plan_batches_disjoint_images_with_same_resource_key_into_one_draw --locked --jobs 1 --message-format short --color never`.
  - Debug/correction loop: If a test fails because dependencies are lower or higher than expected, inspect `dependency_depths` and clipped rects first; if a test fails before reaching UI batching due unrelated render compile drift, record the failing file/test and do not modify render-owned sprite modules from this milestone.
  - Acceptance evidence: command output showing the new focused planner tests passed, or a recorded external blocker that occurs before those tests can compile.
- Lightweight checks:
  - `rustfmt --edition 2021 --check zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs` is allowed before testing stage if only this file changes.
- Exit evidence:
  - The planner test names above pass under `cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1`, and `docs/zircon_runtime/rhi/ui_surface.md` records the evidence.

## Milestone 2: Runtime SpriteAtlas Data Model

- Goal: Add a neutral, serializable runtime atlas layout model that editor artifacts and runtime UI consumers can share without introducing renderer-specific objects.
- In-scope behaviors:
  - Atlas manifest stores atlas texture URI/resource key separately from region entries.
  - Entries are stable by name and optional source URI, with pixel rects and UV rects derived from atlas size.
  - Validation rejects zero atlas dimensions, zero entry dimensions, out-of-bounds rects, duplicate entry names, non-finite UVs, and UVs outside 0..1.
  - The data model avoids direct `wgpu`, editor, Slint, or sprite-renderer types.
- Dependencies:
  - `TextureAsset` remains the image bytes container.
  - `AssetUri` remains the serializable project locator type.
- Implementation slices:
  - [ ] Create `zircon_runtime/src/asset/assets/sprite_atlas/mod.rs` with `mod layout; mod validation;` and public re-exports for the data model and validation error.
  - [ ] Create `layout.rs` with these initial public types:

```rust
use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteAtlasAsset {
    pub atlas_texture: AssetUri,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub padding: SpriteAtlasPadding,
    #[serde(default)]
    pub entries: Vec<SpriteAtlasEntry>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAtlasPadding {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteAtlasEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<AssetUri>,
    pub pixel_rect: SpriteAtlasRect,
    pub uv_rect: SpriteAtlasUvRect,
    pub source_width: u32,
    pub source_height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteAtlasUvRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}
```

  - [ ] Add `SpriteAtlasUvRect::from_pixel_rect(rect, atlas_width, atlas_height) -> Result<Self, SpriteAtlasValidationError>` and keep all division guarded against zero dimensions.
  - [ ] Create `validation.rs` with `SpriteAtlasValidationError` and `validate_sprite_atlas_asset(&SpriteAtlasAsset) -> Result<(), SpriteAtlasValidationError>`.
  - [ ] Modify `zircon_runtime/src/asset/assets/mod.rs` to re-export `SpriteAtlasAsset`, `SpriteAtlasEntry`, `SpriteAtlasPadding`, `SpriteAtlasRect`, `SpriteAtlasUvRect`, and `SpriteAtlasValidationError` only.
  - [ ] Add unit tests under `layout.rs` or `validation.rs` for UV derivation, duplicate names, out-of-bounds rect, zero atlas size, and zero entry size.
  - [ ] Create `docs/zircon_runtime/asset/assets/sprite_atlas.md` with the required YAML header and behavior details.
- Testing stage: `M2 atlas data contract gate`.
  - Compile/build command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_runtime --lib --locked --message-format short --color never`.
  - Unit-test command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never`.
  - Serialization compatibility command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib asset --locked --jobs 1 --message-format short --color never`.
  - Debug/correction loop: If serde or `AssetUri` serialization fails, fix the atlas data model rather than adding editor-only serialization wrappers; if a public `ResourceKind` change becomes necessary, stop and update this milestone to include `zircon_runtime_interface` and all asset-kind matches.
  - Acceptance evidence: runtime atlas model tests pass and docs list the exact model tests.
- Lightweight checks:
  - `cargo check -p zircon_runtime --lib --locked --message-format short --color never` is allowed before the testing stage after adding module wiring.
- Exit evidence:
  - The atlas model serializes, validates, and exports from `zircon_runtime::asset` without adding editor/runtime boundary violations.

## Milestone 3: Editor Atlas Packer And Artifact Writer

- Goal: Generate deterministic editor-owned atlas artifacts from texture sources and write atlas image plus manifest into the project library.
- In-scope behaviors:
  - Decode RGBA8 source images with `image`.
  - Pack source rectangles with `rectangle-pack` using deterministic insertion order.
  - Grow atlas dimensions from config initial size to max size.
  - Copy source image rows into a transparent RGBA atlas image.
  - Write an atlas PNG and atlas manifest under `ProjectPaths::library_root()`.
  - Return structured diagnostics for success, skipped sources, and pack failures.
  - Do not change runtime importers or plugin importer workspaces in this milestone.
- Dependencies:
  - Milestone 2 atlas data model.
  - Existing editor dependency on `image`.
  - New workspace dependency `rectangle-pack = "0.4"`.
- Implementation slices:
  - [ ] Add `rectangle-pack = "0.4"` to root `[workspace.dependencies]` and `rectangle-pack.workspace = true` to `zircon_editor/Cargo.toml`.
  - [ ] Create `sprite_atlas/config.rs` with `SpriteAtlasBuildConfig { output_stem: String, padding: (u32, u32), initial_size: (u32, u32), max_size: (u32, u32) }` and defaults `output_stem = "editor-atlas"`, `padding = (1, 1)`, `initial_size = (256, 256)`, `max_size = (2048, 2048)`.
  - [ ] Create `sprite_atlas/packer.rs` with `SpriteAtlasSourceImage { name, source_uri, width, height, rgba }` and `pack_sprite_atlas_sources(config, sources) -> Result<PackedSpriteAtlas, SpriteAtlasBuildError>`.
  - [ ] In `packer.rs`, use `GroupedRectsToPlace::<usize>`, `RectToInsert::new(width + padding.x, height + padding.y, 1)`, `TargetBin::new(current_width, current_height, 1)`, `volume_heuristic`, and `contains_smallest_box`, matching the Bevy reference behavior while returning Zircon-owned structs.
  - [ ] In `packer.rs`, copy each source row into the atlas buffer at the packed location and subtract padding from the region max when deriving `SpriteAtlasRect`.
  - [ ] Create `sprite_atlas/artifact.rs` with `write_sprite_atlas_artifacts(project: &ProjectManager, config: &SpriteAtlasBuildConfig, packed: &PackedSpriteAtlas) -> Result<SpriteAtlasArtifactReport, SpriteAtlasBuildError>`.
  - [ ] Use `project.paths().library_root()` only if `ProjectManager` exposes paths; if it does not, add the narrowest existing accessor in the project manager module instead of reaching into private fields from editor code.
  - [ ] Create `diagnostics.rs` with `SpriteAtlasBuildDiagnostics` fields: `source_count`, `packed_count`, `atlas_width`, `atlas_height`, `padding`, `packed_area`, `atlas_area`, `skipped_sources`, and `message`.
  - [ ] Add tests using two or three tiny in-memory RGBA images to assert deterministic entry order, in-bounds rects, atlas byte copying, padding separation, and pack failure when max size is too small.
  - [ ] Create `docs/zircon_editor/ui/host/editor_asset_manager/sprite_atlas.md` with the required YAML header.
- Testing stage: `M3 editor atlas artifact gate`.
  - Compile/build command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_editor --lib --locked --message-format short --color never`.
  - Unit-test command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_editor --lib sprite_atlas --locked --jobs 1 --message-format short --color never`.
  - Runtime contract command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never`.
  - Debug/correction loop: If artifact-path tests need filesystem access, use a temporary project directory through existing project-manager test helpers; do not hardcode `E:\` or `D:\` paths in tests.
  - Acceptance evidence: editor atlas tests prove deterministic packing and artifact manifest output; runtime atlas validation still passes.
- Lightweight checks:
  - `cargo check -p zircon_editor --lib --locked --message-format short --color never` after dependency and module wiring.
- Exit evidence:
  - Editor can generate atlas bytes and a runtime-valid manifest without changing runtime importers or concrete renderers.

## Milestone 4: Retained Host And UI Surface Atlas Payload

- Goal: Thread optional atlas UV metadata from retained-host image commands into runtime UI surface geometry without changing non-atlas image behavior.
- In-scope behaviors:
  - Non-atlas images continue to sample full texture UV 0..1.
  - Atlas-backed images sample the specified UV rect while clipping still trims within the command frame.
  - `ChromeImagePayload` and `UiSurfaceImagePayload` carry equivalent optional atlas UV data.
  - Existing viewport image tests still pass with `atlas_uv = None`.
  - Softbuffer command-stream replay either ignores atlas metadata when RGBA bytes represent the subimage, or uses atlas metadata only if the RGBA bytes represent the full atlas. This milestone must choose one explicit behavior and test it.
- Dependencies:
  - Milestone 2 atlas UV type or an equivalent `UiSurfaceImageUvRect` if keeping RHI DTOs independent from asset DTOs.
  - Milestone 1 batching proof.
- Implementation slices:
  - [ ] Add a small DTO in `zircon_runtime/src/rhi/ui_surface.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSurfaceImageUvRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}
```

  - [ ] Add `#[serde]` only if the surrounding RHI payload becomes serialized; otherwise keep the DTO plain Rust like the existing UI surface types.
  - [ ] Add `pub atlas_uv: Option<UiSurfaceImageUvRect>` to `UiSurfaceImagePayload`.
  - [ ] Add the matching editor-side `ChromeImageUvRect` and `atlas_uv: Option<ChromeImageUvRect>` to `ChromeImagePayload` in `command_stream.rs`.
  - [ ] Update all existing `ChromeImagePayload` construction sites to set `atlas_uv: None` unless they consume a SpriteAtlas manifest.
  - [ ] Update `gpu.rs` to forward the UV rect field into `UiSurfaceImagePayload`.
  - [ ] Update `geometry.rs` so `image_vertices(frame, visible_rect, surface_size, atlas_uv)` maps local clipped UVs into the atlas range using `atlas_min + local_uv * (atlas_max - atlas_min)`.
  - [ ] Validate atlas UV rects at geometry time by falling back to full-image UVs only for `None`; invalid `Some` values should be rejected before command creation or skipped in geometry with a focused test. Prefer rejecting/skipping invalid atlas commands over silently sampling wrong pixels.
  - [ ] Add tests in `geometry.rs`: full-image clipped UVs remain `[0.25, 0.25]` to `[0.75, 0.75]`; atlas UV `[0.5, 0.25]` to `[0.75, 0.5]` with the same clip maps to `[0.5625, 0.3125]` to `[0.6875, 0.4375]`.
  - [ ] Add a `gpu.rs` test that creates a `ChromeImagePayload` with atlas UV and asserts the recorded `UiSurfaceDrawList` contains the same UV rect.
  - [ ] Update `docs/zircon_runtime/rhi/ui_surface.md` and `docs/zircon_editor/ui/retained_host/performance.md`.
- Testing stage: `M4 retained UI atlas payload gate`.
  - Compile/build commands:
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_runtime --lib --locked --message-format short --color never`
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_editor --lib --locked --message-format short --color never`
  - Unit-test commands:
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never`
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format short --color never`
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never`
  - Debug/correction loop: If non-atlas viewport image parity breaks, revert the atlas payload path to `None` at the producer and fix geometry composition before touching WGPU cache/upload code.
  - Acceptance evidence: non-atlas command-stream and viewport-image tests pass, atlas metadata reaches runtime, and atlas UV clipping is covered by numeric geometry tests.
- Lightweight checks:
  - `rustfmt --edition 2021 --check zircon_runtime/src/rhi/ui_surface.rs zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs` after DTO and geometry edits.
- Exit evidence:
  - Atlas metadata is a normal optional image payload field, not a special renderer branch.

## Milestone 5: Atlas-Backed UI Batching And Profile Evidence

- Goal: Use editor-generated atlas outputs in retained UI image commands so WGPU binds one atlas texture and draws independent subimages in one image draw op per partial-order layer.
- In-scope behaviors:
  - A UI image producer can resolve an atlas entry into `resource_key = atlas texture key` plus `atlas_uv = entry.uv_rect`.
  - Multiple disjoint atlas entries from the same atlas batch together as one image draw op.
  - Different atlases remain separate draw ops.
  - Overlapping atlas entries preserve painter order through existing partial-order layers.
  - Runtime image cache uploads the atlas texture once per `resource_key` and touches it on repeated atlas subimages.
  - Live profiling records improved image draw reduction for an atlas-heavy scenario without reintroducing software fallback.
- Dependencies:
  - Milestones 1 through 4.
  - A concrete retained-host image source suitable for static atlas grouping, such as repeated icons or preview images. Dynamic viewport frames remain non-atlas.
- Implementation slices:
  - [ ] Add an editor atlas manifest resolver near the image producer selected for this milestone. The resolver returns `(atlas_resource_key, atlas_uv, atlas_width, atlas_height, atlas_rgba_or_none)` for a named source entry.
  - [ ] Update the chosen retained-host image producer to use atlas-backed `ChromeImagePayload` only when an atlas manifest entry exists; otherwise keep the existing non-atlas payload.
  - [ ] Add a command-stream unit test with two same-atlas images and one different-atlas image; assert two distinct `resource_key` values and two `atlas_uv` values.
  - [ ] Add a runtime planner test with two disjoint atlas-backed images sharing a key and different UV rects; assert one `ImageDraw` and the expected vertices carry different UV ranges.
  - [ ] Add or update a headless WGPU presenter stats test proving `visible_draw_item_count = 2` and `draw_calls = 1` for disjoint same-atlas images.
  - [ ] Keep `record_draw_ops_to_view` unchanged except for any necessary debug assertions; the existing bind-group-per-`resource_key` draw path is the intended atlas batching mechanism.
  - [ ] Update profile metric docs to state the ideal image term is `image_resource_key_count`, where atlas subimages should share one resource key per atlas.
- Testing stage: `M5 atlas UI batching evidence gate`.
  - Compile/build commands:
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_runtime --lib --locked --message-format short --color never`
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_editor --lib --locked --message-format short --color never`
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_app --features target-editor-host --locked --message-format short --color never`
  - Unit-test commands:
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never`
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format short --color never`
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never`
  - Live profile commands after a profiling build exists:
    - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo build -p zircon_app --bin zircon_editor --profile profiling --features "target-editor-host profiling profiling-chrome" --locked --message-format short --color never`
    - `tools/ui-profile-capture.ps1 -Scenario viewport_image -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 4 -SkipBuild`
    - `tools/ui-profile-capture.ps1 -Scenario startup -RequireScenarioEvidence -AutoCloseSeconds 4 -SkipBuild`
  - Debug/correction loop: If live profiles show no draw reduction, inspect `ui_batch_metrics.json` and command-stream image keys first; do not alter WGPU batching until evidence shows same-atlas images are reaching runtime with the same `resource_key`.
  - Acceptance evidence: focused runtime/editor tests pass, profile artifacts show `software_fallback_present_count = 0`, and an atlas-heavy command stream reduces image draws to the number of atlas resource keys per independent layer.
- Lightweight checks:
  - `cargo check -p zircon_app --features target-editor-host --locked --message-format short --color never` is allowed before live profiling if editor/runtime contracts changed.
- Exit evidence:
  - Atlas-backed retained UI produces measurable batching without changing non-atlas viewport image behavior.

## Milestone 6: Optional Sprite Renderer/Product Pipeline Integration Gate

- Goal: Decide whether the editor-generated SpriteAtlas artifact should also feed `RenderSpriteAtlasRegion` and concrete sprite rendering, after active render-lane coordination confirms ownership.
- In-scope behaviors:
  - Read `.codex/sessions/20260508-2100-render-m4-plus-design.md` immediately before starting.
  - Coordinate with the render owner before editing `zircon_runtime/src/graphics/scene/scene_renderer/sprite/**`.
  - Map `SpriteAtlasEntry.uv_rect` into `RenderSpriteAtlasRegion` only if render product requirements need sprite draw batching in this slice.
  - Keep runtime UI atlas batching complete without this milestone.
- Dependencies:
  - Milestones 2 and 3.
  - Active render M7A/M6A status.
- Implementation slices:
  - [ ] Refresh cross-session context with `.\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4`.
  - [ ] If render session is still active on sprite/default-2D modules, ask the user or render owner for an explicit handoff before editing those files.
  - [ ] If handoff is granted, add a narrow conversion function from `SpriteAtlasEntry` to `RenderSpriteAtlasRegion` in a shared render-contract location, not inside the concrete WGPU sprite renderer.
  - [ ] Add tests under the render contract module for UV conversion and atlas texture identity.
  - [ ] Do not alter the concrete sprite renderer until render contract tests and current render-session blockers are understood.
- Testing stage: `M6 render integration coordination gate`.
  - Coordination command: `.\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4`.
  - Compile/build command after any render contract edit: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_runtime --lib --locked --message-format short --color never`.
  - Unit-test command after any render contract edit: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --locked render_product_sprite --jobs 1 --message-format short --color never`.
  - Debug/correction loop: If render tests fail in pre-existing sprite harness drift, record the failure and stop this milestone rather than making opportunistic renderer repairs.
  - Acceptance evidence: either a clean coordination note says render integration is deferred, or render contract tests pass after explicit handoff.
- Lightweight checks:
  - None before coordination; no source edit should occur in this milestone until the render lane is refreshed.
- Exit evidence:
  - UI atlas batching remains accepted regardless of whether sprite renderer integration is deferred.

## Final Workspace And Documentation Gate

- Goal: Expand validation after milestones touch shared contracts, workspace dependencies, editor/runtime crates, and documentation.
- Required docs review:
  - `docs/zircon_runtime/rhi/ui_surface.md` lists atlas UV semantics, same-resource partial-order batching tests, and profile commands.
  - `docs/zircon_editor/ui/retained_host/performance.md` lists atlas-backed image command behavior and profile evidence.
  - `docs/zircon_runtime/asset/assets/sprite_atlas.md` exists and explains model invariants.
  - `docs/zircon_editor/ui/host/editor_asset_manager/sprite_atlas.md` exists and explains artifact ownership and project-library output.
- Workspace commands:
  - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo fmt --all -- --check`
  - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo build --workspace --locked --verbose --jobs 1`
  - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test --workspace --locked --verbose --jobs 1`
  - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked --verbose --jobs 1`
  - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1`
  - `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose --jobs 1`
- Debug/correction loop:
  - If workspace validation fails in a touched crate, fix the lowest shared support layer first and rerun the focused failing command before rerunning workspace commands.
  - If workspace validation fails in an unrelated active session area, record the failing crate/test/log and the matching `.codex/sessions` note; do not modify unrelated files.
  - If plugin workspace validation fails only because no plugin files were touched but the shared lock/dependency changed, keep the plugin failure in the final risk statement until rerun passes.
- Acceptance evidence:
  - Focused milestone gates pass.
  - Workspace build/test and plugin workspace validation either pass or have clearly recorded unrelated blockers.
  - Live profile artifacts for the atlas-heavy UI scenario show no software fallback and image draw-call reduction consistent with atlas resource-key grouping.

## Open Risks

- Adding `ResourceKind::SpriteAtlas` is intentionally deferred because it changes `zircon_runtime_interface` and all asset-kind matches. Promote only when runtime project loading requires first-class registry identity rather than a data manifest plus texture artifact.
- Softbuffer atlas semantics must be explicit. If atlas-backed payloads carry full-atlas RGBA bytes, software replay needs UV sampling. If they carry subimage RGBA bytes, software replay can ignore atlas UV. The implementation must choose and test one behavior in Milestone 4.
- Live profile improvement depends on choosing an atlas-heavy retained-host image source. Static icons/previews are suitable; dynamic viewport images are not.
- Concrete sprite renderer integration is coordination-sensitive and not required for UI atlas batching acceptance.
- Existing broad workspace tests may still be blocked by unrelated dirty sessions; focused gates must stay precise and final reporting must not overclaim workspace health.

## Self-Review Notes

- Spec coverage: The plan covers same-resource UI batching proof, editor-generated atlas artifacts, runtime atlas layout, retained-host payload threading, WGPU UV adjustment, live profile evidence, documentation, and render-lane coordination.
- Placeholder scan: No milestone uses unresolved placeholder tasks; deferred work is named as a coordination gate or promotion condition.
- Type consistency: Atlas manifest types, editor payload types, and UI surface payload types are named consistently; `resource_key` remains the texture-cache authority across command stream, RHI payload, WGPU cache, and batching.
