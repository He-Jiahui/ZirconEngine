---
related_code:
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/schema.rs
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_zui_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/assets/data.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/texture/payload.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/core/framework/render/image/descriptor.rs
  - zircon_runtime/src/core/framework/render/image/asset_usage.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/asset/assets/shader.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_asset_manager.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/asset_importer_install.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container.rs
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
  - zircon_runtime/src/asset/importer/ingest/import_ui_zui_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/texture/payload.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
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
  - zircon_plugins/asset_importers/model/runtime/Cargo.toml
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/cad.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/asset_importers/data/runtime/Cargo.toml
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/runtime/Cargo.toml
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/Cargo.toml
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/Cargo.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container.rs
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
  - user: 2026-05-03 Opus/libopus NativeDynamic importer gap
  - user: 2026-05-16 continue Bevy-style asset/image completion toward M4
  - user: 2026-05-20 implement ZirconEngine asset/texture/model/ZShader/ZMaterial/ZMesh completion plan
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/Zircon UI .zui 组件资产与 Unreal 风格入口重构计划.md
  - docs/superpowers/specs/2026-05-03-opus-native-dynamic-importer-design.md
  - docs/superpowers/plans/2026-05-03-opus-native-dynamic-importer.md
tests:
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked asset::tests::project::package_assets --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 3 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --locked package --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed after warm cache, 43 package-filtered runtime lib tests plus package-filtered integration binaries)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-asset-package-m2 cargo test --manifest-path zircon_plugins/Cargo.toml --locked --jobs 1 --message-format short --color never package -- --test-threads=1 (2026-05-20 package roots M2: passed after moving off full D: target dir)
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs
  - zircon_runtime/src/asset/tests/assets/render_product.rs
  - zircon_runtime/src/asset/importer/native.rs::native_import_response_preserves_schema_migration_report
  - zircon_runtime/src/asset/importer/native.rs::native_import_command_errors_preserve_status_diagnostics_without_payload
  - zircon_runtime/src/asset/importer/native.rs::native_import_command_requires_payload_only_after_ok_status
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs::native_loader_fixture_can_import_data_asset_through_native_importer_handler
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
  - zircon_runtime/src/asset/tests/assets/importer.rs::importer_emits_bevy_style_gltf_labeled_subassets
  - CARGO_TARGET_DIR=/tmp/zircon-gltf-m4-wsl-fast cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels: blocked before test execution by unrelated zircon_runtime_interface/src/ui/dispatch/navigation/result.rs E0277, UiBindingUpdateReport does not implement Eq)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels: Windows attempt timed out after 304s before Rust test diagnostics; matching residual Cargo child processes were stopped)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --message-format short --color never (2026-05-20 runtime glTF labels retry: passed, confirming the earlier WSL Eq error is not present in the current Windows source tree)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels retry: passed, 1 passed, after replacing the invalid fixture PNG data URI with a valid CRC 1x1 RGBA PNG)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF animation/skin labels: passed, 1 passed, after extending the fixture with Animation0, Skin0, and Skin0/InverseBindMatrices placeholder labels)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels final: passed, 1 passed, 1720 filtered out, after warming the runtime test harness and restoring the top-level WGSL capture facade export)
  - zircon_plugins/gltf_importer/runtime/src/lib.rs::importer_decodes_triangle_gltf_into_model_asset
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib importer_decodes_triangle_gltf_into_model_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF plugin labels retry: passed, 1 passed, after the same fixture PNG replacement)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF plugin animation/skin labels: passed, 3 passed, after extending the fixture with Animation0, Skin0, and Skin0/InverseBindMatrices placeholder labels)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --lib importer --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 model plugin subasset labels: passed, 5 passed, covering STL/PLY/DXF root dependencies and Mesh0/Primitive0 MeshAsset payloads)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime --lib obj_importer_decodes_model_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 OBJ plugin subasset label: passed, 1 passed, covering root dependency and Mesh0/Primitive0 MeshAsset payload)
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container.rs
  - zircon_plugins/texture_importer/runtime/src/container.rs::dds_dx10_container_importer_reads_cubemap_array_layers
  - zircon_plugins/texture_importer/runtime/src/container.rs::ktx1_3d_container_keeps_depth_separate_from_array_layers
  - zircon_plugins/texture_importer/runtime/src/container.rs::astc_container_importer_reads_3d_block_and_depth
  - zircon_plugins/texture_importer/runtime/src/container.rs::ktx2_3d_container_keeps_depth_separate_from_array_layers
  - zircon_plugins/texture_importer/runtime/src/container.rs::container_importer_applies_descriptor_settings_without_expanding_payload
  - zircon_plugins/texture_importer/runtime/src/container.rs::container_importer_rejects_array_layout_without_decoded_rgba
  - zircon_plugins/texture_importer/runtime/src/container.rs::container_importer_reports_layer_count_overflow_diagnostics
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::render_asset_usage_alias_accepts_single_token
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::depth_or_array_layers_updates_array_layer_count_for_2d_arrays
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::array_layer_count_updates_depth_or_array_layers_for_2d_arrays
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::mismatched_2d_extent_settings_report_error
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::dimension_3d_rejects_multiple_array_layers
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::dimension_3d_keeps_depth_and_single_array_layer
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::import_extent_override_replaces_existing_2d_container_layers
  - zircon_runtime/src/asset/assets/texture/descriptor.rs::bevy_alias_diagnostics_report_actual_setting_keys
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
  - passed: cargo test -p zircon_runtime --lib importer_registry_rejects_non_fixture_legacy_ui_toml_importer_registration --jobs 1 --target-dir target\codex-ui-v2-guard
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
  - 2026-05-03: cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed again after gating legacy first-wave helper modules as test-only; existing runtime warnings only)
  - 2026-05-03: cargo test -p zircon_runtime importer_decodes_obj_and_gltf_into_model_assets --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (passed, 1 fixture-backed test, with existing runtime warnings)
  - 2026-05-03: cargo test -p zircon_runtime runtime_extension_registry_installs_asset_importers_before_project_open --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-lib-importer-contract --message-format short --color never (timed out after 10 minutes during Windows test build/link while other Cargo jobs were active; no Rust diagnostics returned)
  - passed: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_ui_document_importer_runtime --lib typed_toml_importer_decodes_ui_v2_view_asset --jobs 1 --target-dir target\codex-ui-v2-plugin-guard
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
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
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

