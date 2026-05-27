---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime_interface/src/resource/resource_id.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/package_assets.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/project/manager/source_uri_for_path.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/property_values.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs
  - zircon_runtime/src/core/framework/render/material/property_value.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/layouts/create_material_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/render_pass_bindings.rs
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/records.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/folder_projection.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/constants.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_ensure_runtime_assets.rs
  - zircon_editor/src/ui/workbench/project/runtime_asset_helpers.rs
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.zshader
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.wgsl
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/assets/shader_readiness.rs
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - docs/assets-and-rendering/fixtures/zmeta-shader-material
implementation_files:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/layouts/create_material_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/render_pass_bindings.rs
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/property_values.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs
  - zircon_runtime/src/core/framework/render/material/property_value.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/records.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/folder_projection.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/constants.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_ensure_runtime_assets.rs
  - zircon_editor/src/ui/workbench/project/runtime_asset_helpers.rs
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.zshader
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.wgsl
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - docs/assets-and-rendering/fixtures/zmeta-shader-material
plan_sources:
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
  - user: 2026-05-19 finish runtime UI graph and direct-surface damage, then close the `.zmaterial` workspace blocker
  - docs/superpowers/specs/2026-05-24-shader-readiness-report-design.md
  - docs/superpowers/plans/2026-05-24-shader-readiness-report.md
  - docs/superpowers/specs/2026-05-25-typed-shader-definitions-design.md
  - docs/superpowers/plans/2026-05-25-typed-shader-definitions.md
  - user: 2026-05-27 continue shader/material management
