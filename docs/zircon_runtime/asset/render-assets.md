---
related_code:
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
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container.rs
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_editor/src/ui/workbench/project/constants.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_ensure_runtime_assets.rs
  - zircon_editor/src/ui/workbench/project/runtime_asset_helpers.rs
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.zshader
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.wgsl
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources
  - zircon_runtime/tests/virtual_geometry_visibility_debug_contract.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/mod.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
implementation_files:
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
  - zircon_runtime/src/asset/importer/image_decode.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/container.rs
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/asset/assets/model/virtual_geometry.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/language.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/alpha_mode.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_editor/src/ui/workbench/project/constants.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_ensure_runtime_assets.rs
  - zircon_editor/src/ui/workbench/project/runtime_asset_helpers.rs
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.zshader
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.wgsl
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/mod.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
plan_sources:
  - user: 2026-05-09 implement M3A from render M4+ product pipeline plan
  - user: 2026-05-16 continue Bevy-style asset/image completion toward M4
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
  - user: 2026-05-19 finish runtime UI graph and direct-surface damage, then close the `.zmaterial` workspace blocker
  - user: 2026-05-20 implement ZirconEngine asset/texture/model/ZShader/ZMaterial/ZMesh completion plan
tests:
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/assets/render_product.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/material.rs
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 required shader property diagnostics: passed, 1 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked shader --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader shader validation: passed, 15 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked material --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader material validation: passed, 71 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked asset::tests::project --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader project validation: passed, 26 passed)
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::texture_upload_readiness_reports_compressed_container_support
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::texture_upload_readiness_rejects_compressed_mips_and_arrays_until_full_upload_exists
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::texture_upload_readiness_reports_supercompression_and_astc_3d_boundaries
  - cargo test -p zircon_runtime --lib texture_upload_readiness_rejects_compressed_mips_and_arrays_until_full_upload_exists --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never -- --test-threads=1 (2026-05-20 M3 compressed texture upload shape boundaries: passed, 1 passed, 1723 filtered out)
  - cargo test -p zircon_runtime --lib texture_upload_readiness --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never -- --test-threads=1 (2026-05-20 M3 compressed texture broader filter: blocked before texture tests by unrelated `zircon_runtime/src/scene/tests/ecs_systems.rs` large tuple `assert_eq!` E0369/E0277)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never (2026-05-20 M3 compressed texture upload shape boundaries: passed; existing scene/world warnings only)
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs::render_product_streamer_reports_shader_texture_slot_upload_fallback_by_slot_key
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_shader_texture_slot_upload_fallback_by_slot_key --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M5 shader texture slot readiness: attempted, Cargo failed before test execution while writing default target dep-info after concurrent target directory mutation)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_shader_texture_slot_upload_fallback_by_slot_key --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m5-renderer-slot-0520 --message-format short --color never -- --test-threads=1 (2026-05-20 M5 shader texture slot readiness: first independent-target attempt timed out during compile; matching residual Cargo child processes were stopped)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_shader_texture_slot_upload_fallback_by_slot_key --locked --jobs 1 --target-dir E:\cargo-targets\zircon-m5-renderer-slot-0520 --message-format short --color never -- --test-threads=1 (2026-05-20 M5 shader texture slot readiness: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_shader_texture_slot_upload_fallback_by_slot_key --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never -- --test-threads=1 (2026-05-20 M5 shader texture slot final: passed, 1 passed, 1720 filtered out)
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-asset-parity-runtime-lib-0520 --message-format short --color never (2026-05-20 asset parity implementation: passed; existing warnings only)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --locked --jobs 1 --message-format short --color never (2026-05-20 asset parity implementation: passed; existing runtime warning only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --lib --locked --offline --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 asset parity implementation: timed out during Windows test build/link before Rust test diagnostics)
  - cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (2026-05-20 glTF labeled subassets: passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --message-format short --color never (2026-05-20 glTF labeled subassets: passed; existing runtime dead_code warning only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF labeled subassets: timed out during Windows runtime test build/link; matching residual Cargo chain was stopped after timeout)
  - zircon_runtime/src/asset/tests/assets/importer.rs::importer_emits_bevy_style_gltf_labeled_subassets
  - CARGO_TARGET_DIR=/tmp/zircon-gltf-m4-wsl-fast cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels: blocked before test execution by unrelated zircon_runtime_interface/src/ui/dispatch/navigation/result.rs E0277, UiBindingUpdateReport does not implement Eq)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels: Windows attempt timed out after 304s before Rust test diagnostics; matching residual Cargo child processes were stopped)
  - cargo check -p zircon_runtime_interface --locked --jobs 1 --message-format short --color never (2026-05-20 runtime glTF labels retry: passed, confirming the earlier WSL Eq error is not present in the current Windows source tree)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF labels retry: passed, 1 passed, after replacing the invalid fixture PNG data URI with a valid CRC 1x1 RGBA PNG)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 runtime glTF animation/skin labels: passed, 1 passed, after extending the fixture with Animation0, Skin0, and Skin0/InverseBindMatrices placeholder labels)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib importer_decodes_triangle_gltf_into_model_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF plugin labels retry: passed, 1 passed, after the same fixture PNG replacement)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 glTF plugin animation/skin labels: passed, 3 passed, after extending the fixture with Animation0, Skin0, and Skin0/InverseBindMatrices placeholder labels)
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources
  - zircon_runtime/tests/virtual_geometry_visibility_debug_contract.rs
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked shader --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked project_manager_imports_zshader_with_wgsl_capture_diagnostics --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib material --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib shader --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib asset::tests::project::zmeta --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --test virtual_geometry_visibility_debug_contract --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib material_asset_serialization_rewrites_stale_canonical_overrides --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib material_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib asset::tests::pipeline::manager --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test --workspace --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs
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
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
  - tests/acceptance/render-product-m3a-assets.md
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo test -p zircon_runtime --locked material
  - cargo check -p zircon_runtime --lib --locked
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
  - rustfmt --edition 2021 --config skip_children=true --check on shared image source decode files (2026-05-16 image source format selection: passed)
  - git diff --check on shared image source decode/docs/session files (2026-05-16 image source format selection: passed with CRLF warnings only)
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
doc_type: module-detail
---

