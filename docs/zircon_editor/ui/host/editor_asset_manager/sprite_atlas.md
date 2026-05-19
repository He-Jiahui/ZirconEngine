---
related_code:
  - Cargo.toml
  - Cargo.lock
  - zircon_editor/Cargo.toml
  - zircon_editor/src/ui/host/editor_asset_manager/manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/config.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/diagnostics.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/artifact.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/mod.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/layout.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/validation.rs
implementation_files:
  - Cargo.toml
  - Cargo.lock
  - zircon_editor/Cargo.toml
  - zircon_editor/src/ui/host/editor_asset_manager/manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/config.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/diagnostics.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/artifact.rs
plan_sources:
  - docs/superpowers/plans/2026-05-18-editor-sprite-atlas-ui-batching.md
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/artifact.rs
  - 'rustfmt --edition 2021 --check zircon_editor/src/ui/host/editor_asset_manager/manager/mod.rs zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/mod.rs zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/config.rs zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/diagnostics.rs zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/packer.rs zircon_editor/src/ui/host/editor_asset_manager/manager/sprite_atlas/artifact.rs'
  - '$env:CARGO_TARGET_DIR = ''D:\cargo-targets\zircon-shared\sprite-atlas-ui''; cargo check -p zircon_editor --lib --locked --message-format short --color never'
  - '$env:CARGO_TARGET_DIR = ''D:\cargo-targets\zircon-shared\sprite-atlas-ui''; cargo test -p zircon_editor --lib sprite_atlas --locked --jobs 1 --message-format short --color never'
  - '$env:CARGO_TARGET_DIR = ''D:\cargo-targets\zircon-shared\sprite-atlas-ui''; cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never'
doc_type: module-detail
---

# Editor SpriteAtlas Artifact Generation

## Purpose

`zircon_editor::ui::host::editor_asset_manager::manager::sprite_atlas` owns editor-time SpriteAtlas generation. It packs deterministic RGBA source images into one PNG atlas, writes a TOML manifest into the project library, and uses the runtime `SpriteAtlasAsset` contract so runtime UI and later render consumers can read the result without depending on editor code.

The module is intentionally editor-owned. Runtime importers, `ResourceKind`, WGPU UI payloads, and concrete sprite renderer/product-pipeline modules are not modified in M3.

## Related Files

`config.rs` defines `SpriteAtlasBuildConfig` and validates safe output stems plus atlas size bounds. `diagnostics.rs` defines `SpriteAtlasBuildDiagnostics`, the structured report used by packer output. `packer.rs` owns `SpriteAtlasSourceImage`, `PackedSpriteAtlas`, `SpriteAtlasBuildError`, and `pack_sprite_atlas_sources`. `artifact.rs` owns `write_sprite_atlas_artifacts` and the library URI/path report.

The folder root `mod.rs` is structural only. The parent editor asset-manager `manager/mod.rs` only declares the child module and keeps behavior in the folder-backed subtree.

## Behavior Model

The packer accepts already-decoded RGBA8 sources. Each source supplies a stable name, an optional source URI, dimensions, and exact RGBA bytes. `decode_sprite_atlas_source_image` is the editor helper for converting encoded image bytes into RGBA8 source records before packing. The packer rejects empty source lists, zero dimensions, byte-length mismatches, invalid output config, and max atlas sizes that cannot fit the sources.

Packing uses `rectangle-pack` with Bevy-style `GroupedRectsToPlace`, `RectToInsert`, `TargetBin`, `volume_heuristic`, and `contains_smallest_box`. Atlas dimensions grow from `initial_size` toward `max_size` by doubling width and height until packing succeeds or the configured maximum is exhausted.

When packing succeeds, source rows are copied into a transparent RGBA atlas buffer. `SpriteAtlasEntry` values are emitted in the original source order even though placement is chosen by the packer. Padding participates in placement, but `pixel_rect` records only the actual source extent and leaves padding out of the sampled region. UVs are derived through `SpriteAtlasUvRect::from_pixel_rect`, the atlas texture URI is derived from `SpriteAtlasBuildConfig::output_stem`, and the resulting `SpriteAtlasAsset` is validated through the runtime contract before returning.

