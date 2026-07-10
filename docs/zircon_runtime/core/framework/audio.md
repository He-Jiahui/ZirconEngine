---
related_code:
  - zircon_runtime/src/core/framework/audio/mod.rs
  - zircon_runtime/src/core/framework/audio/channel_layout.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/channel_layout/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/audio/mod.rs
  - zircon_runtime/src/core/framework/audio/channel_layout.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
  - zircon_runtime/src/core/framework/sound/tests.rs
  - zircon_runtime/src/asset/tests/assets/sound.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features sound-contracts --locked --offline --jobs 1
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --offline --jobs 1
  - cargo check --manifest-path zircon_plugins/audio_importer/runtime/Cargo.toml --locked --offline --jobs 1
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml channel_layout --locked --offline --jobs 1
doc_type: module-detail
---

# Audio Format Contracts

## Purpose

`zircon_runtime::core::framework::audio` owns backend-neutral audio format data that must remain available to assets and importers even when the optional Sound service is absent. It exposes `AudioChannelLayout` and `AudioSpeakerChannel`; it does not own playback, mixer graphs, output devices, DSP, manager traits, or plugin lifecycle.

This owner prevents the foundational WAV/asset path from depending on `sound-contracts`. Runtime sound implementations consume the same format DTOs, so asset decoding and mixing agree on channel count, speaker order, and named-layout vocabulary without duplicating types.

## Ownership Boundary

- `audio/channel_layout.rs` is the only declaration owner for `AudioChannelLayout` and `AudioSpeakerChannel`.
- `asset/assets/sound.rs` decodes WAV channel masks into this neutral format.
- `zircon_plugins/audio_importer` consumes the neutral format without enabling `sound-contracts`.
- `core/framework/sound` and the Sound plugin consume the format but do not re-export it from the old Sound namespace.
- The removed `core/framework/sound/channel_layout.rs` path has no alias or compatibility re-export.

## Behavior

Named layouts cover mono, stereo, quad, 5.0, 5.1 rear, 5.1 side, 7.0, and 7.1 speaker orders. Unknown channel counts use `discrete_N`. Validation requires a positive channel count, exact canonical data for named layouts, speakerless count-derived discrete layouts, and unique speakers for custom layouts.

The Rust owner/name hard cut does not change serialized field names, layout string values, or speaker enum variant serialization. Existing asset data therefore retains its schema while compile-time ownership changes.

## Validation

Frameworks 03 guards assert the unique owner, absence of the old symbols/path, feature forwarding, declaration gates, and explicit Sound plugin dependencies. WSL nightly checks passed for Runtime `sound-contracts` alone, `target-server` without Sound, the Sound runtime/editor plugins, and the audio importer without `sound-contracts`; the Sound runtime full suite passes 368/368.