Plain `.toml` is a `DataAsset`. Typed `*.xxx.toml` requires a registered full-suffix importer; unknown typed TOML fails as an error resource instead of silently becoming a generic data file. The registry now rejects legacy `.ui.toml` and broad `.v2.ui.toml` importer descriptors on the production path, so plugin manifests cannot reintroduce either the old recursive UI schema or the pre-`.zui` mixed view/component/style UI v2 importer. Only explicit unit-test migration fixtures are allowed to register those matchers for schema migration coverage.

## Built-In Coverage

The production default importer registry installs real Rust paths for runtime-core formats only: plain TOML/JSON data, `.zui` UI component documents, typed Zircon source assets such as `.zmaterial`, `.zshader`, `.zmesh`, material/font/model/physics material/scene/prefab/authoring navigation assets, animation `.zranim` contracts that have not yet moved fully to the animation plugin, and the remaining GLSL/SPIR-V shader paths. It no longer decodes the first-wave independent plugin formats directly.

Common image textures, WGSL, OBJ, glTF/GLB, and WAV now register diagnostic-only `zircon.plugin_required.*` descriptors by default. These descriptors preserve output kind, matcher, importer version, and capability metadata so scans produce stable error records when a plugin is disabled or missing, but they do not perform decoding in production runtime code. Legacy UI `.ui.toml` and `.v2.ui.toml` no longer register production plugin-required fallbacks, and `AssetImporterRegistry` rejects non-fixture matcher registration for both suffixes. They remain reachable only through exact migration fixtures used by unit tests. The real stable split backends live in `texture_importer`, `shader_wgsl_importer`, `obj_importer`, `gltf_importer`, and `audio_importer`, while `ui_document_importer` mirrors the `.zui` component payload path for plugin packaging.

Runtime tests that intentionally exercise these first-wave formats install explicit fixture importers with the same package ids and higher priority as the split plugin crates. The fixtures still call test-only legacy runtime helper modules so the runtime test crate can validate artifact/project behavior without taking a dev-dependency on `zircon_plugins`; the production default path is diagnostic-only. Graphics project-render and M4 behavior-layer tests now use that explicit fixture path for PNG/WGSL/OBJ projects, so the tests prove render behavior with installed importer plugins instead of silently reintroducing production built-in decoders.

