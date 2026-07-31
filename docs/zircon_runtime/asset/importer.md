---
related_code:
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/schema.rs
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_theme_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_icon_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_cube_lut.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/material.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/gltf_external_fixtures.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs
  - zircon_runtime/src/asset/tests/assets/gltf_primitive_fixtures.rs
  - zircon_runtime/src/asset/tests/assets/gltf_scene_fixtures.rs
  - zircon_runtime/src/asset/tests/assets/obj_importer.rs
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/src/asset/tests/project/binary_artifact_cache_assertions.rs
  - zircon_runtime/src/asset/tests/project/binary_artifact_cache.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/resource_revisions.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample/fixtures.rs
  - zircon_runtime/src/asset/tests/support.rs
  - zircon_runtime/src/asset/migration/transaction/journal.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/metadata.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_gltf_labeled_subassets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_project_scan_import.rs
  - zircon_runtime/src/asset/assets/data.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/descriptor/settings.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/texture/payload.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/core/framework/render/image/descriptor.rs
  - zircon_runtime/src/core/framework/render/image/asset_usage.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs
  - zircon_runtime/src/tests/plugin_extensions/asset_importer_install.rs
  - tools/tests/test_frameworks_05_asset_ui_boundary.py
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/asset_importers/model/plugin.toml
  - zircon_plugins/asset_importers/model/dist/Cargo.toml
  - zircon_plugins/asset_importers/model/dist/src/lib.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/Cargo.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/dist/Cargo.toml
  - zircon_plugins/obj_importer/dist/src/lib.rs
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/runtime/Cargo.toml
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/gltf_importer/runtime/src/tests.rs
  - zircon_plugins/gltf_importer/runtime/src/test_fixtures.rs
  - zircon_plugins/gltf_importer/dist/Cargo.toml
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/data/plugin.toml
  - zircon_plugins/asset_importers/data/dist/Cargo.toml
  - zircon_plugins/asset_importers/data/dist/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/dist/Cargo.toml
  - zircon_plugins/audio_importer/dist/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/plugin.toml
  - zircon_plugins/asset_importers/audio/dist/Cargo.toml
  - zircon_plugins/asset_importers/audio/dist/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/dist/Cargo.toml
  - zircon_plugins/texture_importer/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/plugin.toml
  - zircon_plugins/asset_importers/texture/dist/Cargo.toml
  - zircon_plugins/asset_importers/texture/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/plugin.toml
  - zircon_plugins/asset_importers/shader/dist/Cargo.toml
  - zircon_plugins/asset_importers/shader/dist/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
