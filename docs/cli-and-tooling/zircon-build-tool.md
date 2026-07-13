---
related_code:
  - tools/dev-fast-build.ps1
  - tools/zircon_build.py
  - tools/zircon_build_config.py
  - tools/zircon_build_font_sdf.py
  - tools/zircon_build_asset_staging.py
  - tools/zircon_build_plugin_assets.py
  - tools/zircon_build_plugin_manifest_contract.py
  - tools/zircon_build_plugin_packages.py
  - tools/zircon_build_plugin_selection.py
  - tools/zircon_build_plugin_shader_descriptors.py
  - tools/zircon_build_plugin_workspace_crates.py
  - tools/zircon_build_zui_assets.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_resource_registry.py
  - tools/zircon_build_shader_prewarm_report_contract.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/zircon_build_shader_prewarm_acceptance.py
  - tools/zircon_build_shader_prewarm_written_variants.py
  - tools/tests/test_zircon_build_shader_prewarm.py
  - tools/tests/test_zircon_build_font_sdf.py
  - tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_cache_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py
  - tools/tests/test_zircon_build_plugin_carriers.py
  - tools/tests/test_zircon_build_asset_staging_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_asset_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_catalog_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_manifest_contract_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_shader_descriptor_owner_boundaries.py
  - tools/tests/test_plugin_docs_current_status_zircon_build_plugin_catalog_owner_split.py
  - tools/tests/test_plugin_docs_current_status_zircon_build_asset_staging_owner_split.py
  - tools/tests/test_plugin_docs_current_status_zircon_build_plugin_manifest_contract_owner_split.py
  - tools/tests/test_zircon_build_zui_asset_owner_boundaries.py
  - Cargo.toml
  - zircon_hub/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/platform.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/timestamp.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support/runtime_ui_fixture.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/assets/shader.wgsl
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_multi_root_dedupe.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_asset_roots_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_asset_root_plan_visibility.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_staged_wgpu_handoff_command_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_no_proxy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_geometry_source_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_validation_report_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_export_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/manifest_contract.rs
implementation_files:
  - tools/dev-fast-build.ps1
  - tools/zircon_build.py
  - tools/zircon_build_asset_staging.py
  - tools/zircon_build_plugin_assets.py
  - tools/zircon_build_plugin_manifest_contract.py
  - tools/zircon_build_plugin_packages.py
  - tools/zircon_build_plugin_selection.py
  - tools/zircon_build_plugin_shader_descriptors.py
  - tools/zircon_build_plugin_workspace_crates.py
  - tools/zircon_build_zui_assets.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_resource_registry.py
  - tools/zircon_build_shader_prewarm_report_contract.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/zircon_build_shader_prewarm_acceptance.py
  - tools/zircon_build_shader_prewarm_written_variants.py
  - tools/tests/test_zircon_build_shader_prewarm.py
  - tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_command_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_cache_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py
  - tools/tests/test_zircon_build_plugin_carriers.py
  - tools/tests/test_zircon_build_asset_staging_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_asset_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_catalog_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_manifest_contract_owner_boundaries.py
  - tools/tests/test_zircon_build_plugin_shader_descriptor_owner_boundaries.py
  - tools/tests/test_plugin_docs_current_status_zircon_build_plugin_catalog_owner_split.py
  - tools/tests/test_plugin_docs_current_status_zircon_build_asset_staging_owner_split.py
  - tools/tests/test_plugin_docs_current_status_zircon_build_plugin_manifest_contract_owner_split.py
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_validation_report_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_export_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/manifest_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_multi_root_dedupe.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_asset_root_plan_visibility.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_staged_wgpu_handoff_command_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_no_proxy.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/platform.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/timestamp.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support/runtime_ui_fixture.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - user: 2026-05-03 add tools/zircon_build.py for staged editor/runtime/plugin builds
  - user: 2026-05-04 confirm editor/runtime asset staging and exported lookup support
  - user: 2026-05-04 add file-backed exported editor/runtime diagnostics
  - user: 2026-05-13 stop packaging legacy `.ui.toml` schema assets after the editor UI v2 hard cut
  - user: 2026-05-26 keep packaged `.zui` component assets alongside v2 UI documents
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - .codex/plans/zircon_hub 独立启动器设计.md
  - docs/engine-architecture/runtime-editor-pluginized-export.md
  - docs/superpowers/plans/2026-05-01-runtime-interface-cdylib-loader.md