When packing fails because the configured `max_size` cannot contain the padded source set, `SpriteAtlasBuildError::PackFailed` returns `SpriteAtlasBuildDiagnostics`. Failure diagnostics record the attempted maximum atlas size, source count, total packed-area request, skipped source names, and a message suitable for editor UI reporting.

## Artifact Output

`write_sprite_atlas_artifacts` returns `SpriteAtlasBuildError` and writes under `ProjectPaths::library_root()/editor-sprite-atlases/`:

- `<output_stem>.png` contains the atlas RGBA image.
- `<output_stem>.toml` contains the runtime `SpriteAtlasAsset` manifest.

The report returns both library URIs, `lib://editor-sprite-atlases/<output_stem>.png` and `lib://editor-sprite-atlases/<output_stem>.toml`, plus the concrete output paths. The artifact writer rewrites the manifest's `atlas_texture` URI to the actual written PNG URI before serializing and revalidating the manifest.

`output_stem` must be a single safe file stem: ASCII letters, digits, `-`, `_`, and `.` only; no leading/trailing whitespace, no trailing dot, no Windows reserved device stems, and not `.` or `..`. This keeps editor-generated atlas artifacts inside the configured project-library folder and keeps generated `lib://editor-sprite-atlases/<stem>` URIs stable.

## Test Coverage

M3 unit tests are colocated with the editor packer and artifact writer because they cover private placement and artifact-output helpers. Planned focused test names are:

- `sprite_atlas_packer_packs_sources_in_deterministic_entry_order`
- `sprite_atlas_packer_copies_source_rgba_rows_into_atlas`
- `sprite_atlas_packer_keeps_padding_out_of_pixel_rects`
- `sprite_atlas_packer_derives_uvs_from_pixel_rect_without_padding`
- `sprite_atlas_packer_reports_pack_failure_when_max_size_is_too_small`
- `sprite_atlas_packer_rejects_unsafe_output_stem`
- `sprite_atlas_packer_rejects_uri_metacharacter_output_stem`
- `sprite_atlas_packer_keeps_padded_sources_separate`
- `sprite_atlas_packer_decodes_source_images_to_rgba8`
- `sprite_atlas_artifact_writer_writes_png_and_runtime_valid_manifest`
- `sprite_atlas_artifact_writer_rejects_uri_metacharacter_output_stem`

Fresh implementation and validation evidence from 2026-05-18:

- `rectangle-pack v0.4.2` was added to `Cargo.lock` by running `cargo check -p zircon_editor --lib --message-format short --color never` without `--locked` after adding the dependency.
- `rustfmt --edition 2021 --check ...` over the touched editor SpriteAtlas files passed with no output after review hardening.
- `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_editor --lib --locked --message-format short --color never` finished successfully. It emitted warnings because the editor-local SpriteAtlas API is not consumed by later retained-host milestones yet.
- `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_editor --lib sprite_atlas --locked --jobs 1 --message-format short --color never` passed after review hardening: `11` passed, `0` failed, `1380` filtered out.
- `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never` passed before later unrelated render-framework drift: `16` passed, `0` failed, `1577` filtered out.

Current validation notes:

- Earlier attempts to run the editor and runtime SpriteAtlas test gates exceeded shell timeouts while compiling dependencies. Reruns after the shared target cache warmed produced the passing evidence above.
- The active render anti-alias compile drift recorded in the session note did not block the latest M3 focused gates.
- A later post-review rerun of `cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never` is currently blocked before SpriteAtlas tests by unrelated render-framework constructor drift at `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs:136:8`: `FrameSubmissionContext::new` takes 30 arguments but 31 were supplied. This is outside the editor SpriteAtlas M3 lane and render-owned coordination area.

## Plan Sources And Scope

This module implements Milestone 3 from `docs/superpowers/plans/2026-05-18-editor-sprite-atlas-ui-batching.md`. The plan selected editor-generated atlas artifacts first, with runtime-neutral manifests from M2 and without promoting `ResourceKind::SpriteAtlas`.

Future milestones thread atlas UV metadata into retained-host UI image payloads and WGPU geometry. Concrete sprite renderer integration remains a separate coordination gate with the render session and is not part of this module.
