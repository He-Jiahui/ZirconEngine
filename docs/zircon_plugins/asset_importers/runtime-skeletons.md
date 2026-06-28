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
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/runtime/Cargo.toml
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/dist/Cargo.toml
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/Cargo.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/dist/Cargo.toml
  - zircon_plugins/obj_importer/dist/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/dist/Cargo.toml
  - zircon_plugins/texture_importer/dist/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/dist/Cargo.toml
  - zircon_plugins/audio_importer/dist/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/mesh_importer.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/mod.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/registration.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/importers.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/support.rs
  - zircon_plugins/asset_importers/model/plugin.toml
  - zircon_plugins/asset_importers/model/dist/Cargo.toml
  - zircon_plugins/asset_importers/model/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/plugin.toml
  - zircon_plugins/asset_importers/texture/dist/Cargo.toml
  - zircon_plugins/asset_importers/texture/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/plugin.toml
  - zircon_plugins/asset_importers/audio/dist/Cargo.toml
  - zircon_plugins/asset_importers/audio/dist/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/plugin.toml
  - zircon_plugins/asset_importers/shader/dist/Cargo.toml
  - zircon_plugins/asset_importers/shader/dist/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/data/plugin.toml
  - zircon_plugins/asset_importers/data/dist/Cargo.toml
  - zircon_plugins/asset_importers/data/dist/src/lib.rs
  - tools/plugin_structure_audits/registration.py
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
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
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/dist/Cargo.toml
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/Cargo.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/dist/Cargo.toml
  - zircon_plugins/obj_importer/dist/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/dist/Cargo.toml
  - zircon_plugins/texture_importer/dist/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/dist/Cargo.toml
  - zircon_plugins/audio_importer/dist/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/mesh_importer.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/mod.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/registration.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/importers.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/support.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/plugin.toml
  - zircon_plugins/asset_importers/texture/dist/Cargo.toml
  - zircon_plugins/asset_importers/texture/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/plugin.toml
  - zircon_plugins/asset_importers/audio/dist/Cargo.toml
  - zircon_plugins/asset_importers/audio/dist/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/plugin.toml
  - zircon_plugins/asset_importers/shader/dist/Cargo.toml
  - zircon_plugins/asset_importers/shader/dist/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/data/plugin.toml
  - zircon_plugins/asset_importers/data/dist/Cargo.toml
  - zircon_plugins/asset_importers/data/dist/src/lib.rs
  - tools/plugin_structure_audits/registration.py
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
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
  - 2026-06-04: Model Asset Importer runtime root/test split static checks: rustfmt check, diff hygiene, trailing-whitespace scan, and conflict-marker scan over `model/runtime/src/{lib.rs,registration.rs,mesh_importer.rs,cad.rs,tests/*}` plus this doc/session note passed; focused Cargo validation is pending while other Cargo/rustc lanes are active.
  - 2026-06-23: Plugins 12 M3/T1 importer family registration cutover: `asset_importers/{data,model,shader}/runtime/src/plugin.rs` owns trait-backed RuntimePlugin entries; `asset_importers/*` public `pub fn register(...)` scan is empty; `tools/audit_plugin_structure.py --json` reports `registration_conformance.m3_t1_gate_status=family-single-entry-clean`; `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_data_runtime -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_shader_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never` passed with existing runtime warnings.
  - 2026-06-24: Plugins 13 M5/T1 asset_importer.data dist rollout: `asset_importers/data/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `asset_importer.data.dist`, and TOML/JSON/YAML/XML capabilities. Scoped rustfmt passed; dist check/test passed; `data_asset_importer_package_manifest_declares_dist_contract` passed 1/1; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=17`, `dist_build_matrix_count=17`, and zero dist boundary/distribution violations; real `plugin build asset_importer.data --form dist --offline` passed with `fatal=false`.
  - 2026-06-24: Plugins 13 M5/T1 asset_importer.model dist rollout: `asset_importers/model/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `asset_importer.model.dist`, and root/mesh/CAD capabilities. Scoped rustfmt passed; dist locked check/test passed; `model_asset_importer_package_manifest_declares_dist_contract` first timed out after 604s during compile without a test result, then passed 1/1 on rerun; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=18`, `dist_build_matrix_count=18`, and zero dist boundary/distribution violations; real `plugin build asset_importer.model --form dist --offline` passed with `fatal=false`.
  - 2026-06-24: Plugins 13 M5/T1 asset_importer.shader dist rollout: `asset_importers/shader/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `asset_importer.shader.dist`, and root/WGSL/Naga capabilities. Scoped rustfmt passed; dist locked check/test passed; `shader_asset_importer_package_manifest_declares_dist_contract` first timed out after 904s during compile without a test result and residual cargo/rustc processes were stopped, then passed 1/1 on rerun with `CARGO_PROFILE_DEV_DEBUG=0`; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=19`, `dist_build_matrix_count=19`, and zero dist boundary/distribution violations; real `plugin build asset_importer.shader --form dist --offline` passed with `fatal=false`.
  - 2026-06-24: Plugins 13 M5/T1 asset_importer.audio dist rollout: `asset_importers/audio/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `asset_importer.audio.dist`, and root/codec capabilities. Scoped rustfmt passed; dist locked check/test passed; `audio_asset_importer_package_manifest_declares_dist_contract` first timed out after 604s during compile without a test result and residual cargo/rustc processes were stopped, then passed 1/1 on rerun with `CARGO_PROFILE_DEV_DEBUG=0`; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=20`, `dist_build_matrix_count=20`, and zero dist boundary/distribution violations; real `plugin build asset_importer.audio --form dist --offline` passed with `fatal=false`.
  - 2026-06-24: Plugins 13 M5/T1 asset_importer.texture dist rollout: `asset_importers/texture/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `asset_importer.texture.dist`, and root/container/PSD capabilities. Scoped rustfmt passed; dist locked check/test passed; `texture_asset_importer_package_manifest_declares_dist_contract` first timed out after 1204s during parallel compile without a test result, then passed 1/1 on rerun with `CARGO_PROFILE_DEV_DEBUG=0`; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=21`, `dist_build_matrix_count=21`, and zero dist boundary/distribution violations; real `plugin build asset_importer.texture --form dist --offline` passed with `fatal=false`.
  - 2026-06-24: Plugins 13 M5/T1 audio_importer split importer dist rollout: `audio_importer/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `audio_importer.dist`, and root/WAV/codec capabilities. Scoped rustfmt passed; dist test passed; `package_manifest_declares_audio_importer_dist_contract` first cold compile produced no target test result and left same target-dir cargo processes, then passed 1/1 after clearing overlap and rerunning with `CARGO_PROFILE_DEV_DEBUG=0`; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=22`, `dist_build_matrix_count=22`, and zero dist boundary/distribution violations; real `plugin build audio_importer --form dist --offline` passed with `fatal=false`.
  - 2026-06-24: Plugins 13 M5/T1 obj_importer split importer dist rollout: `obj_importer/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `obj_importer.dist`, and root/OBJ model importer capabilities. Scoped rustfmt passed; dist test passed; `package_manifest_declares_obj_importer_dist_contract` first cold compile attempts produced no target test result, then passed 1/1 after the residual cargo process finished and the lane reran with `CARGO_PROFILE_DEV_DEBUG=0`; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=23`, `dist_build_matrix_count=23`, and zero dist boundary/distribution violations; real `plugin build obj_importer --form dist --offline` passed with `fatal=false`.
  - 2026-06-24: Plugins 13 M5/T1 gltf_importer split importer dist rollout: `gltf_importer/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `gltf_importer.dist`, and root/glTF model importer capabilities. Scoped rustfmt passed; dist test passed; `package_manifest_declares_gltf_importer_dist_contract` first long compile attempt timed out before target test result and a follow-up run exposed/fixed the PluginModuleKind assertion, then passed 1/1 with `CARGO_PROFILE_DEV_DEBUG=0`; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=24`, `dist_build_matrix_count=24`, and zero dist boundary/distribution violations; real `plugin build gltf_importer --form dist --offline` passed with `fatal=false`.
  - 2026-06-25: Plugins 13 M5/T1 texture_importer split importer dist rollout: `texture_importer/dist` exports ABI v3 via `native_dist_runtime_plugin_v3!`; runtime package manifest declares `[distribution]`, native module `texture_importer.dist`, and root/image/container/PSD texture importer capabilities. Scoped rustfmt passed; dist test passed; `package_manifest_declares_texture_importer_dist_contract` first two 305s compile attempts timed out before target test result, then passed 1/1 with a 20-minute timeout and `CARGO_PROFILE_DEV_DEBUG=0`; CI matrix unittest passed; audit JSON reports `dist_capable_plugin_count=25`, `dist_build_matrix_count=25`, and zero dist boundary/distribution violations; real `plugin build texture_importer --form dist --offline` passed with `fatal=false`.
  - 2026-06-22: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-importer-m3-0622 --message-format short --color never registration_contributes_stl_ply_and_dxf_importers -- --test-threads=1 --nocapture was blocked by unrelated zircon_runtime MaterialCaptureSeed / MaterialRuntime::capture_seed lib-test drift after an earlier 904s timeout without a test result.
  - 2026-06-23: Plugins 12 M3/T1 split importer registration cutover: `zircon_plugins/{gltf_importer,obj_importer,texture_importer,audio_importer,opus_importer,shader_wgsl_importer,ui_document_importer}/runtime/src/plugin.rs` owns trait-backed RuntimePlugin entries; split importer public `pub fn register(...)` scan is empty; `tools/audit_plugin_structure.py --json` reports `registration_conformance.m3_split_importer_gate_status=split-importer-single-entry-clean` and aggregate `m3_importer_gate_status=importer-single-entry-clean`; split importer focused cargo check passed with existing zircon_runtime warnings.
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
trait-backed `RuntimePlugin` entries in `runtime/src/plugin.rs`. They are the package-selection and
capability surface that export planning and plugin enablement should target for glTF, OBJ, texture,
audio, Opus, WGSL shader, and UI document import.