The `asset_importer.data` runtime plugin now registers real TOML/JSON/YAML/XML `DataAsset`
backends so project/plugin selection can move structured data loading out of the built-in fallback
path. The `asset_importer.model` family plugin now registers real STL, PLY, and DXF model backends.
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
wiring. The split `texture_importer` package decodes common image formats to RGBA8 through the shared
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
override role (`dev/bevy/crates/bevy_image/src/image_loader.rs:122`).

The split `gltf_importer` package now emits Bevy-style labeled subassets while keeping the root
`ModelAsset.primitives` compatibility path. `runtime/src/lib.rs` still owns plugin registration and
primitive decoding, while `runtime/src/subassets.rs` owns labeled subasset expansion. The
runtime-only first-wave fixture mirrors the same label semantics in
`zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs` so project/meta tests that
cannot depend on `zircon_plugins` still exercise the public `AssetImportOutcome` shape. The importer
and fixture descriptors declare additional output kinds for mesh, scene, material, texture, and data, then emit
`Texture{n}` as `TextureAsset`, `Material{n}` and `DefaultMaterial` as `MaterialAsset`,
`Mesh{n}` as a mesh-local `ModelAsset`, `Mesh{m}/Primitive{p}` as first-class `MeshAsset`,
`Node{n}` and `Scene{n}` as `SceneAsset`, and diagnostic placeholder `DataAsset` rows for
`Animation{n}`, `Skin{n}`, and `Skin{n}/InverseBindMatrices`. The primitive `MeshAsset` payloads
preserve glTF morph target position/normal/tangent displacement channels and attach node skin
inverse bind matrices while standalone skin and animation labels stay diagnostic placeholders. The labels intentionally match
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
The runtime fixture tests for this texture source-format, descriptor, and `[array_layout]` behavior
are split into `zircon_runtime/src/asset/tests/assets/texture_importer.rs`; the generic
`importer.rs` module stays focused on registry routing plus non-texture fixture contracts.
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
neutral `RenderImageDescriptor` contract as the runtime-facing output. The split `audio_importer` package decodes WAV
directly and decodes MP3/OGG/Vorbis/FLAC/AIFF/AIF through Symphonia into `SoundAsset` f32 PCM. Opus
now has a split `opus_importer` package that owns the `.opus` `SoundAsset` importer slot and
NativeDynamic/libopus command contract; importing still requires an installed native backend, and
missing backend cases remain stable importer errors. The
`asset_importer.shader` family package now owns a real Naga path for WGSL validation plus
GLSL/vertex/fragment/compute and SPIR-V conversion into normalized WGSL `ShaderAsset` payloads. The
split `ui_document_importer` package imports only `.zui` component documents and emits
`UiV2ComponentAsset` payloads. The older `.ui.toml` migration path, pre-`.zui` `.v2.ui.toml`
view/style/component importer, and serialized `.ui.json`/`.uidoc` `UiAssetDocument` paths are not production
plugin importers anymore; migration coverage must install explicit test fixtures.

Heavy or toolchain-backed formats are registered as diagnostic importers until a plugin backend is installed. This includes FBX/DAE/3DS/USD-family model containers, cubemap/DXGI texture authoring formats, and HLSL/CG/FX shader toolchains. The Opus split package uses the same diagnostic path when its NativeDynamic/libopus backend is absent. DXF linework, curves, blocks, and solid-kernel BREP payloads are still outside the Rust DXF mesh-surface backend. First-wave plugin-required diagnostics follow the same stable error-record path when the corresponding split plugin is absent.

`TextureAsset` keeps the existing RGBA8 payload and the container payload used by DDS/KTX/KTX2/ASTC import paths. The optional descriptor field is backward-compatible: old artifacts without it derive render metadata from `TexturePayload`, while newly imported assets store the descriptor explicitly for diagnostics, support queries, and render prepare. Container payloads are not decoded into RGBA by the importer; the render preparation layer decides whether the current GPU feature set can upload the compressed format or should emit a deterministic fallback diagnostic. `ShaderAsset` records source language, original source, normalized WGSL source, entry points, and validation diagnostics. `DataAsset` preserves source text and canonical JSON for TOML, JSON, YAML, and XML data. XML is normalized into a stable element tree JSON object with element name, optional namespace, attributes, text, and children.