tests:
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/assets/shader_readiness.rs
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime_interface --locked resource --jobs 1 --message-format short --color never (2026-05-20 package roots M2: passed, 12 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked asset::tests::project::package_assets --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 3 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked asset::tests::project::zmeta --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 8 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked plugin_package_manifest --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 6 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --locked package --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed after warm cache, 43 package-filtered runtime lib tests plus package-filtered integration binaries)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-asset-package-m2 cargo test --manifest-path zircon_plugins/Cargo.toml --locked --jobs 1 --message-format short --color never package -- --test-threads=1 (2026-05-20 package roots M2: passed after moving off full D: target dir)
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs::render_product_streamer_prepares_shader_property_runtime_values
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs::render_product_streamer_reports_shader_material_layout_abi_diagnostics
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs::render_product_streamer_reports_imported_zshader_material_layout_abi_diagnostics
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_material_properties_prepare_uniform_payload
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_streamer_exposes_material_uniform_debug_counts
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_streamer_reports_material_uniform_diagnostics_in_readiness_report
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_streamer_reports_material_uniform_diagnostics_for_shader_string_defaults
  - zircon_runtime/src/graphics/tests/render_product_submit.rs::render_product_submit_material_stats_count_material_uniform_diagnostics
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs::tests::material_readiness_report_deduplicates_material_uniform_diagnostics
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs::tests::material_property_uniform_payload_aligns_and_encodes_numeric_values
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs::tests::material_property_uniform_payload_records_unsupported_strings
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs::tests::material_property_uniform_payload_reports_unsupported_diagnostics
  - cargo test -p zircon_runtime --lib material_property_uniform_payload_reports_unsupported_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 material uniform diagnostics: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_submit_material_stats_count_non_blocking_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 material diagnostic stats validation retry: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_submit_material_stats_count_material_uniform_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 MaterialUniform submit stats: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - D:/cargo-targets/zircon-material-diagnostic-stats-0527/debug/deps/zircon_runtime-b34ee8d8fc52f1fd.exe render_product_streamer_reports_material_uniform_diagnostics_in_readiness_report --test-threads=1 --nocapture (2026-05-27 MaterialUniform readiness detail: passed, 1 passed after the Cargo wrapper timed out during concurrent build activity)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_material_uniform_diagnostics_for_shader_string_defaults --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 MaterialUniform shader default detail: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - F:/cargo-targets/zircon-platform-m5-workspace/debug/deps/zircon_runtime-030785730509538c.exe material_readiness_report_deduplicates_material_uniform_diagnostics --test-threads=1 --nocapture (2026-05-27 MaterialUniform diagnostic dedup: passed, 1 passed; standard Cargo wrappers timed out under concurrent workspace/editor build load before producing a local material target binary)
  - zircon_runtime/src/asset/tests/assets/material.rs::material_asset_readiness_reports_material_local_diagnostics_without_blocking
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/material/diagnostic_source.rs zircon_runtime/src/core/framework/render/material/readiness_report.rs zircon_runtime/src/core/framework/render/material/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/asset/assets/material/material_asset.rs zircon_runtime/src/asset/tests/assets/material.rs zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs (2026-05-27 material-local readiness diagnostics: passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-local-diagnostics-0527 --message-format short --color never (2026-05-27 material-local readiness diagnostics: blocked before material tests by unrelated UI a11y private re-export errors in zircon_runtime/src/ui/accessibility/action/text.rs)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-local-diagnostics-0527-core-min --message-format short --color never (2026-05-27 material-local readiness diagnostics: same unrelated UI a11y private re-export blocker)
  - cargo test -p zircon_runtime --lib material_asset_readiness_reports_material_local_diagnostics_without_blocking --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-local-diagnostics-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 material-local readiness diagnostics: attempted; wrapper timed out before producing a test binary or Rust diagnostics)
  - git diff --check -- touched material-local readiness diagnostics files (2026-05-27 material-local readiness diagnostics: passed with LF-to-CRLF warnings only)
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/assets/shader/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs (2026-05-27 shader pipeline layout readiness summary: passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group --message-format short --color never (2026-05-27 shader pipeline layout readiness summary: passed with existing warnings)
  - D:/cargo-targets/zircon-shader-readiness-layout-0527/debug/deps/zircon_runtime-b34ee8d8fc52f1fd.exe shader_readiness --test-threads=1 --nocapture (2026-05-27 shader pipeline layout readiness summary: passed, 6 passed; standard cargo test wrapper timed out after producing the test binary during concurrent Cargo activity)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/mod.rs zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs (2026-05-27 imported zshader renderer ABI diagnostics: passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 required shader property diagnostics: passed, 1 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked shader --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader shader validation: passed, 15 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked material --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader material validation: passed, 71 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked asset::tests::project --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader project validation: passed, 26 passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never (2026-05-20 WGSL capture facade re-export: initially failed with E0425 for `crate::asset::validate_wgsl_captures`; passed after top-level re-export, existing warnings only)
  - cargo test -p zircon_runtime --lib --locked documented_zmeta_shader_material_fixture_parses --jobs 1 --target-dir F:\cargo-targets\zircon-zmeta-shader-material-m3 --message-format short --color never -- --test-threads=1 (2026-05-20 M4 fixture capture closeout: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --locked asset::tests::project::zmeta --jobs 1 --target-dir F:\cargo-targets\zircon-zmeta-shader-material-m3 --message-format short --color never -- --test-threads=1 (2026-05-20 M4 fixture capture closeout: passed, 8 passed)
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts -- --test-threads=1 (2026-05-24 shader defs slice: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never project_manager_imports_compound_zshader_package_with_subassets -- --test-threads=1 (2026-05-24 shader defs slice: passed, 1 passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_shader_runtime --lib --locked --jobs 1 --message-format short --color never (2026-05-24 shader defs slice: passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_shader_wgsl_importer_runtime --lib --locked --jobs 1 --message-format short --color never (2026-05-24 shader defs slice: passed)
  - cargo test -p zircon_editor --lib --locked --jobs 1 --message-format short --color never material_editor_projection_groups_shader_properties_and_material_overrides -- --test-threads=1 (2026-05-24 shader defs slice: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never project_manager_imports_compound_zshader_package_with_subassets -- --test-threads=1 (2026-05-24 zshader import preservation: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never shader -- --test-threads=1 (2026-05-24 zshader import preservation: passed, 19 passed)
  - cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 5 passed)
  - cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 1 passed)
  - cargo test -p zircon_runtime --lib project_manager_imports_zshader_with_wgsl_capture_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 1 passed)
  - cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 24 passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/assets/shader/zshader.rs zircon_runtime/src/asset/importer/ingest/import_shader_package.rs zircon_runtime/src/asset/tests/project/zmeta.rs (2026-05-27 zshader pipeline layout persistence: passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group --message-format short --color never (2026-05-27 imported zshader renderer ABI diagnostics: passed with existing warnings)
  - cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 zshader pipeline layout persistence: passed, 1 passed)
  - cargo test -p zircon_runtime --lib documented_zmeta_shader_material_fixture_parses --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 zshader pipeline layout persistence: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_shader_material_layout_abi_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 renderer material ABI diagnostics: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_imported_zshader_material_layout_abi_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 imported zshader renderer ABI diagnostics: passed, 1 passed)
  - rustfmt --edition 2021 --check on touched typed shader-definition Rust files (2026-05-26 typed shader definitions: passed)
  - cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-26 typed shader definitions: passed, 30 passed after retrying a transient target-dir dep-info write failure)
  - cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-26 typed shader definitions: passed, 1 passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs (2026-05-26 typed shader definitions: passed with existing warnings)
  - cargo test -p zircon_runtime --lib render_product_assets_shader_defs_accept_legacy_flags_and_typed_values --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib zshader_typed_shader_definition_rows_validate_kind_and_value --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 5 passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs --message-format short --color never (2026-05-25 typed shader definitions: passed with existing warnings)
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - cargo check -p zircon_runtime_interface --locked
  - cargo check -p zircon_runtime --locked --lib --message-format=short
  - cargo test -p zircon_runtime_interface --locked resource --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::project::zmeta --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib asset::tests::project::zmeta --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::watcher --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::assets::material --jobs 1 -- --nocapture
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked shader --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked project_manager_imports_zshader_with_wgsl_capture_diagnostics --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib material --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib shader --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib asset::tests::project::zmeta --locked --offline --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked package_manifest --jobs 1 -- --nocapture
  - cargo check -p zircon_editor --locked --lib --message-format=short
  - cargo check -p zircon_editor --locked --tests --message-format=short
  - cargo test -p zircon_editor --lib --locked sync_from_project_exposes_zmeta_package_and_compound_shader_details --jobs 1 -- --nocapture
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_editor --lib create_renderable_template_scaffolds_directory_project_defaults --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-presenter-check-wsl cargo test -p zircon_editor --lib create_renderable_template_scaffolds_directory_project_defaults --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --test virtual_geometry_visibility_debug_contract --locked --offline --jobs 1 --message-format short --color never
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir F:\cargo-targets\zircon-zmeta-validation
  - cargo test --manifest-path zircon_plugins\Cargo.toml --locked --target-dir F:\cargo-targets\zircon-zmeta-validation
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib runtime_backed_workspace_plugin_manifests_are_present_in_builtin_catalog -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib documented_zmeta_shader_material_fixture_parses -- --nocapture
  - cargo test -p zircon_editor --lib --locked --target-dir F:\cargo-targets\zircon-zmeta-validation -- --nocapture
  - cargo build -p zircon_hub --locked --target-dir F:\cargo-targets\zircon-zmeta-validation
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-zmeta-validation
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib material_asset_serialization_rewrites_stale_canonical_overrides --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib material_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib asset::tests::pipeline::manager --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test --workspace --locked --jobs 1 --message-format short --color never -- --test-threads=1
doc_type: module-detail
---

# ZMeta Shader Material Assets

## Purpose

The asset identity path is now owned by `zircon_runtime::asset` and `zircon_runtime::core::resource`: `.zmeta` stores UUID identity, human-readable URL, source unit, included files, subasset entries, importer state, artifact locators, and dependency locators. There is no second asset database.

## Locator Rules

- `res://path/to/asset` maps to `{project_root}/assets/path/to/asset`.
- `package://com.zircon.navigation/path/to/asset` maps through `PackageAssetRegistry` to a registered package `assets/` root.
- `PluginPackageManifest::package_id()` composes `package_prefix.package_company.package_name` when all three fields are present, otherwise it falls back to the manifest id. `asset_roots_or_default()` returns explicit `asset_roots` or `assets` when the manifest omits roots.
- `ProjectManager::register_package_asset_root(...)` registers an explicit package root. `register_package_manifest_asset_roots(...)` reads a manifest root and currently requires exactly one relative, contained root so every `package://{package_id}/...` path has one unambiguous filesystem base.
- Public references serialize as `{ uuid, url }`. Runtime lookup tries UUID first; stale URLs are retained as diagnostics/repair hints through the manager lookup surface.
- `url#label` remains the subasset address form, but every root and subasset entry persists its own UUID and derives `AssetId` from that UUID.

## Source Units

Single assets use sidecars such as `assets/textures/hero.png.zmeta`.

Compound assets use a `.zmeta` root and same-name directory, for example:

```text
assets/shaders/unlit_shader.zmeta
assets/shaders/unlit_shader/unlit.zshader
assets/shaders/unlit_shader/unlit.wgsl
```

The scanner treats the `.zmeta` root as one `AssetSourceUnit::Compound`, records the directory files in `included_files`, and prevents those included files from being imported again as standalone assets. The same scanner handles registered package roots: project sources keep `res://` URLs, while package sources and compound included files use `package://{package_id}/...` URLs and the same `.zmeta` schema, importer registry, artifact writer, dependency resolver, and UUID index.

## Shader And Material

`.zshader` is TOML. It describes the optional shader import path, WGSL source files, entry points, import declarations, redirect targets, shader definition flags, material property schema, texture slots, optional `[pipeline_layout]`, and editor hints. The compound shader importer reads the `.zmeta` root, loads the same-name directory, emits a root `ShaderAsset`, and emits `.zshader`/`.wgsl` files as labeled data subassets.

`ShaderAsset` carries `import_path`, `source_files`, `imports`, typed `shader_defs`, `property_schema`, `texture_slots`, `pipeline_layout`, `editor`, and `validation_diagnostics`. `import_path` is authored as an optional `.zshader` string such as `zircon::unlit` or `zircon::pbr`; it is preserved on compound shader roots so later shader composition/cache layers have a stable import namespace separate from the asset URL. Raw single-file shader importers and built-in shader fixtures leave it empty. Every `[[imports]]` row is preserved by source name, even when it has no redirect yet. Rows with a `{ uuid, url }` redirect are additionally projected into `ShaderAsset.dependencies` and the import outcome dependency graph; source-only rows remain authoring metadata for later shader composition and editor repair. Legacy `shader_defs = ["FEATURE"]` rows become bool-true `RenderShaderDefinitionValue` entries, while `[[shader_def_values]]` rows can store bool, int, and uint values; `ShaderAsset::variant_keys()` copies these typed values into every `RenderShaderVariantKey`, so render prepare/cache code can distinguish the same entry point compiled with different definition sets. Texture slot schema is shader-owned: each `ShaderTextureSlotAsset` records the slot `name`, `kind`, whether the material must bind it with `required`, optional fallback class, sampler hint, grouping label, and editor metadata. During compound shader import, `validate_wgsl_captures(...)` scans the combined WGSL source for declared property and texture-slot names; missing captures are recorded as `wgsl_capture` diagnostics on the shader asset but do not stop import. The helper is re-exported through the top-level `zircon_runtime::asset` facade so fixture tests and public callers do not need to depend on the internal `assets::material` module path.

Shader definition authoring has two compatible forms. `shader_defs = ["FEATURE"]` is the legacy flag array and imports each string as a bool-true `RenderShaderDefinitionValue`. Each `[[shader_def_values]]` table must provide `name`, `kind`, and `value`; accepted kind aliases are `bool`/`boolean`, `int`/`i32`/`integer`, and `uint`/`u32`. Invalid typed rows fail compound shader import with `AssetImportError::Parse("parse zshader shader_def_values: ...")`; diagnostic fragments include unsupported-kind text, `value is not a boolean`, `value is not an i32 integer`, and `value is not a u32 integer`.

Pipeline layout authoring has the same asset-owned persistence model. A `.zshader` may include `[pipeline_layout]`, `[[pipeline_layout.bind_groups]]`, nested `[[pipeline_layout.bind_groups.bindings]]`, and `push_constant_ranges`; the importer preserves those rows as `ShaderAsset.pipeline_layout`. Empty or omitted layouts still deserialize to the default descriptor and remain accepted, while authored bind groups give renderer material readiness enough data to report ABI drift before full reflection exists. The shader-owned readiness report also exposes the layout shape as a read-only summary of bind groups, bindings, binding resource types, visibility, and push-constant labels, so editor diagnostics can display authored layout context without rerunning renderer ABI validation. `render_product_streamer_reports_imported_zshader_material_layout_abi_diagnostics` verifies that this survives the full compound `.zmeta` import/artifact path before `ResourceStreamer` emits the renderer ABI diagnostics.

Standalone shader readiness is visible directly on `ShaderAsset` through `readiness_report()`. Compound `.zshader` packages therefore expose WGSL capture diagnostics and authored pipeline-layout context without requiring a material instance: the import can succeed, the shader asset can keep its authoring rows, and readiness consumers can still see that `wgsl_capture` diagnostics make the shader not ready for downstream material/render preparation. Source-only imports are reported as non-dependency authoring rows, while redirected imports appear as dependency-contributing readiness rows.

`.zmaterial` is the only built-in material source suffix. It references one shader with `{ uuid, url }`, stores scalar/vector instance state in `[overrides]`, and stores texture bindings under `[textures.<slot>]`. The built-in importer id is `zircon.builtin.zmaterial`; `.material.toml` is intentionally not registered and now reports as an unknown typed TOML suffix. `ZMaterialDocument` denies unknown top-level fields, so the old top-level PBR `.material.toml` shape is rejected instead of being silently translated. `MaterialAsset` keeps legacy PBR runtime fields as transitional in-memory data, but source parse/serialization flows through `ZMaterialDocument` and shader-driven overrides/texture slots.

Shader-driven material properties now have a runtime projection path. `MaterialAsset::shader_property_values_for_shader(...)` reads the loaded `ShaderAsset.property_schema`, takes a `[overrides]` value first and then the shader default, and converts supported TOML bool, float, int, uint, string, vec2, vec3, and vec4/color values into `RenderMaterialPropertyValue`. The resource streamer stores that map on `MaterialRuntime.shader_property_values` so renderer uniform or bind-group work can consume already-typed material values without reparsing `.zmaterial` TOML. `RenderMaterialPropertyValueSummary` summarizes the projected map before uniform encoding by total, per-kind, uniform-eligible, and non-uniform counts. This still does not perform automatic shader reflection.

The resource streamer also prepares `MaterialRuntime.shader_property_uniform_payload` from those typed values. Numeric scalar/vector properties are encoded into deterministic CPU-side bytes with field offsets and alignments; string properties are reported in the payload's unsupported list and projected into non-blocking readiness diagnostics with `MaterialUniform` source. Prepared materials now own a `GpuMaterialUniformResource` that uploads those bytes into a WGPU uniform buffer and exposes a material bind group consumed by mesh/deferred draw paths as group 3. `ResourceStreamer::material_readiness_report(...)` keeps the same unsupported string row visible with the `uniform.<name>` path for both material overrides and shader schema defaults, and the stored report now carries `RenderMaterialPropertyUniformSummary` in `uniform_summary` once material preparation has run. `ResourceStreamer::material_uniform_summary(...)` remains the compact direct accessor for payload byte length, encoded field count, and unsupported row count after preparation. `RenderMaterialPropertyValueSummary::from_values(...)` remains the pre-encoding companion for editor/runtime panels that need to show whether the shader property projection itself produced the expected bool/float/vector/string mix. The individual debug accessors remain available for backing buffer length and legacy counter reads. Submit-time material stats cover the unsupported row through `RenderStats.last_material_diagnostic_count` without changing ready, fallback, or validation-error counts. Duplicate `MaterialUniform` diagnostics are merged by the shared readiness report de-duplication helper, so repeated unsupported rows do not produce duplicate editor/report/stat rows. This advances renderer consumption without changing the `.zmaterial` source schema; large custom layouts, non-standard texture bind arrays, and non-uniform property kinds remain later binding/reflection work.

If a shader asset already carries an authored `pipeline_layout`, including one persisted from `.zshader`, renderer material preparation validates the material portion against the current fixed group-3 ABI. The accepted material binding is group 3 binding 0 as a uniform buffer with vertex or fragment visibility; wrong resource types, missing group/binding rows, duplicate descriptors, compute-only visibility, and extra group-3 material bindings become material readiness diagnostics under `pipeline_layout.group3...`. Shaders without serialized bind groups are still accepted because the renderer owns the fallback mesh/material layouts until automatic reflection and custom texture binding land.

When a hydrated `MaterialAsset` is serialized back to `.zmaterial`, the canonical runtime fields are authoritative over stale matching entries inside `property_values` and `texture_slots`. `base_color`, `metallic`, `roughness`, `emissive`, non-opaque `alpha_mode`, and `double_sided` rewrite or remove their corresponding `[overrides]` entries according to the current field value; canonical texture references rewrite their `[textures.<slot>]` reference while preserving fallback metadata. Unknown shader-specific overrides and fallback-only texture slots are preserved. This keeps editor/runtime mutations such as changing `MaterialAsset.base_color` from re-emitting the old `overrides.base_color` bytes and blocking asset watcher reimport/revision updates.

New editor renderable projects scaffold the same contract: `default.zmaterial` points at the compound shader root `res://shaders/pbr_shader`, `pbr_shader.zmeta` marks that root as `AssetSourceUnit::Compound`, and the included `pbr.zshader`/`pbr.wgsl` files live under `assets/shaders/pbr_shader/`. The raw WGSL remains an included shader source, not the material's referenced shader identity.

Material direct dependencies include the shader reference and every texture slot that carries a concrete `AssetReference`. Texture slots may also contain only a fallback class, such as `white`, `black`, `normal`, or `missing`; fallback-only slots do not become `.zmeta` dependencies.

Material/schema mismatches are represented as typed readiness diagnostics rather than importer failures. `MaterialAsset::shader_contract_diagnostics(...)` compares `[overrides]` with `ShaderAsset.property_schema` and `[textures.<slot>]` with `ShaderAsset.texture_slots`; it records unknown overrides, override type mismatches, missing required shader properties, unknown texture slots, and missing required texture-slot references with stable document paths. A fallback-only slot such as `[textures.base_color] fallback = "white"` remains valid authoring metadata, but it does not satisfy a shader slot marked `required = true` because no concrete texture asset reference can enter the dependency graph or renderer upload path. `MaterialAsset::readiness_report_with_shader_contract(...)` merges those diagnostics with dependency-resolution readiness and consumes `ShaderAsset::readiness_report()` so missing runtime WGSL, invalid entry-point stage tokens, duplicate or empty shader definitions, and shader-side WGSL capture diagnostics all reach material/runtime readiness reports. `MaterialAsset.validation_diagnostics` now flows into the report's non-blocking `diagnostics` list with `MaterialAsset` source and paths like `material.validation_diagnostics[0]`; uniform payload unsupported rows flow into the same list with `MaterialUniform` source and paths like `uniform.debug_label`. The readiness report de-duplicates those rows by full diagnostic identity before storage. Importer notes from glTF, generated default materials, and non-uniform property retention stay visible without making the material fail readiness.

The persistent fixture under `docs/assets-and-rendering/fixtures/zmeta-shader-material/` mirrors a project `assets/` tree and includes a compound `unlit_shader.zmeta`, `unlit.zshader`, `unlit.wgsl`, and `hero_unlit.zmaterial` with `{ uuid, url }` shader and texture references. The fixture `.zshader` includes `import_path = "zircon::unlit"` plus a legacy `shader_defs` flag array, the fixture WGSL references every property and texture-slot name declared by the `.zshader`, and `documented_zmeta_shader_material_fixture_parses` checks the same WGSL capture rule as the importer so the example stays diagnostic-clean as the schema evolves.

## Editor Surfacing

The editor asset manager keeps using the runtime project registry as the authority. Asset details now include package id, source unit, included files, and labeled subassets from the loaded `.zmeta` document. The Asset Browser metadata tab displays the adapter/package/unit summary and lists included files plus subassets beside runtime diagnostics. Package assets are projected into their own `package://{package_id}` folder roots instead of being folded under `res://`.

## Validation

Runtime/interface scoped checks and the focused zmeta, watcher, package manifest, shader-selection, and material tests pass. Editor library and editor test-target checks pass, and the direct editor sync test passes after the editor test harness finishes linking.

The 2026-05-27 shader pipeline-layout readiness summary passed Rust formatting checks and scoped `zircon_runtime` lib/test type checking. The focused `shader_readiness` filter passed by running the generated lib-test binary directly with 6 tests; two standard Cargo wrapper attempts were affected by concurrent target-directory mutation or long Cargo-layer timeouts after the binary was produced.

The 2026-05-27 material-local readiness diagnostics slice keeps imported material notes in `RenderMaterialReadinessReport.diagnostics` instead of `validation_errors`, so those rows remain visible to renderer/editor consumers without forcing fallback or making `is_ready()` false. Formatting and diff checks passed for the touched material/readiness files. Scoped Cargo validation is currently blocked before material test execution by unrelated UI accessibility private re-export errors in `zircon_runtime/src/ui/accessibility/action/text.rs`, and the focused material test wrapper timed out before producing a test binary or Rust diagnostics.

The 2026-05-27 material uniform diagnostics slice projects unsupported uniform payload rows into non-blocking `MaterialUniform` readiness diagnostics. The focused DTO payload regression passed, the earlier renderer stats diagnostic-count test passed after the unrelated UI accessibility private re-export blocker was resolved by its owning session, the submit-level string-property regression passed with `last_material_diagnostic_count` reporting the non-blocking uniform note while ready/fallback/validation-error counts stayed unchanged, the resource-streamer readiness-detail regression passed by directly running the generated runtime lib-test binary after the Cargo wrapper timed out under concurrent build load, and the shader-default string regression passed through the normal Cargo wrapper with existing lib-test warnings only. The follow-up DTO de-dup regression passed by directly running the workspace-generated `zircon_runtime` test binary after fresh Cargo wrappers timed out during concurrent workspace/editor build load.

The runtime package validator passes with `-TargetDir F:\cargo-targets\zircon-zmeta-validation`, and the plugin workspace test command passes against the same external target directory. The `zr_vm_language` catalog consistency gap is closed by registering that package id, crate, target modes, and both capabilities in `RuntimePluginDescriptor::builtin_catalog()`, so runtime-backed package manifest projection can see it through the same path as the other built-in plugin packages.

The final acceptance matrix passes with `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-zmeta-validation`: the validator performed the target-dir cleanup gate, then completed workspace `cargo build --workspace --locked` and `cargo test --workspace --locked`. The only observed follow-up diagnostics are non-blocking Cargo warnings about the `zircon_runtime.pdb` output-name collision and an unused `RuntimeSession::create` helper outside the `.zmeta` asset path.

The 2026-05-19 `.zmaterial` hard-cutover closeout passed the focused WSL runtime checks on `/mnt/f/cargo-targets/zircon-zmaterial-final-wsl`: runtime `material` tests (`68` passed), runtime `shader` tests (`13` passed), runtime `asset::tests::project::zmeta` tests (`8` passed), the Virtual Geometry runtime plugin library check, and `virtual_geometry_visibility_debug_contract` (`3` passed). The editor renderable scaffold command initially exposed unrelated retained-host GPU presenter test-scope drift; after that presenter state was corrected by the active UI changes, the same scaffold test passed on `/mnt/f/cargo-targets/zircon-presenter-check-wsl` with `1` passed and `1400` filtered out.

The later 2026-05-19 runtime UI graph closeout exposed and fixed a Windows workspace blocker in this material serialization layer: asset watcher/reimport tests mutated `MaterialAsset.base_color`, but `to_toml_string()` preserved stale `overrides.base_color`, so source bytes and resource revisions did not change. The fix is covered by `material_asset_serialization_rewrites_stale_canonical_overrides`, the focused `material_asset` filter (`8` passed), the asset manager pipeline filter (`9` passed), full `zircon_runtime --lib` (`1634` passed), and full workspace `cargo test --workspace --locked --jobs 1 --message-format short --color never -- --test-threads=1`, all using `CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage`.

The 2026-05-20 M3 required-property closeout records missing required shader schema properties as typed material diagnostics instead of importer failures. The focused regression passed on `F:\cargo-targets\zircon-zmeta-shader-material-m3` with `1` test, followed by broader scoped runtime filters: `shader` (`15` passed), `material` (`71` passed), and `asset::tests::project` (`26` passed), all with `--locked` and `--test-threads=1`.

The 2026-05-20 M4 fixture capture closeout updated the documented `unlit.wgsl` fixture so it references the declared `base_color` property and texture slot, then extended `documented_zmeta_shader_material_fixture_parses` to call the same WGSL capture validator as the importer. The focused fixture test passed with `1` test, and the broader `asset::tests::project::zmeta` filter passed with `8` tests on `F:\cargo-targets\zircon-zmeta-shader-material-m3`.