The older family packages still exist as migration packages:

- `zircon_plugin_asset_importer_model_runtime`
- `zircon_plugin_asset_importer_texture_runtime`
- `zircon_plugin_asset_importer_audio_runtime`
- `zircon_plugin_asset_importer_shader_runtime`
- `zircon_plugin_asset_importer_data_runtime`

Most family crates remain declaration aggregators during migration so existing workspace and test
callers do not lose their importer declarations while the split package ids are adopted. The model,
data, shader, audio, and texture family crates are the current exceptions: model owns real STL/PLY
mesh interchange backends plus a DXF CAD mesh-surface backend, data owns real TOML/JSON/YAML/XML
backends, shader owns the real Naga-backed WGSL/GLSL/SPIR-V path, audio owns WAV plus
Symphonia-backed codec declarations, and texture owns common image/container/PSD declarations. As of
Plugins 12 M3/T1 plus the runtime-only owner follow-up, those family crates and the root-level split
importers use `src/plugin.rs` as the single RuntimePlugin registration owner; the importer public
free-function registration audit is clean for both tracks. As of Plugins 13 M5/T1,
`asset_importer.data`, `asset_importer.model`, `asset_importer.shader`, `asset_importer.audio`,
`asset_importer.texture`, and the split `audio_importer`, `obj_importer`, `gltf_importer`, and `texture_importer` packages also have standalone dist wrapper packages
(`zircon_plugin_asset_importer_data_dist`, `zircon_plugin_asset_importer_model_dist`,
`zircon_plugin_asset_importer_shader_dist`, `zircon_plugin_asset_importer_audio_dist`,
`zircon_plugin_asset_importer_texture_dist`, `zircon_plugin_audio_importer_dist`, and
`zircon_plugin_obj_importer_dist`, `zircon_plugin_gltf_importer_dist`, and
`zircon_plugin_texture_importer_dist`). They
expose family data/model/shader/audio/texture capabilities plus split audio/OBJ/glTF/texture capabilities through
ABI v3 while the actual importer implementations remain in the runtime modules.

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

