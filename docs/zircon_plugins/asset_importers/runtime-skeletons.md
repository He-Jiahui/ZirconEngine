---
related_code:
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
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
implementation_files:
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
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
plan_sources:
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/多插件组合可选功能规则设计.md
  - active session: .codex/sessions/20260502-1935-independent-plugin-implementation.md
tests:
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1 --locked --offline (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_obj_importer_runtime -p zircon_plugin_texture_importer_runtime -p zircon_plugin_audio_importer_runtime -p zircon_plugin_shader_wgsl_importer_runtime -p zircon_plugin_ui_document_importer_runtime --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-importers --message-format short --color never (timed out after 10 minutes without Rust diagnostics while other Cargo jobs were active)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-importers --message-format short --color never (timed out after 5 minutes without Rust diagnostics while other Cargo jobs were active)
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
- `zircon_plugin_shader_wgsl_importer_runtime`
- `zircon_plugin_ui_document_importer_runtime`

These packages carry `plugin.toml` manifests, runtime workspace members, runtime module manifest
entries, capability-gated `AssetImporterDescriptor` rows, `ProjectPluginSelection` helpers, and
`RuntimePluginRegistrationReport` smoke surfaces. They are the package-selection and capability
surface that export planning and plugin enablement should target for glTF, OBJ, texture, audio,
WGSL shader, and UI document import.

The older family packages still exist as declaration aggregators:

- `zircon_plugin_asset_importer_model_runtime`
- `zircon_plugin_asset_importer_texture_runtime`
- `zircon_plugin_asset_importer_audio_runtime`
- `zircon_plugin_asset_importer_shader_runtime`
- `zircon_plugin_asset_importer_data_runtime`

The family crates remain loadable during migration so existing workspace and test callers do not
lose their importer declarations while the split package ids are adopted.

## Public Surface

Each importer crate exports stable marker constants:

- `PLUGIN_ID`
- `RUNTIME_CRATE_NAME`
- `MODULE_NAME`
- importer capability constants

Each root-level crate also exposes:

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

These crates are still importer registration packages. They do not yet decode source files directly
or own project scan/artifact state. Real first-wave decoding lives in
`zircon_runtime::asset::AssetImporter` for stable Rust-backed formats. Native/toolchain formats
remain descriptor-driven until a backend plugin is installed.

The new root-level descriptors use higher priority than the family aggregators where they overlap.
That lets the split packages become the preferred declaration path while keeping the old family
crates loadable during the migration window.

## Split Packages

`gltf_importer` declares `gltf` and `glb` model inputs with
`runtime.asset.importer.model.gltf`.

`obj_importer` declares Wavefront `obj` model inputs with
`runtime.asset.importer.model.obj`.

`texture_importer` declares common image formats as the primary image importer and reserves a lower
priority optional-container row for PSD, DDS, KTX/KTX2, ASTC, cubemap, and DXGI-style payloads that
still need a native or codec backend.

`audio_importer` declares WAV as the first-wave audio path and keeps compressed codecs such as MP3,
OGG, FLAC, AIFF, and Opus behind the native importer capability.

`shader_wgsl_importer` declares the WGSL shader importer package separately from the older shader
family, so WGSL can be enabled without pulling GLSL/SPIR-V/HLSL declaration rows.

`ui_document_importer` declares typed `.ui.toml` documents plus serialized `.ui.json`, `.zui`, and
`.uidoc` documents. Its descriptors output `UiLayout` and advertise `UiWidget`/`UiStyle` as
additional output kinds.

## Legacy Families

`model` declares glTF/GLB, OBJ, and optional model containers such as FBX, DAE, 3DS, PLY, STL, and
USD-family extensions.

`texture` declares standard image inputs and optional container/compressed texture formats including
PSD, DDS, KTX/KTX2, ASTC, cubemap, and DXGI-style payloads.

`audio` declares WAV plus optional codec-backed formats such as MP3, OGG, FLAC, AIFF, and Opus.

`shader` declares WGSL, Naga-backed GLSL/SPIR-V, and optional NativeDynamic shader toolchains for
HLSL/CG/FX.

`data` declares TOML, JSON, YAML, YML, and XML data importers. Runtime built-ins currently decode
TOML/JSON and keep YAML/XML as plugin-backed capability declarations.