tests:
  - rustc --edition=2021 --test zircon_runtime/src/tests/runtime_absorption/structure_convention.rs; exact `runtime_15_no_oversized_production_files`, `runtime_15_shader_prewarm_manifest_tests_are_folder_backed`, `runtime_15_shader_prewarm_asset_revision_export_is_wired`, and `runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired` passed 1/1 each (2026-07-03 Runtime 15 M4 shader prewarm manifest path helper owner split; package Cargo deferred)
  - rustc --edition=2021 --test zircon_runtime/src/tests/runtime_absorption/structure_convention.rs; exact shader-prewarm owner guard sync set passed 8/8 and full standalone structure sweep passed 622/622 (2026-07-03 Runtime 15 M4 shader prewarm owner guard sync; package Cargo deferred)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never (2026-07-01 shader prewarm source-hash helper support fix: passed with existing warnings)
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never (2026-07-01 editor text swash/subpixel build gate after shader prewarm source-hash helper support fix: passed with existing warnings)
  - python -m py_compile tools/zircon_build.py
  - python -m py_compile tools/zircon_build.py tools/zircon_build_asset_staging.py tools/tests/test_zircon_build_asset_staging_owner_boundaries.py tools/tests/test_plugin_docs_current_status_zircon_build_asset_staging_owner_split.py (2026-07-03 plugins_13_m5_t1_zircon_build_asset_staging_owner_split: build asset staging owner split passed)
  - python -m unittest tools.tests.test_zircon_build_asset_staging_owner_boundaries tools.tests.test_zircon_build_plugin_carriers.ZirconBuildPluginCarrierTests.test_zircon_build_rejects_staged_zui_document_kind_drift tools.tests.test_plugin_docs_current_status_zircon_build_asset_staging_owner_split (2026-07-03 plugins_13_m5_t1_zircon_build_asset_staging_owner_split: `stage_engine_assets`, `copy_tree_contents`, and `copy_resource_dirs` live in `zircon_build_asset_staging.py`; staged `.zui` `asset.kind` gate remains wired through `validate_staged_engine_asset_suffix`)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_plugin_packages.py tools/zircon_build_plugin_selection.py tools/zircon_build_plugin_workspace_crates.py tools/tests/test_zircon_build_plugin_catalog_owner_boundaries.py tools/tests/test_plugin_docs_current_status_zircon_build_plugin_catalog_owner_split.py (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_catalog_owner_split: build plugin catalog owner split passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_catalog_owner_boundaries tools.tests.test_zircon_build_plugin_carriers.ZirconBuildPluginCarrierTests.test_zircon_build_classifies_forms_from_manifest tools.tests.test_plugin_docs_current_status_zircon_build_plugin_catalog_owner_split (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_catalog_owner_split: `CargoPackage` and `PluginPackage` live in `zircon_build_plugin_packages.py`; `filter_plugins_by_carrier` and `select_plugins` live in `zircon_build_plugin_selection.py`; `discover_plugin_workspace_crates` lives in `zircon_build_plugin_workspace_crates.py`)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_plugin_manifest_contract.py tools/tests/test_zircon_build_plugin_manifest_contract_owner_boundaries.py tools/tests/test_plugin_docs_current_status_zircon_build_plugin_manifest_contract_owner_split.py (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_manifest_contract_owner_split: build plugin manifest contract owner split passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_manifest_contract_owner_boundaries tools.tests.test_zircon_build_plugin_carriers.ZirconBuildPluginCarrierTests.test_zircon_build_rejects_plugin_manifest_missing_distribution_forms tools.tests.test_zircon_build_plugin_carriers.ZirconBuildPluginCarrierTests.test_zircon_build_rejects_plugin_manifest_unknown_distribution_form tools.tests.test_plugin_docs_current_status_zircon_build_plugin_manifest_contract_owner_split (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_manifest_contract_owner_split: `PLUGIN_DISTRIBUTION_FORM_DIST`, `require_distribution_forms`, and `collect_module_crate_names` live in `zircon_build_plugin_manifest_contract.py`; `distribution.forms`, `modules`, `optional_features`, and `feature_extensions` behavior remained covered)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_plugin_shader_descriptors.py tools/tests/test_zircon_build_plugin_shader_descriptor_owner_boundaries.py (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_shader_descriptor_owner_split: build plugin shader descriptor owner split passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_shader_descriptor_owner_boundaries tools.tests.test_zircon_build_plugin_carriers.ZirconBuildPluginCarrierTests.test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids tools.tests.test_zircon_build_plugin_carriers.ZirconBuildPluginCarrierTests.test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_shader_descriptor_owner_split: `collect_shader_permutation_id_specs`, `collect_geometry_source_descriptors`, and `collect_shading_model_descriptors` live in `zircon_build_plugin_shader_descriptors.py`; `shader_permutation`, `geometry_sources`, and `shading_models` behavior remained covered)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_plugin_assets.py tools/tests/test_zircon_build_plugin_carriers.py tools/tests/test_zircon_build_plugin_asset_owner_boundaries.py (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_distribution_assets_zui_document_gate: build plugin distribution.assets `.zui` document gate passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_carriers.ZirconBuildPluginCarrierTests.test_zircon_build_rejects_distribution_assets_zui_document_kind_drift tools.tests.test_zircon_build_plugin_asset_owner_boundaries (2026-07-03 plugins_13_m5_t1_zircon_build_plugin_distribution_assets_zui_document_gate: `validate_plugin_distribution_assets_for_build` calls `plugin_validate_distribution_assets` and rejects `distribution.assets` matched .zui asset `asset.kind` drift; expected one of `component, style, theme_tokens, view`; `test_zircon_build_rejects_distribution_assets_zui_document_kind_drift` and `test_plugin_asset_roots_live_in_plugin_assets_owner` passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_zui_assets.py tools/tests/test_zircon_build_plugin_carriers.py tools/tests/test_zircon_build_zui_asset_owner_boundaries.py (2026-07-03 plugins_13_m5_t1_zircon_build_staged_zui_document_gate: staged `.zui` document gate passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_carriers tools.tests.test_zircon_build_zui_asset_owner_boundaries (2026-07-03 plugins_13_m5_t1_zircon_build_staged_zui_document_gate: `validate_staged_engine_asset_suffix` rejects staged engine asset matched .zui asset `asset.kind` drift; `test_zircon_build_rejects_staged_zui_document_kind_drift` and `test_staged_zui_asset_checks_live_in_zui_asset_owner` passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-27 build-tool prewarm dimension summary, shader permutation registry overlay, and CLI-id auto-export: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-27 shader permutation registry CLI-id auto-export: passed, 9 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin shader permutation registry auto-export: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin shader permutation registry auto-export: passed, 13 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_permutation_registry_contract.py (2026-06-28 Plugin shader permutation registry export contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_permutation_registry_contract (2026-06-28 Plugin shader permutation registry export contract: passed, 32 tests)
  - python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: passed; status render_plan08_project_plugin_registry_production_wrapper_no_proxy_wgpu_passed_product_renderdoc_deferred)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_wrapper_orchestration.ZirconBuildShaderPrewarmWrapperOrchestrationTests.test_runtime_server_wrapper_uses_client_features_for_preview_binary (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: passed; guard runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired)
  - cargo test -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-wrapper-no-proxy-0630-guard --message-format short --color never runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired -- --nocapture --test-threads=1 (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: passed, 1/1 with repository warnings)
  - python -u tools\zircon_build.py --targets runtime --plugins native_dynamic_fixture --out target\codex-plan08-wrapper-no-proxy-0630 --mode debug --runtime-features target-server --jobs 1 --prewarm-shaders --validate-wgpu-shaders --shader-asset-root target\codex-plan08-wrapper-no-proxy-0630\project_assets --shader-quality-tier medium --shader-geometry-source static (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: real no-proxy public command passed; runtime lib used target-server, preview executable used target-client, prewarm cargo run used target-server, report closed 18/18 written and 18/18 WGPU module validated; RenderDoc/product remains deferred)
  - python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin shading-model descriptor registration: passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin shading-model descriptor registration: passed, 3 tests)
  - python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin geometry-source descriptor registration: passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin geometry-source descriptor registration: passed, 4 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin shader asset roots auto-export: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin shader asset roots auto-export: passed, 20 tests)
  - python tools/zircon_build.py --targets runtime,plugins --plugins native_dynamic_fixture --out target/codex-plan08-plugin-asset-roots-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-27 Plugin shader asset roots auto-export: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_build_command_auto_export_registry_scans_all_asset_roots tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_plan_lists_asset_roots_for_registry_export tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_zircon_build_resolves_project_shader_asset_roots_for_prewarm (2026-06-29 Project shader asset roots auto-export: passed, 3 focused tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-29 Project shader asset roots auto-export: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_asset_roots_auto_export.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-29 Project shader asset roots auto-export: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed, 13 tests)
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root (2026-06-28 Runtime custom id staged fallback lookup contract: added; Cargo deferred under milestone-first cadence)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Runtime custom id staged fallback lookup contract: passed)
  - source/docs anchor scan, conflict marker scan, trailing-whitespace scan, scoped git diff --check (2026-06-28 Runtime custom id staged fallback lookup contract: passed; diff-check only reported LF/CRLF warnings)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm acceptance contract: RED then passed, 3 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/shader_prewarm.rs zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_shader_prewarm_tests.rs (2026-07-01 Runtime 15 M4 dynamic API shader prewarm tests owner split: status runtime_15_dynamic_api_shader_prewarm_tests_owner_split_static_passed_cargo_deferred; standalone structure guard runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner passed 1/1; standalone runtime_15_no_oversized_production_files passed 1/1; standalone plan_status passed 42/42; package-level Cargo deferred while external cargo/rustc lanes were active)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm runtime fallback layout contract: RED then passed, 5 tests; status render_plan08_build_tool_staged_prewarm_runtime_fallback_layout_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool staged prewarm runtime fallback layout contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_rejects_empty_success_report (2026-06-28 Build-tool staged prewarm nonempty success report acceptance: RED with old source-provenance error)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm nonempty success report acceptance: passed, 7 tests; status render_plan08_build_tool_staged_prewarm_nonempty_success_report_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_requires_written_variant_identity (2026-06-28 Build-tool staged prewarm written variant identity acceptance: RED with old source-provenance error)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm written variant identity acceptance: passed, 9 tests; status render_plan08_build_tool_staged_prewarm_written_variant_identity_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_rejects_partial_written_success_report (2026-06-28 Build-tool staged prewarm complete written count acceptance: RED with old source-provenance error, then passed after helper check)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool staged prewarm acceptance contract: passed, 30 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_permutation_registry_contract (2026-06-28 Build-tool staged prewarm acceptance contract closeout: passed, 3 tests after old build-root report-validator patch was moved to the staged acceptance entry point)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool staged prewarm written variant identity acceptance: passed, 64 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool staged prewarm complete written count acceptance: passed, 65 tests; status render_plan08_build_tool_staged_prewarm_complete_written_count_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_rejects_source_provenance_count_mismatch tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_accepts_source_provenance_counts (2026-06-28 Build-tool source provenance totals match contract: RED then passed, 2 tests; status render_plan08_build_tool_source_provenance_totals_match_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool source provenance totals match contract: passed, 66 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool source provenance report test owner split: passed, 32 tests; status render_plan08_build_tool_source_provenance_report_tests_owner_split_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool source provenance report test owner split: passed, 66 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_report_contract.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py tools/tests/test_zircon_build_shader_prewarm_command_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_permutation_registry_contract.py (2026-06-28 Build-tool source provenance report test owner split: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool WGPU validation totals match contract: RED first failed with RuntimeError not raised, then passed, 3 tests; status render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool WGPU validation totals match contract: passed, 30 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_report_contract.py tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool WGPU validation totals match contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool WGPU validation totals match contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool product Base pass acceptance contract: RED first failed with unexpected keyword argument expected_pass_types, then passed, 32 tests; status render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool product Base pass acceptance contract: passed, 70 tests)
  - python -m py_compile tools/zircon_build_shader_prewarm_report_contract.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py (2026-06-28 Build-tool product Base pass acceptance contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs (2026-06-28 Build-tool product Base pass acceptance contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool product material mesh pass acceptance contract: RED then passed, 12 tests; status render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool product material mesh pass acceptance contract: passed, 90 tests)
  - python -m py_compile tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool product material mesh pass acceptance contract: passed with PYTHONPYCACHEPREFIX isolation)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs (2026-06-28 Build-tool product material mesh pass acceptance contract: passed)
  - source/docs anchor scan, conflict marker scan, trailing-whitespace scan, line-count scan, scoped git diff --check (2026-06-28 Build-tool product material mesh pass acceptance contract: passed; diff-check only reported LF/CRLF warnings)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool cache quality/geometry identity contract: RED first failed with unexpected keyword argument expected_quality_tiers/expected_geometry_sources, then passed, 29 tests; status render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool cache quality/geometry identity contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool cache quality/geometry identity contract: passed, 73 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs (2026-06-28 Build-tool cache quality/geometry identity contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool cache dimension combination contract: RED first failed with RuntimeError not raised, then passed, 20 tests; status render_plan08_build_tool_cache_dimension_combination_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool cache dimension combination contract: passed, 30 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool cache dimension combination contract: passed, 74 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool cache dimension combination contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs (2026-06-28 Build-tool cache dimension combination contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool cache custom id combination contract: RED first failed with RuntimeError not raised, then passed, 21 tests; status render_plan08_build_tool_cache_custom_id_combination_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool cache custom id combination contract: passed, 31 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool cache custom id combination contract: passed, 75 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool cache custom id combination contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs (2026-06-28 Build-tool cache custom id combination contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_report_contract.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py tools/tests/test_zircon_build_shader_prewarm_command_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_permutation_registry_contract.py (2026-06-28 Build-tool staged prewarm acceptance contract closeout: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool staged prewarm acceptance contract closeout: passed)
  - rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs zircon_runtime/src/bin/zircon_shader_prewarm/run.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_multi_root_dedupe.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Staged shader resource registry multi-root dedupe: passed)
  - python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader asset-root plan visibility: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool shader asset-root plan visibility: passed, 15 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_plan_lists_runtime_fallback_handoff_paths (2026-06-28 Build-tool shader asset-root plan visibility fallback handoff extension: RED then passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader asset-root plan visibility fallback handoff extension: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_asset_root_plan_visibility.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader asset-root plan visibility: passed)
  - python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Prewarm WGPU validation report summary: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_dimension_summary_lines_accept_rust_count_field_names tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_dimension_summary_lines_format_wgpu_module_validation_counts (2026-06-28 Prewarm WGPU validation report summary: passed, 2 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Prewarm WGPU validation report summary: passed, 17 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/dynamic_api/shader_prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_validation_report_summary.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm WGPU validation report summary: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool WGPU validation report contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_prints_summary_before_raising_nonzero_exit tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_requires_wgpu_validation_when_requested tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_accepts_wgpu_validation_counts (2026-06-28 Build-tool WGPU validation report contract: passed, 4 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool WGPU validation report contract: passed, 20 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool WGPU validation report contract: passed)
  - python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Shader prewarm source provenance summary: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_dimension_summary_lines_format_source_provenance (2026-06-28 Shader prewarm source provenance summary: RED then passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Shader prewarm source provenance summary: passed, 21 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs zircon_runtime/src/dynamic_api/shader_prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_summary.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Shader prewarm source provenance summary: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool source provenance report contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_requires_source_provenance_when_requested tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_accepts_source_provenance_counts (2026-06-28 Build-tool source provenance report contract: RED then passed, 3 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool source provenance report contract: passed, 23 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_report_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool source provenance report contract: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader resource registry export contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_uses_same_acceptance_entry_for_explicit_registry tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_requires_resource_records tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_wrapped_resources tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_raw_array (2026-06-28 Build-tool shader resource registry export contract: RED then passed, 5 tests; explicit registry handoff later moved into staged acceptance)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool shader resource registry export contract: passed, 27 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_export_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader resource registry export contract: passed)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_resource_registry_export_contract_is_wired --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-registry-export-contract-0628 --message-format short --color never -- --nocapture (2026-06-28 Build-tool shader resource registry export contract: blocked before compile because Cargo.lock would need update under --locked; not counted as passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader resource registry report correlation: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_rejects_missing_report_source_locator tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_report_source_locator tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_ignores_builtin_report_sources (2026-06-28 Build-tool shader resource registry report correlation: RED then passed, 4 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool shader resource registry report correlation: passed, 30 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader resource registry report correlation: passed)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-registry-report-correlation-0628 --message-format short --color never -- --nocapture (2026-06-28 Build-tool shader resource registry report correlation: timed out after 120 seconds with no test result; no residual cargo/rustc process; not counted as passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool resource registry written source-label correlation: RED first failed with RuntimeError not raised, then passed, 28 tests; status render_plan08_build_tool_resource_registry_written_source_label_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool resource registry usable shader revision contract: RED first failed with RuntimeError not raised for non-Shader and zero-revision records, then passed, 30 tests; status render_plan08_build_tool_resource_registry_usable_shader_revision_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry contract test owner split: passed, 30 tests; status render_plan08_build_tool_resource_registry_contract_tests_owner_split_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry ResourceRecord wire-shape contract: RED first failed with RuntimeError not raised, then passed, 10 tests; status render_plan08_build_tool_resource_registry_record_shape_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry enum wire-shape contract: RED first failed with TypeError on dict enum, then passed, 11 tests; status render_plan08_build_tool_resource_registry_enum_wire_shape_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry numeric width contract: RED first failed with RuntimeError not raised for u64/u32 overflow, then passed, 13 tests; status render_plan08_build_tool_resource_registry_numeric_width_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry locator wire-shape contract: RED first failed with RuntimeError not raised and a rejected valid duplicate-separator path, then passed, 16 tests; status render_plan08_build_tool_resource_registry_locator_wire_shape_python_passed_cargo_deferred)
  - PYTHONPYCACHEPREFIX=%TEMP%\zircon-codex-pycache-plan08-locator python -m py_compile tools\zircon_build_shader_prewarm.py tools\zircon_build_shader_resource_registry.py tools\tests\test_zircon_build_shader_prewarm_resource_registry_contract.py (2026-06-28 Build-tool resource registry locator wire-shape contract: passed after local __pycache__ lock was bypassed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool resource registry locator wire-shape contract: passed, 87 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool shader resource registry-backed locator correlation: RED first failed with RuntimeError not raised for missing lib/package report source locators, then passed, 18 tests; status render_plan08_build_tool_resource_registry_backed_locator_correlation_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Resource registry ready shader revision contract: passed, 19 tests; status render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred; includes test_validate_registry_export_contract_rejects_non_ready_report_source_record)
  - python -m py_compile tools/zircon_build_shader_resource_registry.py tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py (2026-06-28 Resource registry ready shader revision contract: passed)
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs::shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only (2026-06-28 Resource registry ready shader revision contract: added; Cargo deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_rejects_duplicate_written_variant_identity (2026-06-28 Build-tool written variant uniqueness contract: RED first failed with RuntimeError not raised, then passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract.ZirconBuildShaderPrewarmCacheContractTests.test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity (2026-06-28 Build-tool written variant uniqueness contract: RED first failed with RuntimeError not raised, then passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension contract: RED then passed, 4 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py (2026-06-28 Build-tool shader prewarm report dimension contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader prewarm report dimension contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension complete-count contract: RED then passed, 7 tests; status render_plan08_build_tool_report_dimension_complete_counts_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension totals match contract: RED then passed, 8 tests; status render_plan08_build_tool_report_dimension_totals_match_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool shader prewarm report dimension totals match contract: passed, 92 tests)
  - python -m py_compile tools/zircon_build_shader_prewarm_report_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py (2026-06-28 Build-tool shader prewarm report dimension totals match contract: passed with isolated PYTHONPYCACHEPREFIX)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs (2026-06-28 Build-tool shader prewarm report dimension totals match contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_uses_same_acceptance_entry_for_explicit_registry tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool shader prewarm cache artifact contract: RED then passed, 5 tests; explicit registry handoff later moved into staged acceptance)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Build-tool shader prewarm cache artifact contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool shader prewarm cache artifact contract: passed, 38 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader prewarm cache artifact contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm report cache identity contract: RED then passed, 8 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Prewarm report cache identity contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm report cache identity contract: passed, 41 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm report cache identity contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache runtime layout contract: RED then passed, 10 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Prewarm cache runtime layout contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache runtime layout contract: passed, 43 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm cache runtime layout contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache hash shape contract: RED then passed, 11 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Prewarm cache hash shape contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache hash shape contract: passed, 44 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm cache hash shape contract: passed)
- python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_report_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_permutation_registry_contract.py (2026-06-28 Build-tool shader permutation id report dimension contract: passed)
- python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract (2026-06-28 Build-tool shader permutation id report dimension contract: passed, 8 tests)
- PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_report_contract.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_permutation_registry_contract.py (2026-06-28 Prewarm cache custom id correlation contract: passed)
- PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract (2026-06-28 Prewarm cache custom id correlation contract: passed, 52 tests)
- PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_command_contract (2026-06-28 Build-tool staged WGPU handoff command contract: RED then passed, 2 tests; status render_plan08_build_tool_staged_wgpu_handoff_command_contract_python_passed_cargo_deferred)
- PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool staged WGPU handoff command contract: passed, 30 tests; runtime_15_shader_prewarm_staged_wgpu_handoff_command_contract_is_wired guards docs/status anchors)
  - python tools/zircon_build.py --help
  - python tools/zircon_build.py --list-plugins
  - python tools/zircon_build.py --targets editor,runtime --out <dir> --mode debug --dry-run
  - python tools/zircon_build.py --targets hub,editor,runtime --out <dir> --mode debug --dry-run
  - python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out <dir> --mode debug --dry-run
  - python tools/zircon_build.py --targets runtime --out <dir> --mode profiling --runtime-features target-client,profiling,profiling-tracy --dry-run
  - python tools/zircon_build.py --targets runtime --out <dir> --mode debug --prewarm-shaders --dry-run
  - python tools/zircon_build.py --targets runtime --out target/codex-plan08-build-tool-prewarm-summary-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-27 build-tool prewarm dimension summary: passed)
  - python tools/zircon_build.py --targets runtime --out <dir> --mode debug --prewarm-shaders --shader-shading-model-id custom:subsurface=16 --dry-run
  - python tools/zircon_build.py --targets runtime --out <dir> --mode debug --prewarm-shaders --shader-geometry-source-id custom:gpu-driven=4 --dry-run
  - python tools/zircon_build.py --targets runtime --out <dir> --mode debug --prewarm-shaders --shader-permutation-registry <registry.json> --dry-run
  - python tools/zircon_build.py --targets runtime --out target/codex-plan08-permutation-registry-dry-run --mode debug --prewarm-shaders --shader-permutation-registry Project/shader_permutation_registry.json --dry-run (2026-06-27 shader permutation registry overlay: passed)
  - python tools/zircon_build.py --targets runtime --out target/codex-plan08-permutation-registry-auto-export-dry-run --mode debug --prewarm-shaders --shader-geometry-source-id custom:gpu-driven=4 --shader-shading-model-id custom:toon=16 --dry-run (2026-06-27 shader permutation registry auto-export: passed)
  - python tools/zircon_build.py --targets runtime,plugins --plugins virtual_geometry --out target/codex-plan08-plugin-permutation-registry-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-27 Plugin shader permutation registry auto-export: passed)
  - python tools/zircon_build.py --targets runtime --out <dir> --mode debug --prewarm-shaders --shader-resource-registry <resources.json> --dry-run
  - target: ./tools/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride "target-client profiling profiling-tracy"
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir <target-dir>
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-prewarm-diagnostics-check-0627 --message-format short --color never (2026-06-27 prewarm report dimension diagnostics: passed with existing warnings)
  - cargo test -p zircon_runtime --lib render_shader_variant_prewarm_report_groups_written_and_failed_dimensions --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-prewarm-diagnostics-check-0627 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27 prewarm report dimension diagnostics: timed out after 604s with no test result; not counted as passed)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug
  - python -c "from pathlib import Path; import importlib.util, sys; spec = importlib.util.spec_from_file_location('zb', 'tools/zircon_build.py'); zb = importlib.util.module_from_spec(spec); sys.modules[spec.name] = zb; spec.loader.exec_module(zb); assert zb.should_skip_staged_engine_asset(Path('ui/editor/editor_widgets.ui.toml')); assert not zb.should_skip_staged_engine_asset(Path('ui/editor/ui_asset_editor.zui')); assert not zb.should_skip_staged_engine_asset(Path('ui/editor/components/workbench/shell/activity_drawer_window.zui')); assert not zb.should_skip_staged_engine_asset(Path('fonts/default.font.toml'))"
  - powershell: Get-ChildItem E:\zircon-build\ZirconEngine\assets\ui -Recurse -File -Filter *.ui.toml | Where-Object { $_.Name -notlike '*.v2.ui.toml' } returns no files
  - E:\zircon-build\ZirconEngine\zircon_editor.exe smoke run with E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log
doc_type: workflow-detail
---

# Zircon Build Tool

`tools/zircon_build.py` is the staged local build entry point for producing a
runnable `ZirconEngine` directory from the repository checkout. It builds Hub,
editor, runtime, and selected plugins into separate Cargo target directories, then
copies only deployable runtime artifacts into the staged engine directory.

Status `plugins_13_m5_t1_zircon_build_asset_staging_owner_split` keeps staged
engine asset copying and plugin resource directory copy in
`tools/zircon_build_asset_staging.py`. The owner provides
`stage_engine_assets`, `copy_tree_contents`, `copy_asset_file`, and
`copy_resource_dirs`, and continues to call `validate_staged_engine_asset_suffix`
so staged `.zui` files must declare `asset.kind` as one of `component, style,
theme_tokens, view`. Behavior coverage is
`test_zircon_build_rejects_staged_zui_document_kind_drift`, and owner coverage is
`test_asset_staging_lives_in_asset_staging_owner` plus
`test_asset_staging_owner_preserves_zui_and_resource_copy_semantics`.

Status `plugins_13_m5_t1_zircon_build_plugin_catalog_owner_split` keeps plugin
catalog models, selection rules, and workspace crate discovery outside the main
build script. `tools/zircon_build_plugin_packages.py` owns `CargoPackage` and
`PluginPackage`; `tools/zircon_build_plugin_selection.py` owns
`filter_plugins_by_carrier` and `select_plugins`; and
`tools/zircon_build_plugin_workspace_crates.py` owns
`discover_plugin_workspace_crates`. Behavior coverage is
`test_zircon_build_classifies_forms_from_manifest`, and owner
coverage is `test_plugin_catalog_models_selection_and_workspace_crates_have_owners`
plus `test_plugin_catalog_owners_preserve_package_workspace_and_selection_semantics`.

Status `plugins_13_m5_t1_zircon_build_plugin_manifest_contract_owner_split`
keeps plugin manifest distribution contracts in
`tools/zircon_build_plugin_manifest_contract.py`. The owner provides
`PLUGIN_DISTRIBUTION_FORM_DIST`, `require_distribution_forms`,
`distribution_table`, `normalize_optional_string`, and
`collect_module_crate_names`, covering `distribution.forms`, root `modules`,
`optional_features[].modules`, and `feature_extensions[].modules` projection into
build plugin package distribution forms and crate selection. Behavior coverage is
`test_zircon_build_rejects_plugin_manifest_missing_distribution_forms` and
`test_zircon_build_rejects_plugin_manifest_unknown_distribution_form`, and owner
coverage is `test_plugin_manifest_contract_lives_in_plugin_manifest_contract_owner`.

Status `plugins_13_m5_t1_zircon_build_plugin_distribution_assets_zui_document_gate`
keeps plugin asset-root discovery in `tools/zircon_build_plugin_assets.py`.
`validate_plugin_distribution_assets_for_build` reuses
`plugin_validate_distribution_assets`, so any plugin `distribution.assets`
entry that matches a `.zui` file must declare `asset.kind` as one of
`component, style, theme_tokens, view`; failures use the shared
`matched .zui asset ... asset.kind ...` diagnostic. Behavior coverage is
`test_zircon_build_rejects_distribution_assets_zui_document_kind_drift`, and
owner coverage is `test_plugin_asset_roots_live_in_plugin_assets_owner`.

Status `plugins_13_m5_t1_zircon_build_plugin_shader_descriptor_owner_split`
keeps plugin shader contribution manifest parsing in
`tools/zircon_build_plugin_shader_descriptors.py`. The owner provides
`collect_shader_permutation_id_specs`, `collect_geometry_source_descriptors`,
and `collect_shading_model_descriptors`, covering `shader_permutation`,
`geometry_sources`, and `shading_models` projection into shader prewarm ID specs
and descriptor rows. Behavior coverage includes
`test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids`
and `test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids`;
owner coverage is `test_plugin_shader_descriptors_live_in_plugin_shader_descriptor_owner`.

## Output Layout

Given `--out E:\builds\zircon`, the tool writes:

```text
E:\builds\zircon\
  ZirconEngine\
    zircon_hub.exe
    zircon_editor.exe
    zircon_runtime.exe
    zircon_runtime.dll
    assets\
      ui\
      fonts\
      icons\
      viewport_gizmos\
    plugins\
      native_plugins.toml
      <plugin-id>\
        plugin.toml
        native\
          <native-plugin-dylib>
  targets\
    editor\
    hub\
    runtime\
    plugins\
      <plugin-id>\
```

`ZirconEngine` is the runnable/staged payload. `targets` contains Cargo
intermediate artifacts and stays outside the runtime payload. This split prevents
Cargo `debug/deps` layout details from leaking into the final engine directory.

The `assets` directory is a merged engine asset root. The build tool stages
`zircon_editor/assets` and `zircon_runtime/assets` into the same payload root so
authored `res://ui/...`, runtime fixture, icon, font, and viewport-gizmo paths
work from the exported directory. If both crate asset roots provide the same
relative file with different bytes, staging fails instead of silently choosing
one copy; identical duplicates are treated as idempotent.

UI template staging is now v2-only for packaged payloads. Legacy `.ui.toml`
authoring and migration inputs live under
`zircon_editor/src/tests/fixtures/ui_zui/**`, outside the deployable asset
roots. `tools/zircon_build.py` still defensively skips non-v2 `assets/ui/**`
files if one is reintroduced, and guard tests reject that regression. Files
ending in `.v2.ui.toml` are staged for root view/style documents, and `.zui`
component assets are staged for imported widget prototypes. Non-UI assets such
as fonts, icons, SVGs, and viewport gizmo resources are staged unchanged. This
prevents a packaged `zircon_editor.exe` from resolving old schema assets from
the exported directory while the source tree can keep focused migration coverage
until those fixtures are rewritten.

The hub target stages `zircon_hub.exe` as the default launcher entry. It does not
replace the editor/runtime targets: a complete local desktop payload should include
`hub,editor,runtime` so Hub can stay open while launching `zircon_editor` child
processes against the staged runtime library.

`--prewarm-shaders` adds a shader-cache prewarm step after runtime/editor assets
are staged. The tool runs the `zircon_shader_prewarm` binary with the staged
`ZirconEngine` directory as its project root, writes cache entries into
`ZirconEngine/cache/shader_variants`, and writes
`ZirconEngine/cache/shader_variants_report.json`. The report contains the
top-level requested/written/failed totals plus
`dimension_summary.pass_types`, `dimension_summary.geometry_source_ids`,
`dimension_summary.shading_model_ids`, and
`dimension_summary.quality_tiers`, each with requested/written/failed counts so
build logs can identify which prewarm dimension missed. After a non-dry-run
prewarm command returns, the build tool reads the same report and prints a
compact `shader prewarm dimension summary` grouped by pass type, geometry source
id, shading model id, and quality tier before propagating any non-zero exit code
so CI output exposes missed dimensions without opening the JSON artifact. The
prewarm producer accepts
explicit manifest JSON and can scan staged asset roots for `.zshader`, `.wgsl`,
and `.zmaterial` sources. Shader-prewarm parsing and command assembly live in
`tools/zircon_build_shader_prewarm.py`, while `tools/zircon_build.py` keeps the
top-level staging orchestration. `--shader-quality-tier` expands requested
quality tiers, and `--shader-geometry-source` expands built-in static, skinned,
morphed, and skinned-morphed shader variant keys. `--shader-shading-model-id
custom:name=16` forwards explicit project/plugin shading-model ids to the
staged prewarm tool so `.zmaterial` files with `lighting_model =
"custom:name"` can write plugin-range `ShaderVariantKey.shading_model` values
instead of falling back to StandardPBR. `--shader-geometry-source-id
custom:name=4` forwards explicit project/plugin geometry-source ids to the
staged prewarm tool; the tool validates the id is in the plugin range and then
adds it to the prewarm geometry dimension so asset-root shader requests can
write plugin-range `ShaderVariantKey.geometry_source` values. Implementation
files are `bin/zircon_shader_prewarm/args.rs`,
`bin/zircon_shader_prewarm/manifest.rs`, `tools/zircon_build.py`, and
`tools/zircon_build_shader_prewarm.py`; the focused manifest regression is
`shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids`,
the structure guard is
`runtime_15_shader_prewarm_custom_geometry_source_id_is_wired`, and current
status is
`render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`.
Shader permutation registry overlay is wired for staged prewarm build
invocation: `--shader-permutation-registry <registry.json>` forwards a
project/plugin shader permutation registry JSON file to the staged prewarm tool,
and `zircon_shader_prewarm` also auto-discovers
`shader_permutation_registry.json` at each `--asset-root` through
`shader_permutation_registry_paths`. The registry owner is
`bin/zircon_shader_prewarm/manifest/permutation_registry.rs`; it accepts
`geometry_source_ids` and `shading_model_ids` records, normalizes custom
tokens, validates plugin id ranges, and merges those ids into prewarm
`ShaderVariantKey.geometry_source` / `ShaderVariantKey.shading_model`
dimensions before asset manifests expand. The focused regressions are
`shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids`,
`shader_prewarm_permutation_registry_discovers_asset_root_registry`, and
`test_build_command_forwards_shader_permutation_registries`; the structure
guard is `runtime_15_shader_prewarm_permutation_registry_overlay_is_wired`,
and current status is
`render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred`.

Shader permutation registry auto-export is wired for the staged build path when
`--prewarm-shaders` receives explicit custom id inputs but no explicit
`--shader-permutation-registry`. `tools/zircon_build.py` owns
`shader_prewarm_permutation_registry_path`, which resolves to
`ZirconEngine/cache/shader_permutation_registry.json`; before a non-dry-run
prewarm command, `tools/zircon_build_shader_prewarm.py` writes that file with
`geometry_source_ids` and `shading_model_ids` records derived from
`--shader-geometry-source-id` and `--shader-shading-model-id`. The generated
file uses the same registry schema as the overlay reader, while explicit
`--shader-permutation-registry` remains the override path. The focused
regressions are
`test_build_command_uses_generated_shader_permutation_registry_for_custom_ids`,
`test_build_command_prefers_explicit_shader_permutation_registry`,
`test_generated_shader_permutation_registry_document_groups_custom_ids`, and
`test_write_generated_shader_permutation_registry_writes_json`; the structure
guard is
`runtime_15_shader_prewarm_permutation_registry_auto_export_is_wired`, and
current status is
`render_plan08_build_tool_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`.

Plugin shader permutation registry auto-export extends that generated-registry
path to selected plugin manifests. A plugin `plugin.toml` may include optional
`[shader_permutation]` records with `[[shader_permutation.geometry_source_ids]]`
or `[[shader_permutation.shading_model_ids]]` entries; each entry supplies a
custom `token` and plugin-range `id`. `tools/zircon_build.py` discovers those
records only for the selected plugins, and `tools/zircon_build_shader_prewarm.py`
merges them with explicit CLI id records before writing the generated
`shader_permutation_registry.json`. The focused regressions are
`test_zircon_build_discovers_plugin_shader_permutation_records`,
`test_generated_shader_permutation_registry_document_merges_selected_plugin_ids`,
and
`test_build_command_uses_generated_shader_permutation_registry_for_selected_plugin_ids`;
the structure guard is
`runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired`, and
current status is
`render_plan08_plugin_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`.
The non-dry-run generated-registry path is also validated before launching the
prewarm subprocess: `validate_shader_permutation_registry_export_contract(...)`
requires the generated `geometry_source_ids` and `shading_model_ids` arrays to
contain every selected-plugin and explicit CLI id spec expected by the current
build config. The focused regressions are
`test_validate_generated_registry_requires_selected_plugin_ids` and
`test_prewarm_shaders_validates_generated_registry_before_run`; current status
is
`render_plan08_plugin_shader_permutation_registry_export_contract_python_passed_cargo_deferred`.
After a successful prewarm subprocess, the report dimension contract also
requires those same selected-plugin and explicit CLI ids to appear in
`dimension_summary.geometry_source_ids` and `dimension_summary.shading_model_ids`
with positive requested counts. `tools/zircon_build_shader_prewarm_report_contract.py`
owns report summary formatting and report contract validation, while
`tools/zircon_build_shader_prewarm.py` re-exports the public report helpers for
existing callers. `prewarm_shaders(...)` passes
`expected_geometry_source_ids=shader_geometry_source_id_specs(config)` and
`expected_shading_model_ids=shader_shading_model_id_specs(config)` into
`validate_shader_prewarm_report_contract(...)`. The focused regressions are
`test_validate_report_contract_requires_requested_geometry_source_ids`,
`test_validate_report_contract_requires_requested_shading_model_ids`, and
`test_prewarm_shaders_passes_selected_custom_ids_to_report_contract`; current
status is
`render_plan08_build_tool_permutation_id_report_dimension_contract_python_passed_cargo_deferred`.

Selected plugin `[[shading_models]]` descriptors now feed the same staged
prewarm id discovery path. `tools/zircon_build.py::discover_plugins(...)` derives
`shader_shading_model_ids` from descriptor rows with `token` and `id`, so custom
shading-model plugins can keep `PluginPackageManifest.shading_models` as the
authoritative descriptor owner instead of duplicating a
`shader_permutation.shading_model_ids` row. The focused regression is
`test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids`; the
structure guard is
`runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired`,
and current status is
`render_plan08_plugin_shading_model_descriptor_registration_typecheck_python_passed_libtest_blocked_by_ui_input_error`.

Selected plugin `[[geometry_sources]]` descriptors now feed staged geometry-source
id discovery as well. `tools/zircon_build.py::discover_plugins(...)` derives
`shader_geometry_source_ids` from descriptor rows with `token` and `id`, then
dedupes those ids with legacy `shader_permutation.geometry_source_ids` entries.
This keeps `PluginPackageManifest.geometry_sources` authoritative for custom
geometry-source descriptors while preserving the old id-row input for staged
registries. The focused regression is
`test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids`;
the structure guard is
`runtime_15_shader_prewarm_plugin_geometry_source_descriptor_registration_is_wired`,
and current status is
`render_plan08_plugin_geometry_source_descriptor_registration_typecheck_python_cargo_check_passed_renderdoc_deferred`.

Explicit `--plugins` selection is also valid for `runtime` staged prewarm even
when the `plugins` target is not being built. In that mode the selected packages
only contribute shader ids, `geometry_source_descriptors`, and asset roots to the
prewarm command; plugin binary packaging still runs only when `plugins` appears
in `--targets`. This allows
`python tools/zircon_build.py --targets runtime --plugins virtual_geometry --prewarm-shaders --validate-wgpu-shaders`
to validate the VirtualGeometry descriptor shader path without requiring an
editor host artifact. The focused regression is
`test_zircon_build_selects_plugin_contributions_for_runtime_prewarm`, and the
2026-06-29 live gate passed in
`target\codex-plan08-live-wgpu-prewarm-0629`: report `requested=12`,
`written=12`, `failed=0`, WGPU module validation `validated=12`, and geometry
source ids `0` plus `4` across the six material pass types.

Plugin shader asset roots auto-export extends staged prewarm and staged shader
resource registry export to selected plugin package assets. `tools/zircon_build.py`
stores existing package asset roots on `PluginPackage.asset_roots`, resolving
top-level `asset_roots`, defaulting to an existing `assets` directory, and
supporting legacy `[distribution] assets = ["assets/**"]` declarations such as
`native_dynamic_fixture`. `tools/zircon_build_shader_prewarm.py` keeps staged
`ZirconEngine/assets` as the first `--asset-root` and appends selected plugin
asset roots, so the same `--export-resource-registry
ZirconEngine/cache/shader_resource_records.json` run scans engine assets and
selected plugin shader payloads. The focused regressions are
`test_build_command_includes_selected_plugin_asset_roots`,
`test_zircon_build_discovers_plugin_asset_roots_for_shader_prewarm`,
`test_zircon_build_discovers_distribution_assets_as_plugin_asset_roots`, and
`test_zircon_build_uses_existing_default_plugin_assets_root`; the structure guard
is `runtime_15_shader_prewarm_plugin_asset_roots_auto_export_is_wired`, and
current status is
`render_plan08_plugin_shader_asset_roots_auto_export_focused_tests_passed_cargo_deferred_renderdoc_deferred`.

Project shader asset roots auto-export extends the same staged prewarm and staged
shader resource registry export path to explicit project asset roots. Repeat
`--shader-asset-root <path>` to add project-local shader assets to the staged
prewarm scan; `BuildConfig.shader_asset_roots` stores the resolved roots, and
`shader_asset_root_paths_for_prewarm(config)` appends them after staged
`ZirconEngine/assets` and before selected plugin package roots. The final
prewarm command therefore passes one ordered `--asset-root` set to
`zircon_shader_prewarm`, and the default
`--export-resource-registry ZirconEngine/cache/shader_resource_records.json`
scan can include project `.zmeta` shader records without requiring a separate
explicit `--shader-resource-registry`. The focused regressions are
`test_zircon_build_resolves_project_shader_asset_roots_for_prewarm`,
`test_build_command_auto_export_registry_scans_all_asset_roots`, and
`test_prewarm_plan_lists_asset_roots_for_registry_export`; the structure guard
is `runtime_15_shader_prewarm_project_asset_roots_auto_export_is_wired`, and
current status is
`render_plan08_project_shader_asset_roots_auto_export_python_static_passed_cargo_deferred`.
This closes project asset-root participation in auto-export, not real staged
WGPU execution, full live project/plugin registry export, RenderDoc/product
capture, or product miss=0 acceptance.

Build-tool shader asset-root plan visibility makes that selected-root set
auditable before a prewarm run starts. `print_shader_prewarm_plan(...)` prints
`shader asset roots: ...` from `shader_asset_root_paths_for_prewarm(config)`,
so dry-run output and the final `--asset-root` command arguments share one root
owner. The focused regressions are
`test_prewarm_plan_lists_asset_roots_for_registry_export` and
`test_build_command_auto_export_registry_scans_all_asset_roots`; the structure
guard is `runtime_15_shader_prewarm_asset_root_plan_visibility_is_wired`, and
current status is
`render_plan08_build_tool_shader_asset_root_plan_visibility_python_passed_cargo_deferred`.
The same plan visibility owner now also prints `shader prewarm cache root`,
`shader prewarm report`, and `shader runtime fallback root`, so dry-run output
shows the staged cache/report handoff path before the acceptance helper reads a
real report. `test_prewarm_plan_lists_runtime_fallback_handoff_paths` locks
those lines. Closeout verification passed the build-helper Python combo 60/60
plus py_compile, rustfmt, per-doc anchors, conflict/trailing-whitespace,
line-budget, and scoped diff-check.

Prewarm opt-in WGPU shader-module validation is available through
`--validate-wgpu-shaders` when `--prewarm-shaders` is enabled. The build helper
keeps normal staged prewarm on the existing Naga-only cache-write path unless
that flag is present; when enabled, it appends `--validate-wgpu-modules` to the
internal `zircon_shader_prewarm` command. The runtime prewarm tool then creates
an offscreen WGPU shader module for each request after WGSL validation and before
writing `ZirconEngine/cache/shader_variants`. The focused regression is
`test_build_command_forwards_wgpu_shader_module_validation`, the structure guard
is `runtime_15_shader_prewarm_wgpu_module_validation_is_wired`, and current
status is
`render_plan08_prewarm_wgpu_module_validation_gate_python_cargo_check_passed_runtime_run_timeout_deferred`.
Python and scoped Cargo checks passed for this handoff; an actual
`cargo run ... --validate-wgpu-modules` attempt timed out during Windows
compilation and is not counted as runtime execution evidence.

Prewarm opt-in WGPU render-pipeline validation is available through
`--validate-wgpu-pipelines` when `--prewarm-shaders` is enabled. This is stricter
than `--validate-wgpu-shaders`: the build helper appends
`--validate-wgpu-pipelines`, the runtime prewarm tool creates an offscreen WGPU
device and mesh validation pipeline layout, and each full-template mesh prewarm
request must create the corresponding WGPU render pipeline before its staged
cache entry is written. The pipeline report field is
`wgpu_pipeline_validation`, the focused command regression is
`test_build_command_forwards_wgpu_shader_pipeline_validation`, the structure
guard is `runtime_15_shader_prewarm_wgpu_render_pipeline_validation_is_wired`,
and current status is
`render_plan08_prewarm_wgpu_render_pipeline_validation_gate_focused_tests_passed_product_deferred`.
Python command/report/acceptance contracts passed for this handoff, and the
focused WGPU render-pipeline validation tests passed by running the generated
`zircon_runtime` lib-test binary filters directly after Cargo had produced the
test executable. Cargo wrapper reruns for those filters timed out while
rebuilding/relinking the large lib-test binary, so those wrapper timeouts are
not counted as failed WGPU assertions. RenderDoc/product capture and
second-launch miss=0 remain separate Plan 08 gates.

Project/plugin registry production wrapper no-proxy WGPU run is covered by
status
`render_plan08_project_plugin_registry_production_wrapper_no_proxy_wgpu_passed_product_renderdoc_deferred`.
For `--targets runtime --runtime-features target-server`, `build_runtime(...)`
now keeps the runtime library and `zircon_shader_prewarm` path on the requested
`target-server` feature set, while `BuildConfig.runtime_preview_feature_arg`
builds `zircon_app --bin zircon_runtime` with `target-client` plus any
non-target extra features. The focused regression is
`test_runtime_server_wrapper_uses_client_features_for_preview_binary`, and the
structure guard is
`runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired`.
The guard passed as a focused target-server `zircon_runtime` lib test after the
wrapper feature split was applied.
The real no-proxy public wrapper command with `native_dynamic_fixture`,
`--prewarm-shaders`, `--validate-wgpu-shaders`, a project shader asset root, and
medium/static shader dimensions completed with runtime lib `target-server`,
preview executable `target-client`, prewarm cargo run `target-server`,
18/18 requested/written variants, 18/18 WGPU module validation, and Ready
project/plugin shader records. This run uses WGPU module validation; the
stricter render-pipeline validation flag is recorded by the separate pipeline
validation slice.

Prewarm WGPU validation report summary makes that opt-in path auditable after a
real run. `ShaderVariantPrewarmReport.wgpu_module_validation` records whether
module validation was enabled and how many variants were requested, validated,
failed, or skipped because WGSL validation failed first. The build helper prints
that data as `WGPU module validation: enabled requested=... validated=...
failed=... skipped=...`, and it also accepts Rust report fields named
`requested_count`, `written_count`, and `failed_count` when formatting dimension
summaries. The focused regressions are
`test_dimension_summary_lines_accept_rust_count_field_names` and
`test_dimension_summary_lines_format_wgpu_module_validation_counts`; the same
summary path prints `WGPU render pipeline validation: ...` when
`wgpu_pipeline_validation` is present. The
structure guard is
`runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired`, and current
status is
`render_plan08_prewarm_wgpu_validation_report_summary_python_passed_cargo_deferred`.
Cargo/WGPU runtime execution remains a later Plan 08 acceptance gate.

Build-tool WGPU validation report contract turns that report into a success
condition when `--validate-wgpu-shaders` is requested. After a successful
`zircon_shader_prewarm` process, `prewarm_shaders(...)` calls
`validate_shader_prewarm_report_contract(...)`; the helper requires
`wgpu_module_validation.enabled`, a positive requested count, every requested
variant counted as validated, and zero failed/skipped entries. Non-zero prewarm
exits still print the summary and then propagate `CalledProcessError` without
running the contract check. The focused regressions are
`test_prewarm_shaders_validates_wgpu_report_after_success`,
`test_validate_report_contract_requires_wgpu_validation_when_requested`, and
`test_validate_report_contract_accepts_wgpu_validation_counts`; the structure
guard is `runtime_15_shader_prewarm_wgpu_report_contract_is_wired`, and current
status is
`render_plan08_build_tool_wgpu_report_contract_python_passed_cargo_deferred`.
The same contract helper also accepts `require_wgpu_pipeline_validation=True`
and applies the totals rule to `wgpu_pipeline_validation` for the stricter
render-pipeline prewarm path.

Build-tool WGPU validation totals match contract extends that success condition
to the top-level report counts. `validate_shader_prewarm_report_contract(...)`
now compares `wgpu_module_validation.requested_count`, `validated_count`, and
`failed_count` with top-level `requested_count`, `written_count`, and
`failed_count`; mismatches raise
`shader prewarm WGPU module validation counts did not match report totals`.
The dedicated test owner is
`tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py`, covering
the required/positive WGPU report cases plus
`test_validate_report_contract_rejects_wgpu_validation_total_mismatch`.
`runtime_15_shader_prewarm_wgpu_report_contract_is_wired` reads that owner and
keeps the WGPU report-contract regressions out of the general prewarm test file.
Status:
`render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred`.
This is report-contract evidence only; real WGPU runtime execution remains open.

Shader prewarm source provenance summary makes the same report artifact explain
which shader source/template payload produced each written or failed variant.
`ShaderVariantPrewarmRequest.source_label` is filled from the asset scan stable
label or `builtin://shader/pbr.wgsl`, and
`ShaderVariantPrewarmReport.source_provenance` groups request outcomes by source
label, WGSL source hash, include hashes, template revision, Naga version, and
WGPU version. `tools/zircon_build_shader_prewarm.py` prints a compact
`source provenance:` line from that JSON, so dry-run/runtime logs can be tied
back to the staged report without dumping full WGSL. Status:
`render_plan08_shader_prewarm_source_provenance_summary_python_passed_cargo_deferred`;
`test_dimension_summary_lines_format_source_provenance` and
`runtime_15_shader_prewarm_source_provenance_summary_is_wired` lock the helper
format and structure anchors. Cargo/WGPU runtime execution, RenderDoc capture,
full registry export, and product miss=0 remain later Plan 08 gates.

Build-tool source provenance report contract turns that report field into a
success condition for staged builds. After a successful prewarm process,
`prewarm_shaders(...)` now calls `validate_shader_prewarm_report_contract(...)`
with `require_source_provenance=True` regardless of whether WGPU module
validation was requested. The helper requires a non-empty
`source_provenance.sources` map, matching `source_count`, a `variant_count` that
covers the report `requested_count`, and per-source `source_label`,
`source_hash`, `template_revision`, and closed requested/written/failed counts.
Status:
`render_plan08_build_tool_source_provenance_report_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_requires_source_provenance_when_requested`,
`test_validate_report_contract_accepts_source_provenance_counts`, and
`runtime_15_shader_prewarm_source_provenance_report_contract_is_wired` lock this
contract. It still does not count as real staged WGPU execution or product
miss=0 acceptance.

Build-tool source provenance totals match contract closes the next report
consistency gap. The same report helper now sums
`source_provenance.sources[*].requested/written/failed` and requires those
values to match report-level `requested_count`, `written_count`, and
`failed_count`, so a report cannot claim top-level success while a source entry
records a failed variant. Status:
`render_plan08_build_tool_source_provenance_totals_match_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_source_provenance_count_mismatch` and
`runtime_15_shader_prewarm_source_provenance_report_contract_is_wired` lock this
behavior. Closeout verification passed the build-helper Python combo 66/66.

Build-tool source provenance report tests now have a dedicated owner:
`tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py`.
The Runtime 15 guard reads that file directly and asserts the source-provenance
mismatch regression does not move back into the general
`test_zircon_build_shader_prewarm.py` owner. Status:
`render_plan08_build_tool_source_provenance_report_tests_owner_split_python_passed_cargo_deferred`;
the split leaves the general owner at 694 lines and the dedicated owner at 101
lines.

Build-tool shader resource registry export contract makes the automatic
`--export-resource-registry` artifact a parseable staged prewarm product. The
helper accepts a raw `ResourceRecord` array or `{ resources: [...] }` /
`{ records: [...] }` wrapper, allows an empty array, and rejects missing,
non-JSON, non-array, or non-object record shapes. Explicit
`--shader-resource-registry` inputs are consumed rather than produced by this
build step, so they are not counted as auto-export evidence; the staged
acceptance bundle validates those explicit inputs against the report later.
Status:
`render_plan08_build_tool_resource_registry_export_contract_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_requires_resource_records`,
`runtime_15_shader_prewarm_resource_registry_export_contract_is_wired` lock this
gate. It still does not count as a real staged WGPU run, RenderDoc/product
capture, full live registry export, or miss=0 acceptance. A focused Cargo guard
attempt was blocked before compile because `Cargo.lock` would need update under
`--locked`; no Rust diagnostics were produced and that run is not counted as
passed.

Build-tool shader resource registry report correlation now ties that exported
registry to the successful prewarm report. When the build helper validates an
auto-exported registry, it also receives `report_path` and requires every
`res://` shader source reported in `source_provenance.sources` to appear in a
`ResourceRecord.primary_locator` or `ResourceRecord.artifact_locator`. Builtin
sources such as `builtin://shader/pbr.wgsl` and raw path-like sources remain
outside the registry requirement because they are not produced by staged
`.zmeta` registry export. Status:
`render_plan08_build_tool_resource_registry_report_correlation_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_missing_report_source_locator`,
`test_validate_registry_export_contract_accepts_report_source_locator`,
`test_validate_registry_export_contract_ignores_builtin_report_sources`, and
`runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired` lock
this gate. It still does not count as real staged WGPU execution,
RenderDoc/product capture, full live registry export, or miss=0 acceptance.

The same registry correlation now also reads actual cache write identity rows.
`_report_resource_source_labels(...)` collects `res://` labels from both
`source_provenance.sources[*].source_label` and
`written_variants[].source_label`, then checks those labels against exported
resource record locators. Status:
`render_plan08_build_tool_resource_registry_written_source_label_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_missing_written_variant_locator`
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock this extension. It remains build-helper evidence only; live WGPU,
RenderDoc/product capture, full live registry export, and miss=0 acceptance are
separate Plan 08 gates.

Registry/report correlation also now requires the matching record to be usable
by the shader revision overlay. `_usable_shader_resource_record_locators(...)`
only accepts records whose `kind` is Shader and whose `revision` is a positive
integer before a `res://` report source is considered covered. Status:
`render_plan08_build_tool_resource_registry_usable_shader_revision_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_non_shader_report_source_record`,
`test_validate_registry_export_contract_rejects_zero_revision_report_source_record`,
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock this extension. This remains build-helper/static evidence, not live WGPU,
RenderDoc/product capture, full live registry export, or miss=0 acceptance.

Registry export/report correlation tests now live in
`tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py`
instead of the general build-helper test owner. Status:
`render_plan08_build_tool_resource_registry_contract_tests_owner_split_python_passed_cargo_deferred`;
the split leaves `tools/tests/test_zircon_build_shader_prewarm.py` at 540 lines
and the registry owner at 234 lines. Future registry/report cases should go to
the dedicated owner.

Registry export validation now lives in
`tools/zircon_build_shader_resource_registry.py`, while
`tools/zircon_build_shader_prewarm.py` only exposes the original
`validate_shader_resource_registry_export_contract(...)` entry point. The
helper validates ResourceRecord wire shape before report/source correlation:
`id` and `dependency_ids` must be UUID strings, `kind` and `state` must be known
resource enums, `primary_locator` and nullable-but-present `artifact_locator`
must use locator strings, `diagnostics` must contain severity/message records,
and `source_hash/importer_id/importer_version/config_hash` must be present.
Status:
`render_plan08_build_tool_resource_registry_record_shape_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_incomplete_resource_record`
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock this wire-shape contract. It remains build-helper evidence only; live WGPU
execution, RenderDoc/product capture, full live registry export, and miss=0
acceptance remain separate Plan 08 gates.

The same ResourceRecord gate now matches Rust unit-enum JSON shape for
`ResourceKind` and `ResourceState`: `kind` and `state` must be strings such as
`"Shader"` and `"Ready"`, not tagged objects. Status:
`render_plan08_build_tool_resource_registry_enum_wire_shape_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_tagged_enum_resource_record`
locks the negative case.

Numeric ResourceRecord fields are also clamped to Rust serde widths. `revision`
must fit `u64`, and `importer_version` must fit `u32`. Status:
`render_plan08_build_tool_resource_registry_numeric_width_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_u64_revision_overflow` and
`test_validate_registry_export_contract_rejects_u32_importer_version_overflow`
lock the overflow cases.

ResourceRecord locators now follow Rust `ResourceLocator::parse(...)` instead
of the old loose `://` check. Accepted schemes are `res`, `lib`, `package`,
`builtin`, and `mem`; paths must stay relative, non-empty after normalization,
without Windows drive prefixes or root escapes, and labels cannot be empty.
`package://` additionally requires a single package id plus a package-local
path. Status:
`render_plan08_build_tool_resource_registry_locator_wire_shape_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_invalid_locator_wire_shape`,
`test_validate_registry_export_contract_accepts_locator_wire_shape_variants`,
and `test_validate_registry_export_contract_rejects_invalid_artifact_locator`
lock the primary and artifact locator cases.

Resource registry/report correlation now treats registry-backed locator schemes
consistently with `ResourceLocator::parse(...)`. `res://`, `lib://`,
`package://`, and `mem://` report source labels must match exported
`ResourceRecord.primary_locator` or `ResourceRecord.artifact_locator` entries,
while `builtin://` remains an internal shader source outside staged project
registry export. Status:
`render_plan08_build_tool_resource_registry_backed_locator_correlation_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_accepts_registry_backed_source_locators`,
`test_validate_registry_export_contract_rejects_missing_registry_backed_source_locator`,
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock this correlation extension. It remains build-helper evidence only; live
WGPU execution, RenderDoc/product capture, full live registry export, and miss=0
acceptance are separate Plan 08 gates.

Resource registry ready shader revision contract keeps build-tool report
correlation aligned with runtime `ResourceManager::ready_records_for_kind(...)`
and the Rust prewarm overlay. `_is_usable_shader_record(...)` now requires
`kind=Shader`, `state=Ready`, and positive `revision`; non-Ready shader records
with a nonzero revision no longer satisfy report-source coverage. Status:
`render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_non_ready_report_source_record`,
`shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only`,
`runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`, and
`runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired` lock the
helper, Rust overlay, docs/status anchors, and child test owner. This remains
Python/static evidence only; real staged WGPU execution, full live registry
export, RenderDoc/product capture, fallback-root hit, and miss=0 acceptance are
separate Plan 08 gates.

Build-tool shader prewarm report dimension contract now verifies that a
successful report covers the requested staged build dimensions, not only that a
report exists. `prewarm_shaders(...)` passes
`expected_pass_types`, `expected_quality_tiers=config.shader_quality_tiers`, and
`expected_geometry_sources=config.shader_geometry_sources` into
`validate_shader_prewarm_report_contract(...)`; the helper requires
`dimension_summary.pass_types`, `dimension_summary.quality_tiers`, and
`dimension_summary.geometry_source_ids` to contain positive requested counts for
every requested pass, quality tier, and built-in geometry source. Status:
`render_plan08_build_tool_report_dimension_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_requires_requested_pass_types`,
`test_validate_report_contract_requires_requested_quality_tiers`,
`test_validate_report_contract_requires_requested_geometry_sources`,
`test_validate_report_contract_accepts_requested_dimensions`, and
`runtime_15_shader_prewarm_report_dimension_contract_is_wired` lock this gate.
It prevents `--shader-quality-tier high` or `--shader-geometry-source skinned`
from silently accepting a default `medium/static` report, but it still does not
count as real staged WGPU execution, RenderDoc/product capture, full live
registry export, or miss=0 acceptance.

Build-tool shader prewarm report dimension complete-count contract tightens that
same report gate so requested dimensions must also close their counts. The
helper now rejects expected pass, quality tier, built-in geometry source,
geometry-source id, or shading-model id summary entries where
`requested_count > 0` but `written_count != requested_count` or
`failed_count != 0`. Status:
`render_plan08_build_tool_report_dimension_complete_counts_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_incomplete_requested_dimension_counts`
locks the `forward requested=6 written=5 failed=1` regression. This still does
not count as real WGPU execution or product miss=0 evidence.

Build-tool shader prewarm report dimension totals match contract now requires
dimension-summary group totals to match the top-level report counts. The helper
uses `_validate_dimension_summary_totals_match_report(...)` and
`_dimension_group_totals(...)` to reject reports where entries are individually
complete but the group sums drift, such as `requested=6/7 written=6/7
failed=0/0`. Status:
`render_plan08_build_tool_report_dimension_totals_match_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_dimension_count_total_mismatch` locks the
negative case. This still does not count as real WGPU execution or product
miss=0 evidence.

Build-tool product Base pass acceptance contract ties the generic staged
prewarm validators to the product Base/Opaque path. That earlier slice first
routed `forward` into both the report dimension contract and cache artifact
contract, so successful build output had to report a requested Forward pass and
had to include a written cache identity with `pass=forward`. Status:
`render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_requires_requested_pass_types`,
`test_validate_cache_artifact_contract_requires_requested_pass_types`,
`test_validate_cache_artifact_contract_accepts_requested_pass_types`, and the
three structure guards lock this handoff. This still does not count as real
WGPU execution or product miss=0 evidence.

Build-tool product material mesh pass acceptance contract extends that handoff
from Base-only to the product material mesh pass tuple. The acceptance helper
now owns `_PRODUCT_MATERIAL_MESH_PASS_TYPES = ("forward", "gbuffer",
"depth_prepass", "shadow", "velocity", "taa_reactive_mask")` and passes it to
both validators, so a forward-only staged report/cache bundle is rejected before
build acceptance. Status:
`render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred`;
`test_acceptance_contract_rejects_forward_only_staged_pass_report`, the updated
acceptance handoff assertion for `expected_pass_types`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock this contract.
This still does not count as real WGPU execution, RenderDoc/product capture,
full live registry export, or product miss=0 evidence.

Build-tool cache quality/geometry identity contract extends that same cache
handoff beyond pass and custom ids. The acceptance helper passes
`config.shader_quality_tiers` and `config.shader_geometry_sources` into
`validate_shader_prewarm_cache_artifact_contract(...)`; the cache helper checks
the written cache canonical strings for `quality=<tier>` and built-in
`geometry=<id>` values. Status:
`render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_quality_tiers`,
`test_validate_cache_artifact_contract_requires_requested_geometry_sources`, and
`test_validate_cache_artifact_contract_accepts_requested_quality_and_geometry`
lock the gate. This still does not count as real WGPU execution or product
miss=0 evidence.

Build-tool cache dimension combination contract closes the gap left by
independent dimension coverage. The cache validator now calls
`_validate_expected_written_variant_combinations(...)` after pass, quality, and
geometry checks, parses each written canonical string with
`_canonical_dimension_values(...)`, and requires the requested `pass x quality x
built-in geometry` combinations to exist in a single written variant identity.
Status:
`render_plan08_build_tool_cache_dimension_combination_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_dimension_combinations`
locks the case where `forward/high/static` plus `shadow/medium/skinned` cannot
satisfy `forward/high/skinned`. This still does not count as real WGPU execution
or product miss=0 evidence.

Build-tool cache custom id combination contract applies the same product-key
rule to selected plugin/CLI ids. After individual custom geometry and shading id
checks pass, the cache validator calls
`_validate_expected_written_custom_id_combinations(...)` and requires the
requested custom geometry id and custom shading id to appear in the same written
canonical string; requested pass and quality dimensions are included in that same
match when present. Status:
`render_plan08_build_tool_cache_custom_id_combination_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_custom_id_combinations`
locks the case where `forward/high/geometry=0/shading=0` plus
`shadow/medium/geometry=4/shading=16` cannot satisfy
`forward/high/geometry=4/shading=16`. This still does not count as real WGPU
execution or product miss=0 evidence.

Build-tool cache source-label provenance correlation contract ties written cache
identity back to the source provenance table. When a successful report contains
`source_provenance.sources`, the cache artifact helper now requires every
`written_variants[].source_label` to be present and to match one
`source_provenance.sources[*].source_label`. Status:
`render_plan08_build_tool_cache_source_label_provenance_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_written_variant_source_labels_in_provenance`
locks the mismatch where cache metadata matches the report but the written
variant points at an unprovenanced `res://` shader source. This is still
Python/static build-helper evidence, not a live WGPU run or product miss=0
acceptance.

Build-tool written variant uniqueness contract keeps duplicated report identity
rows from satisfying staged success. The shared
`tools/zircon_build_shader_prewarm_written_variants.py` owner now parses
`written_variants`, validates BLAKE3 cache-hash shape, keeps source-label
provenance correlation out of the oversized cache artifact helper, and exposes
`validate_unique_written_variant_identity(...)` for both the cache artifact
contract and staged acceptance precheck. Status:
`render_plan08_build_tool_written_variant_uniqueness_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity`
and `test_acceptance_contract_rejects_duplicate_written_variant_identity` lock
that duplicate `cache_hash` or duplicate `canonical_string` rows cannot satisfy
`written_count`. This closes a static acceptance gap only; real staged WGPU,
fallback lookup, RenderDoc/product capture, full registry export, and miss=0
product acceptance remain later Plan 08 gates.

Build-tool cache metadata field type contract keeps malformed staged `.meta`
wire shapes from being treated as valid cache evidence. The cache artifact
helper now rejects bool/string schema or timestamp values and non-string
canonical/template/Naga/WGPU fields at metadata-parse time. Status:
`render_plan08_build_tool_cache_metadata_field_type_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_invalid_metadata_field_types`
locks that `schema_version=True`, string timestamps, and list/dict metadata
fields fail as invalid cache metadata instead of passing or surfacing as a late
variant mismatch. This is still Python/static build-helper evidence, not a live
WGPU run or product miss=0 acceptance.

Build-tool staged prewarm written cache-hash shape acceptance now rejects
malformed `written_variants[].cache_hash` values in the success precheck. The
acceptance helper reuses `validate_cache_hash_shape(...)`, so a staged success
report with `cache_hash="not-a-cache-key"` fails before the lower cache artifact
validator is patched in. Status:
`render_plan08_build_tool_staged_prewarm_written_cache_hash_shape_python_passed_cargo_deferred`;
`test_acceptance_contract_rejects_invalid_written_variant_cache_hash_shape`
locks the entry point. This is still Python/static build-helper evidence, not a
live WGPU run or product miss=0 acceptance.

Build-tool source-label nonblank contract rejects whitespace-only source
identity before staged success can rely on it. The report contract now requires
source provenance `source_label`, `source_hash`, and `template_revision` to be
nonblank after trimming, and the written-variant helper plus staged acceptance
precheck reject blank `written_variants[].source_label`. Status:
`render_plan08_build_tool_source_label_nonblank_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_blank_source_provenance_strings` and
`test_acceptance_contract_rejects_blank_written_variant_source_label` lock the
two entry points. Final build-helper aggregate validation passed 99/99, with
py_compile, rustfmt, anchor/conflict/trailing-whitespace and line-budget scans,
and scoped diff-check passing with only LF/CRLF warnings. This is still
Python/static build-helper evidence, not a live WGPU run or product miss=0
acceptance.

Build-tool source-label trim contract requires source evidence strings to be
trim-clean before they can satisfy staged success. The report contract rejects
untrimmed `source_label`, `source_hash`, and `template_revision`; the shared
written-variant helper and staged acceptance precheck reject untrimmed
`written_variants[].source_label`. Status:
`render_plan08_build_tool_source_label_trim_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_untrimmed_source_provenance_strings`,
`test_acceptance_contract_rejects_untrimmed_written_variant_source_label`, and
`test_validate_cache_artifact_contract_rejects_untrimmed_written_variant_source_label`
lock the three entry points. The source, acceptance, and cache helper suite
passed 46/46, and the build-helper aggregate passed 102/102. This is still
Python/static build-helper evidence, not a live WGPU run or product miss=0
acceptance.

Build-tool explicit registry exact revision acceptance validates caller-provided
`--shader-resource-registry` inputs in the staged acceptance bundle. The helper
now validates `config.shader_resource_registry` when present, otherwise the
automatic `shader_prewarm_resource_registry_path`, and always passes the
successful report path to the registry validator. That means explicit live or
project registry records must still provide `usable shader ResourceRecord revisions`
for report-visible shader source labels. Status:
`render_plan08_build_tool_explicit_registry_exact_revision_acceptance_python_passed_cargo_deferred`;
`test_acceptance_contract_validates_explicit_registry_against_report` and
`test_acceptance_contract_rejects_explicit_registry_without_ready_revision` lock
the handoff. This narrows the full live project/plugin registry export gate but
does not close the actual production export, RenderDoc/product capture, or
product miss=0 acceptance.

Build-tool staged prewarm written source-label identity acceptance moves the
same requirement earlier in the success path. The staged acceptance helper now
requires each `written_variants[]` entry to include `source_label` alongside
`cache_hash`, `canonical_string`, template revision, and Naga/WGPU versions
before report/source/cache/registry validators run. Status:
`render_plan08_build_tool_staged_prewarm_written_source_label_identity_python_passed_cargo_deferred`;
`test_acceptance_contract_requires_written_variant_source_label_identity` locks
the precheck. This is still build-helper evidence, not a live WGPU run or
product miss=0 acceptance.

Build-tool shader prewarm cache artifact contract now verifies that a successful
report is backed by staged cache files. After the report contract passes,
`prewarm_shaders(...)` calls
`validate_shader_prewarm_cache_artifact_contract(...)` with
`config.shader_prewarm_cache_root` and the report path. The helper lives in
`tools/zircon_build_shader_prewarm_cache_artifacts.py`, reads report
`written_count`, and scans the cache root for `.wgsl.zst` files with matching
`.meta` siblings whose JSON metadata includes a matching `hash`,
`schema_version`, `canonical_string`, template revision, and Naga/WGPU version
fields. Status:
`render_plan08_build_tool_cache_artifact_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_written_cache_pairs`,
`test_validate_cache_artifact_contract_rejects_orphan_wgsl_artifacts`,
`test_validate_cache_artifact_contract_rejects_invalid_metadata`,
`test_validate_cache_artifact_contract_rejects_metadata_hash_mismatch`,
`test_validate_cache_artifact_contract_accepts_written_cache_pairs`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this gate.
It prevents a zero-exit prewarm command from claiming written variants while
leaving no staged cache artifacts, but it still does not count as real WGPU
execution, runtime key lookup, RenderDoc/product capture, or miss=0 acceptance.

Prewarm report cache identity contract extends the same staged cache gate from
pair presence to exact identity. New reports include
`ShaderVariantPrewarmReport.written_variants`, where each
`ShaderVariantPrewarmWrittenVariant` records the `cache_hash`,
`canonical_string`, source label, template revision, and Naga/WGPU versions
captured from the `ShaderVariantCacheDiskKey` that was successfully written.
When `written_variants` is present, the cache artifact helper requires its
length to match `written_count` and validates every reported hash and canonical
metadata against the staged `.meta` file. Status:
`render_plan08_prewarm_report_cache_identity_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_report_written_variants`,
`test_validate_cache_artifact_contract_rejects_partial_written_variant_report`,
`test_validate_cache_artifact_contract_rejects_wrong_canonical_variant`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this gate.
Legacy reports without `written_variants` still use the previous count/pair
contract, and real WGPU execution, runtime key lookup, RenderDoc/product
capture, full live registry export, and miss=0 acceptance remain deferred.

Prewarm cache runtime layout contract extends the same helper toward the actual
runtime lookup path. The cache artifact helper now mirrors
`ShaderVariantCacheDisk` layout by requiring each staged `.wgsl.zst` file to sit
under `<cache_root>/v1/<hash[0..2]>/<hash>.wgsl.zst` and requiring `.meta`
`schema_version == 1`. Status:
`render_plan08_prewarm_cache_runtime_layout_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_non_runtime_cache_layout`,
`test_validate_cache_artifact_contract_rejects_schema_version_mismatch`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this gate.
This prevents a successful build from accepting cache files that the runtime
fallback root cannot find, but it is still not a live WGPU run, RenderDoc/product
capture, full registry export, or miss=0 acceptance.

Prewarm cache hash shape contract closes the next build acceptance gap. The same
helper now rejects staged artifact names and report `written_variants.cache_hash`
values that are not 64-character lowercase BLAKE3 hex strings, matching
`ShaderVariantCacheDiskKey::from_variant_key(...)` hash output. Status:
`render_plan08_prewarm_cache_hash_shape_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_non_blake3_hex_cache_hash` and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this gate.
This is still static/build-helper evidence, not a live WGPU run or product
miss=0 acceptance.

Prewarm cache custom id correlation contract closes the next staged-cache
identity gap. After successful prewarm, `prewarm_shaders(...)` passes
`shader_geometry_source_id_specs(config)` and
`shader_shading_model_id_specs(config)` into the cache artifact helper. The
helper parses the selected plugin / explicit CLI custom ids and requires
`written_variants[].canonical_string` to include matching `geometry=<id>` and
`shading=<id>` entries before accepting staged cache artifacts. Status:
`render_plan08_prewarm_cache_custom_id_correlation_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_custom_ids`,
`test_validate_cache_artifact_contract_requires_requested_shading_ids`,
`test_validate_cache_artifact_contract_accepts_requested_custom_ids`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this gate.
This is still Python/static build evidence, not a live WGPU run, runtime
fallback-root hit, RenderDoc/product capture, or product miss=0 acceptance.

Runtime prewarm custom id cache lookup contract is the Rust-side pair to the
build-helper custom-id artifact check. The existing prewarm/cache owner now
constructs a request with custom `GeometrySourceId(4)` and `ShadingModelId(16)`,
writes it with `prewarm_shader_variants_to_disk`, and verifies the same
`ShaderVariantCacheDiskKey` can be looked up from disk with canonical
`geometry=4` and `shading=16`. Status:
`render_plan08_runtime_prewarm_custom_id_cache_lookup_static_passed_cargo_deferred`;
`render_shader_variant_prewarm_custom_ids_survive_disk_lookup` and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this owner.
This remains static Rust coverage; the real staged `zircon_shader_prewarm`
process, WGPU validation, RenderDoc/product capture, full registry export, and
product miss=0 acceptance are still later Plan 08 gates.

Runtime custom id staged fallback lookup contract proves that the same custom-id
cache entry is addressable through the runtime fallback chain that product
launches use for staged payloads. The prewarm/cache owner writes the custom
`GeometrySourceId(4)` / `ShadingModelId(16)` request into a staged
`cache/shader_variants` root, confirms the empty runtime cache root misses, then
confirms `ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root])`
hits without creating or writing the runtime root. Status:
`render_plan08_runtime_custom_id_staged_fallback_lookup_static_passed_cargo_deferred`;
`render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root` and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this owner.
This remains static Rust coverage; live `zircon_shader_prewarm`, WGPU validation,
RenderDoc/product capture, full registry export, and product miss=0 acceptance
are still later Plan 08 gates.

Build-tool staged prewarm acceptance contract now owns the zero-exit build
acceptance bundle. `tools/zircon_build_shader_prewarm_acceptance.py::validate_staged_shader_prewarm_acceptance_contract`
collects selected plugin and CLI custom ids once, then runs the report contract,
cache artifact contract, and automatic resource-registry/report correlation from
a single helper. `prewarm_shaders(...)` calls this helper after the subprocess
returns success instead of directly calling each low-level validator. Status:
`render_plan08_build_tool_staged_prewarm_acceptance_contract_python_passed_cargo_deferred`;
`test_acceptance_contract_validates_report_cache_and_exported_registry` and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock this behavior.
This is still build-helper acceptance evidence only; live
`zircon_shader_prewarm --validate-wgpu-modules`, RenderDoc/product capture, full
live registry export, and second-launch miss=0 remain separate Plan 08 gates.

Build-tool staged prewarm runtime fallback layout contract is the path gate
inside the same acceptance helper. Before reading report JSON,
`validate_staged_shader_prewarm_runtime_fallback_layout(...)` requires the cache
root to be `ZirconEngine/cache/shader_variants`, the report to be
`ZirconEngine/cache/shader_variants_report.json`, and the automatic resource
registry export to be `ZirconEngine/cache/shader_resource_records.json` under
the staged engine root. Status:
`render_plan08_build_tool_staged_prewarm_runtime_fallback_layout_python_passed_cargo_deferred`;
`test_acceptance_contract_rejects_runtime_fallback_layout_drift`,
`test_acceptance_contract_accepts_runtime_fallback_layout`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock the runtime fallback root behavior. This remains build-helper evidence; the real WGPU run,
RenderDoc/product capture, full live registry export, and product miss=0 gates
are still separate Plan 08 checks.

Build-tool staged prewarm nonempty success report acceptance is the result gate
inside the same helper. After path layout passes and before lower report/cache
validators run, `validate_staged_shader_prewarm_nonempty_success_report(...)`
requires `requested_count > 0`, `written_count > 0`, and `failed_count == 0` in
`shader_variants_report.json`. This prevents a zero-exit prewarm command with no
written variants, or with any failed variant, from being accepted as a build
success. Status:
`render_plan08_build_tool_staged_prewarm_nonempty_success_report_python_passed_cargo_deferred`;
`test_acceptance_contract_rejects_empty_success_report`,
`test_acceptance_contract_rejects_failed_success_report`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock this behavior.
Closeout verification passed focused acceptance 7/7 and the build-helper Python
combo 62/62.

Build-tool staged prewarm written variant identity acceptance is the cache-key
handoff gate inside the same helper. Successful reports must include
`written_variants` with a row for every `written_count` entry, and each row must
name `cache_hash`, `canonical_string`, `template_revision`, `naga_version`, and
`wgpu_version` before the lower cache artifact helper validates staged `.meta`
files. Status:
`render_plan08_build_tool_staged_prewarm_written_variant_identity_python_passed_cargo_deferred`;
`test_acceptance_contract_requires_written_variant_identity`,
`test_acceptance_contract_rejects_incomplete_written_variant_identity`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock this behavior.
Closeout verification passed focused acceptance 9/9 and the build-helper Python
combo 64/64.

Build-tool staged prewarm complete written count acceptance is the final count
relation gate inside the same helper. Successful reports must have
`written_count == requested_count` before source-provenance, dimension, cache,
and registry validators run; a zero-exit report with `requested=2`,
`written=1`, `failed=0`, and one `written_variants` row is rejected as partial
success. Status:
`render_plan08_build_tool_staged_prewarm_complete_written_count_python_passed_cargo_deferred`;
`test_acceptance_contract_rejects_partial_written_success_report` and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock this behavior.
Closeout verification passed the build-helper Python combo 65/65.

The same acceptance owner first carried the product Base pass handoff, then was
extended to full product material mesh pass acceptance. A successful staged
prewarm report/cache bundle must now satisfy `expected_pass_types=("forward",
"gbuffer", "depth_prepass", "shadow", "velocity", "taa_reactive_mask")`, so the
product material consumer is not handed a cache populated only with Forward.
`test_acceptance_contract_rejects_forward_only_staged_pass_report` locks the
negative case under status
`render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred`.

The cache identity handoff now also requires written variants for requested
quality tiers and built-in geometry sources. A staged report may no longer
claim `high` or `skinned` in `dimension_summary` while only writing
`quality=medium` or `geometry=0` cache keys. Focused cache/acceptance tests
passed 29/29 under status
`render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred`.

Build-tool staged WGPU handoff command contract keeps the real prewarm command
from drifting before the next controlled WGPU run. `build_shader_prewarm_command`
now validates that opt-in WGPU validation, generated permutation registry,
engine/plugin asset roots, resource-registry export, cache/report paths, and
quality/geometry/custom-id dimensions are all present and in config order.
The contract is locked by `test_full_staged_wgpu_handoff_keeps_generated_registries_and_roots`
and `runtime_15_shader_prewarm_staged_wgpu_handoff_command_contract_is_wired`.
Status:
`render_plan08_build_tool_staged_wgpu_handoff_command_contract_python_passed_cargo_deferred`.
It is command-shape evidence only; real `zircon_shader_prewarm --validate-wgpu-modules`,
RenderDoc/product capture, full live registry export, and second-launch miss=0
remain separate acceptance gates.

`--shader-resource-registry <path>`
forwards a serialized `ResourceRecord` array, or a JSON object with a
`resources`/`records` array, to `zircon_shader_prewarm --resource-registry`.
Asset-root resource registry revision overlay is owned by
`bin/zircon_shader_prewarm/manifest/resource_registry.rs`: matching shader
records override `.zmeta.source_digest`-derived revisions with the live
`ResourceRecord.revision`, while unmatched raw sources keep content-hash
revisions. The focused wiring guard is
`runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired`, the
manifest regression is
`shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay`,
and current status is
`render_plan08_asset_root_resource_registry_revision_overlay_typecheck_passed_test_timeout_no_result`.
Built-in fallback and
`builtin://shader/pbr.wgsl` material references use the standard-material
template builder for each requested geometry source; custom scanned shader
payloads remain raw WGSL requests. Runtime lookup checks the writable
`.zircon/cache/shader_variants` cache first and then the staged
`cache/shader_variants` payload, so packaged prewarm entries can satisfy the
first matching shader-module lookup.

Staged shader resource registry auto-export is the default when
`--prewarm-shaders` runs without an explicit `--shader-resource-registry`.
The build helper forwards `--export-resource-registry` to write
`ZirconEngine/cache/shader_resource_records.json`, then the prewarm binary
immediately consumes those generated records as an overlay for the same
asset-root manifest scan. `shader_resource_records_from_asset_root(...)` reads
staged `.zmeta` documents, exports shader-only ready `ResourceRecord` rows, and
keeps explicit `--resource-registry` input as the override path. The focused
manifest regression is
`shader_prewarm_asset_root_exports_shader_resource_records`, the structure
guard is `runtime_15_shader_prewarm_registry_auto_export_is_wired`, and status
is
`render_plan08_shader_resource_registry_auto_export_focused_tests_passed_renderdoc_deferred`.

Staged shader resource registry multi-root dedupe keeps that automatic export
stable when selected plugin asset roots overlap the engine staged assets.
`shader_resource_records_from_asset_roots(...)` now owns aggregation for all
asset roots before the build writes `shader_resource_records.json`; duplicate
shader records collapse once, while conflicting id/locator mappings return an
error from the prewarm binary. The focused regression is
`shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records`,
the structure guard is
`runtime_15_shader_prewarm_resource_registry_multi_root_dedupe_is_wired`, and
current status is
`render_plan08_shader_resource_registry_multi_root_dedupe_static_passed_cargo_deferred`.

The editor target also stages a sibling `zircon_runtime.dll`/`so`/`dylib`, because
`zircon_editor` resolves the runtime library from `ZIRCON_RUNTIME_LIBRARY` or the
current executable directory. Keeping the library beside the executable fixes the
common local Cargo layout issue where the dynamic library remains under
`debug/deps` and `LoadLibraryExW` cannot find it.

Built-in engine asset lookup follows the staged layout. Runtime/editor code first
honors `ZIRCON_ASSET_ROOT`, then checks `assets` beside the current executable,
then `assets` under the current working directory, and finally falls back to the
crate-local `assets` directory for `cargo run` and unit-test workflows. Editor
call sites pass `zircon_editor/assets` as their development fallback so editor
templates still resolve from source when no staged payload exists.

The lookup returns the first candidate root that contains the requested relative
file. It only falls back to the first existing root when no candidate contains the
file, which keeps a staged payload from masking editor-only development assets and
keeps missing built-ins visible in diagnostics.

## Startup Logs

Exported editor/runtime binaries initialize a lightweight startup diagnostic sink
before their main host work. Each line is mirrored to stderr and, when possible,
to a per-run file named `logs/<yyyy-MM-dd-hh-mm-ss>/<channel>.log`.

Editor logs prefer the staged executable directory. For example, running
`E:\zircon-build\ZirconEngine\zircon_editor.exe` from the staged payload writes a
file such as:

```text
E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log
```

Standalone runtime logs still honor `ZIRCON_LOG_ROOT` first, then prefer a
Unity-compatible user log directory before local fallbacks:

```text
Windows: %USERPROFILE%\AppData\LocalLow\ZirconEngine\ZirconEngine\logs\<timestamp>\runtime.log
macOS:   $HOME/Library/Logs/ZirconEngine/ZirconEngine/logs/<timestamp>/runtime.log
Linux:   $HOME/.config/unity3d/ZirconEngine/ZirconEngine/logs/<timestamp>/runtime.log
```

Set `ZIRCON_LOG_ROOT` to force both editor and runtime logs under a known root.
The asset resolver, editor template loader, native host window, and native
presenter currently write startup diagnostics there, so exported-display issues
can be classified without guessing whether the failure is asset staging,
presentation data, window creation, or rendering.

## CLI And Interactive Use

The three required decisions are build targets, output directory, and mode:

```powershell
    python tools/zircon_build.py --targets hub,editor,runtime --out E:\builds\zircon --mode debug
```

`--targets` accepts `hub`, `editor`, `runtime`, `plugins`, or comma-separated
combinations. `--mode` is `debug`, `release`, or `profiling`. If any required
value is missing and stdin is interactive, the tool prompts for the missing
selection; if stdin is not interactive, it exits with a clear error.

`--mode profiling` maps to Cargo's root `[profile.profiling]` profile and emits
`--profile profiling` rather than `--release`. Use `--runtime-features` to name
the runtime/app feature set that should be measured:

```powershell
python tools/zircon_build.py --targets runtime --out E:\builds\zircon-profile --mode profiling --runtime-features target-client,profiling,profiling-tracy
```

The profiling mode is intentionally limited to normal Cargo targets. It rejects
the Hub/Tauri target and the separate plugin workspace target because those
pipelines do not use the same root runtime profiling profile. Runtime 07 M0.2
also has the equivalent quick-check path through the PowerShell helper:

```powershell
./tools/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride "target-client profiling profiling-tracy"
```

Plugin builds add a fourth selection:

```powershell
python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\builds\zircon --mode debug
```

`--plugins` accepts plugin ids, menu numbers, ranges, `all`, `native`, or `rlib`.
`--plugin-carrier native_dynamic` or `--plugin-carrier rlib_static` filters the
plugin catalog before selection. `--list-plugins` prints the discovered catalog
and carrier classification without building.

Plugin builds require a real staged host. If the same invocation does not build
`editor`, an existing `zircon_editor` executable must already be present under
`<out>/ZirconEngine`; if the invocation builds neither `editor` nor `runtime`, the
runtime library must already be present too. This keeps incremental plugin
compilation tied to a real engine payload instead of producing detached plugin
artifacts with no host to load or link them.

## NativeDynamic Boundary

`native_dynamic` plugins are Rust `cdylib` crates selected through a package
`plugin.toml` and copied into the runtime payload:

```text
ZirconEngine/plugins/<plugin-id>/plugin.toml
ZirconEngine/plugins/<plugin-id>/native/<crate>.dll|so|dylib
ZirconEngine/plugins/native_plugins.toml
```

These libraries are valid dynamic plugin artifacts because they export the native
plugin ABI symbols consumed by `NativePluginLoader`. The loader expects the
package manifest at `plugins/<plugin-id>/plugin.toml` and the dynamic library
under `plugins/<plugin-id>/native/` using the crate name declared by the runtime
or editor module.

The current native ABI is intentionally byte-oriented and manifest-oriented. It
can report package manifests, entry diagnostics, capability negotiation, command
metadata, serialized command callbacks, state callbacks, unload callbacks, and
plugin-owned buffers. It does not pass Rust trait objects, editor state, ECS
objects, `wgpu` objects, or borrowed runtime references across the dynamic
boundary.

## rlib Static Boundary

Most `zircon_plugins/*/{runtime,editor}` crates intentionally build as rlib
crates. They are Rust static-link plugin packages, not dynamic plugin payloads.
Their real behavior enters the engine through `LibraryEmbed` or `SourceTemplate`
builds that call crate functions such as `plugin_registration()` and merge the
resulting registration reports into runtime/editor registries.

The build tool can compile these crates ahead of time into:

```text
<out>/targets/plugins/<plugin-id>/
```

That proves the selected rlib crates and their dependencies are valid static-link
inputs. The tool does not copy rlib outputs into `ZirconEngine/plugins`, does not
generate fake dynamic libraries for them, and does not claim they are loadable by
`NativePluginLoader`.

Turning an rlib plugin into an independently loadable plugin requires a real ABI
adapter milestone first. That adapter must convert the plugin's runtime/editor
registration data into stable DTOs or C ABI records and must not move Rust-only
types, references, or host-owned objects across a dynamic library boundary.

## Font SDF Artifact Target

`--targets font-sdf` is a build-only target for versioned Runtime text distance-field artifacts. It requires `--font-sdf-manifest`; it does not stage an executable payload and it does not infer a font asset UUID from a path. The manifest is the authoritative association between a project font asset, its `.zmeta` UUID, a cache root, bake parameters, and a glyph selection.

```json
{
  "format_version": 1,
  "bakes": [
    {
      "font": "zircon_runtime/assets/fonts/FiraSans-Regular.ttf",
      "cache_root": "E:/builds/zircon-font-cache",
      "asset_guid": "12345678-90ab-4cde-8f01-234567890abc",
      "face_index": 0,
      "mode": "msdf",
      "codepoints": ["U+0020-U+007E", "U+4E2D"],
      "page_size": 1024,
      "bake_em_px": 48,
      "spread_px_milli": 8000
    }
  ]
}
```

Each bake selects exactly one of `all_cmap: true` or a non-empty `codepoints` list. A codepoint entry is either one Unicode scalar (`U+0041`) or an inclusive range (`U+0041-U+005A`). The Python owner expands ranges in scalar order and deduplicates them. The feature-gated `zircon_font_sdf_bake` Rust binary then decodes the selected font face, deduplicates cmap aliases by glyph id, invokes the shared Runtime fdsm generator, packs the existing R8/RGBA storage formats, and atomically writes one embedded-page `.zsdf` file under `cache_root`.

```powershell
python tools/zircon_build.py --targets font-sdf --font-sdf-manifest E:\manifests\font-sdf.json --out E:\builds\zircon-text
```

The Cargo target stays beneath the build tool's managed targets root (`<out>/targets/font-sdf`). Generated `.zsdf` files belong beneath the manifest's project/library cache root, never beneath repository `target/` or the visual framebuffer evidence directory. The binary format and runtime fallback contract are documented in `docs/zircon_runtime/graphics/text/offline-sdf.md`.

## Validation Scope

Use these fast checks for script changes:

```powershell
python -m py_compile tools/zircon_build.py
python tools/zircon_build.py --help
python tools/zircon_build.py --list-plugins
python tools/zircon_build.py --targets hub,editor,runtime --out E:\builds\zircon-smoke --mode debug --dry-run
python tools/zircon_build.py --targets editor,runtime --out E:\builds\zircon-smoke --mode debug --dry-run
python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\builds\zircon-smoke --mode debug --dry-run
python tools/zircon_build.py --targets runtime --out E:\builds\zircon-smoke --mode profiling --runtime-features target-client,profiling,profiling-tracy --dry-run
python tools/zircon_build.py --targets runtime --out E:\builds\zircon-smoke --mode debug --prewarm-shaders --dry-run
python tools/zircon_build.py --targets runtime --out E:\builds\zircon-smoke --mode debug --prewarm-shaders --validate-wgpu-shaders --dry-run
python tools/zircon_build.py --targets runtime --plugins virtual_geometry --out E:\builds\zircon-smoke --mode debug --prewarm-shaders --validate-wgpu-shaders --dry-run
cargo check -q -p zircon_runtime --bin zircon_font_sdf_bake --no-default-features --features font-sdf-build-tool --target-dir E:\cargo-targets\zircon-font-sdf-bin
python -m unittest tools.tests.test_zircon_build_font_sdf -v
cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir E:\cargo-targets\zircon-shader-prewarm-bin
```

Use a real build when validating executable staging or NativeDynamic publishing:

```powershell
python tools/zircon_build.py --targets hub,editor,runtime --out E:\builds\zircon-smoke --mode debug
python tools/zircon_build.py --targets runtime --plugins virtual_geometry --out E:\builds\zircon-smoke --mode debug --prewarm-shaders --validate-wgpu-shaders
python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\builds\zircon-smoke --mode debug
```

The first command should leave `zircon_hub`, `zircon_editor`, and the platform runtime library
as siblings under `ZirconEngine`. The second command should leave a
`plugins/native_plugins.toml` file and a copied native dynamic library under the
selected plugin package.

2026-07-03 Plugins 13 build-tool owner note:
`plugins_13_m5_t1_zircon_build_hub_owner_split` moved Hub/Tauri build orchestration and installers staging into `tools/zircon_build_hub.py`. That owner now holds `build_hub`, `run_tauri_build`, `stage_hub_tauri_outputs`, and `stage_hub_tauri_installers`; `tools/zircon_build.py` only imports `build_hub` and is down to 872 lines. The contract is covered by `test_hub_tauri_build_lives_in_hub_owner` and `test_hub_owner_preserves_staging_semantics`. This is script-structure evidence only and does not claim a real Hub/editor E2E, full export matrix, startup-to-first-frame, Cargo build/test/export, real plugin build, or full plugin compile/export regression.

2026-07-01 support note: while validating the retained-host text swash/subpixel
slice, the full editor build first failed because
`dynamic_api/shader_prewarm.rs` referenced a prewarm source-hash helper that had
not been landed with the plugin shading-model manifest path. The helper is now
local to the same owner and uses `blake3::hash(source.as_bytes())`, matching the
existing shader content-hash convention. The follow-up runtime lib check and
editor-host build passed in `D:\cargo-targets\zircon-editor-text-tabs-0701`
with existing warnings; repo `target` was not used for these outputs.

2026-07-01 Plan 08 structure mirror: `Plugin shader permutation registry auto-export`, `render_plan08_plugin_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`, `Plugin shader permutation registry export contract`, `render_plan08_plugin_shader_permutation_registry_export_contract_python_passed_cargo_deferred`, `test_zircon_build_discovers_plugin_shader_permutation_records`, `test_validate_generated_registry_requires_selected_plugin_ids`, `runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired`, `Build-tool staged WGPU handoff command contract`, `render_plan08_build_tool_staged_wgpu_handoff_command_contract_python_passed_cargo_deferred`, `test_full_staged_wgpu_handoff_keeps_generated_registries_and_roots`, and `runtime_15_shader_prewarm_staged_wgpu_handoff_command_contract_is_wired`.

2026-07-02 Plan 08 shader prewarm build-tool mirror: `Shader prewarm geometry-source enumeration`, `render_plan08_shader_prewarm_geometry_source_enumeration_static_passed_cargo_deferred_implementation_cadence`, `shader_prewarm_asset_root_manifest_expands_requested_geometry_sources`, `runtime_15_shader_prewarm_geometry_source_enumeration_is_wired`, `Asset-root custom geometry-source id prewarm`, `shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids`, `Asset-root custom shading-model id prewarm`, `render_plan08_asset_root_custom_shading_model_id_prewarm_static_passed_cargo_deferred_implementation_cadence`, `shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids`, and `runtime_15_shader_prewarm_custom_shading_model_id_is_wired`. This note documents static/build-tool ownership only; executable WGPU/product acceptance keeps its separate evidence gates.

2026-07-02 shader prewarm full structure mirror: `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`, `bin/zircon_shader_prewarm/args.rs`, `bin/zircon_shader_prewarm/manifest.rs`, `tools/zircon_build.py`, `tools/zircon_build_shader_prewarm.py`, `runtime_15_shader_prewarm_custom_geometry_source_id_is_wired`, `Build-tool staged prewarm acceptance contract`, `render_plan08_build_tool_staged_prewarm_acceptance_contract_python_passed_cargo_deferred`, `Build-tool staged prewarm nonempty success report acceptance`, `render_plan08_build_tool_staged_prewarm_nonempty_success_report_python_passed_cargo_deferred`, `Build-tool staged prewarm written variant identity acceptance`, `render_plan08_build_tool_staged_prewarm_written_variant_identity_python_passed_cargo_deferred`, `Build-tool staged prewarm written source-label identity acceptance`, `render_plan08_build_tool_staged_prewarm_written_source_label_identity_python_passed_cargo_deferred`, `Build-tool staged prewarm complete written count acceptance`, `render_plan08_build_tool_staged_prewarm_complete_written_count_python_passed_cargo_deferred`, `Build-tool product Base pass acceptance contract`, `render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred`, `Build-tool product material mesh pass acceptance contract`, `render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred`, `Build-tool written variant uniqueness contract`, `render_plan08_build_tool_written_variant_uniqueness_contract_python_passed_cargo_deferred`, `Build-tool staged prewarm written cache-hash shape acceptance`, `render_plan08_build_tool_staged_prewarm_written_cache_hash_shape_python_passed_cargo_deferred`, `Build-tool source-label nonblank contract`, `render_plan08_build_tool_source_label_nonblank_contract_python_passed_cargo_deferred`, `Build-tool source-label trim contract`, `render_plan08_build_tool_source_label_trim_contract_python_passed_cargo_deferred`, `Build-tool explicit registry exact revision acceptance`, `render_plan08_build_tool_explicit_registry_exact_revision_acceptance_python_passed_cargo_deferred`, `expected_pass_types`, `taa_reactive_mask`, `test_acceptance_contract_validates_report_cache_and_exported_registry`, `test_acceptance_contract_validates_explicit_registry_against_report`, `test_acceptance_contract_rejects_explicit_registry_without_ready_revision`, `test_acceptance_contract_rejects_forward_only_staged_pass_report`, `test_acceptance_contract_rejects_duplicate_written_variant_identity`, `test_validate_cache_artifact_contract_requires_requested_pass_types`, `test_validate_cache_artifact_contract_accepts_requested_pass_types`, `test_validate_cache_artifact_contract_requires_requested_quality_tiers`, `test_validate_cache_artifact_contract_requires_requested_geometry_sources`, `test_validate_cache_artifact_contract_accepts_requested_quality_and_geometry`, `tools/zircon_build_shader_prewarm_written_variants.py`, `duplicate written cache variant identity`, `runtime fallback root`, `usable shader ResourceRecord revisions`, `Build-tool shader prewarm cache artifact contract`, `Prewarm report cache identity contract`, `Prewarm cache runtime layout contract`, `Prewarm cache hash shape contract`, `Prewarm cache custom id correlation contract`, `Runtime prewarm custom id cache lookup contract`, `Runtime custom id staged fallback lookup contract`, `render_plan08_runtime_custom_id_staged_fallback_lookup_static_passed_cargo_deferred`, `Build-tool cache quality/geometry identity contract`, `render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred`, `Build-tool cache dimension combination contract`, `render_plan08_build_tool_cache_dimension_combination_contract_python_passed_cargo_deferred`, `Build-tool cache custom id combination contract`, `render_plan08_build_tool_cache_custom_id_combination_contract_python_passed_cargo_deferred`, `Build-tool cache source-label provenance correlation contract`, `render_plan08_build_tool_cache_source_label_provenance_contract_python_passed_cargo_deferred`, `Build-tool cache metadata field type contract`, `render_plan08_build_tool_cache_metadata_field_type_contract_python_passed_cargo_deferred`, `test_validate_report_contract_rejects_untrimmed_source_provenance_strings`, `runtime_15_shader_prewarm_acceptance_contract_is_wired`, `runtime_15_shader_prewarm_cache_artifact_contract_is_wired`, `Asset-root resource registry revision overlay`, `render_plan08_asset_root_resource_registry_revision_overlay_typecheck_passed_test_timeout_no_result`, `render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred`, `bin/zircon_shader_prewarm/manifest/resource_registry.rs`, `shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay`, `shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only`, `runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired`, `Prewarm WGPU validation report summary`, `render_plan08_prewarm_wgpu_validation_report_summary_python_passed_cargo_deferred`, `Build-tool WGPU validation report contract`, `render_plan08_build_tool_wgpu_report_contract_python_passed_cargo_deferred`, `Build-tool WGPU validation totals match contract`, `render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred`, `test_zircon_build_shader_prewarm_wgpu_report_contract.py`, `test_dimension_summary_lines_format_wgpu_module_validation_counts`, `test_validate_report_contract_requires_wgpu_validation_when_requested`, `test_validate_report_contract_rejects_wgpu_validation_total_mismatch`, `runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired`, and `runtime_15_shader_prewarm_wgpu_report_contract_is_wired`.

2026-07-03 Runtime 15 shader prewarm manifest path helper owner note:
`runtime_15_shader_prewarm_manifest_path_helpers_owner_split_static_passed_cargo_deferred` moves package-local shader path discovery and content hashing out of `bin/zircon_shader_prewarm/manifest.rs` and into `bin/zircon_shader_prewarm/manifest/paths.rs`. The child owns `wgsl_files_for_document(...)`, `primary_zshader_path(...)`, `content_hash(...)`, and `ShaderSourceOutsidePackageDir`; the parent keeps manifest assembly, source/material expansion, resource registry overlay, and tests mount. The structure guard `runtime_15_shader_prewarm_manifest_tests_are_folder_backed` now locks the paths child and the current 10 manifest tests, while `runtime_15_no_oversized_production_files` confirms `manifest.rs` is back under the Runtime 15 production budget. Verification also passed `status_output` 143/143 and full plan-status 42/42; full structure sweep is 614/622 with the remaining 8 failures in shader-prewarm plugin/registry/product staged-cache anchors outside this slice. This is structure evidence only; package Cargo, WGPU validation, product miss=0, and RenderDoc capture remain separate Plan 08 gates.

2026-07-03 Runtime 15 shader prewarm owner guard sync note:
`runtime_15_shader_prewarm_owner_guard_sync_static_passed_cargo_deferred` updates the Runtime 15 structure guards to the current build-tool owners. `tools/zircon_build.py` remains the orchestrator; plugin asset-root and `distribution.assets` helpers live in `tools/zircon_build_plugin_assets.py`; geometry/shading descriptor parsing lives in `tools/zircon_build_plugin_shader_descriptors.py`; plugin package descriptor fields live in `tools/zircon_build_plugin_packages.py`. Exact affected structure guards passed 8/8 and full standalone structure sweep passed 622/622. This note does not claim a new build-tool behavior change, package Cargo pass, WGPU/product miss=0, or RenderDoc capture.