# Render Assets

## Purpose

M3A turns render-facing assets into product contracts for `RenderProductFeature::{Image, Mesh, Shader, Material}`. The asset structs remain under `zircon_runtime::asset`, while the neutral descriptors they produce live under `zircon_runtime::core::framework::render`.

## Asset Contracts

`TextureAssetDescriptor` is the render-facing texture metadata payload beside the legacy CPU/container bytes. It records Bevy-style extent depth-or-array-layers, dimension, format, color space, sampler, render usage, render asset residency usage, mip count, array-layer count, and fallback class, then projects through `TextureAsset::render_image_descriptor()` into `RenderImageDescriptor`. This follows the Bevy split where `Image` stores raw data plus `texture_descriptor`, `sampler`, and `asset_usage` (`dev/bevy/crates/bevy_image/src/image.rs:606`), where that descriptor owns `Extent3d` including `depth_or_array_layers` (`dev/bevy/crates/bevy_image/src/image.rs:1105`), where it explicitly covers whether texture data is 1D, 2D, or 3D (`dev/bevy/crates/bevy_image/src/image.rs:618`), where Bevy's `RenderAssetUsages` defines main-world and render-world residency flags (`dev/bevy/crates/bevy_asset/src/render_asset.rs:32`), and where `ImageLoaderSettings` can override format, sRGB interpretation, sampler, asset usage, and array layout (`dev/bevy/crates/bevy_image/src/image_loader.rs:120`).

Texture importers now construct textures through `TextureAsset::new_rgba8(...)` or `TextureAsset::new_container(...)`, so the imported asset carries an explicit descriptor. The RGBA8 image path uses `decode_texture_source_image(...)`, which follows Bevy's default extension-selected image loading (`dev/bevy/crates/bevy_image/src/image_loader.rs:120` and `dev/bevy/crates/bevy_image/src/image_loader.rs:188`) instead of guessing byte format unless `image_format = "guess"` is supplied. Explicit source formats such as `image_format = "jpeg"` and aliases such as `source_format = "open_exr"` are accepted separately from the render-facing `format` texture descriptor override, preserving Bevy's distinction between source image format and GPU texture format. Invalid source-format settings report the actual key name so importer diagnostics stay actionable. The render-facing override also accepts Bevy names: `texture_format` aliases Zircon's `format`, `is_srgb = false` maps to linear color interpretation, `sampler = "linear"`/`"nearest"` mirrors Bevy's `ImageSamplerDescriptor::linear()` and `ImageSamplerDescriptor::nearest()` filter shorthands (`dev/bevy/crates/bevy_image/src/image.rs:856` and `dev/bevy/crates/bevy_image/src/image.rs:867`), and `asset_usage = "render_world"` is accepted as a single-token residency setting as well as the explicit array form. Invalid Bevy-alias descriptor settings report the actual key name, including `texture_format`, `is_srgb`, `sampler`, and `render_asset_usage`. Existing serialized assets that predate the field remain valid: `descriptor = None` falls back to `TexturePayload` metadata, preserving compressed container format, mip count, and array-layer count from old artifacts. `RenderImageDimension` defaults to `D2`, matching the existing image path while leaving an explicit contract for 1D and 3D texture preparation. `RenderImageAssetUsage` defaults to `MainWorld + RenderWorld`, matching Bevy's default CPU/GPU residency policy without conflating it with GPU texture binding usages such as sampled, storage, or copy destination.