The package manifest records the runtime crate, editor/client targets, platform support, package
capabilities, and the `AssetImporterDescriptor` rows for that importer package.

Importer runtime crates no longer expose public `pub fn register(...)` free functions. As of the
2026-06-28 D13 importer runtime export macro convergence, all 12 importer runtime `plugin.rs` owners
generate `runtime_plugin()`, `package_manifest()`, `runtime_selection()`, and `plugin_registration()`
through `zircon_plugin_sdk::runtime_plugin_exports!`. No importer runtime crate hand-writes
`ProjectPluginSelection` or `RuntimePluginRegistrationReport` helper blocks.

As of the 2026-06-28 D13 importer runtime manifest builder convergence, those same `plugin.rs`
owners route shared targets, platforms, runtime module manifest, native dist module manifest,
NativeDynamic distribution, ABI v3 symbol/version, and asset-importer manifest projection through
`ImporterRuntimeManifestBuilder`. The `RuntimePlugin` implementation remains the owner of the
descriptor, importer descriptors, private registry mutation, and crate-specific registration logic.
Guards `review_d13_importer_runtime_exports_use_sdk_macro` and
`review_d13_importer_runtime_manifests_use_sdk_builder`, with statuses
`d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred` and
`d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred`, lock this helper
and manifest builder surface.

