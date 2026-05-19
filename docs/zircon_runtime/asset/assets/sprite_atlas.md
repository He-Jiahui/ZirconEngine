---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/mod.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/layout.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/validation.rs
implementation_files:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/mod.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/layout.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/validation.rs
plan_sources:
  - docs/superpowers/plans/2026-05-18-editor-sprite-atlas-ui-batching.md
tests:
  - zircon_runtime/src/asset/assets/sprite_atlas/layout.rs
  - zircon_runtime/src/asset/assets/sprite_atlas/validation.rs
  - 'rustfmt --edition 2021 --check zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/assets/sprite_atlas/mod.rs zircon_runtime/src/asset/assets/sprite_atlas/layout.rs zircon_runtime/src/asset/assets/sprite_atlas/validation.rs'
  - '$env:CARGO_TARGET_DIR = ''D:\cargo-targets\zircon-shared\sprite-atlas-ui''; cargo check -p zircon_runtime --lib --locked --message-format short --color never'
  - '$env:CARGO_TARGET_DIR = ''D:\cargo-targets\zircon-shared\sprite-atlas-ui''; cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never'
doc_type: module-detail
---

# SpriteAtlas Asset Contract

## Purpose

`zircon_runtime::asset` exports the neutral runtime data model for editor-generated SpriteAtlas manifests. The private implementation module lives under `zircon_runtime::asset::assets::sprite_atlas`, while public consumers should use the curated facade exports such as `SpriteAtlasAsset`, `SpriteAtlasEntry`, `SpriteAtlasUvRect`, and `validate_sprite_atlas_asset`.

The model describes how named sprite regions map into one atlas texture without depending on editor types, WGPU objects, Slint UI state, or concrete sprite renderer internals.

The atlas texture remains a normal `TextureAsset` payload. `SpriteAtlasAsset` only records the manifest: the atlas texture locator, atlas dimensions, padding metadata, and named entries with pixel and UV rectangles.

## Behavior Model

`SpriteAtlasAsset::atlas_texture` is the binding authority for the atlas image. Runtime UI or future render consumers should use that URI/resource key when sampling the atlas texture, while each `SpriteAtlasEntry` selects a subregion through `pixel_rect` and `uv_rect`.

Entries are stable by `name`. `source` is optional so generated manifests can preserve the original source texture URI when one exists, while still allowing generated or synthetic regions. `source_width` and `source_height` capture the original source extent for consumers that need layout or diagnostics independent of the packed atlas rect.

`SpriteAtlasUvRect::from_pixel_rect` derives normalized UV coordinates from a pixel rect and atlas extent. It rejects zero atlas dimensions before division, zero pixel extents, and pixel rects that overflow or exceed the atlas extent.

## Invariants

Valid SpriteAtlas manifests must satisfy these invariants:

- Atlas `width` and `height` are non-zero.
- The atlas contains at least one entry.
- Entry names are non-empty after trimming whitespace.
- Entry names are unique within one atlas manifest.
- Entry names cannot contain leading or trailing whitespace, preventing whitespace-equivalent stable keys.
- Entry pixel rect `width` and `height` are non-zero.
- Entry source dimensions are non-zero.
- Entry source dimensions are at least as large as the packed pixel rect until explicit trim/scale metadata exists.
- Entry pixel rects remain within the atlas extent, including overflow-safe right and bottom edge checks.
- UV coordinates are finite, stay inside `0..=1`, preserve strict ordering on both axes, and match the UVs derived from the pixel rect and atlas dimensions.

## Validation

`validate_sprite_atlas_asset` returns `Result<(), SpriteAtlasValidationError>` and stops at the first invalid invariant. The error variants include enough context for artifact-generation diagnostics: atlas size, entry name where available, pixel rect values, atlas extent, UV min/max values, and expected-vs-actual UV mismatch data.

Validation intentionally lives beside the manifest types instead of in editor code. This keeps M3 editor artifact generation and later runtime/UI consumers aligned on one shared contract.

## ResourceKind Scope

M2 deliberately does not promote SpriteAtlas to a first-class `ResourceKind`. Atlas manifests can be carried as serialized data artifacts while `TextureAsset` remains the atlas image payload container. Promoting `ResourceKind::SpriteAtlas` would be a public runtime-interface contract change and is out of scope until a later milestone proves direct registry identity is required.

## Testing Stage

M2 follows the milestone-first testing cadence. Unit-test code lives in `layout.rs` and `validation.rs`; validation for the `M2 atlas data contract gate` used a scoped runtime check plus focused SpriteAtlas tests:

- Compile/build command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo check -p zircon_runtime --lib --locked --message-format short --color never`
- Focused unit-test command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never`
- Planned broader serialization compatibility command: `$env:CARGO_TARGET_DIR = 'D:\cargo-targets\zircon-shared\sprite-atlas-ui'; cargo test -p zircon_runtime --lib asset --locked --jobs 1 --message-format short --color never`

The focused tests cover TOML serialization roundtrip, UV derivation, zero atlas dimensions, empty entries, empty/whitespace/duplicate names, out-of-bounds pixel rects, zero entry and source dimensions, source dimensions smaller than packed pixel rects, invalid UV values, invalid UV ordering, UV-to-pixel mismatch, and valid asset acceptance.

Concrete focused test names:

- `sprite_atlas_uv_rect_derives_from_pixel_rect`
- `sprite_atlas_uv_rect_rejects_zero_atlas_extent`
- `sprite_atlas_uv_rect_rejects_zero_pixel_extent`
- `sprite_atlas_validation_accepts_valid_asset`
- `sprite_atlas_asset_roundtrips_through_toml_and_remains_valid`
- `sprite_atlas_validation_rejects_zero_atlas_size`
- `sprite_atlas_validation_rejects_empty_entry_name`
- `sprite_atlas_validation_rejects_empty_entries`
- `sprite_atlas_validation_rejects_duplicate_entry_names`
- `sprite_atlas_validation_rejects_whitespace_variant_entry_names`
- `sprite_atlas_validation_rejects_out_of_bounds_pixel_rect`
- `sprite_atlas_validation_rejects_zero_entry_and_source_dimensions`
- `sprite_atlas_validation_rejects_source_smaller_than_pixel_rect`
- `sprite_atlas_validation_rejects_non_finite_and_out_of_range_uvs`
- `sprite_atlas_validation_rejects_invalid_uv_ordering`
- `sprite_atlas_validation_rejects_uv_rect_that_does_not_match_pixel_rect`

Fresh M2 evidence from 2026-05-18:

- `rustfmt --edition 2021 --check zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/assets/sprite_atlas/mod.rs zircon_runtime/src/asset/assets/sprite_atlas/layout.rs zircon_runtime/src/asset/assets/sprite_atlas/validation.rs` passed with no output.
- `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-shared\sprite-atlas-ui`.
- `cargo test -p zircon_runtime --lib sprite_atlas --locked --jobs 1 --message-format short --color never` passed `16` SpriteAtlas tests with `0` failures and `1569` filtered out.
- The broader `cargo test -p zircon_runtime --lib asset --locked --jobs 1 --message-format short --color never` command is not current acceptance evidence for this slice. One run reached tests and failed in unrelated asset-manager watcher/revision and render-framework tests that do not reference `sprite_atlas`; a later rerun failed before compilation because concurrent dirty manifest/lockfile changes required a `Cargo.lock` update while `--locked` was active.