## Project Scan Behavior

`ProjectManager::scan_and_import` now processes every source file independently. A successful import validates that the outcome has exactly one unlabeled root entry, rejects duplicate subasset labels, writes one artifact per entry, updates `.zmeta` with source hash, import settings hash, importer id/version, root artifact locator, labeled `entries`, dependency locators, schema migration details, and `preview_state = ready`, then publishes ready `ResourceRecord` rows for the root and each subasset. Each entry has its own persistent UUID, and `ResourceId` is derived from that UUID instead of from the source UUID plus label.

Project scanning also walks every registered `PackageAssetRegistry` root after the project `assets/` root. Project files keep `res://` locators; package files are converted to `package://{package_id}/...` locators from the package asset root, then enter the same importer, artifact, `.zmeta`, dependency, and registry flow. `source_path_for_uri(package://...)` maps back through the package registry, rejects unknown packages, and relies on locator/package-root validation so package paths cannot escape the registered root. Package subassets keep the package id when entry URLs are remapped during restore or failed reimport, so `package://com.zircon.navigation/bundles/atlas.multi#Texture0` remains a UUID-addressable record and artifact across restarts.

If an importer is missing, unsupported, malformed, or fails validation, the scan writes meta with the same source hash and importer identity when known, sets `preview_state = error`, and registers `ResourceState::Error` with diagnostics. The live registry only publishes the failed root record, but `.zmeta.entries` preserves prior root/subasset UUID rows with cleared artifact locators so transient failures do not break saved subasset references after a later successful reimport. The next source file continues importing. Runtime resource sync registers error records without trying to load a missing artifact.

Editor catalog sync mirrors the same contract. `DefaultEditorAssetManager::sync_from_project` keeps failed assets visible in the catalog, carries their diagnostics, and leaves direct-reference edges empty instead of calling `load_artifact_by_id` on records that have no artifact locator. This keeps missing-plugin and parse-error assets inspectable without blocking editor manager startup.

Runtime meta documents are `.zmeta` format version 6. The schema uses `uuid`, `url`, `asset_kind`, `unit`, `included_files`, importer metadata fields, `artifact_locator`, `config_hash`, root dependencies, and per-entry `uuid/url/asset_kind/artifact_locator/dependencies`. Future meta versions fail so the engine does not downgrade unknown schema, and old `*.meta.toml` files are ignored rather than treated as compatibility inputs.

Ready meta can now restore an already-imported artifact after editor restart without rerunning the importer. The restore path requires `preview_state = ready`, unchanged source hash, unchanged import settings hash, a matching importer id/version contract when the importer is present, and a readable artifact at `artifact_locator`. It remaps every entry URL to the current source URI before building `ResourceRecord` rows, preserving UUID identity while allowing source files and their `.zmeta` sidecars to move together. This keeps model, texture, material/data, scene, and UI document imports stable across restarts even when only the artifact store and meta are available. If the artifact is missing, the source/config changed, or the importer contract no longer matches, the project scan falls back to a normal import attempt and rewrites meta from the fresh result.

Successful imports now clear stale schema migration fields when the selected importer does not
return a migration report. Failed imports clear the same fields before recording error state, so an
old upgraded asset does not leave misleading schema metadata on a later non-migrating or failed
import.

The split `ui_document_importer` runtime package routes `.zui` TOML through `UiZuiAssetLoader`.
The importer descriptor and package `plugin.toml` both expose a single `ui_document_importer.zui_component`
entry for `.zui` with importer version 2 and `UiWidget` output. Legacy `.ui.toml` and `.v2.ui.toml` are intentionally absent from
production registration so they cannot silently route through `UiAssetLoader`, the recursive
`UiAssetDocument` migration chain, or the old mixed-kind v2 importer. `.ui.json` and `.uidoc` are also absent from production
registration; the plugin no longer depends on `serde_json` or `bincode` for UI document import.

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
`zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs` contains the host-side fixture
test that loads the real DLL, routes it through `NativeAssetImporterHandler`, and asserts that the
native response migration report survives as `ImportedAssetEntry::migration_report`; the source is in
place, while full runtime type/test validation is currently blocked by unrelated plugin catalog
compile errors from adjacent optional-feature work.
