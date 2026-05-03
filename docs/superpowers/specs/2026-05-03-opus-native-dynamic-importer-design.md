---
related_code:
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/Cargo.toml
  - zircon_runtime/src/asset/importer/native.rs
  - docs/zircon_runtime/asset/importer.md
  - docs/zircon_plugins/asset_importers/runtime-skeletons.md
implementation_files:
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/Cargo.toml
  - docs/zircon_runtime/asset/importer.md
  - docs/zircon_plugins/asset_importers/runtime-skeletons.md
plan_sources:
  - user: 2026-05-03 Opus/libopus NativeDynamic importer gap
  - .codex/sessions/20260503-1940-importer-gap-design.md
tests:
  - cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --check
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --lib --locked --jobs 1
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1
  - git diff --check
doc_type: module-detail
---

# Opus NativeDynamic Importer Design

## Purpose

Close the Opus audio importer gap without expanding the sound mixer/runtime workstream or turning the existing audio importer package into a mixed backend umbrella. The first slice should make `.opus` a real plugin-owned importer contract with a NativeDynamic/libopus backend boundary, while preserving deterministic diagnostics when the native backend is not installed.

## Current Context

`zircon_plugins/audio_importer/runtime` already owns real WAV decoding plus Symphonia-backed MP3, OGG/Vorbis, FLAC, AIFF, and AIF decoding into `SoundAsset` interleaved f32 PCM. Its current Opus row is diagnostic-only and requires `runtime.asset.importer.native`.

`zircon_plugins/asset_importers/audio/runtime` is a lightweight family manifest package. It advertises WAV, codec formats, and Opus, but does not decode audio. It is useful for package catalog coverage but should not become the real libopus host.

An active sound mixer graph session is editing `zircon_plugins/sound/**` and `zircon_runtime::core::framework::sound`. This Opus importer slice must not touch those modules. Imported Opus output remains the existing neutral `SoundAsset` artifact consumed later by asset/resource paths.

## Chosen Approach

Add a split `zircon_plugins/opus_importer/runtime` package. The linked Rust package uses `LibraryEmbed` packaging and owns the concrete Opus importer manifest, module descriptor, runtime selection, and importer registration. It declares `.opus` as `AssetKind::Sound`, publishes the `runtime.asset.importer.audio.opus` slot capability, and keeps `runtime.asset.importer.native` as a descriptor requirement for the actual libopus execution path.

The existing `audio_importer` package keeps WAV and Symphonia-backed formats. Its diagnostic Opus row may remain as a lower-priority fallback until a broader package split removes Opus from that package. The new split Opus importer should register with a higher priority so an installed real backend wins selection for `.opus` files.

## Package Boundary

The new package has these responsibilities:

- Define `PLUGIN_ID = "opus_importer"` and `RUNTIME_CRATE_NAME = "zircon_plugin_opus_importer_runtime"`.
- Declare a runtime module such as `opus_importer.runtime` for `ClientRuntime` and `EditorHost`.
- Publish capabilities `runtime.plugin.opus_importer` and `runtime.asset.importer.audio.opus`; the Rust package does not publish `runtime.asset.importer.native` until a separate native backend is installed.
- Register one importer descriptor for `opus` with output kind `Sound` and a priority above the old diagnostic row.
- Register a linked Rust diagnostic shim for the missing-backend path while the descriptor requires the NativeDynamic importer capability.

The package must not introduce sound mixer, playback, output-device, DSP, timeline, or sound framework dependencies.

## NativeDynamic Contract

Opus decoding is modeled as a host-owned NativeDynamic importer command rather than a Rust trait object. The command name is `asset.import/opus_importer.opus`. The request envelope is the existing `ZRIMP001` NativeDynamic asset import request containing metadata JSON and raw source bytes. The response envelope is the existing `ZRIMO001` import response containing a neutral import DTO and diagnostics.

On success, the native libopus side returns a `SoundAsset`-equivalent neutral response with URI, sample rate, channel count, and interleaved f32 samples. On failure, it returns a stable parse or backend diagnostic. Host-side validation rejects malformed response magic, mismatched importer id, wrong output kind, reserved artifact bytes, zero sample rate, zero channel count, or invalid sample layout through the existing NativeDynamic validation path.

This design does not require implementing libopus C bindings inside the Rust package. The split package creates the importer slot and the command contract that a separately installed NativeDynamic backend can satisfy.

## Error Behavior

Missing native backend must stay deterministic. Importing `.opus` without the backend returns an importer error that states Opus requires a NativeDynamic/libopus backend. The importer must not fall through to the Symphonia codec row or generic data importers.

Bad native responses fail at the NativeDynamic envelope validation layer. Bad decoded audio metadata fails before a `SoundAsset` artifact is accepted. These failure records remain inspectable in project scans like other missing-backend or parse-error assets.

## Tests

Package-local tests should cover:

- Manifest declares Opus source extension, sound output kind, target modes, package id, and required capabilities.
- Runtime registration contributes the module and one Opus importer descriptor.
- `.opus` selection chooses the split Opus importer ahead of the old diagnostic row when both are installed in one registry.
- Missing backend produces a stable diagnostic instead of succeeding or falling through.

If the current `NativeAssetImporterHandler` API can be instantiated with a fixture command without touching unrelated runtime compile blockers, add a host-side response validation test for an Opus-shaped `SoundAsset` response. If not, keep this milestone package-local and document the fixture gap for a later NativeDynamic integration milestone.

## Documentation

Update `docs/zircon_runtime/asset/importer.md` and `docs/zircon_plugins/asset_importers/runtime-skeletons.md` in the same change. The docs should move Opus from a generic missing backend gap to a split NativeDynamic/libopus package contract, while still stating that decoding requires an installed native backend.

## Validation

Use the repository milestone cadence. The implementation stage may use scoped syntax checks only if needed. The milestone testing stage should run:

- `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --check`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --lib --locked --jobs 1`
- `cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1`
- `git diff --check`

If validation is blocked by unrelated dirty workspace or adjacent session changes, record the exact command and blocker rather than expanding this slice into those areas.