Descriptor extent fields are normalized before render preparation sees them. For 1D/2D textures,
`depth_or_array_layers` and `array_layer_count` describe the same layer count; setting only one import
key updates the other, and setting both to different values returns a parse diagnostic. For 3D
textures, `depth_or_array_layers` is native depth and `array_layer_count` is fixed to one; explicit
multi-layer `array_layer_count`/`array_layers` settings are rejected so serialized descriptors cannot
represent a contradictory 3D array texture.

The decoded image path now applies Bevy-style `[array_layout]` settings after descriptor overrides.
`row_count` treats the image as a vertical stack of that many layers, and `row_height` derives the
layer count from the source height. The texture bytes stay in their decoded order, while
`TextureAsset.height`, `RenderImageDescriptor.height`, `array_layer_count`, and
`depth_or_array_layers` are updated to describe a 2D array texture. Invalid zero values,
non-divisible heights, non-2D descriptors, and already-layered textures produce parse diagnostics
instead of ambiguous render metadata.

Texture container importers seed descriptor dimension and extent depth-or-array-layers from native metadata where possible. The plugin root delegates DDS/KTX/KTX2/ASTC header parsing to `zircon_plugins/texture_importer/runtime/src/container.rs`, keeping registration and decode orchestration separate from container byte layout rules. DDS remains 2D and now covers DX10 array/cubemap layer counts, KTX1/KTX2 map zero height to 1D and positive depth to 3D, and ASTC maps positive depth or 3D block dimensions to 3D. For 1D/2D array textures, `depth_or_array_layers` follows the parsed layer/face count; for 3D textures, it follows native depth while `array_layer_count` stays one, even if a malformed KTX header also sets layer/face counts. Broken DDS, KTX1, KTX2, and ASTC headers return format-specific diagnostics before artifact output, and DDS/KTX layer-face products are checked for `u32` overflow before they can become descriptor metadata. Texture import settings can still override descriptor fields with `format`, `color_space`, `dimension`, `depth_or_array_layers` or `depth`, `usage`, `asset_usage`, `render_asset_usage`, `mip_count`, `array_layer_count` or `array_layers`, `sampler = "default" | "linear" | "nearest"`, and partial `sampler` table settings for address/filter modes. `usage`, `asset_usage`, and `render_asset_usage` accept either one string or an array of strings. Invalid setting types or unsupported enum strings fail the import with a parse diagnostic instead of silently producing an ambiguous image contract.

Container descriptor settings do not expand compressed payloads: DDS/KTX/KTX2/ASTC bytes remain in
`TexturePayload::Container` while render-facing format, sampler, and residency fields can be
overridden for diagnostics and later prepare stages. `[array_layout]` is rejected on those container
imports because it requires decoded RGBA bytes that can be reinterpreted as a vertical layer stack.

`TextureAsset::upload_readiness(...)` is the support query between imported texture bytes and renderer upload. It accepts a `TextureUploadSupport` feature record, returns a `TextureUploadPlan` for uploadable payloads, and returns deterministic unsupported reasons for cases the current device or runtime cannot upload. The first supported container path covers single-layer mip0 DDS BC formats and ASTC 2D blocks when the corresponding GPU features are present, while uncompressed RGBA8 keeps the existing byte-size validation. KTX/KTX2 level extraction, KTX2 supercompression/transcoding, ASTC 3D block payload upload, cubemap/array-layer compressed upload, and full mip-chain upload remain explicit unsupported diagnostics instead of falling through to a generic missing-texture path.