implementation_files:
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/schema.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_theme_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_icon_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_cube_lut.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/material.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/descriptor/settings.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/texture/payload.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/core/framework/render/image/descriptor.rs
  - zircon_runtime/src/core/framework/render/image/asset_usage.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/metadata.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_gltf_labeled_subassets.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/Cargo.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/dist/Cargo.toml
  - zircon_plugins/obj_importer/dist/src/lib.rs
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/runtime/Cargo.toml
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/gltf_importer/dist/Cargo.toml
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/data/plugin.toml
  - zircon_plugins/asset_importers/data/dist/Cargo.toml
  - zircon_plugins/asset_importers/data/dist/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/dist/Cargo.toml
  - zircon_plugins/audio_importer/dist/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/plugin.toml
  - zircon_plugins/asset_importers/audio/dist/Cargo.toml
  - zircon_plugins/asset_importers/audio/dist/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs
  - zircon_plugins/asset_importers/texture/plugin.toml
  - zircon_plugins/asset_importers/texture/dist/Cargo.toml
  - zircon_plugins/asset_importers/texture/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/Cargo.toml
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/plugin.toml
  - zircon_plugins/asset_importers/shader/dist/Cargo.toml
  - zircon_plugins/asset_importers/shader/dist/src/lib.rs
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/ui_document_importer/runtime/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - user: 2026-05-02 Asset Importer 插件化补齐计划
  - user: 2026-05-03 Opus/libopus NativeDynamic importer gap
  - user: 2026-05-16 continue Bevy-style asset/image completion toward M4
  - user: 2026-05-20 implement ZirconEngine asset/texture/model/ZShader/ZMaterial/ZMesh completion plan
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/Zircon UI .zui 组件资产与 Unreal 风格入口重构计划.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/superpowers/specs/2026-05-03-opus-native-dynamic-importer-design.md
  - docs/superpowers/plans/2026-05-03-opus-native-dynamic-importer.md
  - user: 2026-06-03 implement ZirconEngine WGPU render main-chain closure plan, M7 LUT asset ingress slice
  - docs/superpowers/specs/2026-06-09-vampire-dark-content-upgrade-design.md
  - docs/superpowers/plans/2026-06-09-vampire-dark-content-upgrade.md
  - user: 2026-06-10 vampire roguelite animation state-machine follow-up
  - user: 2026-07-13 implement the complete zircon_runtime architecture plan, prioritizing structure and review findings
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
tests:
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - zircon_runtime/tests/runtime_environment_ibl_source_import_staging_contract.rs
  - zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - zircon_runtime/tests/runtime_texture_external_cubemap_source_only_contract.rs
  - docs/tests/runtime/render/plan08_skinning_gltf_importer_channels_after_vg_skinning_split_20260705.out.log (2026-07-05 Plan 08 skinning/VG split: current-source focused glTF skinning channel regression passed 1/1)
  - docs/tests/runtime/render/plan08_skinning_production_check_after_vg_skinning_split_20260705.err.log (2026-07-05 Plan 08 skinning/VG split: current-source production `cargo check -p zircon_runtime --lib` passed; exit stored beside the log)
  - docs/tests/runtime/render/plan08_default_features_skinning_filter_direct_binary_after_vg_skinning_split_20260705.out.log (2026-07-05 Plan 08 skinning/VG split: direct generated-binary broad `skinning` filter passed 20/20; fresh current-source Cargo test wrapper remains blocked by active runtime text test compile drift)
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - project_asset_manager_runtime_accessors_recover_poisoned_locks
  - runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample/fixtures.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/resource_revisions.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/watcher.rs
  - zircon_runtime/src/asset/tests/support.rs
  - asset::tests::migration::project_commandlet::crash_windows::minted_sidecar_commit_crash_is_whitelisted_and_next_apply_converges
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked asset::tests::project::package_assets --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 3 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --locked package --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed after warm cache, 43 package-filtered runtime lib tests plus package-filtered integration binaries)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-asset-package-m2 cargo test --manifest-path zircon_plugins/Cargo.toml --locked --jobs 1 --message-format short --color never package -- --test-threads=1 (2026-05-20 package roots M2: passed after moving off full D: target dir)
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/sound.rs::sound_asset_wav_parse_reports_typed_error_variants
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_sound_asset_uses_typed_error
  - zircon_runtime/src/asset/tests/assets/ui.rs::importer_decodes_ui_theme_assets_from_theme_toml
  - zircon_runtime/src/asset/tests/assets/ui.rs::project_manager_scans_ui_theme_assets_and_restores_theme_payloads
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-theme-asset-0612-coremin-check --message-format short --color never (2026-06-12 UiThemeAsset slice: passed with existing warnings)
  - cargo test -p zircon_runtime --lib ui_theme_asset --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-theme-asset-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12 UiThemeAsset slice: timed out after 904s while compiling runtime lib-test target; no Rust diagnostics returned)
  - zircon_runtime/src/asset/tests/assets/ui.rs::importer_decodes_ui_icon_assets_from_icon_toml
  - zircon_runtime/src/asset/tests/assets/ui.rs::project_manager_scans_ui_icon_assets_and_restores_icon_payloads
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-icon-asset-0612-coremin-check --message-format short --color never (2026-06-12 UiIconAsset slice: passed with existing warnings)
  - cargo test -p zircon_runtime --lib ui_icon --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-icon-asset-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12 UiIconAsset slice: passed, 4 passed / 0 failed / 3563 filtered out)
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/asset/tests/project/binary_artifact_cache_assertions.rs
  - zircon_runtime/src/asset/tests/project/binary_artifact_cache.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/artifact/store.rs zircon_runtime/src/asset/artifact/mod.rs zircon_runtime/src/asset/project/manager/scan_and_import.rs zircon_runtime/src/asset/tests/assets/artifact_store.rs zircon_runtime/src/asset/tests/project/binary_artifact_cache_assertions.rs zircon_runtime/src/asset/tests/project/manager.rs (2026-06-12 artifact cache binary wire reaffirmation: passed)
  - git diff --check -- zircon_runtime/src/asset/artifact/store.rs zircon_runtime/src/asset/artifact/mod.rs zircon_runtime/src/asset/project/manager/scan_and_import.rs zircon_runtime/src/asset/tests/assets/artifact_store.rs zircon_runtime/src/asset/tests/project/binary_artifact_cache_assertions.rs zircon_runtime/src/asset/tests/project/manager.rs (2026-06-12 artifact cache binary wire reaffirmation: passed with LF-to-CRLF warnings only)
  - Select-String over artifact store/project restore/binary-cache test files for `artifact_cache_payload_is_current`, `legacy_bincode_artifact_payload_for_test`, cache JSON/legacy constants, and direct artifact-cache `serde_json` encode/decode (2026-06-12 artifact cache binary wire reaffirmation: passed, no matches)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-binary-cache-0611c --message-format short --color never (2026-06-12 artifact cache binary wire reaffirmation: passed with existing warnings only)
  - cargo test -p zircon_runtime --lib artifact_store --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-binary-cache-0611c --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12 artifact cache binary wire reaffirmation: blocked before artifact tests by unrelated active `zircon_runtime::render_graph` lib-test compile errors around missing `TransientTexture`/`TransientBuffer` exports and `RenderGraphBuilder::create_transient_*` methods)
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_clamps_channels_and_builds_3d_texture
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_ignores_common_metadata_rows
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_rejects_1d_shaper_sections
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_rejects_out_of_range_sizes
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_decodes_cube_lut_as_linear_3d_rgba8_texture
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_rejects_cube_lut_with_wrong_sample_count
  - zircon_runtime/src/asset/tests/assets/importer.rs::importer_capability_report_marks_diagnostic_only_backends
  - cargo test -p zircon_runtime --lib cube_lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never (2026-06-03 M7 cube LUT asset ingress before explicit shaper guard: passed, 5 passed; existing warnings only)
  - wsl -e sh -lc 'cd /mnt/e/Git/ZirconEngine && CARGO_TARGET_DIR=/tmp/zircon-render-main-chain-cube-lut-0603 cargo test -p zircon_runtime --lib cube_lut --locked --jobs 1 --message-format short --color never' (2026-06-03 M7 cube LUT shaper guard rerun: passed, 6 passed, 2500 filtered out; existing warnings only)
  - cargo test -p zircon_runtime --lib texture_importer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never (2026-06-03 M7 cube LUT asset ingress: passed, 14 passed; existing warnings only)
  - cargo test -p zircon_runtime --lib importer_capability_report_marks_diagnostic_only_backends --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never (2026-06-03 M7 cube LUT importer capability: passed, 1 passed; existing warnings only)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never (2026-06-03 M7 cube LUT asset ingress: passed; existing warnings only)
  - cargo test -p zircon_runtime --lib default_importer_decodes_gltf_without_first_wave_plugin_fixture --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app (2026-06-09 default runtime glTF: passed, 1 passed)
  - cargo test -p zircon_runtime --lib importer_default_decodes_builtin_png_texture_without_plugin_backend --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app (2026-06-09 default runtime image importer: passed, 1 passed)
  - cargo test -p zircon_runtime --lib importer_default_decodes_txt_as_text_data --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app (2026-06-09 default runtime text importer: passed, 1 passed)
  - cargo test -p zircon_runtime --lib vampire_example --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app (2026-06-09 vampire project built-in importer scan: passed, 2 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app cargo check -p zircon_runtime --lib --locked --message-format short (2026-06-10 glTF animation subasset importer: passed; existing warnings only)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --message-format short -- --nocapture --test-threads=1 (2026-06-10 real glTF AnimationClip/Skeleton labels: passed, 1 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app cargo test -p zircon_runtime --lib importer_emits_synthetic_skeleton_for_node_animation_without_skin --locked --message-format short -- --nocapture --test-threads=1 (2026-06-10 no-skin node animation skeleton label: passed, 1 passed)
  - zircon_runtime/src/asset/tests/assets/render_product.rs
  - zircon_runtime/src/asset/importer/native.rs::native_import_response_preserves_schema_migration_report
  - zircon_runtime/src/asset/importer/native.rs::native_import_command_errors_preserve_status_diagnostics_without_payload
  - zircon_runtime/src/asset/importer/native.rs::native_import_command_requires_payload_only_after_ok_status
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs::native_loader_fixture_can_import_data_asset_through_native_importer_handler
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/native_plugin_loader.rs::runtime_15_native_plugin_loader_real_fixture_tests_are_folder_backed
  - zircon_runtime/src/asset/tests/pipeline/manager.rs::asset_manager_service_reports_importer_capabilities_before_and_after_project_open
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/importer/native.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_plugins/native_dynamic_fixture/native/src/lib.rs (2026-05-26 NativeDynamic migration report DTO: passed after applying standard formatting)
  - git diff --check -- zircon_runtime/src/asset/importer/native.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_plugins/native_dynamic_fixture/native/src/lib.rs docs/zircon_runtime/asset/importer.md .codex/sessions/20260526-1820-asset-system-continuation.md (2026-05-26 NativeDynamic migration report DTO: passed with LF-to-CRLF warnings only)
  - cargo build --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir F:\cargo-targets\zircon-native-fixture-migration-0526 --message-format short --color never (2026-05-26 NativeDynamic migration report DTO: passed)
  - cargo test -p zircon_runtime --lib native_loader_fixture_can_import_data_asset_through_native_importer_handler --locked --jobs 1 --target-dir F:\cargo-targets\zircon-native-loader-migration-0526 --message-format short --color never -- --test-threads=1 (2026-05-26 NativeDynamic migration report DTO: timed out after 304s during Windows runtime test compilation before Rust diagnostics; matching residual processes for this target dir were stopped)
  - cargo test -p zircon_runtime --lib native_import_command_errors_preserve_status_diagnostics_without_payload --locked --jobs 1 --target-dir F:\cargo-targets\zircon-native-command-status-0526 --message-format short --color never -- --test-threads=1 (2026-05-26 NativeDynamic command status handling: timed out after 304s during Windows runtime test compilation before Rust diagnostics; no matching residual target-dir process remained)
  - cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-asset-parity-runtime-lib-0520 --message-format short --color never (2026-05-20 asset parity implementation: passed; existing warnings only)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --locked --jobs 1 --message-format short --color never (2026-05-20 asset parity implementation: passed; existing runtime warning only)
  - cargo test -p zircon_runtime --lib importer_capability_report_marks_diagnostic_only_backends --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-asset-parity-runtime-lib-0520 --message-format short --color never -- --test-threads=1 (2026-05-20 asset parity implementation: timed out during Windows test build/link before Rust test diagnostics)
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (2026-05-20 glTF labeled subassets: passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --message-format short --color never (2026-05-20 glTF labeled subassets: passed; existing runtime dead_code warning only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF labeled subassets: timed out during Windows runtime test build/link; matching residual Cargo chain was stopped after timeout)
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_bevy_style_gltf_labeled_subassets
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_gltf_multi_primitive_material_labels
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_gltf_multi_scene_labels
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_decodes_gltf_external_texture_image
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_reports_missing_gltf_external_buffer
  - CARGO_TARGET_DIR=/tmp/zircon-gltf-m4-wsl-fast cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels: blocked before test execution by unrelated zircon_runtime_interface/src/ui/dispatch/navigation/result.rs E0277, UiBindingUpdateReport does not implement Eq)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels: Windows attempt timed out after 304s before Rust test diagnostics; matching residual Cargo child processes were stopped)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --message-format short --color never (2026-05-20 runtime glTF labels retry: passed, confirming the earlier WSL Eq error is not present in the current Windows source tree)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels retry: passed, 1 passed, after replacing the invalid fixture PNG data URI with a valid CRC 1x1 RGBA PNG)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF animation/skin labels: passed, 1 passed, early placeholder-label phase later superseded by the 2026-06-05 Skin JSON coverage)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels final: passed, 1 passed, 1720 filtered out, after warming the runtime test harness and restoring the top-level WGSL capture facade export)
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_decodes_triangle_gltf_into_model_asset
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_emits_multi_primitive_material_labels
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_rejects_unsupported_gltf_primitive_mode
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_decodes_external_texture_image
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_reports_missing_external_buffer
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib importer_decodes_triangle_gltf_into_model_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF plugin labels retry: passed, 1 passed, after the same fixture PNG replacement)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF plugin animation/skin labels: passed, 3 passed, early placeholder-label phase later superseded by the 2026-06-05 Skin JSON coverage)
  - cargo test -p zircon_runtime --lib importer_rejects_unsupported_gltf_primitive_mode --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 glTF primitive mode guard: failed before implementation because `LINES` imported as `TriangleList`; passed after adding the mode guard; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_gltf_multi_primitive_material_labels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 multi-primitive glTF labels: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_gltf_multi_scene_labels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 multi-scene glTF labels: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 glTF primitive mode guard regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib gltf --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 glTF multi-scene/external texture/missing buffer regression: passed, 8 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime importer_rejects_unsupported_gltf_primitive_mode --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF primitive mode guard: failed before implementation because `LINES` imported as `TriangleList`; passed after adding the mode guard; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF primitive mode and multi-primitive label regression: passed, 5 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime importer_emits_multi_scene_labels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF multi-scene labels: passed, 1 passed; existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF multi-scene/external texture/missing buffer regression: passed, 8 passed plus 0 doc tests; existing zircon_runtime warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::gltf_importer --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gltf-skin-json-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 runtime glTF Skin JSON labels: passed, 8 passed, 2774 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gltf-skin-json-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 split glTF Skin JSON labels: passed, 8 passed; existing zircon_runtime warnings only)
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_gltf_labeled_subassets.rs::runtime_15_gltf_labeled_material_subassets_are_child_owner
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib importer --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 model plugin subasset labels: passed, 5 passed, covering STL/PLY/DXF root dependencies and Mesh0/Primitive0 MeshAsset payloads)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime --lib obj_importer_decodes_model_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 OBJ plugin subasset label: passed, 1 passed, covering root dependency and Mesh0/Primitive0 MeshAsset payload)
  - cargo test -p zircon_runtime --lib importer_emits_obj_multi_mesh_subassets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 OBJ multi-mesh labels: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib obj --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 runtime OBJ regression: passed, 6 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime obj_importer_emits_multi_mesh_subassets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split OBJ multi-mesh labels: passed, 1 passed; existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split OBJ importer regression: passed, 4 passed plus 0 doc tests; existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split STL/PLY/DXF M4 gate: passed, 5 passed plus 0 doc tests; existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_model_runtime model_asset_importer_package_manifest_declares_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-model-runtime-0624 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 model dist package-manifest gate: first 604s compile attempt timed out without a test result, rerun passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build asset_importer.model --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-asset-importer-model-package-0624 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-model-build-0624 --offline (2026-06-24 model dist package smoke: passed with fatal=false, native_plugins.toml manifests and asset_importer_model.sig emitted)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_shader_runtime shader_asset_importer_package_manifest_declares_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-shader-runtime-0624 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 shader dist package-manifest gate: first 904s compile attempt timed out without a test result and residual cargo/rustc processes were stopped, rerun with CARGO_PROFILE_DEV_DEBUG=0 passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build asset_importer.shader --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-asset-importer-shader-package-0624 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-shader-build-0624 --offline (2026-06-24 shader dist package smoke: passed with fatal=false, native_plugins.toml manifests and asset_importer_shader.sig emitted)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_audio_runtime audio_asset_importer_package_manifest_declares_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-audio-runtime-0624-nodebug --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 audio dist package-manifest gate: first 604s compile attempt timed out without a test result and residual cargo/rustc processes were stopped, rerun with CARGO_PROFILE_DEV_DEBUG=0 passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build asset_importer.audio --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-asset-importer-audio-package-0624 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-audio-build-0624 --offline (2026-06-24 audio dist package smoke: passed with fatal=false, native_plugins.toml manifests and asset_importer_audio.sig emitted)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_texture_runtime texture_asset_importer_package_manifest_declares_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-texture-runtime-0624-nodebug --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 texture dist package-manifest gate: first parallel compile attempt timed out after 1204s without a test result, rerun with CARGO_PROFILE_DEV_DEBUG=0 passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build asset_importer.texture --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-asset-importer-texture-package-0624 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-texture-build-0624 --offline (2026-06-24 texture dist package smoke: passed with fatal=false, native_plugins.toml manifests and asset_importer_texture.sig emitted)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_audio_importer_runtime package_manifest_declares_audio_importer_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-audio-importer-runtime-0624-nodebug --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 split audio_importer dist package-manifest gate: first cold compile produced no target test result and left same target-dir cargo processes, rerun with CARGO_PROFILE_DEV_DEBUG=0 passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build audio_importer --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-audio-importer-package-0624 --target-dir D:\cargo-targets\zircon-plugin-audio-importer-build-0624 --offline (2026-06-24 split audio_importer dist package smoke: passed with fatal=false, native_plugins.toml manifests and audio_importer.sig emitted)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_obj_importer_runtime package_manifest_declares_obj_importer_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-obj-importer-runtime-0624-nodebug --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 split obj_importer dist package-manifest gate: first two cold compile commands timed out without a target test result while compiling zircon_runtime; residual cargo was allowed to finish, then rerun with CARGO_PROFILE_DEV_DEBUG=0 passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build obj_importer --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-obj-importer-package-0624 --target-dir D:\cargo-targets\zircon-plugin-obj-importer-build-0624 --offline (2026-06-24 split obj_importer dist package smoke: passed with fatal=false, native_plugins.toml manifests and obj_importer.sig emitted)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_gltf_importer_runtime package_manifest_declares_gltf_importer_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-gltf-importer-runtime-0624-nodebug --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 split gltf_importer dist package-manifest gate: first long compile attempt timed out before target test result and a follow-up run exposed/fixed the PluginModuleKind assertion, then rerun with CARGO_PROFILE_DEV_DEBUG=0 passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build gltf_importer --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-gltf-importer-package-0624 --target-dir D:\cargo-targets\zircon-plugin-gltf-importer-build-0624 --offline (2026-06-24 split gltf_importer dist package smoke: passed with fatal=false, native_plugins.toml manifests and gltf_importer.sig emitted)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_importer_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-texture-importer-dist-0625 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-25 split texture_importer dist tests: passed, 2 passed)
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_texture_importer_runtime package_manifest_declares_texture_importer_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-texture-importer-runtime-0625-nodebug --message-format short --color never -- --test-threads=1 --nocapture (2026-06-25 split texture_importer dist package-manifest gate: first two 305s compile attempts timed out before target result, then rerun with a 20-minute timeout and CARGO_PROFILE_DEV_DEBUG=0 passed 1/1 with existing zircon_runtime warnings)
  - python -m tools.zircon_export plugin build texture_importer --form dist --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-texture-importer-package-0625 --target-dir D:\cargo-targets\zircon-plugin-texture-importer-build-0625 --offline (2026-06-25 split texture_importer dist package smoke: passed with fatal=false, native_plugins.toml manifests and texture_importer.sig emitted)
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs::dds_dx10_container_importer_reads_cubemap_array_layers
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs::ktx1_3d_container_keeps_depth_separate_from_array_layers
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs::astc_container_importer_reads_3d_block_and_depth
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs::ktx2_3d_container_keeps_depth_separate_from_array_layers
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs::container_importer_applies_descriptor_settings_without_expanding_payload
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs::container_importer_rejects_array_layout_without_decoded_rgba
  - zircon_plugins/texture_importer/runtime/src/container/mod.rs::container_importer_reports_layer_count_overflow_diagnostics
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::render_asset_usage_alias_accepts_single_token
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::depth_or_array_layers_updates_array_layer_count_for_2d_arrays
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::array_layer_count_updates_depth_or_array_layers_for_2d_arrays
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::mismatched_2d_extent_settings_report_error
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::dimension_3d_rejects_multiple_array_layers
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::dimension_3d_keeps_depth_and_single_array_layer
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::import_extent_override_replaces_existing_2d_container_layers
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::bevy_alias_diagnostics_report_actual_setting_keys
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/texture_descriptor_settings.rs::runtime_15_texture_descriptor_settings_parser_is_child_owner
  - zircon_runtime/src/asset/importer/image_decode.rs::default_format_reports_missing_extension
  - zircon_runtime/src/asset/importer/image_decode.rs::explicit_source_format_reports_unsupported_token
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_decodes_common_extension_format_matrix
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_uses_extension_format_by_default
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_can_guess_format_when_requested
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_can_use_explicit_source_format
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_accepts_source_format_aliases
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_reports_actual_source_format_key
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_accepts_bevy_image_setting_aliases
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_reinterprets_stacked_array_layout
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_texture_fixture_rejects_invalid_array_layout
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::texture_upload_readiness_rejects_compressed_mips_and_arrays_until_full_upload_exists
  - cargo test -p zircon_runtime --lib texture_upload_readiness_rejects_compressed_mips_and_arrays_until_full_upload_exists --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never -- --test-threads=1 (2026-05-20 texture container upload shape boundaries: passed, 1 passed, 1723 filtered out)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never (2026-05-20 texture container upload shape boundaries: passed; existing scene/world warnings only)
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_decodes_common_extension_format_matrix
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_uses_extension_format_by_default
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_can_guess_format_from_bytes_when_requested
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_can_use_explicit_source_format
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_accepts_source_format_aliases
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_reports_actual_source_format_key
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_accepts_bevy_image_setting_aliases
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_reinterprets_stacked_array_layout
  - zircon_plugins/texture_importer/runtime/src/lib.rs::image_importer_rejects_invalid_array_layout
  - zircon_plugins/texture_importer/runtime/src/lib.rs::psd_importer_applies_texture_descriptor_settings
  - rustfmt --edition 2021 --config skip_children=true --check on touched M4 texture/importer/runtime/plugin files (2026-05-17 focused M4 final: passed)
  - git diff --check on touched M4 texture/importer/docs/session files (2026-05-17 focused M4 final: passed with LF/CRLF warnings only; trailing-whitespace content search found matches only in unrelated docs)
  - cargo metadata --locked --no-deps --format-version 1 (2026-05-17 focused M4 final: passed)
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (2026-05-17 focused M4 final: passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --tests --locked --offline --jobs 1 (2026-05-17 focused M4 final: passed)
  - cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 (2026-05-17 focused M4 final: passed)
  - cargo test -p zircon_runtime --lib texture_importer --locked --offline --jobs 1 (2026-05-17 focused M4 final: passed, 11 passed, 0 failed)
  - cargo test -p zircon_runtime --lib render_product_assets_texture --locked --offline --jobs 1 (2026-05-17 focused M4 final: passed, 3 passed, 0 failed)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --locked --offline --jobs 1 (2026-05-17 focused M4 final: passed, 28 passed, 0 failed)
  - cargo test -p zircon_runtime --lib texture::descriptor --locked --offline --jobs 1 on Windows (2026-05-17 M4 descriptor follow-up: blocked before Zircon tests by root `Cargo.lock` `wgpu-hal`/`windows` D3D12 API mismatch)
  - WSL cargo test -p zircon_runtime --lib texture::descriptor --locked --jobs 1 (2026-05-17 M4 descriptor follow-up: passed, 8 passed, 0 failed)
  - WSL-built zircon_runtime test binary image_decode --nocapture (2026-05-17 M4 image-decode follow-up: passed, 2 passed, 0 failed)
  - rustfmt --edition 2021 --config skip_children=true --check on shared image source decode files (2026-05-16 image source format selection: passed)
  - git diff --check on shared image source decode/docs/session files (2026-05-16 image source format selection: passed with CRLF warnings only)
  - rustfmt --edition 2021 --check on touched texture/importer/render-product files except root mod traversal (2026-05-16 M4 texture descriptor: passed)
  - cargo metadata --locked --no-deps --format-version 1 (2026-05-16 M4 texture descriptor: passed)
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (2026-05-16 M4 texture descriptor: passed)
  - cargo test -p zircon_runtime --lib render_product_assets_texture --locked --jobs 1 (2026-05-16 M4 texture descriptor: attempted, inconclusive because concurrent Cargo package-cache locks/active Cargo jobs prevented completion before test diagnostics)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib image_importer_applies_texture_descriptor_settings --locked --jobs 1 (2026-05-16 M4 texture descriptor: attempted, blocked by current plugin workspace lock/update state before test execution)
  - rustfmt --edition 2021 --config skip_children=true --check on touched M4 image descriptor/importer files (2026-05-16 asset_usage/container dimension: passed)
  - git diff --check on touched M4 image descriptor/importer/docs files (2026-05-16 asset_usage/container dimension: passed with CRLF warnings only)
  - cargo metadata --locked --no-deps --format-version 1 (2026-05-16 asset_usage/container dimension: passed)
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (2026-05-16 asset_usage/container dimension: passed)
  - cargo check/test for this slice (2026-05-16 asset_usage/container dimension: deferred; active unrelated Cargo jobs and lockfile update state prevent reliable `--locked` compile/test evidence)
  - rustfmt --edition 2021 --config skip_children=true --check on texture importer lib/container files (2026-05-16 texture container split: passed)
  - git diff --check on texture importer lib/container docs/session files (2026-05-16 texture container split: passed with CRLF warnings only)
  - cargo metadata --locked --no-deps --format-version 1 (2026-05-16 texture container split: passed)
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (2026-05-16 texture container split: passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-texture-importer-container-split (2026-05-16 texture container split: passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-texture-importer-container-split (2026-05-16 texture container split: passed)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-texture-importer-container-split (2026-05-16 texture container split: attempted; Cargo exited -1 during dependency test-profile compilation before Rust diagnostics)
  - rustfmt --edition 2021 --config skip_children=true --check on texture extent/importer files (2026-05-16 texture extent depth-or-array-layers: passed)
  - git diff --check on texture extent/importer/docs/session files (2026-05-16 texture extent depth-or-array-layers: passed with CRLF warnings only)
  - cargo metadata --locked --no-deps --format-version 1 (2026-05-16 texture extent depth-or-array-layers: passed)
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (2026-05-16 texture extent depth-or-array-layers: passed)
  - cargo check -p zircon_runtime --lib/--tests --locked --offline --target-dir E:\cargo-targets\zircon-texture-extent-runtime-check (2026-05-16 texture extent depth-or-array-layers: attempted; Cargo exited -1 during dependency/runtime compilation before Rust diagnostics)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --tests --locked --offline --target-dir E:\cargo-targets\zircon-texture-importer-container-split (2026-05-16 texture extent depth-or-array-layers: attempted; package-cache lock caused immediate Cargo exit -1 before Rust diagnostics)
  - cargo test -p zircon_runtime_interface --locked resource (2026-05-16 `.zmeta` M1 final: passed, 11 passed, 0 failed, 85 filtered out)
  - cargo test -p zircon_runtime --locked asset::tests::project (2026-05-16 `.zmeta` M1 final: passed, 19 passed, 0 failed, 1350 filtered out)
  - cargo test -p zircon_runtime --locked asset::tests::watcher (2026-05-16 `.zmeta` M1 final: passed, 2 passed, 0 failed, 1367 filtered out)
  - cargo test -p zircon_runtime --locked asset::tests::assets::animation (2026-05-16 `.zmeta` M1 final: passed, 6 passed, 0 failed, 1363 filtered out)
  - cargo test -p zircon_editor --lib --locked editor_asset_manager (2026-05-16 `.zmeta` M1 final: passed, 4 passed, 0 failed, 1315 filtered out)
  - cargo test -p zircon_runtime --lib zui --locked (2026-05-14 .zui M1 importer route: planned for milestone testing stage)
  - cargo check -p zircon_runtime --lib --locked (2026-05-14 .zui M1 importer route: planned for milestone testing stage)
  - 2026-05-03 review correction: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --check (passed)
  - 2026-05-03 review correction: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_opus_importer_runtime --lib --locked --jobs 1 (passed, 4 tests)
  - 2026-05-03 review correction: cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (passed)
  - 2026-05-03 review correction: git diff --check (passed with CRLF normalization warnings only)
  - previously passed: cargo check -p zircon_runtime --locked
  - previously passed: cargo test -p zircon_runtime --locked asset
  - previously passed: cargo test -p zircon_runtime --locked plugin_extensions
  - previously passed: cargo test -p zircon_runtime --locked native_import
  - previously passed: cargo test -p zircon_runtime --locked project_manager_records_failed_imports_and_continues_scanning
  - previously passed: cargo test --manifest-path zircon_plugins/Cargo.toml --locked -j 1 -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_texture_runtime -p zircon_plugin_asset_importer_audio_runtime -p zircon_plugin_asset_importer_shader_runtime -p zircon_plugin_asset_importer_data_runtime
  - fresh-rerun blocked: cargo test -p zircon_runtime --locked asset (unrelated graphics/VG ViewportCameraSnapshot move error)
  - passed: cargo check -p zircon_runtime --lib --tests --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation
  - passed: cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --locked --jobs 1
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib package_declares_only_ui_v2_toml_importer --jobs 1 --target-dir target\codex-ui-v2-plugin-guard
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib plugin_toml_declares_only_ui_v2_toml_importer --jobs 1 --target-dir target\codex-ui-v2-plugin-guard
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib registration_does_not_select_legacy_ui_document_formats --jobs 1 --target-dir target\codex-ui-v2-plugin-guard
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib --jobs 1 --target-dir target\codex-ui-v2-plugin-guard
  - passed: cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --no-deps --format-version 1
  - passed: cargo test -p zircon_runtime --lib importer_registry_rejects_non_fixture_ui_toml_source_importer_registration --jobs 1 --target-dir target\codex-ui-v2-guard
  - passed: cargo test -p zircon_runtime --lib importer_registry_routes_v2_ui_toml_to_v2_document_backend --jobs 1 --target-dir target\codex-ui-v2-guard
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
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed again after gating source-template first-wave helper modules as test-only; existing runtime warnings only)
  - 2026-05-03: cargo test -p zircon_runtime importer_decodes_obj_and_gltf_into_model_assets --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed, 1 fixture-backed test, with existing runtime warnings)
  - 2026-05-03: cargo test -p zircon_runtime runtime_extension_registry_installs_asset_importers_before_project_open --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (timed out after 10 minutes during Windows test build/link while other Cargo jobs were active; no Rust diagnostics returned)
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib typed_toml_importer_decodes_ui_v2_view_asset --jobs 1 --target-dir target\codex-ui-v2-plugin-guard
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_data_runtime --lib --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation-2
  - 2026-06-24: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_data_runtime data_asset_importer_package_manifest_declares_dist_contract --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-data-runtime-0624 --message-format short --color never -- --test-threads=1 --nocapture (passed, 1 passed; existing zircon_runtime warnings only)
  - 2026-06-24: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_data_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-data-dist-0624 --message-format short --color never -- --test-threads=1 --nocapture (passed, 2 passed)
  - 2026-06-24: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_audio_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-audio-dist-0624 --message-format short --color never -- --test-threads=1 --nocapture (passed, 2 passed)
  - 2026-06-24: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_asset_importer_texture_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-asset-importer-texture-dist-0624 --message-format short --color never -- --test-threads=1 --nocapture (passed, 2 passed)
  - 2026-06-24: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_audio_importer_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-audio-importer-dist-0624 --message-format short --color never -- --test-threads=1 --nocapture (passed, 2 passed)
  - 2026-06-24: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_obj_importer_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-obj-importer-dist-0624 --message-format short --color never -- --test-threads=1 --nocapture (passed, 2 passed)
  - 2026-06-24: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_gltf_importer_dist --no-default-features --features dist --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-gltf-importer-dist-0624 --message-format short --color never -- --test-threads=1 --nocapture (passed, 2 passed)
  - passed: cargo build --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 with CARGO_TARGET_DIR=E:\cargo-targets\zircon-asset-importer-gap-continuation-3-plugin
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
  - 2026-05-03: git diff --check (passed with LF-to-CRLF warnings only)
  - 2026-05-03: cargo info bincode (used for UI binary document backend selection)
  - 2026-05-03: cargo generate-lockfile --manifest-path zircon_plugins\Cargo.toml (passed after adding the UI binary document backend dependency)
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime (passed)
  - 2026-05-03: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-asset-importer-ui-binary-backend --message-format short --color never (passed, 8 tests)
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/tests/plugin_extensions/asset_importer_install.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/asset/tests/project/manager.rs::project_manager_restores_ready_artifacts_from_meta_after_restart
  - 2026-05-08 cross-lane compile unblock: cargo test -p zircon_runtime --lib scene::tests::ecs_schedule::render_extract_prepare_flushes_parent_reorder_and_active_changes --locked --message-format short (passed, 1 test, after asset importer M3 hard-cutover fixes)
doc_type: module-detail
---

# Asset Importer Pluginization

## Purpose

Asset import is now routed through `AssetImporterRegistry` instead of hard-coded extension branches in `ProjectManager`. The project scan owns traversal, metadata, artifact writing, failure records, and hot reimport state; importers own only source decoding and conversion to `ImportedAsset`.

This makes import formats a runtime extension point. The runtime still owns the neutral contracts, registry, project scan, artifact metadata, and diagnostics, but the first-wave stable format behavior is now expected to arrive through linked plugin importers. Package manifests can declare importer descriptors, and NativeDynamic plugins can provide external toolchain importers without sharing Rust trait objects or engine state across the ABI.

## Runtime Contract

`AssetImporterDescriptor` is the public routing record. It declares importer id, plugin id, priority, ordinary extensions, full suffixes, output kind, importer version, and required capabilities. Full suffixes are matched before extensions, so `toolbar.zui`, `level.scene.toml`, and `actor.prefab.toml` do not fall through to ordinary extension importers.

`AssetImportContext` carries the source path, normalized asset URI, source bytes, and per-asset import settings from meta. `AssetImportOutcome` is now a labeled entry list rather than a single imported asset. Each `ImportedAssetEntry` owns its locator, asset payload, dependency URIs, optional schema migration report, and diagnostics. The root entry uses the unlabeled source locator, and subassets use the same source path with a label such as `res://model/character.gltf#Mesh0`. The registry validates duplicate importer ids and duplicate matchers at the same priority before a plugin contribution is accepted.

`AssetImporterCapabilityReport` is the public diagnostic view of a registered importer. It pairs the routing descriptor with `AssetImporterCapabilityStatus::Available` or `DiagnosticOnly { message }`. Function-backed and plugin-backed handlers report `Available`; `DiagnosticOnlyAssetImporter` reports the stable reason that a format is recognized but cannot currently produce runtime assets. `AssetImporterRegistry`, the ingest-level `AssetImporter`, `ProjectAssetManager`, and the public `AssetManager` service trait expose source-specific and full capability reports so editor UI can present importer availability without running a scan or creating error artifacts.

The hard-cutover rule is that importer code must call `AssetImportOutcome::new(locator, asset)` with an explicit locator. No compatibility constructor derives a locator from the asset payload, because several asset payloads do not own source URIs and subasset identity is label-based. Structured duplicate-label and missing-label errors carry `source_uri` plus `label` so `thiserror` does not treat the source locator as an error source.

Plain `.toml` is a `DataAsset`. Typed `*.xxx.toml` requires a registered full-suffix importer; unknown typed TOML fails as an error resource instead of silently becoming a generic data file. The registry rejects `.ui.toml` source-template and `.v2.ui.toml` source-template importer descriptors on the production path, so plugin manifests cannot reintroduce recursive UI source schemas or the pre-`.zui` mixed view/component/style UI v2 importer. Only explicit unit-test source-template fixtures are allowed to register those matchers for schema migration coverage.

### Data import error sources

`AssetImportContext::source_text()` reports invalid UTF-8 as
`AssetImportError::SourceTextDecode { path, source }`, preserving the original
`FromUtf8Error`. Plain TOML and JSON data import use the contextual
`TomlDeserialize` and `JsonDeserialize` variants, so callers can inspect the parser error through
`std::error::Error::source()` instead of receiving a flattened `Parse(String)`.

This is a hard cut for these three paths. There is no conversion back to the old string variant,
and semantic import rejections may continue to use their dedicated non-source variants.

### Project reference repair ordering

`asset/reference_resolver.rs` is the single owner for GUID, path-hint, and labeled-subasset repair used by importers. It first validates the project path against the configured roots, then checks the exact labeled registry locator when a subasset label is present, and finally falls back to the canonical base asset only when that label is stale or absent. This ordering preserves valid labeled subassets while allowing a stale `#Mesh0` reference to repair to the existing base asset; the resolver emits the complete `ReferenceRepair` and the model importer only records it in `AssetImportOutcome`.

The importer layer must not catch `Dangling` to retry without a label, add registry aliases, or silently delete subasset identity. Regression coverage lives in `asset::reference_resolver::tests::resolution_reports_guid_path_repair_dangling_and_conflict_states` and `asset::importer::ingest::import_model::tests::importer_outcome_exposes_complete_guid_repair`, sourced from `docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md` and its stale-subasset failure handoff.

Project scan establishes reference identity before any importer reads an authored project document. `scan_and_import` first collects every source, then aligns each existing `.zmeta` root URL and kind with its current source path or mints the missing sidecar. URL alignment emits an in-memory `Renamed` identity change. During watcher scans, that authoritative rename is ordered before raw split `Removed` / `Added` events; the same merged change list drives duplicate-GUID normalization and the atomic resource-registry plus asset-registry commit. Only after path identity and duplicate ownership converge does `AssetRegistryIndex::inspect_project` build the read-only resolver index passed to importers. This ordering preserves the GUID of a source and sidecar moved together while still reminting a copied sidecar whose GUID has another live owner.

Authored material and scene fixtures follow the production project-reference schema instead of the retired runtime `{uuid,url}` serializer. Their references are persisted as `{kind,guid,path_hint,sub}` records. The fixture writer resolves an existing target through exactly one manifest asset root; when a target is intentionally absent, it keeps a path hint under the source asset's root so missing references remain representable. Multiple matching roots remain an error because choosing one would conceal a real project ambiguity. Fixture edits deserialize the project document and write it back through the same formal serializer, so watcher and revision tests exercise the production contract rather than a test-only compatibility path. Runtime04 testing-stage coverage owns first-open resolution, stale-URL GUID preservation, rename preservation, watcher reimport, resource revision isolation, runtime lease rehydration, and the end-to-end asset-flow material fixture; full Cargo acceptance remains a testing-stage result rather than a documentation assumption.

## Built-In Coverage

The production default importer registry installs real Rust paths for runtime-core formats: plain TOML/JSON data, plain `.txt` text data, `.zui` UI component documents, typed Zircon source assets such as `.zmaterial`, `.zshader`, `.zmesh`, material/font/model/physics material/scene/prefab/authoring navigation assets, animation `.zranim` contracts that have not yet moved fully to the animation plugin, the remaining GLSL/SPIR-V shader paths, common image textures, and glTF/GLB models. Plain `.txt` data keeps source notes and license files importable in example projects without forcing an external data plugin.

The image and glTF built-ins use priority `10`, while diagnostic `zircon.plugin_required.texture.image` and `zircon.plugin_required.model.gltf` descriptors stay registered at priority `0`. Registry selection ranks available handlers over diagnostic-only handlers and then compares priority, so the standalone dynamic runtime chooses the built-in PNG/JPEG/etc. and glTF/GLB paths. Linked first-party plugin importers or test fixture importers can still override those matchers by registering available handlers at higher priority. This keeps the plugin override model intact while giving `zircon_runtime.dll` enough independent capability to load simple real-asset game projects even though app-linked Rust plugin registrations do not cross the dynamic ABI boundary.

WGSL, OBJ, WAV, and optional model/audio/container formats still register diagnostic-only `zircon.plugin_required.*` or `zircon.optional.*` descriptors by default. These descriptors preserve output kind, matcher, importer version, and capability metadata so scans produce stable error records when a plugin is disabled or missing. UI source-template `.ui.toml` and `.v2.ui.toml` no longer register production plugin-required fallbacks, and `AssetImporterRegistry` rejects non-fixture matcher registration for both suffixes. They remain reachable only through exact source-template migration fixtures used by unit tests. The real stable split backends live in `texture_importer`, `shader_wgsl_importer`, `obj_importer`, `gltf_importer`, and `audio_importer`, while `ui_document_importer` mirrors the `.zui` component payload path for plugin packaging.

Runtime tests that intentionally exercise plugin-owned first-wave formats install explicit fixture importers with the same package ids and higher priority as the split plugin crates. The fixtures call source helper modules so the runtime test crate can validate artifact/project behavior without taking a dev-dependency on `zircon_plugins`. Graphics project-render and M4 behavior-layer tests use that explicit fixture path for WGSL/OBJ projects, while default-runtime importer tests separately lock the built-in PNG/text/glTF behavior that standalone game examples depend on.

The `asset_importer.data` runtime plugin now registers real TOML/JSON/YAML/XML `DataAsset`
backends so project/plugin selection can move structured data loading out of the built-in fallback
path. Its standalone distribution shape uses `zircon_plugin_asset_importer_data_dist` as the native
ABI v3 wrapper while keeping decoding and importer registration in the runtime importer module. The
`asset_importer.model` family plugin now registers real STL, PLY, and DXF model backends.
STL and PLY decode through `stl_io` and `ply-rs-bw`; DXF decodes through the `dxf` crate and imports
`3DFACE`, `SOLID`, `TRACE`, and `POLYLINE` polyface mesh surfaces. These paths emit `ModelAsset`
primitives with generated virtual-geometry metadata and labeled `MeshAsset` subassets. The root
`ModelAsset.primitives` path stays in place for the current renderer, while each primitive also
receives a label such as `Mesh0/Primitive0` and is emitted through `ImportedAsset::Mesh`. The same
compatibility subasset path is used by the split OBJ and glTF plugins and by built-in `.model.toml`
imports. glTF primitive subassets additionally carry morph target displacement maps and node-linked
skin inverse bind matrices when the source file provides them. Current STL/PLY/DXF and OBJ plugin
tests assert both the root dependency edge and the labeled `Mesh0/Primitive0` `MeshAsset` payload,
including vertex count, indices, and preserved virtual-geometry metadata. The glTF fixture tests now
also assert morph target position deltas and inverse bind matrix propagation. The DXF importer
implementation is isolated in
`asset_importers/model/runtime/src/cad.rs`, while the package root keeps descriptor and registration
wiring. Its standalone distribution shape uses `zircon_plugin_asset_importer_model_dist` as the
native ABI v3 wrapper while keeping STL/PLY/DXF decoding and importer registration in the runtime
module. OBJ multi-object fixtures now assert that a single `.obj` with two object sections produces
two root model primitives plus `Mesh0/Primitive0` and `Mesh1/Primitive0` subassets, with both the
runtime fixture importer and split plugin preserving per-object virtual geometry. The split
`obj_importer` package now has a standalone distribution shape using
`zircon_plugin_obj_importer_dist` as the native ABI v3 wrapper while keeping OBJ triangulation,
descriptors, virtual-geometry metadata, and runtime registration in `obj_importer/runtime`. The split
`texture_importer` package decodes common image formats to RGBA8 through the shared
`zircon_runtime::asset::decode_texture_source_image` helper, delegates DDS, KTX, KTX2, and ASTC
header parsing to its focused `runtime/src/container.rs` module, stores those containers as
`TexturePayload::Container`, and decodes PSD files through the Rust `psd` crate into flattened RGBA8
textures. The shared image helper follows Bevy's default `ImageLoaderSettings.format =
FromExtension` contract (`dev/bevy/crates/bevy_image/src/image_loader.rs:120` and
`dev/bevy/crates/bevy_image/src/image_loader.rs:188`): source bytes are decoded using the source
extension by default, mismatch failures say which extension-selected format was attempted, and
`image_format = "guess"`, `image_format = "jpeg"`, or `source_format = "open_exr"` style settings
opt into byte guessing or an explicit source container format. The parser reports the actual setting
key (`image_format`, `decode_format`, or `source_format`) when a value has the wrong type or an
unsupported token, and the default path reports a distinct missing-extension diagnostic rather than
falling back to byte guessing. This keeps those settings scoped to source decoding while the existing `format`
import setting continues to mean render texture format, matching Bevy's separate `texture_format`
override role (`dev/bevy/crates/bevy_image/src/image_loader.rs:122`). The split `texture_importer`
package now has a standalone distribution shape using `zircon_plugin_texture_importer_dist` as the
native ABI v3 wrapper while common image decode, container parsing, PSD decode, optional native
container diagnostics, descriptors, and runtime registration stay in `texture_importer/runtime`.

The split `gltf_importer` package now emits Bevy-style labeled subassets while keeping the root
`ModelAsset.primitives` compatibility path. `runtime/src/lib.rs` still owns plugin registration and
primitive decoding, while `runtime/src/subassets.rs` owns labeled subasset expansion. The
runtime-only first-wave fixture mirrors the same label semantics in
`zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs` and routes clip/skeleton
payloads through `gltf_animation_subassets.rs`, so project/meta tests that cannot depend on
`zircon_plugins` still exercise the public `AssetImportOutcome` shape. The importer and fixture
descriptors declare additional output kinds for mesh, scene, material, texture, data,
`AnimationClip`, and `AnimationSkeleton`, then emit `Texture{n}` as `TextureAsset`, `Material{n}`
and `DefaultMaterial` as `MaterialAsset`, `Mesh{n}` as a mesh-local `ModelAsset`,
`Mesh{m}/Primitive{p}` as first-class `MeshAsset`, `Node{n}` and `Scene{n}` as `SceneAsset`,
`Animation{n}` as a typed `AnimationClipAsset`, real `Skin{n}/Skeleton` skeleton subassets when
skins are present, and synthetic `Animation{n}/Skeleton` skeleton subassets for node-animation
files that do not define glTF skins. Machine-readable JSON `DataAsset` rows still record `Skin{n}`
metadata and `Skin{n}/InverseBindMatrices`. The primitive `MeshAsset` payloads preserve glTF morph
target position/normal/tangent displacement channels and attach node skin inverse bind matrices.
Weighted skinned glTF primitives also preserve authored `JOINTS_0`/`WEIGHTS_0` channels through
the root `ModelAsset` and labeled `MeshAsset` conversion path. Importer-side automatic Virtual
Geometry cooking/backfill skips those weighted primitives because the current VG ordinal channel
reuses joint-index slots; static, non-skinned primitives keep the existing automatic VG cook/backfill
path.
The split `gltf_importer` package now has a standalone distribution shape using
`zircon_plugin_gltf_importer_dist` as the native ABI v3 wrapper while glTF/GLB parsing, labeled
subassets, scene/material/skin/animation placeholder handling, descriptors, and runtime
registration stay in `gltf_importer/runtime`.
Animation channel import reads translation, rotation, and scale samplers with step, linear, or cubic
spline interpolation, fills missing TRS channels from bind pose, and rejects morph-target animation
channels until the runtime has a typed morph animation target. Synthetic no-skin skeletons are built
from animated target nodes plus their ancestors so node-transform character packs such as the
Kenney vampire assets can be referenced by graph/state-machine assets immediately.
Multi-primitive mesh fixtures now assert that the root model, `Mesh{n}` model subasset, each
`Mesh{n}/Primitive{p}` `MeshAsset`, and each primitive material dependency stay aligned when one
glTF mesh carries more than one material-backed primitive.
Multi-scene fixtures now assert that `Scene0` and `Scene1` are emitted as separate `SceneAsset`
subassets, each scene keeps only its own root node dependency, and the generated scene entity rows
bind back to the shared `Mesh0` model subasset and `Material0` material label.

Runtime 15 M4 glTF labeled material subasset owner split is recorded as `runtime_15_gltf_labeled_material_subasset_owner_split_static_passed_cargo_deferred`. `asset/importer/ingest/gltf_labeled_subassets.rs` now keeps the texture, mesh, and scene labeled subasset entry points, scene dependency collection, shared material URI/reference resolution, root dependency insertion, and label URI/reference helpers. `asset/importer/ingest/gltf_labeled_subassets/material.rs` owns `add_gltf_material_subassets(...)`, default material generation, PBR material projection, texture-slot metadata, KHR_texture_transform projection into `RenderMaterialTextureTransform`, glTF alpha mode mapping, and the default PBR shader reference. Guard `runtime_15_gltf_labeled_material_subassets_are_child_owner` locks those owner boundaries, the two production files staying below the Runtime 15 soft budget, and the Runtime 15/status/importer/module documentation mirrors. This split does not change Bevy-style glTF label names, `Material{n}`/`DefaultMaterial` output shape, texture dependency collection, texture transform metadata, default shader locator, or scene/mesh material reference semantics.

External texture fixtures now assert that a glTF image URI becomes a decoded `Texture0`
subasset and that the material `base_color_texture` locator points at that label. Both the
runtime fixture importer and the split glTF plugin also preflight external buffer URIs from the
source bytes before calling the lower-level glTF loader, so a missing buffer reports the named
URI and buffer index instead of only returning a platform file-open error.
Triangle-list glTF primitives are the supported model/mesh path; non-triangle primitive modes such
as lines now fail with a parse diagnostic before any root model or labeled mesh subasset is emitted. The labels intentionally match
Bevy's glTF label vocabulary (`dev/bevy/crates/bevy_gltf/src/label.rs` and
`dev/bevy/crates/bevy_gltf/src/assets.rs`) while using Zircon's existing neutral asset payloads
instead of introducing Bevy-specific glTF wrapper types.

All texture paths now emit an explicit `TextureAssetDescriptor`, and the same import settings table
can override `format`, `color_space`, `dimension`, `depth_or_array_layers`/`depth`, `usage`,
`asset_usage`, `mip_count`, `array_layer_count`/`array_layers`, and partial sampler address/filter
modes. For 1D/2D textures the depth-or-array-layers and array-layer fields are normalized together:
setting either one updates the other, while setting both to different values is rejected. For 3D
textures, `depth_or_array_layers` remains native depth and explicit multi-layer array settings are
rejected. The parser accepts Bevy `ImageLoaderSettings` aliases `texture_format` for render texture
format and `is_srgb` for sRGB/linear color interpretation while preserving Zircon's existing
`format` and `color_space` names. It also accepts `sampler = "linear"` and `sampler = "nearest"`
as Bevy `ImageSamplerDescriptor::linear()`/`nearest()` shorthands, setting mag/min/mipmap filters
together while preserving the default clamp-to-edge address modes
(`dev/bevy/crates/bevy_image/src/image.rs:856` and
`dev/bevy/crates/bevy_image/src/image.rs:867`). `asset_usage` accepts either a single residency
token such as `"render_world"` or an array of tokens, matching Bevy's single
`ImageLoaderSettings.asset_usage` role while keeping Zircon's explicit serialized residency list.
Invalid Bevy-alias settings report the actual key that failed, including `texture_format`,
`is_srgb`, `sampler`, and `render_asset_usage`.

The import-settings entry is intentionally named `apply_import_settings(...)` because it can fail
while parsing and normalizing authored settings; it is not a builder-style `with_*` chain point.
`review_f8_texture_import_settings_use_fallible_apply_not_with` keeps the runtime importer and
first-party texture importer plugin on that contract. Status: F8 texture import settings apply API /
`texture_import_settings_apply_api_coremin_check_passed`; RuntimePluginDescriptor test fixture migration
remains pending in the broader E3/F8 cleanup.
Runtime 15 M4 texture descriptor settings parser owner split is recorded as
`runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred`.
`asset/assets/texture/descriptor.rs` keeps `TextureAssetDescriptor`, `TextureArrayLayout`,
`apply_import_settings(...)`, extent normalization, and render descriptor projection, while
`asset/assets/texture/descriptor/settings.rs` owns TOML parser helpers for usage tokens, asset
usage tokens, sampler tables/shorthands, array layout, color space, dimension, and Bevy-style token
normalization. Guard `runtime_15_texture_descriptor_settings_parser_is_child_owner` locks the
parent/child boundary, moved parser helper ownership, the 800-line production-file budget, and the
Runtime 15/status/importer/render-assets/module documentation mirrors. This split keeps the F8
fallible apply API and importer diagnostics unchanged.
The runtime fixture tests for this texture source-format, descriptor, and `[array_layout]` behavior
are split into `zircon_runtime/src/asset/tests/assets/texture_importer.rs`; the generic
`importer.rs` module stays focused on registry routing plus non-texture fixture contracts.
The same default importer now owns a narrow built-in `.cube` LUT route through
`zircon.builtin.texture.cube_lut`. It decodes UTF-8 `.cube` files, parses `LUT_3D_SIZE`, accepts
common metadata rows such as `TITLE`, `DOMAIN_*`, and LUT input/output range declarations, rejects
1D shaper sections explicitly until shaper-aware LUT baking is designed, requires exactly `size^3`
RGB samples, and emits a linear `rgba8unorm` 3D `TextureAsset` with clamp/linear
sampler metadata. This keeps post-process LUT authoring on the normal asset pipeline and capability
report path instead of routing `.cube` through a plugin-only image backend or renderer-private WGPU
code.
Decoded RGBA8 image textures also accept Bevy-style
`[array_layout] row_count = N` or
`row_height = pixels` settings: the importer reinterprets a vertical 2D stack as a 2D array texture
by keeping the bytes in place, reducing the stored texture height to one layer, and setting
`array_layer_count` plus `depth_or_array_layers` to the layer count. Invalid zero, non-divisible, or
already-layered layouts fail with parse diagnostics before artifact output. The `dimension` field
accepts 1D/2D/3D tokens and defaults to 2D for old artifacts, matching the existing image decode path
while allowing container and future volume texture importers to advertise the intended render
contract. `depth_or_array_layers` mirrors Bevy's
`Extent3d.depth_or_array_layers`: for 1D/2D arrays it is the parsed layer/face count, and for 3D
textures it is native depth. The `asset_usage` field accepts main-world/render-world residency
tokens and defaults to both, mirroring Bevy's default `RenderAssetUsages` without changing GPU
texture usage flags. DDS defaults to 2D and parses DX10 array/cubemap layer counts, while KTX1,
KTX2, and ASTC header parsing now derives 1D/2D/3D descriptor dimensions from their native header
fields before any import-setting override is applied. For 3D texture containers, native depth maps
to `depth_or_array_layers` while `array_layer_count` remains one even if a malformed KTX header also
sets layer/face counts. Container imports keep compressed
payload bytes in `TexturePayload::Container` even when descriptor settings override render-facing
format, sampler, or residency fields. `[array_layout]` remains decoded-image-only for container
imports and fails with a parse diagnostic before any compressed payload can be misrepresented as an
RGBA stack. Broken DDS, KTX1, KTX2, and ASTC header checks return format-specific parse diagnostics,
and DDS/KTX layer-face products use checked `u32` arithmetic so malformed array counts become parse
errors instead of panic or wraparound behavior. This keeps container failure reporting stable even
when no GPU upload backend is available yet.
The BMP/TGA/TIFF/GIF/WebP/HDR/EXR/QOI/PNM matrix is covered on both the runtime fixture importer
and the split plugin importer, using float image fixtures for the high dynamic range formats. The
PSD path flattens through the `psd` crate and then applies the same descriptor override table as the
image crate formats, so `texture_format`, `is_srgb`, `sampler`, and `asset_usage` remain consistent
across decoded image importers. This mirrors Bevy's `ImageLoaderSettings` role while keeping Zircon's
neutral `RenderImageDescriptor` contract as the runtime-facing output. The split `audio_importer`
package decodes WAV directly and decodes MP3/OGG/Vorbis/FLAC/AIFF/AIF through Symphonia into
`SoundAsset` f32 PCM. Runtime 15 F5 sound asset typed errors
(`runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred`) keep the built-in WAV parser on
`SoundAssetError` / `SoundAssetResult` through `asset/assets/sound.rs`; the importer-facing
`AssetImportError::Parse` boundary only formats the typed error display text with the source path,
and `review_f5_sound_asset_uses_typed_error` guards that split. Its standalone distribution shape uses `zircon_plugin_audio_importer_dist`
as the native ABI v3 wrapper while keeping WAV/Symphonia decode, descriptors, and runtime
registration in `audio_importer/runtime`. Opus
now has a split `opus_importer` package that owns the `.opus` `SoundAsset` importer slot and
NativeDynamic/libopus command contract; importing still requires an installed native backend, and
missing backend cases remain stable importer errors. The `asset_importer.audio` family package now
has a standalone distribution shape using `zircon_plugin_asset_importer_audio_dist` as the native
ABI v3 wrapper while keeping audio importer descriptors and runtime registration in the runtime
module. The `asset_importer.texture` family package likewise has a standalone distribution shape
using `zircon_plugin_asset_importer_texture_dist` as the native ABI v3 wrapper while keeping
image/container/PSD descriptors and runtime registration in the runtime module. The
`asset_importer.shader` family package now owns a real Naga path for WGSL validation plus
GLSL/vertex/fragment/compute and SPIR-V conversion into normalized WGSL `ShaderAsset` payloads. Its
standalone distribution shape uses `zircon_plugin_asset_importer_shader_dist` as the native ABI v3
wrapper while keeping WGSL/Naga parsing and importer registration in the runtime module. The split
`ui_document_importer` package imports only `.zui` component documents and emits
`UiV2ComponentAsset` payloads. The `.ui.toml` source-template migration path, pre-`.zui`
`.v2.ui.toml` view/style/component importer, and serialized `.ui.json`/`.uidoc` `UiAssetDocument`
paths are not production plugin importers anymore; migration coverage must install explicit
source-template test fixtures.

Heavy or toolchain-backed formats are registered as diagnostic importers until a plugin backend is installed. This includes FBX/DAE/3DS/USD-family model containers, cubemap/DXGI texture authoring formats, and HLSL/CG/FX shader toolchains. Text `.cube` LUTs are no longer part of that diagnostic-only bucket: the built-in parser covers the neutral 3D RGBA8 LUT asset shape, while advanced LUT authoring policies such as half-float payloads, shaper LUTs, or GPU-baked LUT generation remain later work. The Opus split package uses the same diagnostic path when its NativeDynamic/libopus backend is absent. DXF linework, curves, blocks, and solid-kernel BREP payloads are still outside the Rust DXF mesh-surface backend. First-wave plugin-required diagnostics follow the same stable error-record path when the corresponding split plugin is absent.

`TextureAsset` keeps the existing RGBA8 payload and the container payload used by DDS/KTX/KTX2/ASTC import paths. The optional descriptor field is backward-compatible: old artifacts without it derive render metadata from `TexturePayload`, while newly imported assets store the descriptor explicitly for diagnostics, support queries, and render prepare. Container payloads are not decoded into RGBA by the importer; the render preparation layer decides whether the current GPU feature set can upload the compressed format or should emit a deterministic fallback diagnostic. `ShaderAsset` records source language, original source, normalized WGSL source, entry points, and validation diagnostics. `DataAsset` preserves source text and canonical JSON for TOML, JSON, YAML, and XML data. XML is normalized into a stable element tree JSON object with element name, optional namespace, attributes, text, and children.

## Project Scan Behavior

`ProjectManager::scan_and_import` now processes every source file independently. A successful import validates that the outcome has exactly one unlabeled root entry, rejects duplicate subasset labels, writes one artifact per entry, updates `.zmeta` with `source_digest`, import settings hash, importer id/version, root artifact locator, labeled `entries`, dependency locators, schema migration details, and `preview_state = ready`, then publishes ready `ResourceRecord` rows for the root and each subasset. Each entry has its own persistent UUID, and `ResourceId` is derived from that UUID instead of from the source UUID plus label.

The importer also exposes `import_context(&AssetImportContext)` so project scanning can pass the same source-byte owner to the selected plugin and post-import staging without cloning large HDR files. After a successful first import, and again after restoring a ready cached asset, the scan invokes both source-format staging entries. `stage_environment_ibl_source(...)` converts HDR/EXR 2:1 images; `stage_external_source_cubemap_texture(...)` converts supported linear-float DDS/KTX cubemap payloads. Both produce or validate current `.zcube` source mips and a companion PMREM/SH9/IEM `.zribl` under `.zircon/cache`. External source mips are preserved for display/FIS input but PMREM is always regenerated. This post-import host step intentionally leaves the plugin ABI neutral: texture plugins decode the ordinary asset/container, while the shared runtime importer owns render-derived files and their current algorithm key. Invalid explicit environment settings and unsupported compressed/supercompressed cubemap formats are reported through the normal failed-import diagnostic path; automatic HDR/EXR mode skips non-equirectangular images.

Runtime 15 M4 asset project scan/import source collection owner split is recorded as `runtime_15_asset_project_scan_import_sources_owner_split_static_passed_cargo_deferred`. `asset/project/manager/scan_and_import.rs` now keeps only import-loop orchestration, artifact restore/writeback, success/failure transitions, shader diagnostics, and registry publication. `scan_and_import/sources.rs` owns project/package enumeration and compound-source byte/mtime assembly; `scan_and_import/dependency_resolution.rs` owns locator-to-id resolution plus handwritten dependency merging; `scan_and_import/metadata.rs` owns importer-contract validation, schema-migration cleanup, stable entry UUID/tag projection, failed-entry preservation, URL remapping, and settings hashing. This follows the Bevy-style split between processor orchestration and source/meta concerns while retaining Zircon's existing runtime/editor boundary. Guard `runtime_15_asset_project_scan_import_sources_are_child_owner` locks all four production owners below the Runtime 15 soft budget and reads completed status only from the Runtime 15 numbered output archive; it no longer requires plan definitions, indexes, review overviews, session notes, or generated status/date mirrors to duplicate the same completion record.

Runtime 15 M3 asset project manager lock poison recovery is recorded as `runtime_15_asset_project_manager_lock_poison_recovery_static_passed_cargo_deferred`. `asset/pipeline/manager/project_asset_manager/runtime.rs` now centralizes project, pending importer registry, change subscriber, watch-error subscriber, and watcher lock access behind poison recovery helpers. `asset/pipeline/manager/project_asset_manager/construction.rs` consumes the importer registry helpers when registering late plugin importers and when cloning pending importers into an active registry. Guard `runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager` locks those helper names, rejects direct lock unwrap/RwLock expect/`lock poisoned` production regressions, and mirrors the Runtime 15/status/module documentation anchors. This keeps ProjectAssetManager API, project scan/import flow, watcher-driven reimport, importer extension handoff, and resource sync behavior unchanged.

`ArtifactStore` treats `lib://` locators as logical identifiers for processed runtime cache rather than authoring documents. Every stored imported entry uses the `.zasset` extension under its asset-family directory rooted at `ProjectPaths::asset_artifact_root()` (`<project>/.zircon/cache/assets`), and the payload is `ZRARTZ01` followed directly by zstd level-1 compressed bincode bytes for an internal `ArtifactCacheAsset` wire type. That wire type converts authoring-friendly shapes such as material `toml::Value` override maps, shader TOML editor metadata/defaults, material texture slots with flattened authoring fields, and flattened physics material metadata into bincode-stable structs before compression. The artifact cache deliberately does not write source-format `.json`, `.toml`, or other readable authoring files and no longer preserves JSON or legacy format-marker read branches; readability stays in source assets, importer metadata, and `.zmeta`, while `.zircon/cache/assets` is optimized for restore speed and compact I/O. Reads reject non-`.zasset` locators, missing magic, failed decompression/deserialization, invalid cached TOML datetime payloads, and payload/path kind mismatches, so stale text artifacts fall back to normal source import instead of keeping an old cache format alive.

Project scan coverage locks the same rule at the real write path. The dedicated binary artifact-cache test asserts that model, mesh, scene, shader, material, and texture registry records all use logical `lib://...zasset` locators, that physical files under `.zircon/cache/assets` start with the `ZRARTZ01` binary-cache magic, that decompressed payloads are not JSON documents and do not carry `JSON` or `BIN\0` compatibility markers, and that every cached artifact uses the `.zasset` extension. The manager-level physics/animation, sound, and restart-restore tests keep the same binary-cache assertions for non-render and restore paths.

Project scanning also walks every registered `PackageAssetRegistry` root after the project `assets/` root. Project files keep `res://` locators; package files are converted to `package://{package_id}/...` locators from the package asset root, then enter the same importer, artifact, `.zmeta`, dependency, and registry flow. `source_path_for_uri(package://...)` maps back through the package registry, rejects unknown packages, and relies on locator/package-root validation so package paths cannot escape the registered root. Package subassets keep the package id when entry URLs are remapped during restore or failed reimport, so `package://com.zircon.navigation/bundles/atlas.multi#Texture0` remains a UUID-addressable record and artifact across restarts.

If an importer is missing, unsupported, malformed, or fails validation, the scan writes meta with the same source digest and importer identity when known, sets `preview_state = error`, and registers `ResourceState::Error` with diagnostics. The live registry only publishes the failed root record, but `.zmeta.entries` preserves prior root/subasset UUID rows with cleared artifact locators so transient failures do not break saved subasset references after a later successful reimport. The next source file continues importing. Runtime resource sync registers error records without trying to load a missing artifact.

Editor catalog sync mirrors the same contract. `DefaultEditorAssetManager::sync_from_project` keeps failed assets visible in the catalog, carries their diagnostics, and leaves direct-reference edges empty instead of calling `load_artifact_by_id` on records that have no artifact locator. This keeps missing-plugin and parse-error assets inspectable without blocking editor manager startup.

Runtime meta documents are `.zmeta` format version 7. The schema uses `uuid`, `url`, `asset_kind`, `unit`, `included_files`, importer metadata fields, `artifact_locator`, `config_hash`, `source_digest`, root dependencies, and per-entry `uuid/url/asset_kind/artifact_locator/dependencies`. The parser accepts only v7, rejects v6 as `UnsupportedOldFormatVersion`, rejects future input as `UnsupportedFutureFormatVersion`, and reports the retired `source_hash` key as `RetiredSourceHashField`; no serde alias or in-place legacy migration remains.

Meta saves validate and serialize before filesystem mutation, then use a unique same-directory staging file with `write_all`, `flush`, and `sync_all`. Replacement keeps the prior sidecar continuously visible: Windows uses one `ReplaceFileW` call with a backup path, while Unix creates a hard-link/copy backup before same-directory rename-overwrite. Injected or OS commit failure leaves the original target readable and cleans transaction files. Backup cleanup after the successful commit point is best-effort, so a durable new sidecar is never reported as a failed save solely because obsolete-backup removal failed.

Project-wide authoring migration uses durable intent journals under `.zircon/asset-migration`. A journal filename retains the complete reserved sibling identity `.SOURCE.zr-migrate-journal-{transaction_id}` and then appends `.toml`; `.toml` is a serialization suffix, not a replacement extension. Recovery strips only that final suffix and verifies the remaining filename ends with the transaction id stored in the journal before it trusts any staged or backup evidence. This makes filename identity and document identity mutually authenticating while preserving the hard rule that recovery never restores untrusted backup bytes into a live asset. The crash-window regression asserts both the reserved filename shape and successful convergence on the next apply.

Runtime 15 F5 asset meta typed errors (`runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred`) remain typed in `asset/project/meta.rs`: `AssetMetaDocument::from_toml_str(...)` returns `AssetMetaResult`, current-version validation distinguishes old, future, retired-field, and deserialize failures, and `AssetMetaDocument::load(...)` stringifies only at the `std::io::ErrorKind::InvalidData` filesystem boundary. `review_f5_asset_meta_uses_typed_error`, `asset_meta_validation_reports_typed_future_version_error`, and `asset/tests/project/zmeta/schema_v7.rs` lock the current contract.

Ready meta can now restore an already-imported artifact after editor restart without rerunning the importer. The restore path requires `preview_state = ready`, unchanged source digest, unchanged import settings hash, a matching importer id/version contract when the importer is present, and a readable artifact at `artifact_locator`. It remaps every entry URL to the current source URI before building `ResourceRecord` rows, preserving UUID identity while allowing source files and their `.zmeta` sidecars to move together. This keeps model, texture, material/data, scene, and UI document imports stable across restarts even when only the artifact store and meta are available. If the artifact is missing, the source/config changed, or the importer contract no longer matches, the project scan falls back to a normal import attempt and rewrites meta from the fresh result.

Successful imports now clear stale schema migration fields when the selected importer does not
return a migration report. Failed imports clear the same fields before recording error state, so an
old upgraded asset does not leave misleading schema metadata on a later non-migrating or failed
import.

The split `ui_document_importer` runtime package routes `.zui` TOML through `UiZuiAssetLoader`.
The importer descriptor and package `plugin.toml` expose `ui_document_importer.zui_document`
for `.zui` with importer version 2, primary `UiWidget` output, and additional UiLayout/UiStyle
outputs. The shared `ui_v2_document_import.rs` owner maps `asset.kind` to component, view, or
style payloads after the `.zui` profile accepts the matching document shape. `.ui.toml` and
`.v2.ui.toml` source-template suffixes are intentionally absent from production registration so
they cannot silently route through `UiAssetLoader`, the recursive `UiAssetDocument` migration
chain, or a second production kind-mapping path. `.ui.json` and `.uidoc` are also absent from
production registration; the plugin no longer depends on `serde_json` or `bincode` for UI
document import.

Standalone theme and icon documents are the exceptions for UI TOML source ingestion. The built-in
`zircon.builtin.ui_theme.toml` importer owns the `.theme.toml` full suffix, parses the source as
`UiThemeAsset`, and stores it as an `ImportedAsset::UiTheme` payload under `AssetKind::UiStyle`.
The built-in `zircon.builtin.ui_icon.toml` importer owns the `.icon.toml` full suffix, parses the
source as `UiIconAsset`, and stores it as an `ImportedAsset::UiIcon` payload under
`AssetKind::Texture`. Icon documents can point at external SVG or bitmap resource locators, and
those locators become normal direct dependency edges before later atlas/rasterization work consumes
the texture-family handle. This keeps theme hot-reload, icon hot-reload, and dependency-index work
on the shared asset pipeline without reclassifying existing `editor_*.v2.ui.toml`
theme-token/style files.

`ProjectAssetManager` keeps a host-owned importer registry for plugin contributions that arrive
before a project is open. `RuntimeExtensionRegistry::apply_asset_importers_to_project_asset_manager`
installs those handlers into that pending registry, and `open_project` applies the registry to the
fresh `ProjectManager` before `scan_and_import` runs. The `AssetManager` service trait now forwards
the same importer capability report helpers, so host/editor tools can ask the service boundary which
importer will handle a source both before a project is opened and after the pending registry is
installed into the active project. This gives linked plugin importers the same first-scan authority
as built-in importers without making `zircon_runtime` depend on `zircon_plugins`.

The built-in `AssetModule` can also carry an `AssetImporterRegistry`. Runtime module load from
plugin registration reports merges active plugin and feature importer handlers into that registry
and constructs the project asset manager with those pending handlers already installed. This closes
the lifecycle gap between catalog selection and the first project scan for linked Rust plugins.

## Plugin Boundary

Frameworks 03 now treats heavy built-in import backends as compile-time domain capabilities. Font source decoding/metadata parsing and the `.font.toml` importer are present only with `text`; Naga shader validation, shader package import, and GLSL/SPIR-V registration are present only with `graphics`. These gates live at child-module declarations and importer-registry assembly sites. `target-server` does not link ttf-parser, woff2-patched, naga, or the graphics/text stack, while the default client check still compiles the complete importer set.

`RuntimeExtensionRegistry` now owns an `AssetImporterRegistry` alongside modules, managers, components, and render extensions. Rust plugins can register real importer handlers. Manifest-only and NativeDynamic declarations can register diagnostic descriptors until a backend is attached.

Applying importer extensions to `ProjectAssetManager` is intentionally host-side. The extension
registry does not open projects, inspect asset files, or write artifacts; it only transfers
capability-gated handlers into the asset manager. If a plugin registers after a project is already
open, the manager preflights the active project registry before accepting the handler into the
pending registry, then installs it into the current project so manual reimport and watcher-driven
reimport can use it immediately.

NativeDynamic importers use the `runtime.asset.importer.native` capability and the `asset.import/<importer_id>` command. The ABI payload is a `ZRIMP001` request envelope containing metadata JSON and raw source bytes. Native code returns a `ZRIMO001` response envelope with a neutral import DTO, dependency locators, optional schema migration reports, and diagnostics. The host validates command status before reading a success payload, so denied, panic, and error statuses preserve native diagnostics even when no output payload is returned. Successful responses then validate importer id, output kind, and malformed buffers before mapping dependency locators, migration reports, and diagnostics into `ImportedAssetEntry` and writing artifacts.

The response validation path is factored separately from dynamic-library invocation, so envelope
decode tests can cover malformed magic, reserved artifact bytes, mismatched importer id, wrong
output kind, command status diagnostics, missing success payloads, declared dependency preservation,
schema migration report propagation, and diagnostic conversion without requiring a native DLL
fixture.

The `native_dynamic_fixture` cdylib now also exposes
`asset.import/native_dynamic_fixture.data_json` in its command manifest. The fixture decodes the
same `ZRIMP001` envelope that production NativeDynamic importers receive, validates the requested
importer id, parses JSON source bytes, and returns a `ZRIMO001` response carrying a neutral
`DataAsset`, a per-entry schema migration report, and diagnostics.
`zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs` now delegates the host-side
fixture test coverage to `zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs`.
That child owner loads the real DLL, routes it through `NativeAssetImporterHandler`, and asserts that
the native response migration report survives as `ImportedAssetEntry::migration_report`; the parent
keeps load-manifest discovery coverage. Runtime 15 records this as
`Runtime 15 M3 native plugin loader real fixture test folder split` /
`runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred`, with
`runtime_15_native_plugin_loader_real_fixture_tests_are_folder_backed` preventing the importer fixture
test from flowing back into the parent. Cargo remains deferred for this structure slice while external
runtime cargo/rustc lanes are active.

## Frameworks 02 M3 Opus Importer Manifest Assertion Hard Cutover

The Opus split importer package manifest test now follows the current runtime-plus-dist module
contract. `zircon_plugins/opus_importer/runtime/src/lib.rs` locates the `opus_importer.runtime`
module by name instead of asserting a single manifest module, so the native dist module can remain
declared in the same package manifest. This keeps the runtime importer contract aligned with the
standalone distribution rollout and avoids reintroducing the old single-module package shape.

Validation for this slice is scoped: `rustfmt --edition 2021 --check
zircon_plugins\opus_importer\runtime\src\lib.rs` passed, the remaining
`manifest.modules.len()` scan hit is the SDK single-module builder example, and focused
`cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_opus_importer_runtime
package_manifest_declares_opus_importer_dist_contract --locked --jobs 1 --target-dir
E:\cargo-targets\zircon-plugin-workspace-test-0703-codex-rerun2 --message-format short --color
never -- --nocapture --test-threads=1` passed 1/1. The full plugin workspace test execution
was subsequently covered by the segmented default-member gate:
`segment1-critical-fixes-rerun2.status.json` covered the Opus runtime package with ExitCode 0, and
the six segment status files under `E:\cargo-targets\zircon-plugin-workspace-segment-0703-codex`
cover all 120 `zircon_plugins` default workspace members with `MissingCount=0` and `ExtraCount=0`.

## Frameworks 02 M3 Texture Importer DDS Caps2 Diagnostic Hard Cutover

The texture importer DDS dual-cubemap regression now follows the current container diagnostic
contract. `zircon_plugins/texture_importer/runtime/src/container/tests/dds.rs` expects the
`DDSCAPS2_CUBEMAP caps2 policy` message produced by the DDS parser instead of the retired
`legacy caps2` wording. This keeps the test aligned with the current explicit caps2 policy and does
not add a parser fallback or old diagnostic alias.

Validation for this slice passed focused status
`E:\cargo-targets\zircon-plugin-workspace-segment-0703-codex\texture-importer-dds-dual-cubemap-focused.status.json`
with ExitCode 0. The follow-up `segment2-importers-dist-rerun2.status.json` gate also passed with
ExitCode 0, covering the importer/dist package segment and confirming
`zircon_plugin_texture_importer_runtime` at 144/144 tests.

## Runtime 15 Shader Schema Diagnostic Hard Cutover

The `.zshader` v2 importer reports old documents by their explicit schema transition. A missing `kind` is diagnosed as a `schema v1` document that must move to schema v2; removed user-authored pipeline-layout and shader-def fields are identified as removed fields that must move to generated ABI/options. No old parser, alias field, or compatibility path is retained.
