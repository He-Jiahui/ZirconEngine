---
related_code:
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/registration.rs
  - zircon_plugins/texture_importer/runtime/src/importers.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs
  - zircon_plugins/texture_importer/runtime/src/tests/mod.rs
  - zircon_plugins/texture_importer/runtime/src/tests/registration.rs
  - zircon_plugins/texture_importer/runtime/src/tests/image.rs
  - zircon_plugins/texture_importer/runtime/src/tests/psd.rs
  - zircon_plugins/texture_importer/runtime/src/tests/support.rs
  - docs/zircon_plugins/texture_importer/container.md
implementation_files:
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/registration.rs
  - zircon_plugins/texture_importer/runtime/src/importers.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs
  - zircon_plugins/texture_importer/runtime/src/tests/mod.rs
  - zircon_plugins/texture_importer/runtime/src/tests/registration.rs
  - zircon_plugins/texture_importer/runtime/src/tests/image.rs
  - zircon_plugins/texture_importer/runtime/src/tests/psd.rs
  - zircon_plugins/texture_importer/runtime/src/tests/support.rs
plan_sources:
  - .codex/plans/Asset Importer 插件化补齐计划.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
tests:
  - rustfmt --edition 2021 --check zircon_plugins/texture_importer/runtime/src/lib.rs zircon_plugins/texture_importer/runtime/src/importers.rs zircon_plugins/texture_importer/runtime/src/registration.rs zircon_plugins/texture_importer/runtime/src/tests/mod.rs zircon_plugins/texture_importer/runtime/src/tests/registration.rs zircon_plugins/texture_importer/runtime/src/tests/image.rs zircon_plugins/texture_importer/runtime/src/tests/psd.rs zircon_plugins/texture_importer/runtime/src/tests/support.rs
  - git diff --check -- zircon_plugins/texture_importer/runtime/src/lib.rs zircon_plugins/texture_importer/runtime/src/importers.rs zircon_plugins/texture_importer/runtime/src/registration.rs zircon_plugins/texture_importer/runtime/src/tests.rs zircon_plugins/texture_importer/runtime/src/tests
doc_type: module-detail
---

# Texture Importer Runtime

## Purpose

`zircon_plugin_texture_importer_runtime` registers the first-party texture asset importer package. It contributes runtime/editor-host plugin metadata and asset importers for decoded image sources, texture containers, and PSD files while keeping the loaded result in Zircon `TextureAsset` DTOs.

## Runtime Boundary

`src/lib.rs` is a structural facade. It owns the plugin constants and re-exports the stable public functions used by plugin catalog and asset import tests. New registration wiring belongs in `src/registration.rs`; new decode/import behavior belongs in `src/importers.rs`; container parsing stays under `src/container/` and is documented in `docs/zircon_plugins/texture_importer/container.md`.

`src/registration.rs` builds the package manifest, runtime module manifest, project selection, module descriptor, importer descriptors, and `RuntimeExtensionRegistry` contributions. It preserves the public importer IDs:

- `texture_importer.image`
- `texture_importer.container`
- `texture_importer.psd`
- `texture_importer.optional_native_container`

`src/importers.rs` owns the callable importer functions. `import_image(...)` delegates source image decoding to the runtime asset helper, `import_texture_container(...)` converts parsed DDS/KTX/KTX2/ASTC metadata into a container `TextureAssetDescriptor`, and `import_psd(...)` decodes flattened PSD RGBA data before applying common texture import settings.

## Tests

The crate-root tests now live under `src/tests/`. `tests/registration.rs` covers plugin manifest and registry contributions, `tests/image.rs` covers source-image decoding, format selection, aliases, texture descriptor settings, and stacked-array reinterpretation, `tests/psd.rs` covers PSD decode/settings behavior, and `tests/support.rs` owns tiny generated image/PSD fixtures.

## Validation

The 2026-06-04 boundary split is static-check complete: rustfmt passed over the structural facade, registration/importer modules, and test tree; `git diff --check` passed for touched Texture Importer Rust files with only the expected line-ending warning on the tracked root file. Focused Cargo validation was not started because active Cargo/rustc lanes were already running for other target directories.