Renderer preparation consumes the same query. `resolve_texture_reference_with_support(...)` uses the current device-derived support to decide whether a referenced texture can be used directly or should emit `TextureNotUploadReady` and fall back. `GpuTextureResource::from_asset(...)` now returns a `Result`, uploads RGBA8 as before, and maps upload-ready DDS BC or ASTC 2D payloads to `wgpu::TextureFormat` before writing the mip-zero container payload to the GPU. This keeps importer diagnostics, asset readiness, and renderer fallback behavior aligned around the same support decision.

PSD imports use the same descriptor settings path after flattening to RGBA8, so Photoshop source
files and image crate source files expose the same render-facing `TextureAssetDescriptor` behavior.
Runtime texture fixture coverage lives in `zircon_runtime/src/asset/tests/assets/texture_importer.rs`
so descriptor/decode matrix tests remain separate from the generic importer registry and model/UI
fixture coverage in `importer.rs`.

`ModelPrimitiveAsset::render_mesh_descriptor()` projects primitive vertex/index data into topology, bounds, primitive kind, 2D/3D suitability, primitive counts, and Virtual Geometry payload presence through `RenderMeshDescriptor`.

`MeshAsset` is the first-class typed mesh asset introduced for the Bevy-style asset plan. It stores topology, a named attribute map, optional u16/u32 indices, main-world/render-world residency intent, and optional Virtual Geometry payload. `MeshAsset::render_mesh_descriptor()` projects the attribute map into the same `RenderMeshDescriptor` surface, with required `position` data driving bounds and planar/spatial classification. Existing model import paths now keep legacy `ModelAsset.primitives` while emitting matching labeled `MeshAsset` subassets for future renderer handle consumption.

`ShaderAsset::runtime_wgsl_source()` chooses runtime WGSL by preferring non-empty `wgsl_source`, then non-empty `source` only when `source_language == ShaderSourceLanguage::Wgsl`. Non-WGSL source without emitted WGSL is not treated as render-ready WGSL.

Shader dependencies are explicit serialized `ShaderDependencyAsset` entries because M3A does not introduce a shader import language. `ShaderAsset::dependencies()` projects those entries into `RenderShaderDependency`. Pipeline layout readiness is explicit through serialized `RenderShaderPipelineLayoutDescriptor`, including bind groups, binding descriptors, resource type, stage visibility, and push-constant range labels.

`MaterialAsset` exposes `dependency_set()`, `direct_references()`, `standard_material_descriptor()`, `color_material_descriptor()`, and `readiness_report()`. Source material files now use `.zmaterial`: shader identity is stored in `[shader]`, instance scalar/vector state is stored under `[overrides]`, and texture references are stored under `[textures.<slot>]`. Legacy PBR fields remain in the Rust struct for the current renderer path, but source parse and serialization hydrate them from shader-style override and texture-slot entries instead of accepting the old `.material.toml` top-level shape. During serialization, canonical PBR fields rewrite stale matching override entries and canonical texture slots before TOML is emitted, while preserving unknown shader-specific overrides and fallback-only texture slot metadata. This keeps a material edited through runtime fields from writing old `[overrides]` bytes back to disk and prevents source-hash/revision no-ops during asset watcher reimport.

Editor renderable project scaffolding follows the same source contract instead of pointing materials at raw WGSL. `default.zmaterial` references `res://shaders/pbr_shader`, `pbr_shader.zmeta` owns the compound shader root, and `pbr.zshader` plus `pbr.wgsl` are written under `assets/shaders/pbr_shader/`. Virtual Geometry fixture helpers now serialize `.zmaterial` material sources as well, so render-product tests exercise the current importer suffix instead of the removed legacy material suffix.

Material direct dependencies include the shader plus every concrete texture-slot reference. Fallback-only texture slots remain authoring/runtime fallback data and are omitted from dependency locators until a real texture reference is authored.

glTF material imports now feed this same contract instead of remaining model-local metadata. The
split glTF importer maps PBR base-color, normal, metallic-roughness, occlusion, and emissive texture
links into `MaterialAsset` legacy fields and shader-style `texture_slots`, while embedded or external
glTF images become labeled `Texture{n}` `TextureAsset` subassets. Scene mesh instances point at
`Mesh{n}` plus `Material{n}` or `DefaultMaterial`, so renderer readiness can trace missing shader,
texture, and fallback state by the same asset locators used by authored `.zmaterial` sources.

