---
related_code:
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/schema.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/assets/data.rs
  - zircon_runtime/src/asset/assets/texture.rs
  - zircon_runtime/src/asset/assets/shader.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/asset_importer_install.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
implementation_files:
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/schema.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
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
  - passed: cargo check -p zircon_runtime --lib --tests --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation
  - passed: cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --locked --jobs 1
  - passed: cargo test -p zircon_runtime --lib importer_reports_ui_toml_schema_migration --locked --jobs 1
  - passed: cargo test -p zircon_runtime --lib native_import_response --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation
  - passed: cargo test -p zircon_runtime --lib project_manager_records_ui_schema_migration_in_meta --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation
  - passed: cargo test -p zircon_runtime --lib project_manager_clears_stale_migration_meta_for_non_migrating_importer --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation
  - 2026-05-03: rustfmt --edition 2021 on the ProjectAssetManager/importer extension touched files (passed)
  - 2026-05-03: cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: rustfmt --edition 2021 on importer default/fixture, ProjectManager/ProjectAssetManager test fixture, plugin catalog/export repair files (passed)
  - 2026-05-03: rustfmt --edition 2021 --check on importer default/fixture and migrated runtime test files (passed)
  - 2026-05-03: git diff --check on importer default/fixture and migrated runtime test files (passed with LF-to-CRLF warnings only)
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings after plugin catalog/export repair)
  - 2026-05-03: cargo test -p zircon_runtime importer_default_reports_missing_first_wave_plugin_backend --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed, 1 test, with existing runtime warnings)
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed again after gating legacy first-wave helper modules as test-only; existing runtime warnings only)
  - 2026-05-03: cargo test -p zircon_runtime importer_decodes_obj_and_gltf_into_model_assets --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed, 1 fixture-backed test, with existing runtime warnings)
  - 2026-05-03: cargo test -p zircon_runtime runtime_extension_registry_installs_asset_importers_before_project_open --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (timed out after 10 minutes during Windows test build/link while other Cargo jobs were active; no Rust diagnostics returned)
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime typed_toml_importer_decodes_ui_layout_asset --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_data_runtime --lib --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation-2
  - passed: cargo build --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation-3-plugin
  - passed: cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1
  - passed: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --check
  - passed: rustfmt --edition 2021 --check zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - blocked: cargo fmt -p zircon_runtime --check (unrelated runtime formatting deltas in importer/project/plugin catalog files owned by adjacent sessions)
  - blocked: cargo check -p zircon_runtime --lib --tests --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation-3 (unrelated plugin optional-feature catalog/export-build-plan errors before the new NativeDynamic importer test can typecheck)
  - blocked: cargo test --manifest-path zircon_plugins/Cargo.toml --locked (unrelated sound/runtime trait drift)
  - inconclusive: .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 timed out before producing a final matrix result
  - passed: cargo test -p zircon_runtime project_manager_restores_ready_artifacts_from_meta_after_restart --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-unity-editor-final-check --message-format short --color never
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding Symphonia audio and Naga shader-family dependencies)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_audio_importer_runtime -p zircon_plugin_asset_importer_audio_runtime --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1 (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_audio_importer_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-audio-real-backend-lib --message-format short --color never (passed, 4 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_audio_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-audio-real-backend-lib --message-format short --color never (passed, 1 test)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_shader_runtime --check (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_shader_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-shader-real-backend --message-format short --color never (passed, 6 tests)
  - 2026-05-03: cargo test -p zircon_editor --lib sync_from_project_keeps_error_assets_without_artifacts_in_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap -- --format terse (passed)
  - 2026-05-03: cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap -- --format terse (passed, 932 passed, 1 ignored)
  - 2026-05-03: cargo test -p zircon_runtime --lib graphics::tests::project_render::directory_project_scene_renders_non_background_frame_with_gizmo_overlay --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap -- --format terse --exact (passed)
  - 2026-05-03: cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap -- --format terse (passed, 759 passed)
  - 2026-05-03: cargo test --workspace --locked --verbose --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap (passed)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding UI JSON importer dependencies)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_importer_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_ui_document_importer_runtime (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-texture-ui-backends --message-format short --color never (passed, 6 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_texture_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-texture-ui-backends --message-format short --color never (passed, 1 test)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-texture-ui-backends --message-format short --color never (passed, 5 tests)
  - 2026-05-03: cargo info stl_io, cargo info ply-rs-bw, cargo info psd (used for third-party backend selection)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding STL/PLY/PSD backend dependencies)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_asset_importer_texture_runtime (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1 (passed)
  - 2026-05-03: cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-third-party-backends-model --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-third-party-backends-model --message-format short --color never (passed, 4 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-third-party-backends-texture --message-format short --color never (passed, 7 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_texture_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-third-party-backends-texture-agg --message-format short --color never (passed, 1 test)
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/tests/plugin_extensions/asset_importer_install.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/asset/tests/project/manager.rs::project_manager_restores_ready_artifacts_from_meta_after_restart
doc_type: module-detail
---

# Asset Importer Pluginization

## Purpose

Asset import is now routed through `AssetImporterRegistry` instead of hard-coded extension branches in `ProjectManager`. The project scan owns traversal, metadata, artifact writing, failure records, and hot reimport state; importers own only source decoding and conversion to `ImportedAsset`.

This makes import formats a runtime extension point. The runtime still owns the neutral contracts, registry, project scan, artifact metadata, and diagnostics, but the first-wave stable format behavior is now expected to arrive through linked plugin importers. Package manifests can declare importer descriptors, and NativeDynamic plugins can provide external toolchain importers without sharing Rust trait objects or engine state across the ABI.

## Runtime Contract

`AssetImporterDescriptor` is the public routing record. It declares importer id, plugin id, priority, ordinary extensions, full suffixes, output kind, importer version, and required capabilities. Full suffixes are matched before extensions, so `main.ui.toml`, `level.scene.toml`, and `actor.prefab.toml` do not fall through to the plain `.toml` data importer.

`AssetImportContext` carries the source path, normalized asset URI, source bytes, and per-asset import settings from meta. `AssetImportOutcome` returns the imported asset, dependency URIs, optional schema migration details, and diagnostics. The registry validates duplicate importer ids and duplicate matchers at the same priority before a plugin contribution is accepted.

Plain `.toml` is a `DataAsset`. Typed `*.xxx.toml` requires a registered full-suffix importer; unknown typed TOML fails as an error resource instead of silently becoming a generic data file.

## Built-In Coverage

The production default importer registry installs real Rust paths for runtime-core formats only: plain TOML/JSON data, typed Zircon TOML assets such as material/font/model/physics material/scene/prefab/authoring navigation assets, animation `.zranim` contracts that have not yet moved fully to the animation plugin, and the remaining GLSL/SPIR-V shader paths. It no longer decodes the first-wave independent plugin formats directly.

UI `.ui.toml`, common image textures, WGSL, OBJ, glTF/GLB, and WAV now register diagnostic-only `zircon.plugin_required.*` descriptors by default. These descriptors preserve output kind, matcher, importer version, and capability metadata so scans produce stable error records when a plugin is disabled or missing, but they do not perform decoding in production runtime code. The real stable backends live in `ui_document_importer`, `texture_importer`, `shader_wgsl_importer`, `obj_importer`, `gltf_importer`, and `audio_importer`.

Runtime tests that intentionally exercise these first-wave formats install explicit fixture importers with the same package ids and higher priority as the split plugin crates. The fixtures still call test-only legacy runtime helper modules so the runtime test crate can validate artifact/project behavior without taking a dev-dependency on `zircon_plugins`; the production default path is diagnostic-only. Graphics project-render and M4 behavior-layer tests now use that explicit fixture path for PNG/WGSL/OBJ projects, so the tests prove render behavior with installed importer plugins instead of silently reintroducing production built-in decoders.

The `asset_importer.data` runtime plugin now registers real TOML/JSON/YAML/XML `DataAsset`
backends so project/plugin selection can move structured data loading out of the built-in fallback
path. The `asset_importer.model` family plugin now registers real STL and PLY mesh interchange
backends through `stl_io` and `ply-rs-bw`, emitting `ModelAsset` primitives with generated
virtual-geometry metadata. The split `texture_importer` package decodes common image formats to
RGBA8, parses DDS, KTX, KTX2, and ASTC container headers into `TexturePayload::Container`, and
decodes PSD files through the Rust `psd` crate into flattened RGBA8 textures. The split
`audio_importer` package decodes WAV directly and decodes MP3/OGG/Vorbis/FLAC/AIFF/AIF through
Symphonia into `SoundAsset` f32 PCM. Opus remains a NativeDynamic/libopus backend gap. The
`asset_importer.shader` family package now owns a real Naga path for WGSL validation plus
GLSL/vertex/fragment/compute and SPIR-V conversion into normalized WGSL `ShaderAsset` payloads. The
split `ui_document_importer` package imports typed `.ui.toml` and serialized `.ui.json` documents;
`.ui.json` uses the neutral UI DTO schema and rejects future schema versions instead of downgrading.

Heavy or toolchain-backed formats are registered as diagnostic importers until a plugin backend is installed. This includes FBX/DAE/3DS/DXF/USD-family model containers, cubemap/DXGI texture authoring formats, ZUI/UIDOC binary UI documents, Opus audio, and HLSL/CG/FX shader toolchains. First-wave plugin-required diagnostics follow the same stable error-record path when the corresponding split plugin is absent.

`TextureAsset` keeps the existing RGBA8 payload while reserving a container payload for future compressed formats. `ShaderAsset` records source language, original source, normalized WGSL source, entry points, and validation diagnostics. `DataAsset` preserves source text and canonical JSON for TOML, JSON, YAML, and XML data. XML is normalized into a stable element tree JSON object with element name, optional namespace, attributes, text, and children.

## Project Scan Behavior

`ProjectManager::scan_and_import` now processes every source file independently. A successful import writes an artifact, updates meta with source hash, import settings hash, importer id/version, artifact locator, schema migration details, and `preview_state = ready`, then publishes a ready `ResourceRecord`.

If an importer is missing, unsupported, malformed, or fails validation, the scan writes meta with the same source hash and importer identity when known, sets `preview_state = error`, and registers `ResourceState::Error` with diagnostics. The next source file continues importing. Runtime resource sync registers error records without trying to load a missing artifact.

Editor catalog sync mirrors the same contract. `DefaultEditorAssetManager::sync_from_project` keeps failed assets visible in the catalog, carries their diagnostics, and leaves direct-reference edges empty instead of calling `load_artifact_by_id` on records that have no artifact locator. This keeps missing-plugin and parse-error assets inspectable without blocking editor manager startup.

Meta documents are format version 3. Older meta files are upgraded in memory and saved with importer metadata fields, `artifact_locator`, and `config_hash`; future meta versions fail so the engine does not downgrade unknown schema.

Ready meta can now restore an already-imported artifact after editor restart without rerunning the importer. The restore path requires `preview_state = ready`, unchanged source hash, unchanged import settings hash, a matching importer id/version contract when the importer is present, and a readable artifact at `artifact_locator`. This keeps model, texture, material/data, scene, and UI document imports stable across restarts even when only the artifact store and meta are available. If the artifact is missing, the source/config changed, or the importer contract no longer matches, the project scan falls back to a normal import attempt and rewrites meta from the fresh result.

Successful imports now clear stale schema migration fields when the selected importer does not
return a migration report. Failed imports clear the same fields before recording error state, so an
old upgraded asset does not leave misleading schema metadata on a later non-migrating or failed
import.

The split `ui_document_importer` runtime package routes typed UI TOML through
`UiAssetLoader::load_toml_str_with_migration_report`. Older `.ui.toml` sources therefore produce
migrated runtime assets and propagate source schema version, target schema version, and a summary of
migration steps into the asset meta document when that plugin is installed. Without the plugin, the
default diagnostic importer records a missing-backend error instead of silently falling back to
plain TOML data.

The same package now routes `.ui.json` through `serde_json` into the neutral
`UiAssetDocument` DTO. The importer applies the same UI source schema version policy as TOML current
tree assets: supported older versions are bumped in memory to the current schema, current versions
are validated as-is, and future versions fail import. `.zui` and `.uidoc` remain reserved binary
document extensions until a codec backend is installed.

`ProjectAssetManager` keeps a host-owned importer registry for plugin contributions that arrive
before a project is open. `RuntimeExtensionRegistry::apply_asset_importers_to_project_asset_manager`
installs those handlers into that pending registry, and `open_project` applies the registry to the
fresh `ProjectManager` before `scan_and_import` runs. This gives linked plugin importers the same
first-scan authority as built-in importers without making `zircon_runtime` depend on
`zircon_plugins`.

The built-in `AssetModule` can also carry an `AssetImporterRegistry`. Runtime module load from
plugin registration reports merges active plugin and feature importer handlers into that registry
and constructs the project asset manager with those pending handlers already installed. This closes
the lifecycle gap between catalog selection and the first project scan for linked Rust plugins.

## Plugin Boundary

`RuntimeExtensionRegistry` now owns an `AssetImporterRegistry` alongside modules, managers, components, and render extensions. Rust plugins can register real importer handlers. Manifest-only and NativeDynamic declarations can register diagnostic descriptors until a backend is attached.

Applying importer extensions to `ProjectAssetManager` is intentionally host-side. The extension
registry does not open projects, inspect asset files, or write artifacts; it only transfers
capability-gated handlers into the asset manager. If a plugin registers after a project is already
open, the manager preflights the active project registry before accepting the handler into the
pending registry, then installs it into the current project so manual reimport and watcher-driven
reimport can use it immediately.

NativeDynamic importers use the `runtime.asset.importer.native` capability and the `asset.import/<importer_id>` command. The ABI payload is a `ZRIMP001` request envelope containing metadata JSON and raw source bytes. Native code returns a `ZRIMO001` response envelope with a neutral import DTO and diagnostics. The host validates status, importer id, output kind, and malformed buffers before writing artifacts.

The response validation path is factored separately from dynamic-library invocation, so envelope
decode tests can cover malformed magic, reserved artifact bytes, mismatched importer id, wrong
output kind, and diagnostic conversion without requiring a native DLL fixture.

The `native_dynamic_fixture` cdylib now also exposes
`asset.import/native_dynamic_fixture.data_json` in its command manifest. The fixture decodes the
same `ZRIMP001` envelope that production NativeDynamic importers receive, validates the requested
importer id, parses JSON source bytes, and returns a `ZRIMO001` response carrying a neutral
`DataAsset`. `zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs` contains the
host-side fixture test that loads the real DLL and routes it through `NativeAssetImporterHandler`;
the source is in place, while full runtime type/test validation is currently blocked by unrelated
plugin catalog compile errors from adjacent optional-feature work.
