---
related_code:
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/tests/assets/mod.rs
  - zircon_runtime/src/asset/tests/assets/sound.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
implementation_files:
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/tests/assets/sound.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime/src/asset/tests/assets/sound.rs::sound_asset_wav_parse_reports_typed_error_variants
  - zircon_runtime/src/asset/tests/assets/sound.rs::sound_asset_rejects_wav_extensible_unsupported_speaker_mask_bits
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_sound_asset_uses_typed_error
  - "2026-06-25 static: scoped rustfmt/static scans/docs-status-session anchors passed; Cargo deferred due active cargo/rustc lanes"
  - "2026-07-03 static: rustfmt --check passed for sound.rs/review guard; standalone review_f5_sound_asset_uses_typed_error passed 1/1; direct sound.rs unwrap/expect scan returned no matches; Cargo deferred due active cargo/rustc lanes"
doc_type: module-detail
---

# Sound Asset Records

`asset/assets/sound.rs` owns the runtime DTO for decoded audio assets. `SoundAsset` stores the asset URI, sample rate, channel count, interleaved speaker layout, and normalized `f32` PCM samples consumed by runtime audio/plugin paths.

## WAV Error Contract

Runtime 15 F5 sound asset typed errors (`runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred`) converted `SoundAsset::from_wav_bytes(...)` and its private WAV parser helpers from `Result<_, String>` to `SoundAssetResult<T>`.

Runtime 15 F5 sound asset panic-free read helpers (`runtime_15_sound_asset_panic_free_read_helpers_static_passed_cargo_deferred`) removed the remaining WAV parser `try_into().unwrap()` conversions. Chunk-size reads and PCM/IEEE sample conversions now go through `read_fixed_bytes<const N>()` plus the typed little-endian readers, so short or overflowing reads stay inside `SoundAssetError::HeaderReadOverflow` instead of relying on infallible conversion assumptions.

`SoundAssetError` models the WAV failure families at the asset boundary:

- RIFF/WAVE container shape errors such as `WavFileTooSmall`, `MissingRiffWaveHeader`, missing `fmt`/`data` chunks, and chunk/header read overflows.
- Format declaration errors such as zero channels, zero sample rate, unsupported bits per sample, unsupported PCM/IEEE float combinations, and block-align mismatches.
- WAVE_FORMAT_EXTENSIBLE errors such as invalid valid-bits declarations, unsupported subformat GUIDs, channel-mask count mismatches, and unsupported speaker-mask bits.
- Data alignment errors for incomplete frames or sample-width boundaries.

The public `asset/assets/mod.rs` and `asset/mod.rs` facades export `SoundAssetError` and `SoundAssetResult`, so callers and tests can match the failure kind directly instead of parsing display text.

## Import Boundary

`asset/importer/ingest/import_sound.rs` still reports `AssetImportError::Parse` for the importer-facing diagnostic surface. That boundary formats the typed `SoundAssetError` display text with the source path, while the asset DTO and its parser retain structured error variants internally.

## Regression Coverage

`sound_asset_wav_parse_reports_typed_error_variants` covers malformed and unsupported WAV input with direct `SoundAssetError` matches. `sound_asset_rejects_wav_extensible_unsupported_speaker_mask_bits` locks the WAVE_FORMAT_EXTENSIBLE speaker-mask failure as `SoundAssetError::UnsupportedSpeakerMaskBits`.

`review_f5_sound_asset_uses_typed_error` rejects reintroducing `Result<_, String>`, `Err(format!(...))`, `.to_string()`, `.unwrap()`, or `.expect(` inside `asset/assets/sound.rs`; it also locks facade exports, importer diagnostic formatting, this document, and the Runtime 15/status docs anchors.
