---
related_code:
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/schema.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/assets/data.rs
  - zircon_runtime/src/asset/assets/texture.rs
  - zircon_runtime/src/asset/assets/shader.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
implementation_files:
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/schema.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
plan_sources:
  - user: 2026-05-02 Asset Importer 插件化补齐计划
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - previously passed: cargo check -p zircon_runtime --locked
  - previously passed: cargo test -p zircon_runtime --locked asset
  - previously passed: cargo test -p zircon_runtime --locked plugin_extensions
  - previously passed: cargo test -p zircon_runtime --locked native_import
  - previously passed: cargo test -p zircon_runtime --locked project_manager_records_failed_imports_and_continues_scanning
  - previously passed: cargo test --manifest-path zircon_plugins/Cargo.toml --locked -j 1 -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_shader_runtime -p zircon_plugin_asset_importer_data_runtime
  - fresh-rerun blocked: cargo test -p zircon_runtime --locked asset (unrelated graphics/VG ViewportCameraSnapshot move error)
  - blocked: cargo test --manifest-path zircon_plugins/Cargo.toml --locked (unrelated sound/runtime trait drift)
  - inconclusive: .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 timed out before producing a final matrix result
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
doc_type: module-detail
---

# Asset Importer Pluginization

## Purpose

Asset import is now routed through `AssetImporterRegistry` instead of hard-coded extension branches in `ProjectManager`. The project scan owns traversal, metadata, artifact writing, failure records, and hot reimport state; importers own only source decoding and conversion to `ImportedAsset`.

This makes import formats a runtime extension point. Built-in Rust importers cover stable first-wave formats, package manifests can declare importer descriptors, and NativeDynamic plugins can provide external toolchain importers without sharing Rust trait objects or engine state across the ABI.

## Runtime Contract

`AssetImporterDescriptor` is the public routing record. It declares importer id, plugin id, priority, ordinary extensions, full suffixes, output kind, importer version, and required capabilities. Full suffixes are matched before extensions, so `main.ui.toml`, `level.scene.toml`, and `actor.prefab.toml` do not fall through to the plain `.toml` data importer.

`AssetImportContext` carries the source path, normalized asset URI, source bytes, and per-asset import settings from meta. `AssetImportOutcome` returns the imported asset, dependency URIs, optional schema migration details, and diagnostics. The registry validates duplicate importer ids and duplicate matchers at the same priority before a plugin contribution is accepted.

Plain `.toml` is a `DataAsset`. Typed `*.xxx.toml` requires a registered full-suffix importer; unknown typed TOML fails as an error resource instead of silently becoming a generic data file.

## Built-In Coverage

The built-in importer registry installs real Rust paths for TOML/JSON data, typed Zircon TOML assets, PNG/JPEG and other `image` crate texture formats, WGSL validation, GLSL/SPIR-V to WGSL through Naga, OBJ, glTF/GLB, and WAV.

Heavy or toolchain-backed formats are registered as diagnostic importers until a plugin backend is installed. This includes FBX and other model containers, PSD/DDS/KTX/KTX2/ASTC/cubemap/DXGI textures, MP3/OGG/FLAC/AIFF/Opus audio codecs, and HLSL/CG/FX shader toolchains. These failures are stable asset records, not scan-stopping exceptions.

`TextureAsset` keeps the existing RGBA8 payload while reserving a container payload for future compressed formats. `ShaderAsset` records source language, original source, normalized WGSL source, entry points, and validation diagnostics. `DataAsset` preserves source text and canonical JSON for TOML/JSON data.

## Project Scan Behavior

`ProjectManager::scan_and_import` now processes every source file independently. A successful import writes an artifact, updates meta with source hash, importer id/version, schema migration details, and `preview_state = ready`, then publishes a ready `ResourceRecord`.

If an importer is missing, unsupported, malformed, or fails validation, the scan writes meta with the same source hash and importer identity when known, sets `preview_state = error`, and registers `ResourceState::Error` with diagnostics. The next source file continues importing. Runtime resource sync registers error records without trying to load a missing artifact.

Meta documents are format version 2. Older meta files are upgraded in memory and saved with importer metadata fields; future meta versions fail so the engine does not downgrade unknown schema.

## Plugin Boundary

`RuntimeExtensionRegistry` now owns an `AssetImporterRegistry` alongside modules, managers, components, and render extensions. Rust plugins can register real importer handlers. Manifest-only and NativeDynamic declarations can register diagnostic descriptors until a backend is attached.

NativeDynamic importers use the `runtime.asset.importer.native` capability and the `asset.import/<importer_id>` command. The ABI payload is a `ZRIMP001` request envelope containing metadata JSON and raw source bytes. Native code returns a `ZRIMO001` response envelope with a neutral import DTO and diagnostics. The host validates status, importer id, output kind, and malformed buffers before writing artifacts.
