---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/runtime/Cargo.toml
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/Cargo.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/runtime/Cargo.toml
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/Cargo.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
plan_sources:
  - user: 2026-05-03 Opus/libopus NativeDynamic importer gap
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/多插件组合可选功能规则设计.md
  - docs/superpowers/specs/2026-05-03-opus-native-dynamic-importer-design.md
  - docs/superpowers/plans/2026-05-03-opus-native-dynamic-importer.md
  - active session: .codex/sessions/20260502-1935-independent-plugin-implementation.md
tests:
  - 2026-05-03 review correction: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --check (passed)
  - 2026-05-03 review correction: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --lib --locked --jobs 1 (passed, 4 tests)
  - 2026-05-03 review correction: cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (passed)
  - 2026-05-03 review correction: git diff --check (passed with CRLF normalization warnings only)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins/Cargo.toml --offline (passed after adding direct importer backend dependencies)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime (passed)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1 --locked --offline (passed)
  - 2026-05-03: rustfmt --check zircon_runtime/src/asset/mod.rs (passed)
  - 2026-05-03: cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing warnings)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --tests --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers-tests --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_audio_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers-tests --message-format short --color never (passed, 3 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers-tests --message-format short --color never (passed, 3 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_shader_wgsl_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers-tests --message-format short --color never (passed, 3 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers-tests --message-format short --color never (passed, 3 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime typed_toml_importer_decodes_ui_layout_asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-gap-continuation --message-format short --color never (passed)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding `serde_yaml` to the data importer plugin)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1 (passed after the data importer dependency update)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_data_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-gap-continuation-2 --message-format short --color never (passed, 5 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers-tests --message-format short --color never (passed, 3 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:/cargo-targets/zircon-independent-plugin-importers-tests --message-format short --color never (previously passed, 2 registration tests before the decode fixture was added)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-importers-tests --message-format short --color never (timed out after 10 minutes during Windows test build/link after the glTF decode fixture was added; no Rust diagnostics returned)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-importers-tests --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-rendering-plugin-runtime-check --message-format short --color never (passed, 3 tests plus doctests, after adding the missing `toml` dev-dependency for the decode fixture)
  - 2026-05-03: rustfmt --edition 2021 on the ProjectAssetManager/importer extension touched files (passed)
  - 2026-05-03: cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings)
  - 2026-05-03: rustfmt --edition 2021 --check zircon_runtime/src/asset/importer/ingest/asset_importer.rs zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs zircon_runtime/src/asset/project/manager/importer_access.rs zircon_runtime/src/asset/tests/assets/importer.rs zircon_runtime/src/asset/tests/project/manager.rs zircon_runtime/src/asset/tests/pipeline/manager.rs zircon_runtime/src/asset/tests/assets/ui.rs (passed)
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed with existing runtime warnings after the production default first-wave importers were changed to plugin-required diagnostics)
  - 2026-05-03: cargo test -p zircon_runtime importer_default_reports_missing_first_wave_plugin_backend --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed, 1 test, with existing runtime warnings)
  - 2026-05-03: cargo test -p zircon_runtime importer_decodes_obj_and_gltf_into_model_assets --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed, 1 fixture-backed test, with existing runtime warnings)
  - 2026-05-03: cargo test -p zircon_runtime runtime_extension_registry_installs_asset_importers_before_project_open --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (timed out after 10 minutes during Windows test build/link while other Cargo jobs were active; no Rust diagnostics returned)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-importers --message-format short --color never (timed out after 10 minutes without Rust diagnostics while other Cargo jobs were active)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-importers --message-format short --color never (timed out after 5 minutes without Rust diagnostics while other Cargo jobs were active)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding Symphonia audio and Naga shader-family dependencies)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_audio_importer_runtime -p zircon_plugin_asset_importer_audio_runtime --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1 (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_audio_importer_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-audio-real-backend-lib --message-format short --color never (passed, 4 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_audio_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-audio-real-backend-lib --message-format short --color never (passed, 1 test)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_shader_runtime --check (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_shader_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-shader-real-backend --message-format short --color never (passed, 6 tests)
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
  - 2026-05-03: cargo info dxf (used for DXF backend selection)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding the DXF backend dependency)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-dxf-backend --message-format short --color never (passed, 5 tests)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-dxf-backend --message-format short --color never (passed again after extracting DXF into `src/cad.rs`, 5 tests)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1 (passed)
  - 2026-05-03: git diff --check (passed with LF-to-CRLF warnings only)
  - 2026-05-03: cargo info bincode (used for UI binary document backend selection)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding the UI binary document backend dependency)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-ui-binary-backend --message-format short --color never (passed, 8 tests)
  - previously passed: cargo test --manifest-path zircon_plugins/Cargo.toml --locked -j 1 -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_shader_runtime -p zircon_plugin_asset_importer_data_runtime
  - fresh runtime rerun blocked: cargo test -p zircon_runtime --locked asset (unrelated graphics/VG ViewportCameraSnapshot move error)
  - blocked: cargo test --manifest-path zircon_plugins/Cargo.toml --locked (unrelated sound/runtime trait drift after the earlier virtual_geometry visibility blocker moved)
doc_type: module-detail
---

# Asset Importer Runtime Packages

## Purpose

The plugin workspace now contains two importer layers.

The new root-level packages match the independent plugin plan's finer package ids:

- `zircon_plugin_gltf_importer_runtime`
- `zircon_plugin_obj_importer_runtime`
- `zircon_plugin_texture_importer_runtime`
- `zircon_plugin_audio_importer_runtime`
- `zircon_plugin_opus_importer_runtime`
- `zircon_plugin_shader_wgsl_importer_runtime`
- `zircon_plugin_ui_document_importer_runtime`

These packages carry `plugin.toml` manifests, runtime workspace members, runtime module manifest
entries, capability-gated `AssetImporterDescriptor` rows, `ProjectPluginSelection` helpers, and
`RuntimePluginRegistrationReport` smoke surfaces. They are the package-selection and capability
surface that export planning and plugin enablement should target for glTF, OBJ, texture, audio,
WGSL shader, and UI document import.

The older family packages still exist as migration packages:

- `zircon_plugin_asset_importer_model_runtime`
- `zircon_plugin_asset_importer_texture_runtime`
- `zircon_plugin_asset_importer_audio_runtime`
- `zircon_plugin_asset_importer_shader_runtime`
- `zircon_plugin_asset_importer_data_runtime`

Most family crates remain declaration aggregators during migration so existing workspace and test
callers do not lose their importer declarations while the split package ids are adopted. The model,
data, and shader family crates are the current exceptions: model owns real STL/PLY mesh interchange
backends plus a DXF CAD mesh-surface backend, data owns real TOML/JSON/YAML/XML backends, and shader
owns the real Naga-backed WGSL/GLSL/SPIR-V path.

## Public Surface

Each importer crate exports stable marker constants:

- `PLUGIN_ID`
- `RUNTIME_CRATE_NAME`
- `MODULE_NAME`
- importer capability constants

Each runtime-backed importer crate exposes:

- `runtime_capabilities()`
- `supported_targets()`
- `supported_platforms()`
- `module_descriptor()`
- `asset_importer_descriptors()`
- `runtime_module_manifest()`
- `package_manifest()`
- `runtime_selection()`
- `plugin_registration()`
- `register_runtime_extensions(...)`

The package manifest records the runtime crate, editor/client targets, platform support, package
capabilities, and the `AssetImporterDescriptor` rows for that importer package.

## Boundaries

These crates now own the first-wave stable Rust-backed importer functions for their primary formats:
glTF/GLB, OBJ, STL/PLY mesh interchange files, DXF mesh-surface CAD files, common image files,
DDS/KTX/KTX2/ASTC texture containers, PSD flattened image data, WAV plus Symphonia-backed
MP3/OGG/Vorbis/FLAC/AIFF/AIF, WGSL, Naga-backed GLSL/SPIR-V, typed `.ui.toml` UI documents,
serialized `.ui.json` UI documents, binary `.zui`/`.uidoc` UI documents, and structured data
documents. The runtime asset contract publicly exposes the neutral `MeshVertex` DTO so model
importer plugins can produce `ModelAsset` primitives without depending on asset pipeline internals.
They still do not own project scan/artifact state; that authority remains in the runtime asset
manager. Optional native/toolchain container formats remain descriptor-driven until a backend plugin
is installed.

The new root-level descriptors use higher priority than the family aggregators where they overlap.
That lets the split packages become the preferred declaration path while keeping the old family
crates loadable during the migration window.

## Host Integration

`RuntimeExtensionRegistry::apply_asset_importers_to_project_asset_manager` is the linked-plugin
handoff point for these packages. The registry transfers real `FunctionAssetImporter` handlers into
`ProjectAssetManager` before a project is opened; the asset manager then applies those pending
handlers to the fresh `ProjectManager` before the first `scan_and_import`. This keeps package
selection and capability gates in the plugin layer while leaving project traversal, artifact writes,
resource records, and dependency invalidation in the runtime asset manager.

Runtime module loading also carries these handlers. When a runtime is built from plugin
registration reports, active package and feature importer handlers are merged into the `AssetModule`
descriptor; activating `AssetModule` constructs `ProjectAssetManager` with the pending registry
already populated.

The pre-open install path is now the authority for first-wave stable format imports. Production
`AssetImporter::default()` keeps only diagnostic `zircon.plugin_required.*` rows for glTF/GLB, OBJ,
common images, WAV, WGSL, and typed UI TOML. Once the host selects the split packages, first-scan
imports come from the plugin registry instead of relying on runtime-built stable backends.

Runtime crate tests install explicit first-wave fixture handlers when they need to exercise project
scan, artifact, meta, or watcher behavior for these formats. Those fixtures use the split package
ids and priority shape but still call the old runtime helper functions so runtime tests stay
self-contained while production behavior remains plugin-owned.

## Split Packages

`gltf_importer` declares `gltf` and `glb` model inputs with
`runtime.asset.importer.model.gltf` and registers a function backend that parses glTF buffers,
preserves skinning channels, and emits `ModelAsset` primitives. Its runtime tests now include a
minimal triangle glTF fixture that exercises the real importer path and validates primitive indices
plus cooked virtual-geometry source metadata; the fixture has a passing locked package test and the
crate declares `toml` as a dev-dependency for the test-side `AssetImportContext` metadata table.

`obj_importer` declares Wavefront `obj` model inputs with
`runtime.asset.importer.model.obj` and registers a function backend that triangulates OBJ meshes and
emits cooked `ModelAsset` primitives.

`texture_importer` declares common image formats as the primary image importer and now has real
rows for DDS, KTX, KTX2, ASTC, and PSD. Common images decode to `TextureAsset` RGBA8 payloads;
container files parse width, height, mip count, array layers, and format metadata, then preserve the
source bytes in `TexturePayload::Container`. PSD files decode through the Rust `psd` crate into a
flattened RGBA8 texture. Cubemap authoring files and loose DXGI-style payloads remain NativeDynamic
diagnostics.

`audio_importer` declares WAV plus a codec row for MP3, OGG/Vorbis, FLAC, AIFF, and AIF. WAV keeps
the direct Rust path, while the codec row decodes through Symphonia and emits interleaved f32
`SoundAsset` PCM samples.

`opus_importer` declares the `.opus` audio importer as a split package. It owns the `SoundAsset`
importer descriptor and NativeDynamic/libopus command contract, registers ahead of the old
audio-family diagnostic row, and reports a stable missing-backend diagnostic until a native libopus
backend is installed.

`shader_wgsl_importer` declares the WGSL shader importer package separately from the older shader
family, so WGSL can be enabled without pulling GLSL/SPIR-V/HLSL declaration rows. It validates WGSL
with Naga in the plugin crate and emits `ShaderAsset` entry points.

`ui_document_importer` declares typed `.ui.toml`, serialized `.ui.json`, and binary `.zui`/`.uidoc`
documents. Its descriptors output `UiLayout` and advertise `UiWidget`/`UiStyle` as additional output
kinds. The typed TOML row decodes in the plugin crate and preserves migration metadata; `.ui.json`
decodes through `serde_json` into the neutral `UiAssetDocument` DTO and applies the same source
schema version policy. `.zui` and `.uidoc` now use a `ZRUI001` container header plus a versioned
bincode payload for the same DTO, then reuse the JSON/TOML schema migration policy. Invalid magic,
unsupported container versions, malformed payloads, and future UI schema versions produce stable
import failures instead of falling back to generic data.

## Legacy Families

`model` declares glTF/GLB, OBJ, real STL/PLY mesh interchange backends, a real DXF CAD backend, and
optional model containers such as FBX, DAE, 3DS, and USD-family extensions. STL is parsed through
`stl_io`, PLY is parsed through `ply-rs-bw`, and DXF is parsed through the `dxf` crate. The DXF path
imports `3DFACE`, `SOLID`, `TRACE`, and `POLYLINE` polyface mesh surfaces into `ModelAsset`
primitives with generated virtual-geometry metadata; linework, curves, blocks, and solid-kernel BREP
payloads remain outside this backend and produce no mesh output. DXF conversion lives in
`asset_importers/model/runtime/src/cad.rs` so the package root stays focused on plugin descriptors,
manifest helpers, and registration wiring.

`texture` declares standard image inputs, real container/compressed texture declarations for DDS,
KTX/KTX2, ASTC, and PSD, plus optional NativeDynamic declarations for cubemap and DXGI-style
payloads.

`audio` declares WAV plus optional codec-backed formats such as MP3, OGG, FLAC, and AIFF; Opus is
now represented by the split `opus_importer` package.

`shader` declares and registers real WGSL validation plus Naga-backed GLSL/SPIR-V conversion into
normalized WGSL `ShaderAsset` payloads. Optional NativeDynamic shader toolchains for HLSL/CG/FX
remain diagnostic until a toolchain backend is installed.

`data` declares and registers runtime backends for TOML, JSON, YAML, YML, and XML data importers.
The plugin emits `DataAsset` values with source text plus canonical JSON. YAML is decoded through
`serde_yaml`; XML is decoded through `roxmltree` into a stable neutral element tree object so XML can
participate in the same artifact path without pretending to be native JSON syntax.

The split `ui_document_importer` runtime package also participates in this importer family at the
package layer. Its `.ui.toml` path now preserves the UI schema migration report returned by
`UiAssetLoader`, matching the built-in runtime importer so package-backed and built-in scans write
the same source/target schema metadata.