The 2026-06-28 D13 importer manifest parity guard adds SDK-level output coverage instead of
repeating manifest assertions in each importer crate. `zircon_plugins/plugin_sdk/src/manifest/tests.rs`
now owns `importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity`,
which checks shared targets/platforms, runtime/dist modules, NativeDynamic distribution packaging,
`NATIVE_ABI_VERSION_V3`, and `NATIVE_DESCRIPTOR_SYMBOL_V3`. Guard
`review_d13_importer_manifest_parity_guard_lives_in_sdk_builder` and status
`d13_importer_manifest_parity_guard_static_passed_cargo_deferred` keep importer runtime owners from
reintroducing local manifest parity boilerplate.

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
preserves skinning channels, emits `ModelAsset` primitives, and expands Bevy-style labeled subassets.
`Mesh0/Primitive0` subassets now preserve morph target displacement maps and node-linked skin inverse
bind matrices beside the compatibility primitive data. Its runtime tests include a minimal triangle
glTF fixture that exercises the real importer path and validates primitive indices, cooked
virtual-geometry source metadata, morph target position deltas, and inverse bind matrices; the crate
declares `toml` as a dev-dependency for the test-side `AssetImportContext` metadata table.
Its standalone distribution shape uses `zircon_plugin_gltf_importer_dist` as the native ABI v3
wrapper while glTF/GLB parsing, labeled subassets, scene/material/skin/animation placeholder
handling, descriptors, and runtime registration stay in `gltf_importer/runtime`.

`obj_importer` declares Wavefront `obj` model inputs with
`runtime.asset.importer.model.obj` and registers a function backend that triangulates OBJ meshes and
emits cooked `ModelAsset` primitives.
Its standalone distribution shape uses `zircon_plugin_obj_importer_dist` as the native ABI v3
wrapper while OBJ triangulation, descriptors, virtual-geometry metadata, and runtime registration
stay in `obj_importer/runtime`.

`texture_importer` declares common image formats as the primary image importer and now has real
rows for DDS, KTX, KTX2, ASTC, and PSD. Common images decode to `TextureAsset` RGBA8 payloads;
container files parse width, height, mip count, array layers, and format metadata, then preserve the
source bytes in `TexturePayload::Container`. PSD files decode through the Rust `psd` crate into a
flattened RGBA8 texture. Cubemap authoring files and loose DXGI-style payloads remain NativeDynamic
diagnostics. Its standalone distribution shape uses `zircon_plugin_texture_importer_dist` as the
native ABI v3 wrapper while image/container/PSD decoding, optional native container diagnostics,
descriptors, and runtime registration stay in `texture_importer/runtime`.

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
payloads remain outside this backend and produce no mesh output. The package root is a structural
facade: `plugin.rs` owns the `ModelAssetImporterRuntimePlugin`, descriptors, manifest projection,
runtime selection, dist module projection, and private registry wiring; `mesh_importer.rs` owns
STL/PLY import plus shared model/mesh subasset packaging and virtual-geometry cooking; `cad.rs`
owns DXF conversion and reuses the shared model mesh helpers; crate tests live under
`tests/{registration,importers,support}.rs`. The `asset_importers/model/dist` wrapper only exports
the ABI v3 descriptor/runtime entry/registration manifest; STL/PLY/DXF decoding remains in the
runtime crate.

`texture` declares standard image inputs, real container/compressed texture declarations for DDS,
KTX/KTX2, ASTC, and PSD, plus optional NativeDynamic declarations for cubemap and DXGI-style
payloads.

`audio` declares WAV plus optional codec-backed formats such as MP3, OGG, FLAC, and AIFF; Opus is
now represented by the split `opus_importer` package.

`shader` declares and registers real WGSL validation plus Naga-backed GLSL/SPIR-V conversion into
normalized WGSL `ShaderAsset` payloads. Optional NativeDynamic shader toolchains for HLSL/CG/FX
remain diagnostic until a toolchain backend is installed. The `src/plugin.rs` owner contains the
trait-backed runtime plugin entry and keeps importer descriptor registration private to the trait.

`data` declares and registers runtime backends for TOML, JSON, YAML, YML, and XML data importers.
The plugin emits `DataAsset` values with source text plus canonical JSON. YAML is decoded through
`serde_yaml`; XML is decoded through `roxmltree` into a stable neutral element tree object so XML can
participate in the same artifact path without pretending to be native JSON syntax. The
`src/plugin.rs` owner contains the trait-backed runtime plugin entry and keeps importer descriptor
registration private to the trait. It also projects the standalone distribution contract consumed by
`asset_importers/data/dist`: the native module name, dist crate name, ABI v3 descriptor symbol,
runtime entry, engine compatibility range, and data importer capability list.

The split `ui_document_importer` runtime package also participates in this importer family at the
package layer. Its `.ui.toml` path now preserves the UI schema migration report returned by
`UiAssetLoader`, matching the built-in runtime importer so package-backed and built-in scans write
the same source/target schema metadata.