Material/schema mismatches are diagnostics, not import blockers. `MaterialAsset::shader_contract_diagnostics(...)` compares `[overrides]` and `[textures.<slot>]` against the loaded `ShaderAsset` contract and emits typed `RenderMaterialValidationError` values for unknown overrides, override type mismatches, missing required shader properties, and unknown texture slots. `readiness_report_with_shader_contract(...)` merges those diagnostics into the neutral readiness report while preserving fallback-only texture slots as non-dependency data.

`MaterialAsset::standard_material_descriptor_for_shader(...)` is the renderer bridge for shader-driven texture slots. It starts from the legacy PBR descriptor for compatibility, then lets shader-declared texture slot aliases such as `base_color`, `albedo`, `normal`, `metallic_roughness`, `occlusion`, and `emissive` override the fixed PBR texture fields. The legacy `standard_material_descriptor()` path remains available for callers without a loaded shader contract, but renderer material preparation now prefers the shader contract when it can load one.

Compound `.zshader` import now performs a lightweight WGSL capture check after reading the declared source files. Every declared property and texture-slot name should appear in the combined WGSL source; misses are recorded in `ShaderAsset.validation_diagnostics` as `wgsl_capture` diagnostics. The import still succeeds so authoring tools and readiness reports can show the mismatch instead of losing the asset.

## Readiness

Material readiness is structured through `RenderMaterialReadinessReport`. `AlphaMode::Mask { cutoff }` rejects non-finite values and values outside `0.0..=1.0` with `RenderMaterialValidationError::InvalidMaskCutoff`. Callers that can resolve asset references use `readiness_report_with_resolution(...)`, which records unresolved shader or concrete texture-slot references as validation errors plus explicit fallback usage records.

Diagnostic sources are explicit through `RenderMaterialDiagnosticSource`: shader schema, WGSL capture, material override, texture slot, and dependency resolution paths are distinguishable in the same readiness payload. This keeps unknown override, unknown texture slot, and WGSL capture mismatch reports machine-readable for the Material Editor without making source import fail.

The resource streamer uses typed material readiness before preparing GPU material state and stores the resulting `RenderMaterialReadinessReport` on `MaterialRuntime`. Renderer-side consumers and tests can query it through `ResourceStreamer::material_readiness_report(...)`. Existing fallback shader and missing-texture behavior remains allowed for compatibility, but unresolved shader references, wrong-kind or load-failing dependencies, unresolved texture references, and the fallback policy used for each slot are preserved in the stored report instead of being discarded.

Material preparation now loads the referenced `ShaderAsset` as a contract when possible, feeds `readiness_report_with_shader_contract(...)`, and resolves every authored material texture slot rather than only the fixed standard PBR slots. Standard PBR aliases are still synchronized into the current runtime material shape, while non-standard shader slots are resolved for readiness and fallback diagnostics so missing or not-upload-ready shader-specific textures can be traced by asset locator and slot key. `render_product_streamer_reports_shader_texture_slot_upload_fallback_by_slot_key` covers this with a shader-declared `mask_map` slot backed by an unsupported compressed container texture.

Shader fallback is exposed through `ensure_shader_source(...)`, which now returns the prepared shader identity and an optional structured readiness report when the requested shader reference resolves through the default fallback shader. A shader that exists but cannot provide runtime WGSL is reported as `MissingRuntimeShaderSource`, stored on the material runtime report, and then treated as a blocking material readiness error.

Texture lookup is exposed through `resolve_texture_reference(...)`, which returns the resolved texture id plus an unresolved-reference validation error and fallback usage when the declared texture locator is missing, is the wrong typed payload, cannot load as a `TextureAsset`, or is not upload-ready. The compatibility helper uses an uncompressed-only support profile; renderer preparation calls `resolve_texture_reference_with_support(...)` with actual device support so uploadable DDS BC and ASTC 2D containers can avoid fallback. Unsupported compression, KTX/KTX2 supercompression or missing level extraction, ASTC 3D, malformed byte lengths, and unavailable GPU features are reported as `TextureNotUploadReady` using the resolved descriptor format and asset locator.

## Scope Boundary

This document covers M3A asset readiness only. It does not implement M4 core phases, sprite rendering, anti-aliasing, Solari, or deeper VG/HGI integration.
