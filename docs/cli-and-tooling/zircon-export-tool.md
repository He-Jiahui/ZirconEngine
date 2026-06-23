---
related_code:
  - tools/zircon_export/__init__.py
  - tools/zircon_export/__main__.py
  - tools/zircon_export/cli.py
  - tools/zircon_export/plugin_build.py
  - tools/zircon_export/compile_host.py
  - tools/zircon_export/command_plan.py
  - tools/zircon_export/cook_assets.py
  - tools/zircon_export/export_strategy_contract.py
  - tools/zircon_export/export_template.py
  - tools/zircon_export/native_build.py
  - tools/zircon_export/native_dynamic_contract.py
  - tools/zircon_export/native_dynamic.py
  - tools/zircon_export/native_dynamic_plan.py
  - tools/zircon_export/native_dynamic_payload.py
  - tools/zircon_export/native_dynamic_templates.py
  - tools/zircon_export/native_signing.py
  - tools/zircon_export/path_resolve.py
  - tools/zircon_export/pipeline_report.py
  - tools/zircon_export/pipeline_report_cook_assets.py
  - tools/zircon_export/pipeline_report_cook_assets_manifest_shape.py
  - tools/zircon_export/pipeline_report_stage_location.py
  - tools/zircon_export/pipeline_report_stage_schema.py
  - tools/zircon_export/pipeline_report_schema_primitives.py
  - tools/zircon_export/pipeline_report_schema_table.py
  - tools/zircon_export/pipeline_report_compile_host_stage_schema.py
  - tools/zircon_export/pipeline_report_cook_assets_stage_schema.py
  - tools/zircon_export/pipeline_report_validate_stage_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_build_audit_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_build_audit_common.py
  - tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_build_execution_schema.py
  - tools/zircon_export/pipeline_report_pack_stage_schema.py
  - tools/zircon_export/pipeline_report_pack_trim_schema.py
  - tools/zircon_export/pipeline_report_pack_manifest_schema.py
  - tools/zircon_export/pipeline_report_pack_delta_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_stage_report.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_operation_audit_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/pipeline_report_schema_table.py
  - tools/zircon_export/pipeline_report_platform_bundle.py
  - tools/zircon_export/pipeline_report_platform_bundle_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_manifest_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_schema_helpers.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_resolution_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_resolution_semantics.py
  - tools/zircon_export/pipeline_report_source_template.py
  - tools/zircon_export/pipeline_report_source_template_stage_schema.py
  - tools/zircon_export/pipeline_report_source_template_string_array_schema.py
  - tools/zircon_export/pipeline_report_source_template_validate_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_linkage_schema.py
  - tools/zircon_export/pipeline_report_validate_plan_vector_schema.py
  - tools/zircon_export/pipeline_report_validate_profile_summary_schema.py
  - tools/zircon_export/pipeline_report_validate_runtime_availability_schema.py
  - tools/zircon_export/pipeline_report_validate_string_array_schema.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/platform_bundle.py
  - tools/zircon_export/report_io.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/stage_handoff.py
  - tools/zircon_export/subprocess_output.py
  - tools/zircon_export/tests/native_dynamic_test_support.py
  - tools/zircon_export/tests/export_test_support.py
  - tools/zircon_export/tests/platform_bundle_report_test_support.py
  - tools/zircon_export/tests/pack_schema_test_support.py
  - tools/zircon_export/tests/pack_test_support.py
  - tools/zircon_export/tests/test_command_plan.py
  - tools/zircon_export/tests/test_compile_host_plan_feature_array_schema.py
  - tools/zircon_export/tests/test_compile_host_plan_linked_crate_schema.py
  - tools/zircon_export/tests/test_compile_host_plan_command_schema.py
  - tools/zircon_export/tests/test_compile_host_output_gate.py
  - tools/zircon_export/tests/test_compile_host_source_template.py
  - tools/zircon_export/tests/test_source_template_project_root_errors.py
  - tools/zircon_export/tests/test_source_template_plan_command_schema.py
  - tools/zircon_export/tests/test_source_template_plan_string_schema.py
  - tools/zircon_export/tests/test_source_template_plan_generated_file_schema.py
  - tools/zircon_export/tests/test_cook_assets_manifest_schema_gate.py
  - tools/zircon_export/tests/test_cook_assets_pack_stage.py
  - tools/zircon_export/tests/test_cook_assets_project_fallback.py
  - tools/zircon_export/tests/test_native_dynamic_build_signing.py
  - tools/zircon_export/tests/test_native_dynamic_copy_file_errors.py
  - tools/zircon_export/tests/test_native_dynamic_signing_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_payload_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_path_resolve_errors.py
  - tools/zircon_export/tests/test_plugin_build.py
  - tools/zircon_export/tests/test_native_dynamic_stage.py
  - tools/zircon_export/tests/test_pipeline_report_source_template.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_validate_build_plan.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_build_validation.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_command_schema.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_schema.py
  - tools/zircon_export/tests/native_dynamic_stage_report_test_support.py
  - tools/zircon_export/tests/native_dynamic_stage_schema_test_support.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_materialized_packages.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_package_report_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_file_manifest_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_top_level_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_manifest_shape_schema.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_manifest_evidence.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_pack_handoff.py
  - tools/zircon_export/tests/test_pipeline_report_stage.py
  - tools/zircon_export/tests/test_pipeline_report_stage_location.py
  - tools/zircon_export/tests/test_pipeline_report_stage_metadata_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_stage_metadata.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_plan_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_plan_command_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_execution_schema.py
  - tools/zircon_export/tests/test_pipeline_report_compile_host_command_semantics.py
  - tools/zircon_export/tests/test_pipeline_report_pack_delta_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_trim_report_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_file_evidence_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_path_string_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_compile_host_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_compile_host_linkage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_plan_vector_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_profile_summary_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_native_dynamic_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_runtime_availability_schema.py
  - tools/zircon_export/tests/test_platform_bundle_cleanup_errors.py
  - tools/zircon_export/tests/test_platform_bundle_native_dynamic.py
  - tools/zircon_export/tests/test_platform_bundle_native_dynamic_operation_audit.py
  - tools/zircon_export/tests/test_platform_bundle_native_payload_loader_manifest.py
  - tools/zircon_export/tests/test_platform_bundle_native_plugins_copy_errors.py
  - tools/zircon_export/tests/test_platform_bundle_delta.py
  - tools/zircon_export/tests/test_platform_bundle_inputs.py
  - tools/zircon_export/tests/test_platform_bundle_path_resolve_errors.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_file_reads.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_manifest_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_manifest_top_level_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_path_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_bundle_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_file_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_file_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_report_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_string_array_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle.py
  - tools/zircon_export/tests/test_report_write_errors.py
  - tools/zircon_export/tests/test_pipeline_resume_flow.py
  - tools/zircon_export/tests/test_stage_directory_errors.py
  - tools/zircon_export/tests/test_subprocess_launch_errors.py
  - tools/zircon_export/tests/test_stage_handoff.py
  - tools/zircon_export/tests/test_export_template_trimmed_schema.py
  - tools/zircon_export/tests/test_templates.py
  - tools/zircon_export/tests/test_native_dynamic.py
  - tools/zircon_export/export-templates/windows-x86_64-library_embed-debug/template.toml
  - tools/zircon_export/export-templates/linux-x86_64-library_embed-debug/template.toml
  - tools/zircon_export/export-templates/macos-aarch64-library_embed-debug/template.toml
  - tools/zircon_export/__init__.py
  - tools/zircon_export/__main__.py
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/bin/zircon_export_pack/main.rs
  - zircon_runtime/src/bin/zircon_export_pack/args.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/asset/pack/trim.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/Cargo.toml
implementation_files:
  - tools/zircon_export/cli.py
  - tools/zircon_export/plugin_build.py
  - tools/zircon_export/compile_host.py
  - tools/zircon_export/command_plan.py
  - tools/zircon_export/cook_assets.py
  - tools/zircon_export/export_strategy_contract.py
  - tools/zircon_export/export_template.py
  - tools/zircon_export/native_build.py
  - tools/zircon_export/native_dynamic_contract.py
  - tools/zircon_export/native_dynamic.py
  - tools/zircon_export/native_dynamic_plan.py
  - tools/zircon_export/native_dynamic_payload.py
  - tools/zircon_export/native_dynamic_templates.py
  - tools/zircon_export/native_signing.py
  - tools/zircon_export/path_resolve.py
  - tools/zircon_export/pipeline_report.py
  - tools/zircon_export/pipeline_report_cook_assets.py
  - tools/zircon_export/pipeline_report_cook_assets_manifest_shape.py
  - tools/zircon_export/pipeline_report_stage_location.py
  - tools/zircon_export/pipeline_report_stage_schema.py
  - tools/zircon_export/pipeline_report_schema_primitives.py
  - tools/zircon_export/pipeline_report_compile_host_stage_schema.py
  - tools/zircon_export/pipeline_report_cook_assets_stage_schema.py
  - tools/zircon_export/pipeline_report_validate_stage_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/pipeline_report_pack_stage_schema.py
  - tools/zircon_export/pipeline_report_pack_trim_schema.py
  - tools/zircon_export/pipeline_report_pack_manifest_schema.py
  - tools/zircon_export/pipeline_report_pack_delta_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_stage_report.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle.py
  - tools/zircon_export/pipeline_report_platform_bundle_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_manifest_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_schema_helpers.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_resolution_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_resolution_semantics.py
  - tools/zircon_export/pipeline_report_source_template.py
  - tools/zircon_export/pipeline_report_source_template_stage_schema.py
  - tools/zircon_export/pipeline_report_source_template_string_array_schema.py
  - tools/zircon_export/pipeline_report_source_template_validate_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_linkage_schema.py
  - tools/zircon_export/pipeline_report_validate_plan_vector_schema.py
  - tools/zircon_export/pipeline_report_validate_runtime_availability_schema.py
  - tools/zircon_export/pipeline_report_validate_string_array_schema.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/platform_bundle.py
  - tools/zircon_export/report_io.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/stage_handoff.py
  - tools/zircon_export/tests/native_dynamic_test_support.py
  - tools/zircon_export/tests/export_test_support.py
  - tools/zircon_export/tests/platform_bundle_report_test_support.py
  - tools/zircon_export/tests/pack_schema_test_support.py
  - tools/zircon_export/tests/pack_test_support.py
  - tools/zircon_export/tests/test_command_plan.py
  - tools/zircon_export/tests/test_compile_host_plan_feature_array_schema.py
  - tools/zircon_export/tests/test_compile_host_plan_linked_crate_schema.py
  - tools/zircon_export/tests/test_compile_host_plan_command_schema.py
  - tools/zircon_export/tests/test_compile_host_output_gate.py
  - tools/zircon_export/tests/test_compile_host_source_template.py
  - tools/zircon_export/tests/test_source_template_project_root_errors.py
  - tools/zircon_export/tests/test_source_template_plan_command_schema.py
  - tools/zircon_export/tests/test_source_template_plan_string_schema.py
  - tools/zircon_export/tests/test_source_template_plan_generated_file_schema.py
  - tools/zircon_export/tests/test_cook_assets_pack_stage.py
  - tools/zircon_export/tests/test_native_dynamic_build_signing.py
  - tools/zircon_export/tests/test_native_dynamic_copy_file_errors.py
  - tools/zircon_export/tests/test_native_dynamic_signing_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_payload_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_stage.py
  - tools/zircon_export/tests/test_plugin_build.py
  - tools/zircon_export/tests/test_platform_bundle_native_dynamic.py
  - tools/zircon_export/tests/test_platform_bundle_native_dynamic_operation_audit.py
  - tools/zircon_export/tests/test_platform_bundle_native_payload_loader_manifest.py
  - tools/zircon_export/tests/test_platform_bundle_delta.py
  - tools/zircon_export/tests/test_platform_bundle_inputs.py
  - tools/zircon_export/tests/test_platform_bundle_path_resolve_errors.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_file_reads.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_manifest_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_manifest_top_level_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_path_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_bundle_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_file_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_file_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_report_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_string_array_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle.py
  - tools/zircon_export/tests/test_report_write_errors.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/tests/native_dynamic_stage_report_test_support.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_materialized_packages.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_package_report_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_file_manifest_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_top_level_trimmed_schema.py
  - tools/zircon_export/tests/test_pipeline_report_source_template.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_validate_build_plan.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_build_validation.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_command_schema.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_schema.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_manifest_shape_schema.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_pack_handoff.py
  - tools/zircon_export/tests/test_pipeline_report_stage.py
  - tools/zircon_export/tests/test_pipeline_report_stage_location.py
  - tools/zircon_export/tests/test_pipeline_report_stage_metadata_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_stage_metadata.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_schema.py
  - tools/zircon_export/tests/test_pipeline_report_compile_host_command_semantics.py
  - tools/zircon_export/tests/test_pipeline_report_pack_delta_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_file_evidence_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_path_string_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_compile_host_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_compile_host_linkage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_diagnostics_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_plan_vector_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_native_dynamic_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_runtime_availability_schema.py
  - tools/zircon_export/tests/test_pipeline_resume_flow.py
  - tools/zircon_export/tests/test_stage_directory_errors.py
  - tools/zircon_export/tests/test_subprocess_launch_errors.py
  - tools/zircon_export/tests/test_export_template_trimmed_schema.py
  - tools/zircon_export/tests/test_templates.py
  - tools/zircon_export/tests/test_native_dynamic.py
  - tools/zircon_export/export-templates/windows-x86_64-library_embed-debug/template.toml
  - tools/zircon_export/export-templates/windows-x86_64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - tools/zircon_export/export-templates/linux-x86_64-library_embed-debug/template.toml
  - tools/zircon_export/export-templates/linux-x86_64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - tools/zircon_export/export-templates/macos-aarch64-library_embed-debug/template.toml
  - tools/zircon_export/export-templates/macos-aarch64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - tools/zircon_export/export-templates/macos-aarch64-library_embed-debug/platform/macos/Info.plist
  - tools/zircon_export/__main__.py
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_validate/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/run.rs
  - zircon_runtime/src/bin/zircon_export_pack/main.rs
  - zircon_runtime/src/bin/zircon_export_pack/args.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs
  - zircon_runtime/src/asset/pack/trim.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/Cargo.toml
plan_sources:
  - docs/plans/zircon_plugins/09-export-publishing.md
tests:
  - python -m py_compile tools\zircon_export\pipeline_report.py tools\zircon_export\pipeline_report_compile_host_stage_schema.py tools\zircon_export\tests\export_test_support.py tools\zircon_export\tests\test_pipeline_report_compile_host_command_semantics.py tools\zircon_export\tests\test_pipeline_report_compile_host_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics.CompileHostCommandSemanticsTests.test_report_stage_rejects_compile_host_command_target_dir_mismatch
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics.CompileHostCommandSemanticsTests.test_report_stage_rejects_compile_host_host_executable_outside_output_root
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics.CompileHostCommandSemanticsTests.test_report_stage_rejects_compile_host_host_executable_target_dir_mismatch
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics.CompileHostCommandSemanticsTests.test_report_stage_rejects_compile_host_host_executable_binary_mismatch
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics.CompileHostCommandSemanticsTests.test_report_stage_rejects_compile_host_missing_host_executable_file
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_with_non_cargo_command
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_command_metadata_mismatch
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_with_invalid_profile_release
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_missing_required_evidence_field
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_malformed_string_evidence_field
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_non_string_evidence_field
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_malformed_array_evidence_field
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema.CompileHostStageSchemaTests.test_report_stage_rejects_compile_host_nonfatal_nonzero_exit_code
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_command_feature_mismatch
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_command_target_broadening
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_command_target_broadening
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_command_target_triple_override
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_command_target_triple_override
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_command_package_broadening
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_command_package_broadening
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_command_profile_override
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_command_profile_override
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_command_wrapper_policy_override
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_command_wrapper_policy_override
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_forbidden_command_equals_form
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate.CompileHostOutputGateTests.test_compile_host_rejects_plan_forbidden_command_equals_form
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_profile_release_mismatch
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_missing_required_field
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_blank_string_field
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_unsafe_path_field
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_command_non_cargo
  - python -m py_compile tools\zircon_export\compile_host.py tools\zircon_export\pipeline_report_validate_compile_host_schema.py tools\zircon_export\tests\export_test_support.py tools\zircon_export\tests\test_compile_host_output_gate.py tools\zircon_export\tests\test_pipeline_report_validate_compile_host_schema.py
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema
  - python -m py_compile tools/zircon_export/pipeline_report_schema_table.py tools/zircon_export/pipeline_report_validate_compile_host_schema.py tools/zircon_export/pipeline_report_compile_host_stage_schema.py tools/zircon_export/tests/test_pipeline_report_validate_compile_host_schema.py tools/zircon_export/tests/test_pipeline_report_compile_host_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_diagnostics_schema tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema
  - python -m py_compile tools\zircon_export\pipeline_report_compile_host_stage_schema.py tools\zircon_export\tests\export_test_support.py tools\zircon_export\tests\test_pipeline_report_compile_host_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema
  - python -m py_compile tools\zircon_export\pipeline_report_validate_compile_host_linkage_schema.py tools\zircon_export\tests\test_pipeline_report_validate_compile_host_linkage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema
  - python -m py_compile tools/zircon_export/pipeline_report_validate_compile_host_linkage_schema.py tools/zircon_export/tests/test_pipeline_report_validate_compile_host_linkage_schema.py tools/zircon_export/tests/test_compile_host_plan_linked_crate_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema.PipelineReportValidateCompileHostLinkageSchemaTests.test_report_stage_rejects_validate_compile_host_linked_crate_path_invalid tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema.PipelineReportValidateCompileHostLinkageSchemaTests.test_report_stage_rejects_compile_host_linked_crate_path_invalid tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema.PipelineReportValidateCompileHostLinkageSchemaTests.test_report_stage_rejects_validate_compile_host_linked_crate_registration_kind_invalid tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema.PipelineReportValidateCompileHostLinkageSchemaTests.test_report_stage_rejects_compile_host_linked_crate_registration_kind_padded
  - python -m unittest tools.zircon_export.tests.test_compile_host_plan_linked_crate_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema
  - python -m unittest tools.zircon_export.tests.test_compile_host_plan_linked_crate_schema tools.zircon_export.tests.test_compile_host_plan_feature_array_schema tools.zircon_export.tests.test_compile_host_plan_command_schema tools.zircon_export.tests.test_compile_host_output_gate
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_diagnostics_schema tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema
  - python -m py_compile tools\zircon_export\pipeline_report_compile_host_stage_schema.py tools\zircon_export\tests\test_pipeline_report_compile_host_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema.CompileHostStageSchemaTests.test_report_stage_rejects_compile_host_padded_command_entry
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics tools.zircon_export.tests.test_compile_host_output_gate tools.zircon_export.tests.test_compile_host_source_template
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema tools.zircon_export.tests.test_pipeline_report_stage
  - python -m py_compile tools\zircon_export\pipeline_report_validate_compile_host_schema.py tools\zircon_export\tests\test_pipeline_report_validate_compile_host_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_padded_command_entry
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_diagnostics_schema tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema
  - python -m py_compile tools/zircon_export/compile_host.py tools/zircon_export/tests/test_compile_host_plan_command_schema.py
  - python -m unittest tools.zircon_export.tests.test_compile_host_plan_command_schema.CompileHostPlanCommandSchemaTests.test_compile_host_rejects_plan_with_padded_command_entry
  - python -m unittest tools.zircon_export.tests.test_compile_host_plan_command_schema tools.zircon_export.tests.test_compile_host_output_gate
  - python -m unittest tools.zircon_export.tests.test_compile_host_source_template tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema
  - python -m py_compile tools/zircon_export/compile_host.py tools/zircon_export/tests/test_compile_host_plan_feature_array_schema.py
  - python -m unittest tools.zircon_export.tests.test_compile_host_plan_feature_array_schema.CompileHostPlanFeatureArraySchemaTests.test_compile_host_rejects_plan_with_padded_feature_entry
  - python -m unittest tools.zircon_export.tests.test_compile_host_plan_feature_array_schema tools.zircon_export.tests.test_compile_host_plan_command_schema tools.zircon_export.tests.test_compile_host_output_gate
  - python -m unittest tools.zircon_export.tests.test_compile_host_source_template tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_compile_host_stage_schema tools.zircon_export.tests.test_pipeline_report_compile_host_command_semantics
  - python -m unittest tools.zircon_export.tests.test_compile_host_plan_command_schema tools.zircon_export.tests.test_compile_host_plan_feature_array_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema.PipelineReportValidateCompileHostSchemaTests.test_report_stage_rejects_validate_compile_host_string_array_entry_non_string tools.zircon_export.tests.test_pipeline_report_validate_schema.PipelineReportValidateSchemaTests.test_report_stage_rejects_validate_compile_host_plan_string_array_fields_non_string_array
  - python -m unittest tools.zircon_export.tests.test_compile_host_output_gate tools.zircon_export.tests.test_compile_host_plan_command_schema tools.zircon_export.tests.test_compile_host_plan_feature_array_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_compile_host_schema tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_compile_host_linkage_schema
  - python -m py_compile tools\zircon_export\compile_host.py tools\zircon_export\pipeline_report_validate_compile_host_schema.py tools\zircon_export\tests\test_compile_host_plan_command_schema.py tools\zircon_export\tests\test_compile_host_plan_feature_array_schema.py tools\zircon_export\tests\test_pipeline_report_validate_compile_host_schema.py tools\zircon_export\tests\test_pipeline_report_validate_schema.py
  - python -m py_compile tools\zircon_export\tests\test_pipeline_report_source_template.py tools\zircon_export\tests\test_pipeline_report_source_template_validate_build_plan.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan
  - python -m py_compile tools/zircon_export/pipeline_report_source_template_stage_schema.py tools/zircon_export/pipeline_report_source_template.py tools/zircon_export/tests/test_pipeline_report_source_template_command_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_command_schema.PipelineReportSourceTemplateCommandSchemaTests.test_report_rejects_source_template_report_padded_command_entry tools.zircon_export.tests.test_pipeline_report_source_template_command_schema.PipelineReportSourceTemplateCommandSchemaTests.test_report_rejects_source_template_build_validation_padded_command_entry
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_command_schema tools.zircon_export.tests.test_pipeline_report_source_template_build_validation
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_command_schema tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema.PipelineReportStageMetadataDiagnosticsSchemaTests.test_report_stage_rejects_non_string_stage_diagnostic_entry_before_array_shape tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema.PipelineReportStageMetadataDiagnosticsSchemaTests.test_handoff_rejects_non_string_stage_diagnostic_entry_before_array_shape
  - python -m py_compile tools\zircon_export\stage_handoff.py tools\zircon_export\tests\test_pipeline_report_stage_metadata_diagnostics_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema tools.zircon_export.tests.test_stage_handoff
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_stage_handoff
  - python -m py_compile tools\zircon_export\stage_handoff.py tools\zircon_export\tests\test_pipeline_report_stage_metadata_diagnostics_schema.py tools\zircon_export\tests\test_pipeline_report_stage_metadata.py tools\zircon_export\tests\test_pipeline_report_stage.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_report_stage_location tools.zircon_export.tests.test_compile_host_source_template tools.zircon_export.tests.test_source_template_project_root_errors
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_command_schema.PipelineReportSourceTemplateCommandSchemaTests.test_report_rejects_source_template_report_non_string_command_entry_before_array_shape tools.zircon_export.tests.test_pipeline_report_source_template_command_schema.PipelineReportSourceTemplateCommandSchemaTests.test_report_rejects_source_template_build_validation_non_string_command_entry_before_array_shape
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan.PipelineReportSourceTemplateValidateBuildPlanTests.test_report_rejects_source_template_validate_build_plan_non_string_command_entry_before_array_shape
  - python -m py_compile tools\zircon_export\pipeline_report_source_template_string_array_schema.py tools\zircon_export\pipeline_report_source_template_stage_schema.py tools\zircon_export\pipeline_report_source_template_validate_schema.py tools\zircon_export\pipeline_report_source_template.py tools\zircon_export\tests\test_pipeline_report_source_template_command_schema.py tools\zircon_export\tests\test_pipeline_report_source_template_validate_build_plan.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_command_schema tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_source_template_plan_command_schema tools.zircon_export.tests.test_source_template_project_root_errors tools.zircon_export.tests.test_compile_host_source_template
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_build_validation.PipelineReportSourceTemplateBuildValidationTests.test_report_rejects_source_template_build_validation_non_string_log_line_entry_before_array_shape
  - python -m py_compile tools\zircon_export\tests\test_pipeline_report_source_template_build_validation.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_pipeline_report_source_template_command_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan
  - python -m py_compile tools/zircon_export/pipeline_report_source_template_validate_schema.py tools/zircon_export/pipeline_report_source_template.py tools/zircon_export/tests/test_pipeline_report_source_template_validate_build_plan.py tools/zircon_export/tests/test_source_template_plan_command_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan.PipelineReportSourceTemplateValidateBuildPlanTests.test_report_rejects_padded_source_template_validate_build_plan_command_entry tools.zircon_export.tests.test_source_template_plan_command_schema.SourceTemplatePlanCommandSchemaTests.test_source_template_rejects_plan_with_padded_command_entry
  - python -m unittest tools.zircon_export.tests.test_source_template_plan_command_schema.SourceTemplatePlanCommandSchemaTests.test_source_template_rejects_plan_with_non_string_command_entry_before_array_shape
  - python -m unittest tools.zircon_export.tests.test_source_template_plan_command_schema tools.zircon_export.tests.test_source_template_build_plan_schema_gate tools.zircon_export.tests.test_source_template_command_gate
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan tools.zircon_export.tests.test_source_template_plan_command_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_command_schema tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan tools.zircon_export.tests.test_source_template_plan_command_schema tools.zircon_export.tests.test_source_template_project_root_errors
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_report_stage_location tools.zircon_export.tests.test_compile_host_source_template
  - python -m py_compile tools/zircon_export/pipeline_report_source_template_validate_schema.py tools/zircon_export/pipeline_report_source_template.py tools/zircon_export/tests/test_pipeline_report_source_template_validate_build_plan.py tools/zircon_export/tests/test_source_template_plan_string_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan.PipelineReportSourceTemplateValidateBuildPlanTests.test_report_rejects_source_template_validate_build_plan_padded_required_string_field tools.zircon_export.tests.test_source_template_plan_string_schema.SourceTemplatePlanStringSchemaTests.test_source_template_rejects_plan_with_padded_required_string_field
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan tools.zircon_export.tests.test_source_template_plan_string_schema tools.zircon_export.tests.test_source_template_plan_command_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_command_schema tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan tools.zircon_export.tests.test_source_template_plan_command_schema tools.zircon_export.tests.test_source_template_plan_string_schema tools.zircon_export.tests.test_source_template_project_root_errors
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_report_stage_location tools.zircon_export.tests.test_compile_host_source_template
  - python -m py_compile tools/zircon_export/pipeline_report_source_template_stage_schema.py tools/zircon_export/pipeline_report_source_template_validate_schema.py tools/zircon_export/pipeline_report_source_template.py tools/zircon_export/tests/test_pipeline_report_source_template_schema.py tools/zircon_export/tests/test_source_template_plan_generated_file_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_schema.PipelineReportSourceTemplateSchemaTests.test_report_rejects_source_template_generated_file_padded_string_field tools.zircon_export.tests.test_pipeline_report_source_template_schema.PipelineReportSourceTemplateSchemaTests.test_report_rejects_source_template_validate_generated_file_padded_string_field tools.zircon_export.tests.test_source_template_plan_generated_file_schema.SourceTemplatePlanGeneratedFileSchemaTests.test_source_template_rejects_plan_with_padded_generated_file_string_field
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_source_template_plan_generated_file_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_command_schema tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan tools.zircon_export.tests.test_source_template_plan_command_schema tools.zircon_export.tests.test_source_template_plan_string_schema tools.zircon_export.tests.test_source_template_plan_generated_file_schema tools.zircon_export.tests.test_source_template_project_root_errors
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_report_stage_location tools.zircon_export.tests.test_compile_host_source_template
  - python -m py_compile tools/zircon_export/pipeline_report_source_template_stage_schema.py tools/zircon_export/pipeline_report_source_template.py tools/zircon_export/tests/test_pipeline_report_source_template_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_schema.PipelineReportSourceTemplateSchemaTests.test_report_stage_rejects_source_template_padded_required_string_field tools.zircon_export.tests.test_pipeline_report_source_template_schema.PipelineReportSourceTemplateSchemaTests.test_report_stage_rejects_source_template_padded_cleanup_reason tools.zircon_export.tests.test_pipeline_report_source_template_schema.PipelineReportSourceTemplateSchemaTests.test_report_rejects_source_template_build_validation_padded_required_string
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_command_schema tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_validate_build_plan tools.zircon_export.tests.test_source_template_plan_command_schema tools.zircon_export.tests.test_source_template_plan_string_schema tools.zircon_export.tests.test_source_template_plan_generated_file_schema tools.zircon_export.tests.test_source_template_project_root_errors
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_report_stage_location tools.zircon_export.tests.test_compile_host_source_template
  - python -m py_compile tools/zircon_export/pipeline_report_cook_assets_stage_schema.py tools/zircon_export/pipeline_report_cook_assets.py tools/zircon_export/tests/test_pipeline_report_cook_assets_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_cook_assets_stage_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_cook_assets_stage_schema tools.zircon_export.tests.test_pipeline_report_cook_assets_manifest_evidence
  - python -m unittest tools.zircon_export.tests.test_cook_assets_pack_stage tools.zircon_export.tests.test_cook_assets_project_fallback tools.zircon_export.tests.test_cook_assets_path_resolve_errors
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema
  - python -m py_compile tools/zircon_export/pipeline_report_cook_assets_manifest_shape.py tools/zircon_export/pipeline_report_cook_assets.py tools/zircon_export/tests/test_pipeline_report_cook_assets_manifest_shape_schema.py tools/zircon_export/tests/test_pipeline_report_cook_assets_manifest_evidence.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_cook_assets_manifest_shape_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_cook_assets_manifest_shape_schema tools.zircon_export.tests.test_pipeline_report_cook_assets_manifest_evidence tools.zircon_export.tests.test_pipeline_report_cook_assets_stage_schema
  - python -m unittest tools.zircon_export.tests.test_cook_assets_pack_stage tools.zircon_export.tests.test_cook_assets_project_fallback tools.zircon_export.tests.test_cook_assets_path_resolve_errors
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_stage_metadata_diagnostics_schema
  - python -m py_compile tools/zircon_export/tests/test_pipeline_report_cook_assets_pack_handoff.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_cook_assets_pack_handoff
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_cook_assets_stage_schema tools.zircon_export.tests.test_pipeline_report_cook_assets_manifest_evidence tools.zircon_export.tests.test_pipeline_report_cook_assets_pack_handoff
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_pack_file_evidence_schema tools.zircon_export.tests.test_pipeline_report_pack_stage_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata.PipelineReportStageMetadataTests.test_report_stage_rejects_pack_string_array_fields_non_string_array
  - python -m py_compile tools\zircon_export\pipeline_report_pack_stage_schema.py tools\zircon_export\tests\test_pipeline_report_stage_metadata.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata.PipelineReportStageMetadataTests.test_report_stage_rejects_pack_string_array_fields_non_string_array tools.zircon_export.tests.test_pipeline_report_pack_delta_top_level_schema.PipelineReportPackDeltaTopLevelSchemaTests.test_report_stage_rejects_pack_delta_asset_list_non_string_entry_before_array_shape
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_pack_stage_schema tools.zircon_export.tests.test_pipeline_report_pack_delta_top_level_schema tools.zircon_export.tests.test_pipeline_report_pack_path_string_schema
  - python -m unittest tools.zircon_export.tests.test_cook_assets_pack_stage tools.zircon_export.tests.test_cook_assets_project_fallback tools.zircon_export.tests.test_cook_assets_path_resolve_errors
  - python -m py_compile tools\zircon_export\tests\test_pipeline_report_validate_schema.py tools\zircon_export\tests\test_pipeline_report_validate_plan_vector_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_plan_vector_schema
  - python -m py_compile tools/zircon_export/__init__.py tools/zircon_export/__main__.py tools/zircon_export/cli.py tools/zircon_export/compile_host.py tools/zircon_export/cook_assets.py tools/zircon_export/export_template.py tools/zircon_export/native_build.py tools/zircon_export/native_dynamic.py tools/zircon_export/native_dynamic_plan.py tools/zircon_export/native_signing.py tools/zircon_export/pipeline_report.py tools/zircon_export/pipeline_report_platform_bundle.py tools/zircon_export/pipeline_stages.py tools/zircon_export/platform_bundle.py tools/zircon_export/source_template.py tools/zircon_export/tests/export_test_support.py tools/zircon_export/tests/test_templates.py tools/zircon_export/tests/test_compile_host_source_template.py tools/zircon_export/tests/test_cook_assets_pack_stage.py tools/zircon_export/tests/test_native_dynamic_stage.py tools/zircon_export/tests/test_pipeline_report_stage.py tools/zircon_export/tests/test_pipeline_resume_flow.py tools/zircon_export/tests/test_native_dynamic.py tools/zircon_export/tests/test_platform_bundle_delta.py
  - python -m tools.zircon_export --help
  - python -m py_compile tools/zircon_export/plugin_build.py tools/zircon_export/cli.py tools/zircon_export/tests/test_plugin_build.py: passed 2026-06-23
  - python -m unittest tools.zircon_export.tests.test_plugin_build: 4 passed, 0 failed on 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m3-package-fixture --target-dir D:\cargo-targets\zircon-plugin-m3-build-fixture: passed 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m3-t3-package-a --target-dir D:\cargo-targets\zircon-plugin-m3-t3-build-fixture: passed 2026-06-23
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m3-t3-package-b --target-dir D:\cargo-targets\zircon-plugin-m3-t3-build-fixture: passed 2026-06-23; package sha256 comparison returned MATCH
  - CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=1 python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --repo-root E:\Git\ZirconEngine --out D:\cargo-targets\zircon-plugin-m4-t1-package-fixture --target-dir D:\cargo-targets\zircon-plugin-m4-t1-build-fixture: timed out after 604s during packer Cargo build/run on 2026-06-23; not counted as passed
  - python -m unittest tools.zircon_export.tests.test_templates
  - python -m unittest tools.zircon_export.tests.test_compile_host_source_template
  - python -m unittest tools.zircon_export.tests.test_cook_assets_pack_stage
  - python -m unittest tools.zircon_export.tests.test_cook_assets_project_fallback
  - python -m unittest tools.zircon_export.tests.test_native_dynamic_stage
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_runtime_availability_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template
  - python -m unittest tools.zircon_export.tests.test_pipeline_resume_flow
  - python -m unittest tools.zircon_export.tests.test_native_dynamic
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_delta
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_platform_bundle
  - python -m unittest tools.zircon_export.tests.test_templates tools.zircon_export.tests.test_platform_bundle_native_dynamic tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit
  - python -m unittest tools.zircon_export.tests.test_native_dynamic tools.zircon_export.tests.test_native_dynamic_build_signing
  - python -m py_compile tools/zircon_export/pipeline_report_schema_table.py tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_plan_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_build_plan_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_build_plan_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_build_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_build_execution_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_native_dynamic
  - python -m py_compile tools/zircon_export/pipeline_report_schema_table.py tools/zircon_export/pipeline_report_native_dynamic_stage_schema.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema.PipelineReportNativeDynamicStageSchemaTests.test_report_stage_rejects_native_dynamic_string_array_fields_padded_entry
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_materialized_packages
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_native_dynamic tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit
  - python -m py_compile tools/zircon_export/pipeline_report_native_dynamic_stage_schema.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema.PipelineReportNativeDynamicStageSchemaTests.test_report_stage_rejects_native_dynamic_padded_required_string_release_evidence_field
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_materialized_packages
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema tools.zircon_export.tests.test_platform_bundle_native_dynamic
  - python -m py_compile tools/zircon_export/pipeline_report_schema_table.py tools/zircon_export/pipeline_report_native_dynamic_operation_audit_schema.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_operation_audit_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_operation_audit_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit tools.zircon_export.tests.test_platform_bundle_native_dynamic
  - python -m py_compile tools/zircon_export/pipeline_report_native_dynamic_package_report_schema.py tools/zircon_export/pipeline_report_native_dynamic_payload.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_package_report_trimmed_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_package_report_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_package_report_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_package_report_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_package_report_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_top_level_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_file_manifest_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_native_dynamic tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest
  - python -m py_compile tools/zircon_export/pipeline_report_schema_table.py tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_top_level_trimmed_schema.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_file_manifest_trimmed_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_top_level_trimmed_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_file_manifest_trimmed_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_top_level_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_file_manifest_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_operation_audit_schema
  - python -m py_compile tools/zircon_export/pipeline_report_schema_table.py tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_operation_audit_schema
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_native_dynamic tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit
  - python -m unittest tools.zircon_export.tests.test_templates tools.zircon_export.tests.test_platform_bundle_native_dynamic tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit tools.zircon_export.tests.test_compile_host_source_template tools.zircon_export.tests.test_cook_assets_pack_stage tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema tools.zircon_export.tests.test_pipeline_report_validate_runtime_availability_schema tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_resume_flow tools.zircon_export.tests.test_native_dynamic_stage tools.zircon_export.tests.test_native_dynamic tools.zircon_export.tests.test_native_dynamic_build_signing tools.zircon_export.tests.test_platform_bundle_delta tools.zircon_export.tests.test_platform_bundle_inputs tools.zircon_export.tests.test_stage_handoff tools.zircon_export.tests.test_pipeline_report_platform_bundle
  - python -m py_compile tools\zircon_export\cli.py tools\zircon_export\compile_host.py tools\zircon_export\export_template.py tools\zircon_export\platform_bundle.py tools\zircon_export\pipeline_report.py tools\zircon_export\pipeline_report_platform_bundle.py tools\zircon_export\native_dynamic.py tools\zircon_export\tests\export_test_support.py tools\zircon_export\tests\test_templates.py tools\zircon_export\tests\test_compile_host_source_template.py tools\zircon_export\tests\test_cook_assets_pack_stage.py tools\zircon_export\tests\test_native_dynamic_stage.py tools\zircon_export\tests\test_pipeline_report_stage.py tools\zircon_export\tests\test_pipeline_resume_flow.py tools\zircon_export\tests\test_platform_bundle_inputs.py tools\zircon_export\tests\test_platform_bundle_delta.py tools\zircon_export\tests\test_pipeline_report_platform_bundle.py
  - python -m py_compile tools\zircon_export\tests\test_templates.py tools\zircon_export\tests\test_platform_bundle_native_dynamic.py tools\zircon_export\tests\test_platform_bundle_native_dynamic_operation_audit.py
  - python -m py_compile tools\zircon_export\native_dynamic.py tools\zircon_export\native_dynamic_contract.py tools\zircon_export\native_dynamic_payload.py tools\zircon_export\native_dynamic_templates.py tools\zircon_export\pipeline_report_platform_bundle.py tools\zircon_export\platform_bundle.py tools\zircon_export\tests\test_native_dynamic.py tools\zircon_export\tests\test_platform_bundle_inputs.py
  - python -m py_compile tools\zircon_export\native_dynamic.py tools\zircon_export\native_dynamic_plan.py tools\zircon_export\tests\test_native_dynamic.py tools\zircon_export\tests\test_native_dynamic_stage.py tools\zircon_export\tests\test_native_dynamic_build_signing.py tools\zircon_export\tests\test_native_dynamic_copy_file_errors.py tools\zircon_export\tests\test_native_dynamic_path_resolve_errors.py tools\zircon_export\tests\test_pipeline_report_validate_native_dynamic_schema.py tools\zircon_export\tests\test_platform_bundle_native_dynamic.py
  - python -m unittest tools.zircon_export.tests.test_native_dynamic tools.zircon_export.tests.test_native_dynamic_stage tools.zircon_export.tests.test_native_dynamic_build_signing tools.zircon_export.tests.test_native_dynamic_copy_file_errors tools.zircon_export.tests.test_native_dynamic_path_resolve_errors tools.zircon_export.tests.test_native_dynamic_payload_file_reads tools.zircon_export.tests.test_native_dynamic_signing_file_reads tools.zircon_export.tests.test_platform_bundle_native_dynamic tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema tools.zircon_export.tests.test_pipeline_report_validate_plan_vector_schema
  - python -m py_compile tools\zircon_export\tests\test_native_dynamic.py tools\zircon_export\tests\test_native_dynamic_build_signing.py tools\zircon_export\tests\native_dynamic_test_support.py
  - python -m py_compile tools\zircon_export\pipeline_report.py tools\zircon_export\pipeline_report_source_template.py tools\zircon_export\pipeline_report_source_template_stage_schema.py tools\zircon_export\tests\export_test_support.py tools\zircon_export\tests\test_pipeline_report_stage.py tools\zircon_export\tests\test_pipeline_report_stage_metadata.py tools\zircon_export\tests\test_pipeline_report_source_template.py tools\zircon_export\tests\test_pipeline_report_source_template_schema.py
  - python -m py_compile tools\zircon_export\pipeline_report.py tools\zircon_export\pipeline_report_stage_schema.py tools\zircon_export\tests\test_pipeline_report_stage_metadata.py tools\zircon_export\tests\test_pipeline_report_validate_schema.py tools\zircon_export\tests\test_pipeline_report_validate_native_dynamic_schema.py tools\zircon_export\tests\test_pipeline_report_validate_runtime_availability_schema.py
  - python -m py_compile tools\zircon_export\pipeline_report_stage_schema.py tools\zircon_export\tests\test_pipeline_report_stage_metadata.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_native_dynamic_schema tools.zircon_export.tests.test_pipeline_report_validate_runtime_availability_schema
  - python -m py_compile tools\zircon_export\pipeline_report_native_dynamic_payload.py tools\zircon_export\pipeline_report_native_dynamic_payload_schema.py tools\zircon_export\pipeline_report_native_dynamic_package_report_schema.py tools\zircon_export\tests\test_pipeline_report_native_dynamic_payload_schema.py tools\zircon_export\tests\test_pipeline_report_native_dynamic_package_report_schema.py
  - python -m py_compile tools\zircon_export\native_dynamic_payload.py tools\zircon_export\pipeline_report_native_dynamic_payload.py tools\zircon_export\pipeline_report_native_dynamic_payload_schema.py tools\zircon_export\pipeline_report_native_dynamic_package_report_schema.py tools\zircon_export\pipeline_report_stage_schema.py tools\zircon_export\tests\test_native_dynamic.py tools\zircon_export\tests\test_pipeline_report_stage_metadata.py tools\zircon_export\tests\test_pipeline_report_native_dynamic_payload.py tools\zircon_export\tests\test_platform_bundle_native_dynamic_operation_audit.py
  - python -m unittest tools.zircon_export.tests.test_native_dynamic tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_platform_bundle_native_dynamic_operation_audit
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_platform_bundle tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_package_report_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_platform_bundle tools.zircon_export.tests.test_pipeline_report_platform_bundle_manifest_schema tools.zircon_export.tests.test_pipeline_report_platform_bundle_file_reads tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_package_report_schema
  - python -m py_compile tools\zircon_export\pipeline_report_native_dynamic_loader_manifest.py tools\zircon_export\pipeline_report_native_dynamic_payload.py tools\zircon_export\tests\test_platform_bundle_native_payload_loader_manifest.py
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest
  - python -m unittest tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest tools.zircon_export.tests.test_pipeline_report_platform_bundle_manifest_schema tools.zircon_export.tests.test_pipeline_report_platform_bundle tools.zircon_export.tests.test_platform_bundle_native_dynamic
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_materialized_packages tools.zircon_export.tests.test_pipeline_report_stage_location tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest tools.zircon_export.tests.test_pipeline_report_platform_bundle_manifest_schema tools.zircon_export.tests.test_pipeline_report_platform_bundle tools.zircon_export.tests.test_platform_bundle_native_dynamic
  - python -m unittest discover tools.zircon_export.tests
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_materialized_packages
  - python -m py_compile tools\zircon_export\tests\test_pipeline_report_stage.py tools\zircon_export\tests\test_pipeline_report_stage_metadata.py tools\zircon_export\tests\test_pipeline_report_source_template.py
  - python -m py_compile tools\zircon_export\compile_host.py tools\zircon_export\source_template.py tools\zircon_export\tests\test_compile_host_source_template.py
  - python -m py_compile tools\zircon_export\cli.py tools\zircon_export\compile_host.py tools\zircon_export\tests\export_test_support.py tools\zircon_export\tests\test_compile_host_source_template.py tools\zircon_export\tests\test_compile_host_output_gate.py tools\zircon_export\tests\test_compile_host_path_resolve_errors.py tools\zircon_export\tests\test_stage_directory_errors.py tools\zircon_export\tests\test_subprocess_launch_errors.py
  - python -m unittest tools.zircon_export.tests.test_compile_host_source_template tools.zircon_export.tests.test_compile_host_output_gate tools.zircon_export.tests.test_compile_host_path_resolve_errors tools.zircon_export.tests.test_stage_directory_errors tools.zircon_export.tests.test_subprocess_launch_errors tools.zircon_export.tests.test_pipeline_resume_flow tools.zircon_export.tests.test_pipeline_report_stage_metadata
  - python -m py_compile tools\zircon_export\pipeline_report_stage_schema.py tools\zircon_export\pipeline_report_source_template.py tools\zircon_export\pipeline_report_source_template_stage_schema.py tools\zircon_export\tests\test_pipeline_report_source_template_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_source_template_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_pack_stage_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema tools.zircon_export.tests.test_pipeline_report_source_template_schema
  - python -m unittest tools.zircon_export.tests.test_compile_host_source_template tools.zircon_export.tests.test_pipeline_report_source_template tools.zircon_export.tests.test_pipeline_report_source_template_build_validation tools.zircon_export.tests.test_pipeline_report_source_template_schema tools.zircon_export.tests.test_pipeline_report_stage tools.zircon_export.tests.test_pipeline_resume_flow
  - python -m py_compile tools\zircon_export\pipeline_report_platform_bundle.py tools\zircon_export\pipeline_report_platform_bundle_schema.py tools\zircon_export\pipeline_report_platform_bundle_template.py tools\zircon_export\tests\test_pipeline_report_platform_bundle_manifest_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_platform_bundle_manifest_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_platform_bundle tools.zircon_export.tests.test_pipeline_report_platform_bundle_manifest_schema tools.zircon_export.tests.test_pipeline_report_platform_bundle_file_reads tools.zircon_export.tests.test_pipeline_report_native_dynamic_payload
  - python -m unittest discover tools.zircon_export.tests
  - python -m py_compile tools\zircon_export\pipeline_report.py tools\zircon_export\pipeline_report_stage_location.py tools\zircon_export\tests\test_pipeline_report_stage_location.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_location
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_validate_schema tools.zircon_export.tests.test_pipeline_report_validate_profile_summary_schema tools.zircon_export.tests.test_pipeline_report_validate_selection_schema
  - python -m py_compile tools\zircon_export\pipeline_report.py tools\zircon_export\pipeline_report_stage_location.py tools\zircon_export\pipeline_report_stage_schema.py tools\zircon_export\pipeline_report_pack_stage_schema.py tools\zircon_export\pipeline_report_pack_manifest_schema.py tools\zircon_export\pipeline_report_pack_delta_schema.py tools\zircon_export\tests\test_pipeline_report_pack_stage_schema.py
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_pack_stage_schema
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_stage_metadata tools.zircon_export.tests.test_pipeline_report_pack_stage_schema tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_schema
  - python -m unittest tools.zircon_export.tests.test_cook_assets_pack_stage tools.zircon_export.tests.test_pack_subprocess_failures tools.zircon_export.tests.test_pipeline_report_stage
  - python -m unittest discover tools.zircon_export.tests
  - python -m py_compile tools\zircon_export\stage_handoff.py tools\zircon_export\pipeline_report.py tools\zircon_export\pipeline_stages.py tools\zircon_export\platform_bundle.py tools\zircon_export\tests\test_stage_handoff.py tools\zircon_export\tests\test_pipeline_report_stage.py tools\zircon_export\tests\test_pipeline_resume_flow.py tools\zircon_export\tests\test_platform_bundle_inputs.py
  - python -m tools.zircon_export --profile windows-release --out D:\zircon-export-platform-manifest-boundary-smoke-0615\out --stage report --pretty
  - test_native_dynamic_stage_writes_package_export_report
  - test_native_dynamic_stage_materializes_package_and_loader_manifest
  - test_native_dynamic_stage_reports_materialized_file_manifest
  - test_native_dynamic_package_report_records_package_payload_hash
  - test_native_dynamic_stage_removes_stale_unselected_packages
  - test_native_dynamic_stage_filters_artifacts_by_target_platform
  - test_native_dynamic_stage_requires_platform_loadable_artifact
  - test_native_dynamic_stage_copies_macos_dsym_bundle
  - test_native_dynamic_stage_reports_package_loadable_artifacts
  - test_native_dynamic_payload_summary_keeps_loadable_artifact_audit
  - test_native_dynamic_payload_summary_rejects_malformed_package_audit
  - test_native_dynamic_payload_summary_rejects_loadable_artifact_not_in_manifest
  - test_native_dynamic_payload_summary_rejects_reported_plugins_dir_resolve_error
  - test_native_dynamic_payload_summary_rejects_current_plugins_dir_resolve_error
  - test_native_dynamic_stage_reports_native_cdylib_build_plan
  - test_native_dynamic_build_plan_records_cargo_features
  - test_native_dynamic_build_plan_rejects_target_dir_resolve_error
  - test_native_dynamic_build_plan_rejects_workspace_manifest_resolve_error
  - test_native_dynamic_build_plan_rejects_member_manifest_resolve_error
  - test_native_dynamic_build_plan_uses_resolved_workspace_manifest_in_command
  - test_native_dynamic_build_plan_rejects_workspace_manifest_directory
  - test_native_dynamic_build_plan_rejects_crate_manifest_directory
  - test_native_dynamic_build_rejects_expected_artifact_resolve_error
  - test_native_dynamic_build_executes_plan_and_stages_cdylib
  - test_native_dynamic_signs_loadable_artifact_before_manifest_hash
  - test_native_dynamic_notarization_runs_after_signing_before_manifest_hash
  - test_native_dynamic_notarization_profile_rejects_platform_mismatch
  - test_native_dynamic_signing_failure_cleans_staged_payload
  - test_native_dynamic_stage_removes_partial_package_on_artifact_filter_fatal
  - test_native_dynamic_stage_removes_all_packages_when_any_package_fails
  - test_native_dynamic_stage_rejects_inconsistent_package_paths
  - test_native_dynamic_stage_rejects_inconsistent_package_report_path
  - test_native_dynamic_stage_derives_missing_package_report_path
  - test_native_dynamic_stage_accepts_sanitized_package_directory
  - test_native_dynamic_stage_rejects_package_directory_id_mismatch
  - test_native_dynamic_stage_rejects_duplicate_package_ids
  - test_native_dynamic_stage_rejects_source_manifest_id_mismatch
  - test_native_dynamic_stage_rejects_source_manifest_parse_error
  - test_native_dynamic_stage_rejects_source_manifest_directory
  - test_native_dynamic_stage_rejects_source_manifest_missing_id
  - test_native_dynamic_stage_rejects_duplicate_recursive_package_sources
  - test_native_dynamic_stage_rejects_non_v3_abi_version
  - test_native_dynamic_stage_rejects_wrong_v3_descriptor_symbol
  - test_native_dynamic_stage_rejects_unselected_package_export
  - test_native_dynamic_stage_rejects_duplicate_selected_package_ids
  - test_native_dynamic_stage_rejects_missing_selected_package_export
  - test_native_dynamic_stage_reports_missing_package_source_fatal
  - test_template_rejects_declared_directory_file
  - test_platform_bundle_copies_native_dynamic_plugins_dir
  - test_platform_bundle_rejects_host_directory_input
  - test_platform_bundle_rejects_pack_directory_input
  - test_platform_bundle_rejects_delta_pack_directory_input
  - test_pipeline_platform_bundle_uses_native_dynamic_report_plugins
  - test_pipeline_platform_bundle_rejects_inherited_native_dynamic_report_directory
  - test_pipeline_platform_bundle_rejects_profile_mismatch_native_dynamic_report
  - test_pipeline_platform_bundle_rejects_invalid_native_dynamic_metadata
  - test_platform_bundle_rejects_invalid_validate_metadata_for_strategy
  - test_platform_bundle_explicit_native_dir_rejects_invalid_validate_metadata
  - test_platform_bundle_explicit_native_dir_rejects_payload_rewrite_resolve_error
  - test_platform_bundle_rejects_native_dynamic_package_report_directory
  - test_pipeline_platform_bundle_requires_native_dynamic_payload_for_native_dynamic_profile
  - test_pipeline_platform_bundle_rejects_invalid_compile_host_report_host_field
  - test_pipeline_platform_bundle_rejects_compile_host_report_host_resolve_error
  - test_pipeline_platform_bundle_rejects_invalid_pack_report_pack_field
  - test_pipeline_platform_bundle_preserves_native_dynamic_payload_hash
  - test_pipeline_platform_bundle_rejects_native_payload_destination_summary_resolve_error
  - test_platform_bundle_rejects_repo_root_resolve_error
  - test_platform_bundle_rejects_template_plugins_filter_resolve_error
  - test_pipeline_platform_bundle_rejects_stale_native_dynamic_payload_hash
  - test_pipeline_platform_bundle_rejects_malformed_native_dynamic_signing_audit
  - test_pipeline_platform_bundle_rejects_native_dynamic_signing_package_count_mismatch
  - test_pipeline_platform_bundle_accepts_disabled_native_dynamic_signing_placeholder
  - test_pipeline_platform_bundle_rejects_fatal_native_dynamic_signing_audit
  - test_pipeline_platform_bundle_rejects_disallowed_native_dynamic_signing_platform
  - test_pipeline_platform_bundle_rejects_spoofed_native_dynamic_signing_platform_allowed
  - test_pipeline_platform_bundle_rejects_malformed_native_dynamic_notarization_audit
  - test_report_rejects_missing_native_plugins_payload_signing_audit
  - test_report_rejects_missing_native_plugins_payload_notarization_audit
  - test_report_rejects_malformed_native_dynamic_report_signing_audit
  - test_report_rejects_malformed_native_dynamic_report_notarization_audit
  - test_report_rejects_native_dynamic_report_signing_package_count_mismatch
  - test_report_accepts_disabled_native_dynamic_report_signing_placeholder
  - test_report_rejects_fatal_native_dynamic_report_signing_audit
  - test_report_rejects_disallowed_native_dynamic_report_signing_platform
  - test_report_rejects_spoofed_native_dynamic_report_signing_platform_allowed
  - test_report_stage_projects_native_dynamic_release_audit
  - test_report_stage_requires_native_dynamic_for_native_dynamic_profile
  - test_pipeline_from_validate_uses_native_dynamic_profile_stages
  - test_pipeline_pack_rejects_invalid_cook_assets_report_manifest_field
  - native_dynamic_only_profile_carries_minimal_compile_host_plan
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-plugin-native-dynamic-host-plan-check-0615 --message-format short --color never
  - native_loader_loads_real_fixture_from_export_load_manifest_payload
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-plugin-native-dynamic-loader-manifest-check-0615 --message-format short --color never
  - cargo test -p zircon_runtime --lib native_loader_loads_real_fixture_from_export_load_manifest_payload --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-plugin-native-dynamic-loader-manifest-check-0615 --message-format short --color never -- --exact --test-threads=1 --nocapture
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --resume-from native_dynamic --dry-run
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --stage native_dynamic
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --stage report --pretty
  - python -m tools.zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --resume-from native_dynamic --dry-run
  - python -m tools.zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --stage native_dynamic
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (file_manifest/content_hash smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (native_dynamic_package.toml payload smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (stale package cleanup smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (target-platform artifact filtering smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (loadable artifact gate smoke)
  - python -m tools.zircon_export --profile macos-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (macOS dSYM bundle copy smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (partial package cleanup smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (fatal stage atomic cleanup smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (inconsistent package path gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (inconsistent package_report gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (derived package_report gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (package directory/package_id gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate package_id gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (source manifest id mismatch smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (source manifest parse error smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate recursive source manifest smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (ABI version gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (ABI v3 contract value gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (selection/export consistency gate smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate selected package_id smoke)
  - python -m tools.zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (fatal package materialization leaves no loader manifest smoke)
  - python -m tools.zircon_export --profile native-dynamic-fixture-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-smoke-0615/out --stage native_dynamic --native-dynamic-build --offline --pretty
  - python -m tools.zircon_export --profile native-dynamic-fixture-v2-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-v2-smoke-0615/out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
  - python -m tools.zircon_export --profile native-dynamic-fixture-release-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-release-smoke-0615/out --stage native_dynamic --native-dynamic-build --offline --pretty
  - python -m tools.zircon_export --profile native-dynamic-fixture-release-v2-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-release-v2-smoke-0615/out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
  - python -m tools.zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --stage report --pretty
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-platform-native-plugins-smoke/out --resume-from platform_bundle
  - python -m tools.zircon_export --profile windows-release --out <temp>/out --resume-from platform_bundle (NativeDynamic payload hash smoke)
  - python -m tools.zircon_export --profile windows-release --out <temp>/out --resume-from platform_bundle (stale NativeDynamic payload hash smoke, expected exit code 2)
  - test_pipeline_platform_bundle_ignores_pack_report_without_profile
  - test_report_stage_uses_source_template_profile_requirements
  - test_report_stage_requires_source_template_for_source_template_profile
  - test_report_stage_ignores_profile_mismatch_validate_strategies
  - test_report_stage_rejects_unknown_validate_strategy_without_defaulting
  - test_report_stage_rejects_empty_validate_strategies_without_defaulting
  - test_report_stage_rejects_invalid_validate_strategies_without_defaulting
  - test_report_stage_rejects_invalid_validate_metadata_without_defaulting
  - test_report_stage_rejects_stage_report_without_profile
  - test_required_path_field_reports_stage_identity_mismatch
  - test_report_stage_rejects_stage_identity_mismatch
  - test_required_path_field_reports_invalid_fatal_metadata
  - test_report_stage_rejects_stage_report_without_boolean_fatal
  - test_required_path_field_reports_invalid_diagnostics_metadata
  - test_report_stage_rejects_stage_report_without_string_diagnostics
  - test_required_path_field_reports_path_resolve_error
  - test_optional_path_field_reports_path_resolve_error
  - test_pipeline_from_validate_uses_source_template_profile_stages
  - test_pipeline_from_validate_rejects_invalid_validate_metadata_without_defaulting
  - test_pipeline_from_validate_rejects_unknown_strategy_without_defaulting
  - test_pipeline_from_validate_rejects_empty_strategies_without_defaulting
  - test_pipeline_from_validate_rejects_invalid_strategies_without_defaulting
  - test_resume_from_invalid_validate_metadata_does_not_use_fallback_stages
  - test_resume_from_validate_report_directory_does_not_use_fallback_stages
  - test_validate_strategy_helpers_reject_report_directory
  - test_validate_rejects_repo_root_resolve_error
  - test_validate_rejects_project_resolve_error
  - test_validate_rejects_validator_resolve_error
  - test_validate_rejects_target_dir_resolve_error
  - test_source_template_rejects_repo_root_resolve_error
  - test_source_template_rejects_validate_report_resolve_error
  - test_source_template_rejects_target_dir_resolve_error
  - test_compile_host_dry_run_rejects_invalid_validate_metadata
  - test_compile_host_reports_repo_root_resolve_error
  - test_compile_host_reports_validate_report_resolve_error
  - test_source_template_stage_rejects_invalid_validate_metadata
  - test_native_dynamic_stage_rejects_invalid_validate_metadata
  - test_report_rejects_source_template_generated_file_read_error
  - test_report_rejects_platform_host_output_read_error
  - test_report_rejects_platform_host_source_read_error
  - test_report_rejects_template_file_read_error
  - test_template_rejects_declared_file_read_error
  - test_native_dynamic_stage_rejects_package_payload_read_error
  - test_report_rejects_native_plugins_payload_file_read_error
  - test_platform_bundle_rejects_native_plugins_payload_source_resolve_error
  - test_native_dynamic_payload_bundle_manifest_rejects_source_resolve_error
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-source-template-path-smoke --resume-from source_template --dry-run
  - test_cook_assets_reports_asset_manifest_resolve_error
  - test_cook_assets_reports_project_manifest_resolve_error
  - test_pack_reports_missing_asset_manifest_before_packer
  - test_pack_reports_failed_packer_without_stage_report
  - test_pack_rejects_repo_root_resolve_error
  - test_pack_rejects_asset_manifest_resolve_error
  - test_pack_rejects_pack_file_resolve_error
  - test_pack_rejects_packer_resolve_error
  - test_pack_rejects_target_dir_resolve_error
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-pack-missing-manifest-smoke --stage pack --pretty (expected exit code 2)
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-resume-smoke --resume-from pack --dry-run
  - python -m tools.zircon_export --profile windows-release --project zircon-project.toml --out D:/zircon-export-m1-smoke --stage validate --dry-run --offline --target-dir D:/cargo-targets/zircon-export-validate-cli-0614
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-compile-host-dryrun --stage compile_host --dry-run --offline
  - test_compile_host_reports_target_dir_resolve_error
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-source-template-dryrun --stage source_template --dry-run --offline
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-m2-smoke --stage pack --asset-manifest D:/zircon-export-m2-smoke/assets/assets.json --determinism-check --offline --target-dir D:/cargo-targets/zircon-export-m1-validate-0614
  - cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-export-pack-profile-0615 --message-format short --color never
  - cargo test -p zircon_runtime --bin zircon_export_pack run_rejects_missing_dependency_without_writing_pack
  - cargo test -p zircon_runtime --bin zircon_export_pack run_rejects_duplicate_trim_input_without_writing_pack
  - cargo test -p zircon_runtime --bin zircon_export_pack run_reports_missing_asset_source_without_writing_pack
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-pack-profile-smoke/out --stage pack --asset-manifest D:/zircon-export-pack-profile-smoke/assets/assets.json --target-dir D:/cargo-targets/zircon-export-pack-profile-0615 --offline --pretty
  - python -m tools.zircon_export --profile windows-release --project D:/zircon-export-cook-project-smoke/project/zircon-project.toml --out D:/zircon-export-cook-project-smoke/out --stage cook_assets --asset-filter shipping --pretty
  - python -m tools.zircon_export --profile windows-release --out D:/zircon-export-m3-template-smoke --stage platform_bundle --pack-file D:/zircon-export-m3-template-smoke/inputs/assets.zrpack --template-dir tools/zircon_export/export-templates/windows-x86_64-library_embed-debug --target-platform windows-x86_64
  - python -m tools.zircon_export --profile linux-release --out D:/zircon-export-template-root-smoke --stage platform_bundle --pack-file D:/zircon-export-template-root-smoke/inputs/assets.zrpack --template-root tools/zircon_export/export-templates --target-platform linux-x86_64
  - validate_report_summarizes_profile_plan_and_fatal_state
  - native_dynamic_generates_loader_manifest_without_source_template
  - validate_report_exposes_native_dynamic_abi_v3_package_exports
  - loader_manifest_deserializes_abi_v3_contract_fields
  - native_dynamic_materialization_copies_runtime_package_without_source_crates
  - delta_pack_contains_only_changed_chunks
  - test_pack_delta_args_are_forwarded_to_packer
  - delta_pack_applies_to_base_pack
  - delta_pack_rejects_wrong_base_manifest
  - test_pack_command_forwards_profile_to_packer
  - test_pipeline_pack_uses_cook_assets_report_manifest
  - test_pipeline_cook_assets_uses_validate_report_asset_filter
  - test_pipeline_cook_assets_rejects_invalid_validate_report_asset_filter
  - test_stage_cook_assets_rejects_invalid_validate_metadata
  - test_cook_assets_preserves_manifest_asset_filter_over_pipeline_default
  - test_cook_assets_rejects_unknown_manifest_fields
  - test_cook_assets_rejects_manifest_missing_references
  - test_cook_assets_rejects_whitespace_explicit_asset_filter
  - test_pipeline_cook_assets_preserves_whitespace_explicit_asset_filter_gate
  - test_cook_assets_dry_run_rejects_whitespace_explicit_asset_filter
  - test_cook_assets_rejects_whitespace_explicit_path_arguments
  - test_cook_assets_dry_run_rejects_whitespace_explicit_path_arguments
  - test_cook_assets_rejects_blank_manifest_asset_filter
  - test_cook_assets_rejects_blank_manifest_path_array_entries
  - test_cook_assets_rejects_non_string_manifest_string_array_entry_before_array_shape
  - test_cook_assets_rejects_blank_manifest_source_when_present
  - test_cook_assets_rejects_unsafe_manifest_asset_paths
  - test_cook_assets_normalizes_explicit_manifest_package_paths
  - test_cook_assets_rejects_duplicate_manifest_paths_after_normalization
  - test_cook_assets_normalizes_manifest_filter_and_labels
  - test_cook_assets_normalizes_manifest_source_when_present
  - test_report_stage_rejects_cook_assets_manifest_shape_mismatch
  - test_cook_assets_derives_project_default_scene_without_manifest
  - test_cook_assets_project_fallback_records_direct_res_asset_references
  - test_cook_assets_project_fallback_records_recursive_direct_references
  - test_cook_assets_project_fallback_treats_binary_reference_as_leaf
  - test_cook_assets_project_fallback_orders_assets_and_dependencies_deterministically
  - test_cook_assets_project_fallback_rejects_default_scene_empty_path_segment
  - test_cook_assets_project_fallback_rejects_direct_reference_empty_path_segment
  - test_cook_assets_project_fallback_rejects_missing_direct_reference
  - test_cook_assets_project_fallback_rejects_unsafe_direct_reference
  - test_cook_assets_reports_missing_project_default_scene_source
  - deterministic_pack_double_run_byte_identical
  - template_version_mismatch_rejected
  - test_template_rejects_aliasing_file_and_host_paths
  - test_omitting_stage_runs_main_pipeline_from_validate
  - test_pipeline_platform_bundle_uses_compile_host_report_host
  - test_pipeline_platform_bundle_rejects_compile_host_report_host_resolve_error
  - test_pipeline_platform_bundle_uses_pack_report_pack_path
  - test_pipeline_platform_bundle_rejects_invalid_pack_report_delta_pack_field
  - test_pipeline_platform_bundle_uses_pack_report_delta_pack_path
  - test_template_delta_pack_path_controls_bundle_location
  - test_checked_in_windows_template_routes_delta_pack_path
  - test_template_root_skips_malformed_template_manifest
  - test_template_root_rejects_workspace_manifest_directory
  - test_platform_bundle_failure_cleans_previous_profile_bundle
  - test_report_stage_rejects_unverified_delta_pack
  - test_report_stage_rejects_invalid_pack_delta_pack_field
  - test_report_stage_rejects_platform_delta_without_pack_verification
  - test_report_stage_rejects_platform_delta_without_source
  - test_report_stage_rejects_platform_delta_source_mismatch
  - test_report_stage_rejects_platform_pack_without_source
  - test_report_stage_rejects_platform_pack_source_mismatch
  - test_report_stage_rejects_platform_host_without_source
  - test_report_stage_rejects_platform_host_source_mismatch
  - test_report_stage_allows_platform_argument_host_source
  - test_report_stage_rejects_platform_bundle_without_manifest
  - test_report_stage_rejects_platform_bundle_manifest_missing_file
  - test_report_stage_rejects_platform_bundle_manifest_host_mismatch
  - test_report_rejects_platform_bundle_without_bundle_root
  - test_report_rejects_platform_bundle_root_outside_current_output
  - test_report_rejects_bundle_manifest_outside_bundle_root
  - test_report_rejects_host_output_outside_bundle_root
  - test_report_rejects_pack_output_outside_bundle_root
  - test_report_rejects_template_file_output_outside_bundle_root
  - test_report_rejects_payload_containment_path_resolve_error
  - test_report_rejects_payload_containment_parent_resolve_error
  - test_report_stage_rejects_missing_platform_bundle_host_output
  - test_report_stage_rejects_missing_platform_bundle_pack_output
  - test_report_stage_rejects_missing_platform_bundle_delta_output
  - test_report_rejects_platform_host_output_hash_mismatch
  - test_report_rejects_platform_pack_output_hash_mismatch
  - test_report_rejects_platform_delta_output_hash_mismatch
  - test_report_rejects_missing_platform_host_source_file
  - test_report_rejects_missing_platform_pack_source_file
  - test_report_rejects_missing_platform_delta_source_file
  - test_report_rejects_stale_native_plugins_payload_hash
  - test_report_rejects_missing_native_plugins_payload_directory
  - test_report_rejects_native_plugins_payload_package_count_mismatch
  - test_report_rejects_native_plugins_payload_stage_report_mismatch
  - test_report_rejects_native_plugins_payload_source_mismatch_for_stage_payload
  - test_report_rejects_native_plugins_payload_bundle_path_mismatch
  - test_report_rejects_native_plugins_payload_package_destination_outside_plugins
  - test_report_rejects_native_plugins_payload_package_report_outside_package
  - test_report_rejects_missing_native_plugins_payload_package_report
  - test_report_rejects_native_plugins_payload_package_report_package_id_mismatch
  - test_report_rejects_native_plugins_payload_package_report_payload_count_mismatch
  - test_report_rejects_native_plugins_payload_package_report_directory_mismatch
  - test_report_rejects_native_plugins_payload_package_report_path_mismatch
  - test_report_rejects_native_plugins_payload_package_report_format_version_mismatch
  - test_report_rejects_native_plugins_payload_package_report_abi_version_mismatch
  - test_report_rejects_native_plugins_payload_loader_manifest_path_mismatch
  - test_report_rejects_native_plugins_payload_loader_manifest_manifest_mismatch
  - test_report_rejects_native_plugins_payload_loader_manifest_package_report_mismatch
  - test_report_rejects_native_plugins_payload_loader_manifest_abi_mismatch
  - test_report_rejects_native_plugins_payload_loader_manifest_bad_abi_table
  - test_report_rejects_native_plugins_payload_loader_manifest_unknown_abi_field
  - test_report_rejects_native_plugins_payload_loader_manifest_unknown_plugin_field
  - test_report_rejects_native_plugins_payload_loader_manifest_string_field_type
  - test_report_rejects_native_plugins_payload_loader_manifest_abi_field_types
  - test_report_rejects_native_plugins_payload_loader_manifest_abi_missing_required_field
  - test_report_rejects_native_plugins_payload_loader_manifest_unknown_top_level_field
  - test_report_rejects_stage_backed_native_plugins_payload_loader_manifest_missing_row_field
  - test_report_rejects_stage_backed_native_plugins_payload_loader_manifest_missing_abi_table
  - test_package_path_diagnostics_rejects_plugins_root_resolve_error
  - test_current_output_stage_report_path_reports_plugins_dir_resolve_error
  - test_package_path_diagnostics_rejects_destination_resolve_error
  - test_package_path_diagnostics_rejects_package_report_resolve_error
  - test_loadable_artifact_manifest_match_reports_destination_resolve_error
  - test_report_rejects_spoofed_native_plugins_payload_signing_audit
  - test_report_rejects_spoofed_native_plugins_payload_notarization_audit
  - test_report_accepts_current_native_plugins_payload
  - test_report_rejects_missing_template_file_output
  - test_report_rejects_template_file_hash_mismatch
  - test_report_accepts_current_template_file_output
  - test_report_rejects_template_file_destination_resolve_error
  - test_path_is_relative_to_treats_resolve_error_as_outside
  - test_report_bundle_path_rejects_stage_path_resolve_error
  - test_template_file_hashes_rejects_declared_path_resolve_error
  - test_template_file_expected_hash_rejects_source_resolve_error
  - test_pipeline_explicit_pack_file_does_not_inherit_report_delta
doc_type: workflow-detail
---

# Zircon Export Tool

`python -m tools.zircon_export` is the staged export pipeline entry point for project-level release
builds. M1 implemented `Validate`; M2 added the first executable asset `Pack` stage and a
`PlatformBundle` staging shell. M3-T1 adds the first `export-template` package contract and version
lock validation. The CLI now also has a `SourceTemplate` generated-project stage, a
`NativeDynamic` package-export report stage, a `CookAssets` handoff stage that normalizes cooked
asset manifests into the standard stage directory before `Pack` consumes them, and a final `Report`
stage that aggregates per-stage JSON into the release-level pipeline report. The main stage machine
can be resumed with `--resume-from <stage>` after a failed or interrupted export. Real host
compilation and full importer-driven asset cooking are still follow-up work. NativeDynamic now
reports the exact cdylib Cargo commands for selected package crates and can execute/copy those built
artifacts only when `--native-dynamic-build` is explicitly requested.

## Ownership

The Python package under `tools/zircon_export` owns process orchestration, output layout, resume
execution, and command construction. It does not duplicate plugin dependency validation or platform
policy checks. Those decisions stay in `zircon_runtime::plugin::export_build_plan` so editor UI,
CLI, CI, and future build stages all consume the same diagnostics.
`report_io.py` owns the final JSON report write boundary for Python-run stages. If a Python stage
cannot write its own `report.json`, the helper marks the in-memory report fatal, appends a typed
`... report ... could not be written` diagnostic, prints that JSON to stdout, and returns exit code
`2`. The final Report stage uses the same helper for both `<out>/stages/report/report.json` and
`<out>/report.json`; if the top-level pipeline report write fails after the stage report write
succeeds, the stage report is rewritten with the pipeline-report write diagnostic. If that rewrite
also fails, `report_io.py` removes the stale stage report and prints the newer in-memory report to
stdout so downstream consumers do not read a success-shaped stale `stages/report/report.json`.
`command_plan.py` centralizes command vector option rewriting and option-value diagnostics for
Python stages that consume trusted Validate build plans; CompileHost and SourceTemplate both use it
for wrapper-controlled Cargo options such as `--target-dir` and `--manifest-path`, and final Report
uses the same diagnostics when auditing SourceTemplate Validate build-plan evidence plus
SourceTemplate stage report command evidence. The shared diagnostic path rejects missing values,
option-looking values, and duplicate controlled options before any stage rewrites or trusts those
command vectors.
`tools.zircon_export.tests.test_command_plan` covers that shared helper directly, while the
stage/report tests cover each consuming boundary. Stage-side duplicate-option regressions are locked
by `test_compile_host_rejects_plan_with_duplicate_target_dir_option` for CompileHost and
`test_source_template_rejects_plan_with_duplicate_manifest_path_option` plus
`test_source_template_rejects_plan_with_duplicate_target_dir_option` for SourceTemplate.
`test_source_template_rejects_target_dir_option_with_option_value` keeps SourceTemplate's
`--target-dir` stage gate aligned with the existing manifest-path and CompileHost option-looking
value coverage.

`tools/zircon_export/__main__.py` is a thin top-level wrapper so the plan command works directly from the
repository root:

```powershell
python -m tools.zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export
```

Omitting `--stage` runs the main pipeline from `validate` through `report`. Passing `--stage
<stage>` keeps the single-stage debugging/CI behavior, and `--resume-from <stage>` starts the same
main pipeline at a later persisted stage.

## Validate Stage

The stage output is:

```text
<out>/
  stages/
    validate/
      report.json
```

When no prebuilt validator is supplied, the Python stage runs:

```text
cargo run -p zircon_runtime --bin zircon_export_validate --locked -- <validator-args>
```

The validator binary is deliberately small. It parses `--project`, `--profile`, optional
`--report`, optional `--stage-output`, and `--pretty`; then it loads `ProjectManifest` and calls
`ExportBuildPlan::from_project_manifest`. A profile or manifest failure still produces JSON with
`fatal = true` and a non-zero exit code, which lets CI and the future editor shell show the same
report shape for both success and failure.
Validate entry paths are canonicalized after the stage output path is known. Explicit `repo_root`
and `project` resolution failures now stay on the Validate diagnostics path, print
`command=<skipped>` during dry-run, and write a fatal Validate stage report during real execution
instead of aborting before `report.json` exists.
After launching the validator, the Python wrapper also requires
`<out>/stages/validate/report.json` to exist as a regular file. If the validator process exits
without writing that report, even with exit code `0`, the wrapper writes a fatal Validate report and
returns exit code `2` so the main pipeline cannot select downstream stages from missing Validate
evidence.

The M1 report contains:

- `stage = "Validate"`, `project_manifest`, `profile`, `stage_output`, `profile_found`, and
  `fatal`.
- `diagnostics` and `fatal_diagnostics` from the effective build plan.
- `profile_summary` with target mode, target platform, build mode, strategies, selected plugins,
  selected features, and asset filter.
- `plan_summary` with enabled runtime plugins, linked runtime crates, native dynamic packages,
  NativeDynamic ABI v3 package exports, generated-file metadata and contents, the LibraryEmbed
  `CompileHost` plan when present, the SourceTemplate generated-project build plan when present, and
  runtime plugin availability categories.

Runtime plugin availability primary buckets are mutually exclusive publish evidence. Final Report
rejects a schema-clean plugin id if it appears more than once across `available`, `linked`,
`native_dynamic`, `externalized_missing`, `stub`, `blocked_by_target`, or `blocked_by_maturity`, so
downstream strategy selection cannot consume contradictory availability states for the same runtime
plugin. `missing_required` is the Rust-side overlay summary for required unavailable plugins: it may
repeat an entry from `externalized_missing`, `stub`, `blocked_by_target`, or `blocked_by_maturity`,
but a standalone `missing_required` row is not publishable.

Successful non-fatal Validate reports are now a required release-evidence shape:
`project_manifest`, `stage_output`, `profile_found`, `fatal_diagnostics`, `profile_summary`, and
`plan_summary` must be present before final Report trusts the stage as publishable. Historical
standalone/debug reports may still omit `profile_summary.strategies` for strategy inspection
compatibility, but the `profile_summary` object itself is required on non-fatal Validate reports.
The selected plugin list remains part of that trusted profile evidence: each
`profile_summary.selected_plugins[]` entry must be a valid project plugin id and duplicate plugin
ids are rejected before later stage selection or feature/link-plan reconciliation can treat repeated
selection as distinct release evidence. `test_report_stage_rejects_validate_selected_plugins_duplicate`
locks the duplicate gate on final Report aggregation.
The feature selection map applies the same rule per plugin: feature ids must remain valid
`<plugin>.<feature>` names for their owner and duplicate entries within
`profile_summary.features.<plugin>[]` are fatal. `test_report_stage_rejects_validate_profile_feature_ids_duplicate`
keeps repeated feature evidence from becoming a duplicated crate/feature projection later in the
export pipeline.
The owner plugin id is now the schema-clean gate for that feature list: final Report only evaluates
feature namespace ownership and duplicate entries after the `profile_summary.features` map key has
passed the complete project plugin id schema. `test_report_stage_rejects_validate_profile_feature_invalid_plugin_id_before_feature_uniqueness`
keeps malformed owner ids such as `Rendering` at the plugin-id diagnostics instead of also emitting
derived `features.<plugin>[index] duplicates entry ...` noise.
The reported `stage_output` must resolve to the current Validate stage directory derived from the
loaded `<out>/stages/validate/report.json` path. `test_report_stage_rejects_validate_stage_output_outside_current_stage`
keeps stale or external Validate output directories from becoming the selected pipeline plan source.

For M2-T1, the `CompileHost` plan is included in the Validate report. The CLI now has an executable
`compile_host` stage that consumes this report, rewrites the planned target directory beneath the
current export output, appends `--locked` by default, passes through `--offline`, and records the
selected command and expected host executable in `<out>/stages/compile_host/report.json`. When
`--target-dir` is supplied, both the Cargo command and reported `host_executable` are derived from
that explicit directory so PlatformBundle consumes the same host path Cargo produced.
CompileHost stage execution now lives in `tools/zircon_export/compile_host.py`; `cli.py` only
dispatches and re-exports `run_compile_host` and `compile_host_command` for compatibility with
existing callers.
The consumed plan must include non-empty `binary`, `cargo_profile`, and `command` fields before
CompileHost launches Cargo, because those values derive the host executable handoff path and the
Cargo process invocation.
The stage also validates the command identity before rewriting the executable token: the trusted
Validate plan must begin with `cargo build`, so a stale or polluted plan cannot be silently
converted from another executable into a Cargo build.
The planned command is then checked against the same Validate plan metadata before launch:
package, binary, `--no-default-features`, feature list, manifest path, target directory, and
release/debug flag must agree with the consumed `library_embed_compile_host` row.
It applies the same execution-time profile contract as final Report: `cargo_profile` must be
`debug` or `release`, `release` must be a boolean, and both fields must agree before any Cargo
process starts.
Final Report treats `cargo_profile` shape as the gate for those profile semantics: a padded or blank
profile value stops at the field-level trimmed-string diagnostic before `debug`/`release` membership,
`release` consistency, or Cargo `--release` flag checks run. `test_report_stage_rejects_validate_compile_host_padded_cargo_profile_before_profile_semantics`
keeps malformed profile evidence from producing secondary release-mode noise.
The standalone CompileHost stage consumes the same Validate plan with the same evidence ordering.
`test_compile_host_rejects_padded_cargo_profile_before_profile_semantics` locks the execution path so
a malformed plan does not launch Cargo and does not degrade into profile-membership noise.
The execution gate also requires the complete CompileHost evidence row before deriving command,
host path, or link-plan data: package, manifest path, target directory, feature arrays, expected
runtime plugin ids, and linked runtime crates must all be present.
Those string evidence fields must be strings first, then non-empty trimmed strings, and the
plan-local manifest/target paths must stay safe and relative, matching the final Report
release-evidence schema.
Feature arrays, expected runtime plugin ids, and linked runtime crate rows are also validated before
subprocess launch, so malformed arrays cannot collapse into an empty `link_plan` or bypass the
linked-crate release-evidence schema.
For both the trusted Validate plan and the produced CompileHost `link_plan`, `app_features` and
`runtime_features` must be non-empty trimmed string entries before command feature matching,
link-plan equality, and duplicate detection consume them. This keeps feature projection evidence
deterministic instead of allowing padded or repeated feature tokens to diverge between plan,
command, and stage report. `test_report_stage_rejects_validate_compile_host_padded_feature_entry`
and `test_report_stage_rejects_validate_compile_host_duplicate_feature_entry` cover the Validate
plan gate, while `test_report_stage_rejects_compile_host_link_plan_padded_feature_entry` and
`test_report_stage_rejects_compile_host_link_plan_duplicate_feature_entry` cover the CompileHost
stage-report gate.
The expected runtime plugin id list follows the same uniqueness contract through the shared project
plugin id helper: `library_embed_compile_host.expected_runtime_plugins[]` and CompileHost
`link_plan.expected_runtime_plugins[]` must be valid project plugin ids and must not repeat. The
Validate-side `test_report_stage_rejects_validate_compile_host_duplicate_expected_plugin` and the
CompileHost-side `test_report_stage_rejects_compile_host_link_plan_duplicate_expected_plugin` keep
that handoff deterministic.
After Cargo returns success, CompileHost now requires the expected host output to be a regular file;
a missing path or directory at that location makes the stage fatal before PlatformBundle can consume
the handoff.
`test_compile_host_reports_repo_root_resolve_error` keeps explicit engine-root canonicalization on
the stage-report path before CompileHost reads the Validate plan or launches Cargo.
`test_compile_host_rejects_plan_without_binary` keeps malformed host plans from reaching Cargo:
without a string `binary`, CompileHost writes a fatal report with no command and no
`host_executable`.
`test_compile_host_rejects_plan_without_cargo_profile` applies the same gate to `cargo_profile`, so
malformed Validate plans cannot silently fall back to the debug output directory.
`test_compile_host_rejects_plan_command_metadata_mismatch` keeps standalone CompileHost execution
aligned with the final Report command provenance rules: package/bin, feature list, manifest path,
target directory, and release/debug flags must all match the validated plan before subprocess
launch. The consumed plan command must include `--manifest-path`; the default success fixture now
publishes `--manifest-path Cargo.toml` so host builds do not implicitly depend on the process
working directory.
The consumed plan command must not carry `--locked`, `--offline`, or `--frozen`; those Cargo
lock/offline policy flags are owned by the CompileHost CLI wrapper through `--no-locked` and
`--offline`, so stale Validate reports cannot override the executor policy.
The consumed plan command must not carry Cargo `--target`; platform target selection belongs to the
export target descriptor, so LibraryEmbed host builds cannot silently switch target triples inside
the command vector.
Forbidden CompileHost command flags are matched both as standalone tokens and as Cargo `--flag=value`
tokens; `test_compile_host_rejects_plan_forbidden_command_equals_form` locks the execution boundary
for target, profile, and wrapper-policy overrides in equals form.
`test_compile_host_rejects_plan_non_string_evidence_field` rejects non-string package, manifest, and
target-dir evidence before command/link-plan derivation can treat those malformed values as
implicitly acceptable.
`test_compile_host_rejects_plan_malformed_array_evidence_field` keeps the execution gate aligned
with final Report link-plan rules: feature arrays must be string arrays without blank entries,
expected runtime plugins must use project plugin ids, and linked runtime crate rows must pass the
same object/schema checks before CompileHost can launch Cargo.
`test_compile_host_rejects_plan_duplicate_array_evidence_entry` applies the same pre-Cargo gate to
duplicate `app_features`, `runtime_features`, and `expected_runtime_plugins` entries, so direct
CompileHost execution cannot produce a stage report from a plan the final Report would later reject.
`test_compile_host_rejects_plan_with_padded_feature_entry` applies the same field-level trimmed
string rule to `app_features[]` and `runtime_features[]`, so padded feature evidence cannot reach
Cargo launch or later command feature matching.
`test_compile_host_rejects_plan_with_empty_command` requires the planned Cargo command to be a
non-empty string array, so CompileHost never launches a command made only from appended flags such as
`--locked` and `--target-dir`.
`test_compile_host_rejects_plan_with_blank_command_entry` applies the same rule to each command
element, rejecting blank entries before wrapper flags are appended.
`test_compile_host_rejects_plan_with_padded_command_entry` keeps padded command tokens on the same
pre-launch path with a field-level `CompileHost plan command[index]` diagnostic, before the broader
`cargo build` identity check can mask the schema issue.
`test_compile_host_rejects_plan_with_dangling_target_dir_option` also validates the planned
`--target-dir` option shape before rewrite, so wrapper target-dir injection cannot hide a malformed
Validate handoff command.
`test_compile_host_rejects_target_dir_option_with_option_value` keeps option-looking values such as
`--release` on the same fatal path.
`test_compile_host_rejects_directory_host_output` keeps the success output boundary strict: Cargo
exit code `0` with a directory at the expected executable path writes a fatal CompileHost report
instead of publishing a success-shaped `host_executable`.
`test_compile_host_reports_target_dir_resolve_error` keeps both explicit and default target-dir
canonicalization on the same boundary: a resolve failure records a fatal CompileHost diagnostic,
leaves `command` empty, and suppresses `host_executable` instead of aborting before report output.
`test_compile_host_reports_validate_report_resolve_error` applies the same boundary to explicit
`--validate-report` input: the path is resolved after the CompileHost report location is known, and
a canonicalization failure writes a fatal CompileHost report with `validate_report = null` instead of
raising `OSError` before report output.
CompileHost final Report stage-report schema now lives in
`tools/zircon_export/pipeline_report_compile_host_stage_schema.py`; the generic stage dispatcher
only registers CompileHost fields and delegates the command/host/exit-code/stdout/stderr typed
checks. Non-dry-run CompileHost execution captures Cargo stdout and stderr with
`subprocess.run(..., capture_output=True, text=True)` and writes them to the stage report as
`stdout_lines` and `stderr_lines`; launch failures keep both arrays empty because no process ran.
`tools/zircon_export/subprocess_output.py` owns the small shared line-splitting helper used by
CompileHost and SourceTemplate.
Successful non-fatal CompileHost reports are now a required release-evidence shape: `link_plan`
must be present as an object beside the Cargo command, host executable, exit code, and captured
stdout/stderr arrays. The stage schema also requires the successful `command` vector to be a
non-empty string array with no empty or whitespace-only entries, so final Report cannot accept a
hand-written CompileHost success report that claims a host build without an executable Cargo command.
Each `command[]` token must also already be a trimmed non-empty string before the Cargo identity and
option checks run; captured `stdout_lines` and `stderr_lines` remain plain string arrays because
real process output may legitimately carry formatting whitespace.
Inside `link_plan`, `app_features`, `runtime_features`,
`expected_runtime_plugins`, and `linked_runtime_crates` are required fields, so an empty or partial
link plan is not publishable. `test_report_stage_rejects_compile_host_missing_release_evidence_field`
keeps missing `link_plan` fatal, and
`test_report_stage_rejects_compile_host_link_plan_missing_evidence_field` keeps missing nested
evidence fatal at stage schema loading, before final Report can trust a host build. After those
schema gates pass, final Report compares the nested execution evidence against Validate's
`plan_summary.library_embed_compile_host` for the same four fields. A mismatch produces a
field-level diagnostic such as
`compile_host report link_plan.expected_runtime_plugins does not match validate report plan_summary.library_embed_compile_host.expected_runtime_plugins`.
`test_report_stage_rejects_compile_host_link_plan_validate_mismatch` keeps this semantic gate
separate from the missing-evidence gate. If the CompileHost wrapper is already fatal, final Report
treats that stage report as non-publishable evidence for these later handoffs: `link_plan` is not
compared with Validate, and PlatformBundle does not emit a secondary missing-host-evidence diagnostic
for `host_source_origin = "compile_host_report"`. `test_report_does_not_compare_fatal_compile_host_link_plan_with_validate`
keeps the field-level CompileHost schema diagnostic as the source of truth.
`test_report_stage_rejects_compile_host_empty_command`,
`test_report_stage_rejects_compile_host_blank_command_entry`, and
`test_report_stage_rejects_compile_host_padded_command_entry` lock the command-vector gate in a
focused CompileHost schema module instead of expanding the already-large stage metadata test file.
`test_report_stage_rejects_validate_compile_host_blank_string_array_entry` and
`test_report_stage_rejects_validate_compile_host_padded_command_entry` mirror the command-vector
gate on the Validate-side `plan_summary.library_embed_compile_host` arrays, so stale plan-side
`app_features`, `runtime_features`, or `command` rows fail before CompileHost link-plan matching and
Cargo command provenance checks.
`test_report_stage_rejects_validate_compile_host_command_metadata_mismatch` also keeps the
Validate-side Cargo command bound to the same plan row: package, binary, required manifest path,
target directory, and release/debug flags must match the published CompileHost plan metadata before
final Report accepts the plan as release evidence.
`test_report_stage_rejects_validate_compile_host_command_feature_mismatch` extends that provenance
gate to the feature cut: the command must retain `--no-default-features`, publish exactly one
`--features` option, and match the plan's `app_features` list after Cargo feature-list parsing.
`test_report_stage_rejects_validate_compile_host_command_wrapper_policy_override` keeps
Validate-side plans from publishing wrapper-owned lock/offline flags (`--locked`, `--offline`,
`--frozen`), matching the execution-time `test_compile_host_rejects_plan_command_wrapper_policy_override`.
`test_report_stage_rejects_validate_compile_host_command_target_triple_override` keeps
Validate-side plans from publishing Cargo `--target`, matching the execution-time
`test_compile_host_rejects_plan_command_target_triple_override`.
`test_report_stage_rejects_validate_compile_host_forbidden_command_equals_form` keeps the same
forbidden-flag policy from being bypassed through Cargo `--flag=value` syntax.
`test_report_stage_rejects_compile_host_command_missing_build_option` applies the same release
evidence stance to the CompileHost execution report: a successful report command must still be an
auditable `cargo build` invocation with package/bin, `--no-default-features`, `--features`, and
`--target-dir` options, so a hand-written `["cargo", "build"]` success report cannot satisfy the
final Report stage by shape alone.
`test_report_stage_rejects_compile_host_command_validate_mismatch` then binds the execution command
back to Validate plan semantics: package, binary, and Cargo feature list must match
`plan_summary.library_embed_compile_host.package`, `.binary`, and `.app_features[]` before final
Report treats the CompileHost evidence as publishable.
`test_report_stage_rejects_compile_host_command_release_flag_mismatch` covers the build-mode half
of the same binding: release plans require `--release` in the CompileHost execution command, while
debug plans reject an unexpected `--release` flag.

NativeDynamic-only profiles also receive the same minimal host plan through the existing
`plan_summary.library_embed_compile_host` field. The plan provides a CompileHost boundary for the
final bundle, while NativeDynamic packages remain loadable plugin payloads and are not linked into
the host's `linked_runtime_crates` list.

The stage intentionally depends on `Validate` output instead of re-running profile dependency logic
in Python. If the Validate report is fatal, missing, for another profile, or lacks a
`library_embed_compile_host` plan, CompileHost returns a fatal report before invoking Cargo.
CompileHost applies the shared Validate stage metadata contract before reading that plan: the report
must identify the `Validate` stage, `fatal` must be a boolean, `diagnostics` must be a string array,
and `profile` must match the requested profile. Dry-run prints those failures as `diagnostic=...`,
so malformed Validate metadata cannot be treated as a usable host build plan.
When the trusted Validate report explicitly carries `profile_summary.strategies`, CompileHost
first applies the shared strategy-list hard gate: the field must be a list, must include at least
one supported export strategy, and may only contain `source_template`, `library_embed`, or
`native_dynamic` aliases. It then requires that normalized list to include either `library_embed` or
`native_dynamic` before consuming `plan_summary.library_embed_compile_host`. This keeps
SourceTemplate-only profiles from using stale host build plans and keeps malformed or future
strategy metadata from falling through to stale CompileHost plan rows, while preserving
NativeDynamic's minimal-host path and historical standalone/debug reports that omit the `strategies`
field entirely.

## SourceTemplate Stage

`SourceTemplate` consumes the Validate report's generated-file rows and
`plan_summary.source_template_build` command. It materializes the generated Cargo project under:

```text
<out>/
  stages/
    source_template/
      project/
        Cargo.toml
        src/...
      report.json
```

Because the generated `Cargo.toml` is authored by the Rust build-plan templates, Python does not
reconstruct project dependencies. It only writes the generated files, rewrites local `zircon_*`
path dependencies from template-relative paths to absolute workspace paths for the current
`--repo-root`, records the validated cargo build command in `report.json`, and records the byte
size plus SHA-256 for each generated file after local dependency paths are rewritten.
Before reading generated files or build-plan rows, SourceTemplate applies the shared Validate stage
metadata contract: the Validate report must identify the `Validate` stage, `fatal` must be a
boolean, `diagnostics` must be a string array, and `profile` must match the requested profile.
Malformed Validate metadata makes SourceTemplate fatal and prevents project materialization.
When the trusted Validate report explicitly carries `profile_summary.strategies`, SourceTemplate
first applies the shared strategy-list hard gate: the field must be a list, must include at least
one supported export strategy, and may only contain `source_template`, `library_embed`, or
`native_dynamic` aliases. It then requires that normalized list to include `source_template` before
consuming `plan_summary.source_template_build`. This keeps standalone `--stage source_template`
from materializing a generated project for a LibraryEmbed-only profile and keeps malformed or
future strategy metadata from falling through to stale SourceTemplate plan rows. Historical
standalone/debug reports that omit the `strategies` field entirely keep the existing inspect
behavior.

Materialization diagnostics that make the generated project untrustworthy are fatal: invalid,
empty, or blank-entry build commands in `plan_summary.source_template_build`, duplicate,
invalid, or escaping generated-file paths, a build `manifest_path` that escapes the generated
project, generated-file rows without
contents, a missing generated `Cargo.toml`, a generated `Cargo.toml` path that is not a regular
file, an unreadable/unwritable generated `Cargo.toml`, or missing rewritten local `zircon_*`
dependency paths.
`test_source_template_stage_rejects_duplicate_generated_file_path` keeps standalone
SourceTemplate from creating `<out>/stages/source_template/project/` when
`plan_summary.generated_files` repeats the same path, matching the final Report duplicate-path
release gate.
The same pre-materialization gate reuses the final Report Validate generated-file row schema:
`plan_summary.generated_files` must be a list of object rows, each row may only contain `path`,
`purpose`, and `contents`, `path` and `purpose` must be trimmed non-empty strings, and `contents`
must be a string. `test_source_template_stage_rejects_invalid_generated_file_plan_rows` keeps bad
rows from producing a command or generated project that final Report would reject later.
Before constructing the rewritten Cargo command, standalone SourceTemplate also reuses the final
Report Validate build-plan schema for `plan_summary.source_template_build`: the field must be an
object containing only `manifest_path`, `target_dir`, `cargo_profile`, `release`, and `command`;
the path/profile fields must be trimmed non-empty strings, `release` must be a boolean, and
`command` must be a non-empty string array whose entries are already trimmed. Final Report applies
the same command evidence rule to the SourceTemplate stage report and nested
`build_validation.command`: every present command token must be non-empty and already trimmed before
manifest-path, target-dir, or report/build-validation command identity checks consume it.
`stdout_lines` and `stderr_lines` stay plain string arrays so real process output can preserve
formatting whitespace; if either field is not an array the diagnostic stays on the field, while
non-string array entries are reported as `stdout_lines[index] must be a string` or
`stderr_lines[index] must be a string` before final Report accepts build output evidence. Existing
direct command-shape diagnostics are preserved for option-value
errors, while `test_source_template_stage_rejects_invalid_build_plan_schema` keeps missing fields,
blank strings, malformed booleans, unknown fields, and blank command entries from producing a
command or generated project. `test_report_rejects_padded_source_template_validate_build_plan_command_entry`
and `test_source_template_rejects_plan_with_padded_command_entry` keep Validate build-plan command
tokens aligned across final Report and standalone SourceTemplate execution.
`test_source_template_rejects_plan_with_non_string_command_entry_before_array_shape` keeps the
standalone SourceTemplate execution path from adding the generic build-plan command fallback after
the Validate build-plan schema has already emitted `command[index] must be a string`.
`test_report_rejects_source_template_report_padded_command_entry` and
`test_report_rejects_source_template_build_validation_padded_command_entry` keep padded final Report
command tokens from being accepted as release evidence or hidden behind command-mismatch diagnostics.
The planned SourceTemplate command is also checked before rewrite: dangling `--manifest-path` or
`--target-dir` options, and option-looking values for those fields, are fatal instead of being
repaired by wrapper injection.
These diagnostics stop the optional build step before Cargo is invoked. The deliberate "build
validation skipped" diagnostic remains non-fatal when `--source-template-build` is not supplied.
Before each non-dry-run materialization, the stage recreates its owned
`<out>/stages/source_template/project/` directory while leaving the sibling Cargo target directory
alone. This prevents files removed from the current generated project plan from remaining beside the
new output after a retry; `test_source_template_stage_cleans_stale_generated_project_files` keeps a
stale `src/stale.rs` from surviving a later SourceTemplate run.
If materialization diagnostics or the optional Cargo build make the stage fatal, the same owned
`project/` directory is removed before `report.json` is written, so a partial generated project does
not remain beside a failed SourceTemplate report. The invalid generated-file regression asserts that
an escaped generated path leaves neither `project/` nor the escaped output behind.

By default this stage materializes the project and skips Cargo execution. Passing
`--source-template-build` executes the validated `cargo build --manifest-path <project>/Cargo.toml`
command, with `--locked` enabled by default and `--offline` forwarded when requested. This keeps the
stage usable during current workspace compile drift while preserving the real build-validation hook
for CI and later clean runs.
Every SourceTemplate report now carries a `build_validation` object with the requested/executed
booleans, status (`skipped`, `passed`, `failed`, or `blocked`), exit code, working directory, and
rewritten Cargo command. When the build is executed, the stage captures Cargo stdout and stderr as
`stdout_lines` and `stderr_lines`; final Report requires both fields to be arrays and rejects
non-string entries at their exact array index before it accepts the build validation evidence. This
keeps generated-project build failures auditable in the stage report instead of reducing them to an
exit code and one diagnostic.

When Validate report `profile_summary.strategies` contains `source_template`, the main pipeline now
includes this stage. A SourceTemplate-only profile runs `Validate -> SourceTemplate -> Report`; a
hybrid SourceTemplate + LibraryEmbed profile runs SourceTemplate first and then the LibraryEmbed
host/assets/bundle stages. This keeps the first-class `python -m tools.zircon_export --profile <name>`
entry point aligned with the profile path instead of requiring a manual `--stage source_template`
detour.

## NativeDynamic Package Manifest

Validate report keeps both native dynamic views: `native_dynamic_packages` is the compatibility
package-id list, and `native_dynamic_package_exports` is the structured ABI v3 package export table.
Each export row records the package id, output directory derived from the Rust build plan's
`package_id` sanitization rule, package path, package manifest path, and ABI v3 contract fields used
by the native loader/tooling boundary. The Python stage derives `package_report =
"<path>/native_dynamic_package.toml"` when the Validate report omits it, matching the current Rust
`NativeDynamicPackageExportPlan` shape; if a report supplies `package_report`, the stage validates it
against that derived path.

The Python implementation is split by publishing responsibility. `native_dynamic.py` owns the
NativeDynamic stage orchestration, package source discovery, staged package materialization, build,
signing and notarization sequencing. `native_dynamic_plan.py` owns Validate report loading, strategy
membership, selected package ids, package export rows, target-platform artifact extension selection,
and ABI/package-path consistency checks before materialization begins. `native_dynamic_contract.py`
owns the shared stage constants and ABI v3 expected contract strings. `native_dynamic_payload.py`
owns deterministic file manifests, content hashes, payload summary fallback/validation, and normalized
release-audit projections used by PlatformBundle and final Report. `native_dynamic_templates.py` owns
the loader manifest and package report TOML rendering. Downstream stages import payload helpers from
`native_dynamic_payload.py` instead of reaching through the stage runner.

The Rust build plan generates `plugins/native_plugins.toml` with `id`, `path`, `manifest`,
`package_report`, and `[plugins.abi]` for each native package. When native packages are materialized,
each copied package receives a `native_dynamic_package.toml` report with the same ABI v3 descriptor,
entry, host function table, behavior, snapshot, and bridge method table contract. The package report
also carries `[payload]` with a package-local file count, content hash, and `[[payload.files]]`
entries for the release-facing files copied into that package, excluding the generated package report
itself. Final Report treats the generated package-report header as required release evidence:
`format_version`, `package_id`, `directory`, `path`, `manifest`, `[abi]`, and `[payload]` must all
be present before package identity, payload, and ABI semantics are trusted.
Package report `payload.files[].path` uniqueness is also schema-clean: only trimmed non-empty path
evidence participates in duplicate detection. A padded payload file path stops at the field-level
trimmed-string diagnostic before final Report derives duplicate path or current package payload
drift diagnostics from the malformed row.
Loader manifest `[plugins.abi]` string contract fields follow the same schema-clean boundary.
Whitespace-only ABI strings are rejected as non-empty string failures, and padded ABI strings are
rejected as non-empty trimmed string failures before the loader manifest row is compared with
NativeDynamic `package_exports[]` or PlatformBundle materialized-package ABI evidence. This keeps
hand-authored `plugins/native_plugins.toml` ABI drift diagnostics tied to the field that is
malformed instead of also producing derived ABI mismatch noise.

The CLI `native_dynamic` stage consumes the Validate report, finds the selected native packages
under `<repo-root>/zircon_plugins` by matching `plugin.toml` ids, and writes:

```text
<out>/
  stages/
    native_dynamic/
      plugins/
        native_plugins.toml
        <package>/
          plugin.toml
          native/...
          resources/...
          native_dynamic_package.toml
      report.json
```

Before reading `native_dynamic_packages` or `native_dynamic_package_exports`, NativeDynamic applies
the shared Validate stage metadata contract: the report must identify the `Validate` stage, `fatal`
must be a boolean, `diagnostics` must be a string array, and `profile` must match the requested
profile. Malformed Validate metadata makes the stage fatal before package materialization, so an
untrusted Validate report cannot generate `plugins/native_plugins.toml`.

Before materializing packages, the stage recreates its owned `<out>/stages/native_dynamic/plugins/`
directory. This makes repeated exports deterministic: a package removed from the active profile
cannot remain in the staged payload, loader manifest, file manifest, or content hash from an earlier
run.

When the selected package maps directly to `<repo-root>/zircon_plugins/<package_id>/plugin.toml`,
that source manifest must be a regular file before TOML parsing. Directory or unreadable manifests
become fatal NativeDynamic diagnostics such as `direct manifest ... is not a file`, trigger the same
owned payload cleanup path, and do not fall through to the recursive package search or a generic
missing-manifest diagnostic.
When the direct source manifest has an `id`, that value is also schema evidence before package-id
matching: it must be a non-empty trimmed string, so padded ids stop at the field-level diagnostic
instead of being reported as a selected-package mismatch.
Non-string `id` values are kept distinct from missing ids and stop at `id must be a string`; this
matches the final Report source-manifest provenance check for generated NativeDynamic reports.
The recursive source search uses the same rule for nested package candidates. A malformed nested
`plugin.toml` id records a `source manifest ... id must be a non-empty trimmed string` diagnostic
instead of being swallowed and later reported as a generic missing package manifest.
Broken nested TOML follows the same ownership rule: recursive search keeps `source manifest ...
could not be parsed` as the actionable diagnostic instead of masking the corrupt manifest as a
missing selected package.

The stage also reads `profile_summary.target_platform` from the Validate report when present and
filters copied native artifacts by platform family. Windows packages copy `.dll` and `.pdb`, Linux
packages copy `.so` and `.dbg`, and macOS packages copy `.dylib` plus `.dSYM`/`.dsym` debug symbol
bundles. macOS debug bundles are copied recursively from `native/` and their nested files are listed
in the staged file manifest. If a legacy or unknown Validate report has no recognizable target
platform, the stage falls back to the full native artifact extension set so older reports remain
diagnosable instead of failing before the package shape can be inspected. Debug symbol files or
directories may accompany a package, but they do not satisfy the loadable library requirement by
themselves: each materialized package must include at least one `.dll`, `.so`, or `.dylib` selected by
the target platform.
Present target-platform evidence is stricter than an absent legacy value: if Validate publishes
`profile_summary.target_platform` or legacy `profile_summary.platform`, the NativeDynamic stage now
requires a non-empty trimmed known export target platform before artifact extension selection runs.
`test_native_dynamic_stage_rejects_padded_target_platform_before_artifact_selection` keeps padded
platform evidence from broadening artifact selection through the unknown-platform fallback.

The `native_dynamic_package_exports` table is validated before any package payload is materialized.
The selected package list and structured export table must match exactly: every
`native_dynamic_packages` id needs one export row, every export row must be selected, and selected
package ids may not repeat. Each `package_id` in the export table may also appear only once. Each
row must also be internally consistent: `directory` must equal the sanitized package id that the
Rust build plan would generate (`animation.fx` becomes `animation_fx`), and for
`package_id = "animation"` / `directory = "animation"` the stage accepts only `path = "plugins/animation"`,
`manifest = "plugins/animation/plugin.toml"`, and
derived or supplied `package_report = "plugins/animation/native_dynamic_package.toml"`. Unselected
export rows, missing export rows, duplicate `package_id` values, mismatched `directory`, or
mismatched `path`/`manifest`/`package_report` entries are fatal and prevent
`plugins/native_plugins.toml` from being written, so the loader manifest cannot point at a different
package location than the staged payload or contain rows that disagree with the profile's selected
plugin set.

NativeDynamic also treats the ABI v3 contract as a hard publishing boundary: each
`native_dynamic_package_exports` row must carry `abi.abi_version = 3`, and every ABI contract string
must match the Rust build-plan generator's fixed v3 values such as
`zircon_native_plugin_descriptor_v3`, `NativePluginAbiV3`, and
`NativePluginBridgeMethodTableV3`. Older ABI versions, future ABI versions, or mismatched v3 contract
names are fatal before package materialization, so an incompatible loader contract cannot be written
into `plugins/native_plugins.toml`.
Package-export path derivation also waits for schema-clean locator strings. Padded `directory`,
`path`, `manifest`, or `package_report` fields stay at their trimmed-string diagnostics;
`test_report_stage_rejects_native_dynamic_package_export_path_field_shape_before_path_semantics`
keeps malformed locator evidence from also emitting derived `plugins/...` path mismatch messages.
The NativeDynamic stage execution path now applies the same ordering before it consumes Validate
plan package exports: `package_id`, `directory`, `path`, `manifest`, present `package_report`, and
ABI v3 string fields must be non-empty trimmed strings before package-id uniqueness, directory/path
derivation, or ABI contract comparison runs. `test_native_dynamic_stage_rejects_padded_package_export_id_before_uniqueness`
locks that stage-entry gate so malformed Validate plan rows cannot create secondary package
selection noise before materialization is skipped.
The selected `native_dynamic_packages[]` list follows the same stage-entry contract:
`test_native_dynamic_stage_rejects_padded_selected_package_id_before_uniqueness` keeps padded selected
package ids at field-level diagnostics before duplicate detection or package-export closure checks
run. Non-string selected package ids are reported per entry as `native_dynamic_packages entry <index>
must be a string`; `test_native_dynamic_stage_rejects_non_string_selected_package_id_before_array_shape`
keeps one bad row from collapsing the whole list into a generic string-array failure.

The stage report records the Validate report path, `native_dynamic_packages`, the full
`native_dynamic_package_exports` table, `package_count`, `loader_manifest`, each materialized
package source/destination/report path, each package's stage-relative `loadable_artifacts` plus
`loadable_artifact_count`, and a `native_build_plan`. The build plan reads the selected source
package `plugin.toml` module crate names, matches them against `cdylib` members declared in
`<repo-root>/zircon_plugins/Cargo.toml`, derives the Cargo profile from Validate
`profile_summary.build_mode`, and records the target directory, exact `cargo build` command
(`--manifest-path`, `-p`, `--target-dir`, lock/offline/release flags), and platform-specific
expected loadable artifact path for every matched package. Final Report applies schema-clean order
to the stage report's top-level `artifact_extensions[]` and `native_dynamic_packages[]`: non-array
values keep the table-level string-array diagnostic, while non-string array entries report
`native_dynamic report <field>[index] must be a string` before blank, trimmed, duplicate,
selected-package, or Validate handoff semantics run. The default target directory remains
`<out>/stages/native_dynamic/target`, but an explicit `--target-dir` feeds the native cdylib build
plan, execution command, and expected loadable artifact path. Repeated
`--native-dynamic-build-feature <feature>` values are accepted only when every provided feature is a
non-empty trimmed string; invalid feature input stops the NativeDynamic stage before package
materialization or Cargo command construction. Schema-clean repeated features are deduplicated,
recorded in `native_build_plan.build_features` and each package plan's `features`, and appended to
the Cargo command as `--features <comma-separated features>`. In final Report schema validation,
both feature arrays may be empty, but non-string entries stop at
`native_build_plan.build_features[index] must be a string` or
`native_build_plan.packages[index].features[index] must be a string` before blank, trimmed,
duplicate, header-match, or Cargo command feature semantics run. Schema-clean entries must be
non-empty trimmed strings and unique.
Feature uniqueness is checked only after entries are schema-clean. Padded duplicate
`build_features[]` or package `features[]` values stop at the trimmed-string entry diagnostic
instead of also emitting duplicate-entry diagnostics or command feature mismatch noise.
The stage-level release arrays follow the same normalization boundary:
`test_report_stage_rejects_native_dynamic_string_array_fields_padded_entry` rejects padded
`artifact_extensions[]` and `native_dynamic_packages[]` entries before artifact-extension policy,
materialized package id closure, or Validate plan-summary matching consumes them.
They must also be unique. `test_report_stage_rejects_native_dynamic_string_array_fields_duplicate_entry`
keeps duplicate `artifact_extensions[]` or `native_dynamic_packages[]` rows at
`native_dynamic report <field>[index] duplicates entry ...`; duplicate selected-package rows no
longer continue into materialized-package or Validate package-set mismatch diagnostics.
Duplicate-index checks only consume schema-clean array entries, so padded top-level array values
stop at their trimmed-string diagnostics before the duplicate index is derived.
`test_report_stage_rejects_native_dynamic_schema_before_payload_semantics` extends that boundary to
the final Report stage-payload checks: schema-invalid `materialized_packages[].package_id` or
`native_dynamic_packages[]` rows stop at field-level diagnostics before loader-manifest,
Validate/package-export, selected-package, or build-plan package-id closure diagnostics run.
For PlatformBundle `native_plugins_payload.materialized_packages[]`, uniqueness checks also require
schema-clean identity and artifact entries: padded duplicate `package_id` or
`loadable_artifacts[]` values now stop at the trimmed-string diagnostic before duplicate-id or
duplicate-loadable-artifact diagnostics are derived.
Negative nested `native_build_execution.package_count` likewise stops at the non-negative and
execution-state diagnostics before package-id payload matching. The same focused gate covers
operation-audit rows: padded signing/notarization `packages[].package_id` or negative
`packages[].artifact_count` stop at operation-audit schema diagnostics before artifact-count,
package-id, or materialized-artifact closure runs. Padded operation-audit
`artifacts[].artifact` and `artifacts[].package_relative_artifact` now stop at their
field-level trimmed-string diagnostics before artifact path or package-relative artifact
closure is derived.
Native build-execution copied artifact paths use the same order. Padded
`copied_loadable_artifact` or `copied_sidecars[]` values stop at trimmed-string diagnostics before
safe-relative, package-directory, materialized loadable-artifact, or current file-manifest checks
consume those paths.
For non-fatal NativeDynamic reports, the top-level string release-evidence fields must be non-empty:
`content_hash`, `loader_manifest`, `native_plugin_root`, `plugins_dir`, `stage_output`,
`target_platform`, and `validate_report`. `test_report_stage_rejects_native_dynamic_empty_required_string_release_evidence_field`
keeps empty strings from bypassing stage location binding, source-root provenance, Validate
handoff, target-platform audit, or the direct content-hash field contract.
`test_report_stage_rejects_native_dynamic_padded_required_string_release_evidence_field` extends
that boundary to non-empty padded values, so these fields must be trimmed before path resolution,
content-hash comparison, stage directory binding, target-platform checks, or Validate handoff
consumes them.
The three current-location bindings reuse that trimmed-string boundary: `stage_output`,
`plugins_dir`, and `loader_manifest` are not canonicalized or compared against the current
NativeDynamic stage directory until their release-evidence fields are schema-clean.
`test_report_stage_rejects_native_dynamic_location_schema_before_location_semantics` keeps padded
location values at field-level diagnostics instead of adding current stage/plugins/loader-manifest
mismatch noise.
Top-level `package_count` follows the same schema-before-semantics rule for count evidence: final
Report only compares it with `materialized_packages[]` after the count is a non-negative integer.
`test_report_stage_rejects_native_dynamic_schema_before_payload_semantics` covers `package_count =
-1` so the report stays at `native_dynamic report package_count must be non-negative` instead of
also adding a count/list mismatch.
`content_hash` must also be a SHA-256 hex digest before any current staged `plugins/`
directory hash comparison runs. `test_report_stage_rejects_native_dynamic_content_hash_shape_before_payload_semantics`
keeps malformed digests at `native_dynamic report content_hash must be a SHA-256 hex digest`
instead of allowing them to fall through to a stale/current directory mismatch diagnostic.
When PlatformBundle reuses a stage-backed NativeDynamic payload summary, the stage report
`plugins_dir` and the caller's current `plugins_dir` are canonicalized separately, so
`test_native_dynamic_payload_summary_rejects_reported_plugins_dir_resolve_error` and
`test_native_dynamic_payload_summary_rejects_current_plugins_dir_resolve_error` preserve which side
failed instead of collapsing both cases into a generic report-field diagnostic.
Legacy stage-backed payload reads, including Validate reports that predate `profile_summary.strategies`,
use the same NativeDynamic report hash schema before comparing PlatformBundle's copied
`native_plugins_payload.content_hash` against the stage report; the legacy schema-before-payload
coverage now includes `content_hash = "not-a-hash"`.
`native_plugin_root` is also a schema gate before stage materialized-package source-root
semantics run. When the root is padded or otherwise not a trimmed non-empty string,
`native_dynamic_package_report_diagnostics(...)` leaves it at the release-evidence field
diagnostic and does not compare `materialized_packages[].source` against a path containing
the invalid root text.
Final Report now treats `native_build_plan` and `native_build_execution` as typed NativeDynamic
stage evidence instead of opaque objects. The stage loader closes build-plan top-level fields,
package command rows, build-execution top-level fields, and per-package execution rows before
PlatformBundle or Report trusts build commands, expected artifacts, stdout/stderr, copied sidecars,
or package counts. Wrong-typed entries produce diagnostics such as
`native_dynamic report native_build_plan.packages[0].command[0] must be a string` and
`native_dynamic report native_build_execution.packages[0].exit_code must be an integer`.
Each `native_build_plan.packages[]` row must carry the complete Cargo package build evidence:
`package_id`, `crate_name`, `manifest_path`, `workspace_manifest`, `target_dir`,
`cargo_profile`, `expected_loadable_artifact`, `release`, `features`, and `command`. The string
row fields must be non-empty trimmed strings before Report trusts the package id, crate, manifests,
target dir, profile, or expected loadable artifact. Padded string evidence stops at field-level
diagnostics before build-plan header equality, Cargo command option matching, profile/release
checks, or expected artifact derivation consumes it. The `command` row must be a non-empty string array;
non-array values keep the table-level string-array diagnostic, non-string entries stop at
`command[index] must be a string`, and blank or padded string entries stop at the non-empty/trimmed
diagnostics before Cargo identity or option matching runs. Its first two tokens must be `cargo` and `build`. It must include
exactly one valued `--manifest-path` matching the same row's `workspace_manifest`, exactly one
valued `-p`/`--package` matching the same row's `crate_name`, and exactly one valued `--target-dir`
matching the same row's `target_dir`. It must include one bare `--release` when the same row's
`release` is true and omit `--release` when `release` is false, so a hand-authored build plan cannot
point Cargo at a different workspace manifest, package selection, target directory, or release/debug
mode than the package evidence declares. `features` may be empty, but non-string entries stop at
the field-level `features[index] must be a string` diagnostic before command feature matching runs.
Schema-clean `features` cannot contain blank entries; when non-empty, `command` must include exactly one valued
`--features` whose value is the comma-joined `features[]` list, and when empty it must omit
`--features`.
The package row `expected_loadable_artifact` is also derived from `target_dir`, `cargo_profile`,
`crate_name`, and the enclosing report `target_platform`; final Report normalizes path separators
and rejects a row whose expected artifact path does not match the platform dynamic-library name.
For non-fatal stage reports, `native_build_plan` must also carry the complete plan header:
`workspace_manifest`, `target_dir`, `cargo_profile`, `release`, `build_features`, `package_count`,
`diagnostics`, `packages`, and `fatal`. Missing any of those header fields marks `NativeDynamic`
fatal before the package command rows are trusted. The string header fields `workspace_manifest`,
`target_dir`, and `cargo_profile` must also be non-empty and trimmed; an empty or padded Cargo
workspace manifest, target dir, or profile is not accepted as build-plan release evidence.
`diagnostics[]` distinguishes array shape from entry type: non-array values keep
`diagnostics must be a string array`, while non-string entries report `diagnostics[index] must be a string`
before blank, trimmed, fatal-explainability, or non-fatal-empty diagnostics semantics run. Schema-clean
diagnostic entries must be meaningful trimmed strings when present. `build_features` may be empty but cannot contain blank, padded,
or duplicate entries, matching the normalized CLI feature input contract.
`test_report_stage_rejects_native_dynamic_build_plan_fatal_without_diagnostics` keeps fatal
build-plan reports explainable: when `native_build_plan.fatal=true`, the nested diagnostics list
must include at least one non-empty reason even if the enclosing NativeDynamic report is already
fatal. Conversely, `native_build_plan.fatal=false` requires an empty diagnostics list, matching the
generation path where plan diagnostics drive the plan fatal flag. After `package_count` has passed
the non-negative integer schema gate, build-plan `package_count` must also equal the length of
`packages[]`, even when the enclosing NativeDynamic stage is already fatal.
Build-plan package ids are unique within `packages[]`, so a hand-authored audit cannot publish two
Cargo build-plan rows for the same plugin package.
That uniqueness check only consumes schema-clean package ids: padded package ids remain field-level
trimmed-string errors and are not normalized into duplicate-id diagnostics.
`native_build_execution` follows the same release-evidence rule for its execution header:
`enabled`, `fatal`, `skipped`, `diagnostics`, `package_count`, and `packages` must be present even
when execution is disabled and the package table is empty. Its `diagnostics[]` entries also distinguish
non-array values from non-string entries before empty/blank/padded/fatal-state rules run; schema-clean
entries cannot be empty, whitespace-only, or padded. Optional `skip_reason` is accepted only for skipped execution, and it
must be trimmed and non-empty, so a skipped execution state cannot publish a blank rationale while a
non-skipped audit cannot carry a stale skip reason. Padded `skip_reason` values are rejected as
field-level shape errors before they can collapse into a generic fatal-stage-only report. Skipped and disabled execution tables must stay
empty: `package_count` must be `0` and `packages` must be `[]`. A skipped execution remains an
enabled, non-fatal audit state, matching the materialization-diagnostics branch where Cargo was
requested but intentionally not launched. When execution package rows are present, each row must carry
`package_id`, `crate_name`, `command`, `exit_code`, `stdout`, `stderr`,
`expected_loadable_artifact`, `copied_loadable_artifact`, and `copied_sidecars`. The package id,
crate name, expected loadable artifact, and copied loadable artifact strings must be trimmed and
non-empty; `stdout` and `stderr` may be empty strings because successful commands often produce no
output. Build-execution `command[]` follows the same type split as the build plan: non-array
commands keep `command must be a string array`, while non-string command entries report
`command[index] must be a string` before build-plan handoff matching or command-array semantics run.
Build-execution `package_count` must also equal the length of `packages[]`.
Build-execution package ids are likewise unique within `packages[]`.
As with the build plan, build-execution duplicate detection only consumes trimmed package ids, so a
malformed padded package id does not also produce a duplicate-id diagnostic.
`test_report_stage_rejects_native_dynamic_build_execution_package_blank_required_string_field`
keeps whitespace-only execution row identity/artifact strings from reaching package-id closure or
copy evidence checks. For non-fatal build execution reports, each execution row is also closed
against the matching `native_build_plan.packages[]` row by `package_id`: `crate_name`, `command`, and
`expected_loadable_artifact` must match exactly before Report trusts the execution as the result of
the planned Cargo build. This handoff comparison is schema-clean only: padded execution or plan
`package_id`, `crate_name`, `expected_loadable_artifact`, or `command[]` entries stop at field-level
diagnostics before the plan/execution mismatch text is derived. A non-fatal NativeDynamic stage report also cannot carry
`native_build_execution.fatal=true`; the nested build execution status must agree with the enclosing
stage success state. The same success-state binding rejects `native_build_execution.skipped=true`
inside a non-fatal stage report. When the execution audit is enabled, the build-plan status must
also agree with that successful stage state: `native_build_plan.fatal=true` is rejected while
`native_build_execution.enabled=true` and the enclosing NativeDynamic report is non-fatal. This
keeps a diagnostic failed Cargo build plan from being published beside concrete build/copy execution
evidence as if the native plugin build had succeeded. Every package row `exit_code` must be
`0`, so a failed native plugin Cargo build cannot be published as successful copied-artifact
evidence. The `copied_loadable_artifact` field is also closed against the current
`materialized_packages[].loadable_artifacts` for that package after normalizing staged absolute
paths back to bundle-relative `plugins/...` paths; a build execution row cannot point at a copied
DLL/shared object that is absent from the published NativeDynamic package. The `command` row must be
a non-empty string array, cannot contain blank entries, and reports non-string entries at
`command[index] must be a string`; `copied_sidecars` may be empty, but non-string entries report
`copied_sidecars[index] must be a string` before blank, trimmed, duplicate, path, package-directory,
or file-manifest semantics run. Schema-clean sidecar entries cannot contain blank, padded, or
duplicate values. Final Report also requires every `copied_sidecars[]` entry to be a trimmed bundle-relative `plugins/...` path, requires it to remain under that package's materialized `native/`
directory, and requires it to be present in the current NativeDynamic plugins `file_manifest`; a
directory sidecar such as `.dSYM` is accepted when the manifest contains files below that directory.
`test_report_stage_rejects_native_dynamic_build_execution_package_duplicate_copied_sidecar_entry`
keeps repeated sidecar copy evidence from publishing the same copied path twice.

By default this plan is deliberately non-executing: missing workspace metadata is reported inside
`native_build_plan.diagnostics`, while package materialization still consumes existing artifacts
under each package's `native/` directory. Passing `--native-dynamic-build` turns the plan into an
execution gate. In that mode the stage may materialize package metadata/resources before source
`native/` artifacts exist, runs each planned Cargo command from `--repo-root`, copies the expected
`.dll`/`.so`/`.dylib` into the staged package `native/` directory, copies adjacent `.pdb`/`.dbg` or
`.dSYM` sidecars when present, then writes `native_dynamic_package.toml` after the built artifact is
part of the package payload. The stage report records this as `native_build_execution` with per
package command output, exit code, expected artifact, copied artifact, copied sidecars, and fatal
diagnostics. As with build plans, non-fatal execution audits must keep `diagnostics=[]`, while fatal
execution audits must include at least one non-empty diagnostic.
The build-plan TOML reader treats existing non-file paths for
`<repo-root>/zircon_plugins/Cargo.toml`, member crate `Cargo.toml`, and package `plugin.toml` as
diagnostics instead of filesystem exceptions. In the default non-executing mode those diagnostics
stay inside `native_build_plan`; with `--native-dynamic-build` enabled they become a build execution
gate before Cargo is invoked.
Path canonicalization failures follow the same NativeBuild reporting lane:
`test_native_dynamic_build_plan_rejects_target_dir_resolve_error`,
`test_native_dynamic_build_plan_rejects_workspace_manifest_resolve_error`, and
`test_native_dynamic_build_plan_rejects_member_manifest_resolve_error` keep `--target-dir`,
`<repo-root>/zircon_plugins/Cargo.toml`, and workspace member crate manifests diagnostic inside
`native_build_plan` instead of raising out of the NativeDynamic stage.
`test_native_dynamic_build_plan_uses_resolved_workspace_manifest_in_command` also locks the Cargo
command builder to the already-resolved workspace manifest path, preventing a redundant second
canonicalization step from bypassing the build-plan diagnostics lane.
The execution side also resolves the successful build's expected loadable artifact through the same
diagnostic helper; `test_native_dynamic_build_rejects_expected_artifact_resolve_error` makes that a
fatal build-execution diagnostic and payload cleanup rather than an uncaught filesystem exception.

NativeDynamic can also run an explicit external signing command before package reports and the
stage-level manifest/hash are written. Passing `--native-dynamic-sign-command <program>` enables the
hook; repeated `--native-dynamic-sign-arg <arg>` values are appended to the signer command and may
use `{artifact}`, `{package_id}`, `{package_dir}`, `{target_platform}`, and `{signing_profile}`
placeholders. `--native-dynamic-sign-profile <name>` records the selected signing profile and passes
that value to the external signer. Repeated `--native-dynamic-sign-platform <prefix>` entries form a
platform gate, so a profile declared for `windows` is accepted for `windows-x86_64` and rejected for
`macos-*` targets before any signer process starts. The stage executes the signer once per staged
loadable artifact selected by the target platform. It records the profile, target platform, allowed
platforms, platform-gate decision, expanded command, stdout/stderr, exit code, and before/after
SHA-256 values in `native_signing`. A platform mismatch, package artifact enumeration failure,
non-zero signer exit code, or a signer that removes the loadable file is fatal and clears the owned
`plugins/` payload. This mirrors Godot's
separation between copying shared objects and code-signing them before final bundle metadata is
sealed, while keeping Zircon's current implementation tool-agnostic instead of baking in platform
certificate stores.

After signing, NativeDynamic can run an explicit external notarization or platform post-processing
command before package reports and the stage-level manifest/hash are written. Passing
`--native-dynamic-notarize-command <program>` enables the hook; repeated
`--native-dynamic-notarize-arg <arg>` values are appended to the command and may use `{artifact}`,
`{package_id}`, `{package_dir}`, `{target_platform}`, `{signing_profile}`, and
`{notarization_profile}` placeholders. `--native-dynamic-notarize-profile <name>` records the audit
profile and passes it to the external command. Repeated `--native-dynamic-notarize-platform <prefix>`
entries form a platform gate before the command starts. The stage records the profile, target
platform, allowed platforms, platform-gate decision, expanded command, stdout/stderr, exit code, and
before/after SHA-256 values in `native_notarization`. A platform mismatch, non-zero command exit
code, or a command that removes the loadable file is fatal and clears the owned `plugins/` payload.
This is an external command boundary for future Windows/macOS/Linux platform services; it does not
integrate OS certificate stores, notary accounts, ticket stapling, or platform package repositories
by itself.

NativeDynamic operation audit validation now separates the complete stage report schema from the
stable PlatformBundle payload summary schema. The complete `native_signing` and
`native_notarization` stage report objects must include `diagnostics` plus `packages[]`, and package
rows may carry `artifacts[]` command/stdout/stderr/hash details. The final Report stage loader and
the standalone PlatformBundle handoff summary both validate those full audit objects before
normalizing their stable header fields. Wrong-typed audit fields now produce path-specific
diagnostics such as `native_dynamic report native_signing.allowed_platforms must be a string array`
or `NativeDynamic report native_notarization.platform_allowed must be a boolean`, while the old
`NativeDynamic report native_signing is malformed` fallback is reserved for schema-clean semantic
inconsistency. Stable payload audit summaries also require `enabled`, `allowed_platforms`,
`platform_allowed`, `fatal`, and `package_count`; missing fields report the same field-level
boolean/string-array/integer diagnostics before normalization. `allowed_platforms` may be empty to
allow every platform, but blank or padded platform entries are rejected before platform-gate
semantics are trusted.
Operation audit count evidence is range-checked as well: stable summary `package_count` and full
stage package `artifact_count` must be non-negative integers before final Report compares them
against normalized package or artifact rows. Operation audit `target_platform` is also required to
be a trimmed non-empty string; disabled operation `profile = null` remains valid, but any present
profile value must likewise be trimmed and non-empty. The target platform evidence must still
identify the stage target that platform gating was evaluated against.
The `allowed_platforms[]` filter accepts an empty array to mean every platform, but non-empty
filters must contain meaningful unique trimmed entries.
For enabled signing/notarization audit summaries, final Report also recomputes `platform_allowed`
from `target_platform` and `allowed_platforms[]` at the schema boundary for both the full
NativeDynamic stage audit and the stable PlatformBundle payload summary; a hand-authored report
cannot publish `platform_allowed = true` for an unsupported target/platform filter combination.
Duplicate `allowed_platforms[]` checks only consume schema-clean platform strings: padded platform
filters stay at field-level trimmed diagnostics and do not also produce duplicate-entry diagnostics.
For non-fatal NativeDynamic stage reports, the stage-only operation audit fields are release
evidence: missing `native_signing.diagnostics`, `native_signing.packages`,
`native_notarization.diagnostics`, or `native_notarization.packages` marks the NativeDynamic stage
fatal before PlatformBundle can project the stable audit summary.
When present, each operation audit `diagnostics[]` entry must be a meaningful string; empty or
whitespace-only rows are rejected at the operation-audit schema boundary instead of being accepted
as release evidence. Non-empty diagnostic rows must also already be trimmed, so hand-authored
reports cannot publish padded signing/notarization reasons that differ from the exact value shown
in final Report diagnostics.
For full NativeDynamic stage audit objects, each `packages[]` row must also carry string
`package_id`, integer `artifact_count`, and object-array `artifacts` evidence before final Report
accepts the operation audit shape. Each artifact row must carry the full execution evidence emitted
by a completed signer/notarizer command: string `artifact`, string `package_relative_artifact`,
string `stdout`, string `stderr`, integer `exit_code`, string `before_sha256`, string
`after_sha256`, and string-array `command`. Missing execution fields make the NativeDynamic stage
fatal before PlatformBundle can project the stable summary, so a non-fatal operation audit cannot
claim a signed or notarized artifact without the command result and before/after bytes evidence.
Package ids, artifact paths, package-relative artifact paths, and before/after SHA-256 values must
also already be trimmed non-empty strings; padded hash values fail this normalization gate before the
hex-shape diagnostic runs.
For non-fatal operation audits, each artifact `exit_code` must also be `0`; a failed signer or
notarizer command cannot be published as successful release evidence by hand-editing the stage
report.
The artifact `command` array must also be non-empty and contain no blank entries. Non-empty
`package_relative_artifact` values must be safe relative paths scoped to the staged package; absolute
or escaping paths are rejected before the materialized loadable artifact comparison runs. Artifact
rows in the same package cannot repeat the same `package_relative_artifact`, so a signing or
notarization audit cannot publish duplicate evidence for one staged loadable artifact. Package
relative artifact duplicate checks also require schema-clean path strings; padded paths stay at
trimmed-string diagnostics before duplicate detection.
identity and artifact identity/byte evidence strings are also non-empty release evidence:
`packages[].package_id`, `artifacts[].artifact`, `artifacts[].package_relative_artifact`,
`artifacts[].before_sha256`, and `artifacts[].after_sha256` cannot be empty, whitespace-only, or
padded with leading/trailing whitespace.
The before/after hash fields must also be SHA-256 hex digests, so successful signing or
notarization audit rows cannot publish arbitrary non-empty placeholders as byte evidence.
Captured `stdout` and `stderr` remain allowed to be empty strings.
The operation-audit schema boundary now lives in
`pipeline_report_native_dynamic_operation_audit_schema.py`; generic table/sequence schema row
dispatch lives in `pipeline_report_schema_table.py`. The payload schema keeps the stable
NativeDynamic payload/file/materialized-package contract and imports the audit entry points for
compatibility, so new audit release-evidence checks should extend the operation-audit module instead
of growing `pipeline_report_native_dynamic_payload_schema.py`.
NativeDynamic stage schema also imports the shared string-array helpers from
`pipeline_report_schema_table.py`, so top-level stage arrays, build plan/execution command arrays,
copied sidecar arrays, and operation-audit command/platform arrays use the same blank-entry,
trimmed-entry, and non-empty command diagnostics where the owning schema opts into each gate.

The downstream payload summary keeps the package audit and rejects malformed `materialized_packages`
rows instead of silently dropping per-package loadable-library evidence. It also cross-checks every
package `loadable_artifacts` path against the stage `file_manifest`, so a report cannot claim a
loadable library that is absent from the staged payload. The package-prefix check uses the
materialized package `destination` relative to the staged `plugins/` directory, not the raw
`package_id`, so package ids that sanitize to a different output directory such as
`animation.fx -> plugins/animation_fx` remain valid.
The same check is now bidirectional for release-facing loadable libraries: every current `.dll`,
`.so`, or `.dylib` file under a materialized package prefix must also appear in that package's
`loadable_artifacts` list. This prevents a stage or bundle report from publishing a package whose
payload contains an executable native library that is not represented in the audit rows consumed by
runtime loaders, Hub, or editor release views.
Stage-backed summaries now run the same typed list schema before normalization: `file_manifest[]`
rows require string `path`/`sha256` and integer `bytes`, while `materialized_packages[]` rows require
string package/path fields, integer `loadable_artifact_count`, and string-array
`loadable_artifacts`. Type errors produce field-level diagnostics such as
`NativeDynamic report file_manifest[0].bytes must be an integer` and do not also emit the broad
`file_manifest is malformed` / `materialized_packages are malformed` fallbacks. NativeDynamic
payload file rows are also required-field evidence: missing top-level `native_plugins_payload
file_manifest[]` fields and package-report `[[payload.files]]` fields now emit the same direct
`path`/`sha256` string or `bytes` integer diagnostics before final Report attempts payload
normalization.
Materialized package `loadable_artifacts[]` rows must also be non-empty trimmed strings before safe
relative path, duplicate, count, and file-manifest membership checks consume them. Non-string
`loadable_artifacts[]` entries are reported as `materialized_packages[index].loadable_artifacts[index]
must be a string` for both NativeDynamic stage reports and PlatformBundle `native_plugins_payload`,
instead of collapsing the whole row into a generic string-array failure.
Top-level `native_plugins_payload.content_hash` is also schema-checked as direct release evidence:
`test_report_rejects_native_plugins_payload_content_hash_blank` requires the field to be trim
non-empty, and `test_report_rejects_native_plugins_payload_content_hash_malformed` requires a
SHA-256 hex digest before final Report compares it to current bundle or NativeDynamic stage bytes.
Top-level payload strings are trim-checked too:
`test_report_rejects_native_plugins_payload_blank_top_level_path_strings` rejects whitespace-only
`bundle_path`, `loader_manifest`, `source`, or `stage_report`, while
`test_report_rejects_native_plugins_payload_padded_top_level_string` rejects leading/trailing
whitespace on `bundle_path`, `content_hash`, `loader_manifest`, `source`, and `stage_report` before
`Path.resolve()` can turn them into the current working directory or SHA-256 shape checks can emit
misleading diagnostics.
Top-level `native_plugins_payload.file_manifest[]` string identities are also trimmed non-empty
evidence: `test_report_rejects_native_plugins_payload_file_manifest_blank_strings` rejects
whitespace-only `path` or `sha256` rows, and
`test_report_rejects_native_plugins_payload_file_manifest_padded_path` /
`test_report_rejects_native_plugins_payload_file_manifest_padded_sha256` reject leading or trailing
whitespace before those rows can degrade into broad file-manifest drift or SHA-256 shape noise.
`test_report_rejects_native_plugins_payload_file_manifest_malformed_sha256` applies the same
digest-shape gate to the top-level payload file manifest: `file_manifest[].sha256` must be a
SHA-256 hex digest before final Report compares it to current bundle or NativeDynamic stage bytes.
`test_report_rejects_native_plugins_payload_file_manifest_unsafe_path` keeps
`file_manifest[].path` on the same safe-relative path contract as package-report payload rows:
no absolute path, empty segment, `.`, or `..` traversal is accepted before file-manifest comparison.
`test_report_rejects_native_plugins_payload_file_manifest_negative_bytes` keeps file byte counts
range-checked as release evidence too: `file_manifest[].bytes` must be non-negative before final
Report compares the top-level payload file manifest to current bundle or NativeDynamic stage bytes.
`test_report_rejects_native_plugins_payload_file_manifest_duplicate_path` also keeps
`file_manifest[].path` unique after normalizing path separators and trimming whitespace, so a
hash-updated payload summary cannot publish two rows for the same bundled native artifact.
`test_report_rejects_native_plugins_payload_missing_top_level_object_arrays` keeps those top-level
lists required as schema evidence too: `file_manifest` and `materialized_packages` must be present
object arrays, so final Report no longer reports only broad malformed fallback diagnostics when
either release-evidence list is omitted.
`test_report_rejects_native_plugins_payload_missing_required_top_level_scalars` applies the same
required-field rule to the scalar release evidence: `bundle_path`, `content_hash`, `loader_manifest`,
and `source` must be strings, while `file_count` and `package_count` must be integers before
path/hash/count semantic checks run.
`test_report_rejects_native_plugins_payload_negative_top_level_counts` applies the same
non-negative evidence rule to top-level `native_plugins_payload.file_count` and `package_count`, so
negative summary counts fail schema validation before they degrade into bundle-directory or package
count mismatch diagnostics.
`test_report_rejects_native_plugins_payload_content_hash_blank` and
`test_report_rejects_native_plugins_payload_content_hash_malformed` keep the top-level
`native_plugins_payload.content_hash` field in that schema lane too: it must be a non-empty
SHA-256 hex digest before final Report compares it with the current bundle plugins directory or
NativeDynamic stage report.
`test_report_rejects_native_plugins_payload_blank_top_level_path_strings` applies the same trimmed
non-empty rule to the top-level payload path/provenance strings. `bundle_path`, `loader_manifest`,
`source`, and `stage_report` must carry concrete string evidence before path resolution,
loader-manifest, or stage-backed provenance checks consume them.
The direct final-report helper also stops after those schema diagnostics:
`test_payload_diagnostics_stop_after_blank_top_level_path_schema` keeps schema-invalid payload
fields from producing secondary path mismatch, directory-read, or stage-backed audit noise.
`test_report_rejects_native_plugins_payload_materialized_package_blank_strings` and
`test_report_rejects_native_plugins_payload_materialized_package_padded_string_field` apply the same
trimmed non-empty evidence rule to `materialized_packages[]` string fields: `package_id`,
`destination`, `package_report`, and `source` cannot be whitespace-only or padded before
loader-manifest, stage-package, or package-report path checks consume the package row.
`test_report_rejects_native_plugins_payload_materialized_package_duplicate_id` keeps
`materialized_packages[].package_id` unique before loader-manifest or stage-package id comparison,
so duplicate package evidence is rejected as a schema violation instead of only as cross-report
drift.
`test_report_rejects_native_plugins_payload_materialized_package_negative_loadable_artifact_count`
keeps package artifact counts range-checked too: `loadable_artifact_count` must be non-negative
before the package row can be normalized or compared against staged loadable artifacts.
`test_report_rejects_native_plugins_payload_materialized_package_loadable_artifact_count_mismatch`
also binds the summary count to the array evidence: `loadable_artifact_count` must equal the
current `loadable_artifacts[]` length before materialized-package normalization can accept the row.
`test_report_rejects_native_plugins_payload_materialized_package_duplicate_loadable_artifact`
rejects duplicate `loadable_artifacts[]` entries within a package after safe-relative path
normalization, keeping the package audit list one row per loadable artifact.
`test_report_rejects_native_plugins_payload_materialized_package_blank_loadable_artifact` closes the
artifact path array itself: `loadable_artifacts[]` may be empty for packages with no loadable native
library, but it cannot contain empty or whitespace-only artifact paths.
`test_report_rejects_native_plugins_payload_materialized_package_unsafe_loadable_artifact` also
applies the safe-relative path gate to those artifact entries. A `loadable_artifacts[]` item cannot
be absolute or escape through `.`/`..` before final Report compares it with the bundled plugin
payload and NativeDynamic stage evidence.
The final Report stage loader now applies the same ABI v3 package export schema to NativeDynamic
stage report `package_exports[]` that Validate uses for
`plan_summary.native_dynamic_package_exports[]`. Stage package export rows reject unknown sidecar
fields, require locator fields such as `package_id`, `directory`, `path`, `manifest`, and
`package_report` to be strings, require `abi` to be an object, and type-check `abi.abi_version` plus
all ABI v3 contract string fields before PlatformBundle can trust staged package publishing
evidence.
Those identity, locator, and ABI fields are required release evidence on every package-export row,
not optional decoration. A non-fatal Validate or NativeDynamic stage report that omits
`package_id`, `directory`, `path`, `manifest`, `package_report`, or `abi` is marked fatal before the
row can be used as a runtime loader handoff.
The nested `abi` table is also complete-by-contract: `abi_version` and every ABI v3 string field
(`descriptor_symbol`, entry source fields, host function table, behavior/snapshot contracts, and
bridge method table) must be present as well as type-correct and value-correct.
Final Report also binds those package-export locator fields back to the current
`materialized_packages[]` destinations. For every package id that appears in a stage or Validate
package-export table, the expected `directory`, `path`, `manifest`, and `package_report` values are
derived from the package directory currently staged under `<out>/stages/native_dynamic/plugins/`.
This rejects a hash-refreshed report whose package bytes and package-local
`native_dynamic_package.toml` have moved to one directory while `package_exports[]` still advertises
an older runtime load path.
The Validate-side wrapper for `plan_summary.native_dynamic_package_exports` now also lives with
that shared package-export schema. It owns the list-level gate and object-row diagnostics before
dispatching each row into the ABI v3 schema, leaving the generic stage dispatcher to route the field
only when present.
On non-fatal exports it also records a sorted `file_manifest` of staged `plugins/` files.
Each file entry contains the stage-relative path, byte length, and sha256; the top-level
`content_hash` is derived from those entries so downstream tooling can verify the exact
NativeDynamic package payload before copying or publishing it. Package materialization copies only the release-facing
`plugin.toml`, target-platform native artifacts under `native/`, and resource directories named
`assets`, `asset`, `resources`, or `resource`; source crates and unrelated development files are not
copied into the stage output. When a direct package directory exists, its source `plugin.toml` is
authoritative for that selected package: parse errors, a missing/non-string `id`, or an id mismatch
are reported directly, and the stage does not fall back to a broader search that could silently pick
another package. When the direct package directory is absent and recursive search finds more than one
`plugin.toml` with the selected id, the stage reports a duplicate source fatal instead of choosing an
arbitrary directory. Source package and native artifact directory enumeration failures are also
fatal NativeDynamic diagnostics, so unreadable package roots or `native/` directories trigger the
same owned payload cleanup path instead of surfacing `OSError` from `iterdir()`.
Selected package destination path resolution is also a hard materialization gate: if a planned
`plugins/<package>` output path cannot be canonicalized, NativeDynamic records the package directory
diagnostic and clears the owned payload instead of aborting before its report is written.
NativeDynamic entry paths now use the shared diagnostics-aware resolver in `path_resolve.py`.
Explicit `repo_root`, `--validate-report`, and `--native-plugin-root` canonicalization failures are
reported by the NativeDynamic stage after `<out>/stages/native_dynamic/report.json` is known; the
failed field is recorded as `null`, no package payload is materialized, and the stage returns fatal
status instead of surfacing `OSError`.
NativeDynamic treats the staged
`plugins/` payload as atomic: if any
selected package cannot be found, lacks usable source native artifacts in the default non-build mode,
has invalid cdylib plan metadata in build mode, fails Cargo execution, does not produce its
expected loadable artifact, has schema-invalid signing/notarization CLI arguments, or fails the
configured signing/notarization command, the stage writes a fatal report,
keeps `loader_manifest = null`, does not write `plugins/native_plugins.toml`, clears the owned staged
`plugins/` payload so no successful package remains publishable beside the failed package, sets
`payload_cleaned = true` with `cleanup_reason = "fatal_diagnostics"`, and returns exit code `2`.
Successful NativeDynamic reports keep `payload_cleaned = false` and `cleanup_reason = null`.
When the trusted Validate report explicitly carries `profile_summary.strategies`, NativeDynamic
first applies the shared strategy-list hard gate: the field must be a list, must include at least
one supported export strategy, and may only contain `source_template`, `library_embed`, or
`native_dynamic` aliases. It then requires that normalized list to include `native_dynamic` before
consuming `plan_summary.native_dynamic_package_exports`. This keeps standalone
`--stage native_dynamic` from materializing ABI v3 plugin packages for a LibraryEmbed-only profile
and keeps malformed or future strategy metadata from falling through to stale NativeDynamic plan
rows. Historical standalone/debug reports that omit the `strategies` field entirely keep the
existing inspect behavior.

This is still an M5 NativeDynamic export slice. Optional cdylib build/copy is implemented behind
`--native-dynamic-build`, and external signer execution plus signing-profile audit/gating is
implemented behind `--native-dynamic-sign-command`; external notarization/post-processing execution
plus profile audit/gating is implemented behind `--native-dynamic-notarize-command`;
the signing/notarization command, repeated argument, profile, and platform-gate inputs must be
non-empty trimmed strings before the stage can materialize packages or launch external tools, so
padded release-command evidence is not silently normalized into the operation audit.
platform-native certificate-store integration, real notary service/ticket workflows, runtime
hot-update end-to-end invocation, and the linux/macos cross-platform real fixture matrix remain
follow-up work. The checked-in `native_dynamic_fixture` currently has one
debug NativeDynamic stage smoke with `--native-dynamic-build --offline` and one debug ABI v2 feature
smoke with `--native-dynamic-build-feature abi_v2_only`, plus matching release-mode smokes for both
variants; all four Windows local rows build and stage the cdylib payload.

## CookAssets Stage

`CookAssets` currently owns the pipeline handoff between future importer-driven asset cooking and
the existing pack writer. It consumes the same cooked asset manifest shape that `Pack` already
understands, validates the basic JSON contract, rewrites relative `source` paths to absolute paths
based on the source manifest directory, requires declared sources to resolve to regular files,
normalizes explicit manifest `roots[]`, `assets[].path`, and `dependencies[]` package paths to
trimmed forward-slash form, sorts explicit manifest `assets[]` by package path, sorts and
deduplicates `roots[]`, `dependencies[]`, and `labels[]`, and writes:

```text
<out>/
  stages/
    cook_assets/
      assets.json
      report.json
```

The stage is intentionally not a real importer yet. It does not scan the full project asset tree,
invoke scene or texture importers, parse asset-specific graph formats, or transform bytes. Those
behaviors still belong to the future real CookAssets implementation. The current stage exists so
`Validate -> CompileHost ->
CookAssets -> Pack -> PlatformBundle` has a stable file boundary and so `Pack` no longer needs a
manual manifest path once CookAssets has run.

Before falling back to textual scene-reference scanning, a project can now declare
`asset_manifest = "export/assets.json"` in `zircon-project.toml`. This field is used only when
`--asset-manifest` is omitted, and the command-line manifest still wins when present. The project
field must be a trimmed safe relative path under the project root; unsafe paths such as `..`,
absolute paths, or blank values are fatal and do not silently fall back to the default-scene scan.
The referenced JSON uses the same closed CookAssets manifest schema as an explicit
`--asset-manifest`, including root/dependency closure checks, deterministic ordering, source-file
existence checks, and default `asset_filter` injection when the manifest itself does not declare a
filter. Relative `assets[].source` rows are still resolved from the source manifest directory, and
the CookAssets report records that project-declared manifest in `source_asset_manifest` while also
preserving `project_default_scene`. This gives importer or editor-side asset graph generation a
stable project-level handoff into Pack without claiming that CookAssets itself has started invoking
real importers.

When no `--asset-manifest` is supplied, CookAssets now has a conservative project fallback. It reads
`--project`, resolves `default_scene = "res://..."` to `<project>/assets/...`, and writes a minimal
cooked manifest with that scene as the only root. It scans the default scene source text for direct
`res://...` references, strips any `#fragment`, records those files as direct dependencies of the
root scene, and recursively scans referenced text assets so their direct references join the same
temporary dependency closure. If the main pipeline supplied a profile `asset_filter`, the fallback
entries receive the same label so the entry is not cut by the temporary label filter. This remains a
project-entry boundary: the scan follows direct textual `res://` strings only, does not parse asset
formats, and does not invoke real importers. Referenced files that are not UTF-8 text are treated as
leaf assets in that temporary closure. The fallback writes `assets[]` and every generated
`dependencies[]` list in lexicographic package-path order, so Pack receives stable input even when
source text lists references in a different order or repeats them. Unsafe direct references use
`project asset reference ...` diagnostics, keep CookAssets fatal, and suppress `assets.json` just
like missing referenced files. The same safe relative asset path rule used by explicit manifests now
applies to `default_scene` and every direct `res://` reference, so empty path segments, `.`, `..`,
absolute paths, and package escapes are rejected instead of being normalized away.
Explicit cooked manifests are a closed schema: the top-level manifest may contain only `roots`,
`assets`, and `asset_filter`, and each asset row may contain only `path`, `source`, `dependencies`,
and `labels`. Unknown fields are fatal before any staged `assets.json` is written. Explicit cooked
manifests use the same normalized ordering before they are staged: path-like
package fields are first trimmed and converted to forward slashes, `assets[]` is sorted by the
normalized package path, while `roots[]`, `dependencies[]`, and `labels[]` are sorted and
deduplicated after the manifest passes the basic shape checks. Top-level `asset_filter` and
`labels[]` entries are trimmed before staging so filter matching uses the same label identity in the
manifest and the stage report. Duplicate asset paths remain a fatal
manifest-shape diagnostic before Pack sees the handoff, and duplicate detection uses the normalized
package path so `textures\hero.png` and `textures/hero.png` cannot describe different assets.
After normalization, every root and every dependency must reference a declared `assets[].path` row;
missing root or dependency references are fatal at CookAssets time instead of being left for Pack to
discover from a malformed staged manifest.
If an explicit cooked manifest declares top-level `asset_filter`, that value must be a trimmed
non-empty string; empty or whitespace-only values are fatal and no staged `assets.json` is written.
String entries inside `roots[]`, `dependencies[]`, and `labels[]` must also be trimmed non-empty
when present, and each asset row `path` must be a trimmed non-empty package path. A malformed
`roots` field that is not an array still reports the field-level string-array diagnostic, while
non-string `roots[]` entries report `roots[index] must be a string` before package-path checks run.
The same split applies to per-asset `dependencies[]` and `labels[]`: a non-array field reports
`assets[index].<field> must be a string array`, and a non-string member reports
`assets[index].<field>[entry] must be a string`.
`roots[]`,
`assets[].path`, and `dependencies[]` must stay inside the package as safe relative asset paths:
absolute paths, empty segments, `.`, and `..` are rejected before normalization can sort or
deduplicate them. Empty arrays remain valid.
Asset `source` remains optional for placeholder rows, but if the field is present it must be a
trimmed non-empty string; CookAssets trims it before resolving relative source paths, and blank
source fields are rejected instead of being treated as missing source metadata or deferred path
errors.
Explicit asset `source` rows that resolve to directories or other non-file paths are fatal at
CookAssets time, so Pack only receives source rows that can represent asset bytes.
The explicit asset manifest and fallback project manifest are checked as files before parsing;
directory paths or unreadable inputs become CookAssets diagnostics and no cooked `assets.json` is
written.
They are also resolved through CookAssets diagnostics before parsing. Explicit `--asset-manifest`
and `--project` values must be trimmed non-empty paths; empty or whitespace-only values produce
field-level argument diagnostics and keep the matching report field `null`, including dry-run.
If path canonicalization fails, the stage writes a fatal report with the failed field set to `null`
and does not emit a cooked manifest.

CookAssets also checks any declared `source` path after normalization. Missing source files or
non-file source paths make the stage fatal and prevent `<out>/stages/cook_assets/assets.json` from
being written, so invalid project entry manifests fail before Pack tries to read bytes.
When the trusted Validate report explicitly carries `profile_summary.strategies`, CookAssets
first applies the shared strategy-list hard gate: the field must be a list, must include at least
one supported export strategy, and may only contain `source_template`, `library_embed`, or
`native_dynamic` aliases. It then requires that normalized list to include either `library_embed` or
`native_dynamic` before cooking or copying an asset manifest. This keeps SourceTemplate-only
profiles from publishing stale cooked assets and keeps malformed or future strategy metadata from
falling through to stale CookAssets inputs, while preserving the bundle paths and historical
standalone/debug reports that omit the `strategies` field entirely.

When `Pack` is launched by the main pipeline, by `--resume-from pack`, or directly through
`--stage pack`, it first reads a non-fatal matching-profile
`<out>/stages/cook_assets/report.json` and uses its `cooked_asset_manifest` path as the Pack
manifest input. An explicit `--asset-manifest` still wins, and the fixed default
`<out>/stages/cook_assets/assets.json` is only used when no current-profile CookAssets report is
present.
When a current-profile CookAssets report exists, `cooked_asset_manifest` must be a non-empty string;
otherwise Pack writes a fatal preflight report and does not fall back to a stale default manifest.
When the trusted Validate report explicitly carries `profile_summary.strategies`, Pack also
first applies the shared strategy-list hard gate, then requires that normalized list to include
either `library_embed` or `native_dynamic` before building a full or delta pack. This prevents
SourceTemplate-only profiles and malformed or unsupported strategy metadata from producing stale
zrpack artifacts while preserving direct debug runs that do not have a Validate report yet.

CookAssets also accepts `--asset-filter <label>` as a default profile filter. When the argument is
omitted, both main-pipeline/resume execution and standalone `--stage cook_assets` read the matching
Validate report `profile_summary.asset_filter` when present. The default is written into the staged
cooked manifest only when the source manifest does not already declare `asset_filter`, so explicitly
cooked manifests keep their own filter decision.
When supplied directly, `--asset-filter` must also be a trimmed non-empty string; an empty or
whitespace-only value is fatal instead of being treated as "no filter." Dry-run uses the same
preflight and prints the same diagnostic. Pipeline/resume execution preserves that explicit invalid
value and does not replace it with a Validate report default, so the same hard gate applies through
every CookAssets entry point.
The Validate report field is optional, but if `profile_summary.asset_filter` is present it must be
a non-empty string; malformed values make CookAssets fatal instead of silently broadening the
cooked manifest to an unfiltered asset set.
If the Validate report file exists but fails the shared stage metadata contract (`stage`, `fatal`,
`diagnostics`, `profile`, or JSON/object shape), standalone CookAssets reports the same fatal
handoff diagnostic instead of treating the report as absent and producing an unfiltered manifest.
Successful CookAssets writes `<out>/stages/cook_assets/assets.json` with a stable LF JSON encoding
and records `cooked_asset_manifest_sha256` as the SHA-256 of the exact staged file bytes. Final
Report treats that hash as release evidence: the non-fatal CookAssets report must carry both
`cooked_asset_manifest` and `cooked_asset_manifest_sha256`, and
`tools/zircon_export/pipeline_report_cook_assets.py` resolves the manifest path, reads the file, and
fails the aggregate report if the byte hash differs. Final Report also requires the resolved
`cooked_asset_manifest` path to match the current CookAssets stage output
`<out>/stages/cook_assets/assets.json`, so a side manifest cannot be published as current stage
evidence even when the CookAssets report, hash, and Pack handoff are edited consistently.
Before those aggregate checks run, CookAssets stage-report schema requires
`cooked_asset_manifest` and `cooked_asset_manifest_sha256` to be non-empty trimmed strings.
Optional string provenance fields (`asset_filter`, `source_asset_manifest`, `project_manifest`,
`project_default_scene`) may stay `null`, but present values must also be non-empty and trimmed.
`pipeline_report_cook_assets_manifest_shape.py` owns the staged manifest shape contract: field
closure, trimmed strings, normalized safe package paths, duplicate-path checks, and root/dependency
reference closure. The same final-report helper also checks the Pack handoff: a non-fatal `pack` report `asset_manifest` must
resolve to the same file as the current non-fatal CookAssets `cooked_asset_manifest`, so Pack cannot
silently switch to a different manifest after CookAssets. It also parses the staged manifest JSON
and checks that report `asset_count` equals `assets[]` length, `root_count` equals `roots[]` length,
and report `asset_filter` equals staged manifest `asset_filter`, so hand-edited report counts and
filter provenance cannot drift from the published CookAssets input. Final Report also validates the
staged manifest shape itself: the top-level manifest and each asset row must stay on the same closed
field set CookAssets accepts; `roots` must be an array whose entries are strings before trimmed,
safe-relative, and normalized path semantics run, `asset_filter` must be a string when present and must
be trimmed non-empty when present, `assets` must be an object array, and each asset row keeps
`path`, `source`, `dependencies`, and `labels` on the same typed contract as CookAssets execution:
`path` and `dependencies[]` are safe relative asset paths in normalized forward-slash form,
normalized asset paths must remain unique, `asset_filter` and `labels[]` must already be in trimmed
string form, and non-string `dependencies[]` or `labels[]` members fail at the exact array entry
before trim, path, closure, or determinism checks run. Optional `source` may be omitted, but when
present it must be trimmed non-empty. When an
asset row
carries `source`, final Report also requires the staged value to be an absolute path that still
points to a regular file, so hand-edited or stale manifests cannot claim Pack-readable byte
provenance. Root and dependency references must also resolve to declared staged asset paths before
the report can be published. Blank or padded package path strings in `roots[]`, `assets[].path`, and
`dependencies[]` fail at this manifest schema layer before normalized-path or Pack trim-closure
checks run. Padded optional `asset_filter`, `assets[].source`, and `labels[]` entries also fail
with field-level non-empty trimmed string diagnostics before absolute-path or trim-evidence
semantics run. The `asset_count` / `root_count` length checks stay behind the same schema-clean
gate: if `roots[]` or `assets[]` still has padded, unsafe, unnormalized, duplicate, or wrong-typed
entries, final Report reports the field-level CookAssets manifest diagnostic and skips the broader
count mismatch. The deterministic ordering checks use that schema-clean boundary too: padded or
otherwise invalid `roots[]`, `assets[].path`, `dependencies[]`, or `labels[]` rows stop at
field-level manifest diagnostics before final Report emits sorted/unique mismatch diagnostics.
The Pack `trim_report` dependency-closure comparison uses the same boundary for package paths:
unsafe or unnormalized `roots[]`, `assets[].path`, or `dependencies[]` rows stop at CookAssets
manifest diagnostics before final Report derives included/trimmed/missing/duplicate closure
evidence from them.
Once the manifest rows are clean, it keeps the staged handoff on CookAssets' deterministic output
contract: `roots[]` and `assets[]` must remain sorted, and each `dependencies[]` and `labels[]`
list must remain sorted and deduplicated. The same cross-stage helper
derives CookAssets' expected trim evidence from `roots`, transitive
`dependencies`, optional `asset_filter`, and per-asset `labels`, then requires a non-fatal Pack
report's `trim_report.included_assets`, `trim_report.trimmed_assets`, and
`trim_report.missing_dependencies`, `trim_report.duplicate_assets`, and `trim_report.diagnostics`
to match that closure. Each `trim_report.diagnostics[]` row must be meaningful when present, so a
hand-authored trim report cannot publish empty or whitespace-only diagnostic evidence. Every path in
`trim_report.included_assets` must also map to a CookAssets manifest asset row with a non-empty
`source`, matching the Rust packer's byte-input contract that included assets without sources fail
before pack bytes are written. For those included assets,
`tools/zircon_export/pipeline_report_cook_assets_source_bytes.py` reuses the Rust writer's four-seed
FNV1a content hash algorithm and compares Pack `manifest.assets[].size` plus `chunk_hash` against
the actual CookAssets source file bytes, so a hand-authored Pack success report cannot claim stale
chunk metadata for different source contents. This byte-level comparison only runs after the
CookAssets `source` field is schema-clean and currently resolves to a regular file; relative,
missing, or otherwise unreadable source paths stop at CookAssets manifest source diagnostics instead
of also producing Pack source-byte read noise.
CookAssets
final Report stage-report schema now lives in
`tools/zircon_export/pipeline_report_cook_assets_stage_schema.py`; the generic stage dispatcher only
registers CookAssets fields and delegates the manifest/count/source/hash typed checks.

The manifest shape is:

```json
{
  "roots": ["scenes/main.zscene"],
  "asset_filter": "shipping",
  "assets": [
    {
      "path": "scenes/main.zscene",
      "source": "scenes/main.zscene",
      "dependencies": ["textures/hero.png"],
      "labels": ["shipping"]
    },
    {
      "path": "textures/hero.png",
      "source": "textures/hero.png",
      "labels": ["shipping"]
    }
  ]
}
```

## Pack Stage

`Pack` consumes a cooked asset manifest rather than reading `zircon-project.toml` directly. If
`--asset-manifest` is supplied, that explicit path is used. Otherwise `Pack` first reads a matching
non-fatal `<out>/stages/cook_assets/report.json` and uses its `cooked_asset_manifest` field. If no
current-profile CookAssets report is present, it falls back to `<out>/stages/cook_assets/assets.json`,
the standard output from the CookAssets handoff stage. This rule is shared by main-pipeline/resume
execution and standalone `--stage pack`.
An explicit `--asset-manifest` value is release input evidence and must be a non-empty trimmed
string. Empty, whitespace-only, or padded explicit values are preserved through pipeline/resume
defaulting, report `asset_manifest=null` in Pack's fatal preflight report, and skip command
construction instead of previewing a default manifest path.
The Pack output path follows the same rule: an explicit `--pack-file` must be non-empty and
trimmed, and an empty or padded value reports `pack=null` plus `command=<skipped>` instead of falling back to
`<out>/stages/pack/assets.zrpack`.
Incremental output uses the same hard preflight: `--previous-pack` and `--delta-pack` must be
supplied together as non-empty trimmed strings. If either delta argument is malformed, dry-run prints the
diagnostic and `command=<skipped>` instead of previewing a partial packer command.
The same trimmed-path argument gate covers Pack's explicit `--packer` and `--target-dir`
overrides before command construction.

In non-dry-run mode, the Python stage preflights that manifest path before invoking the Rust packer.
If the path does not exist or is not a file, the stage writes `<out>/stages/pack/report.json` with
`fatal=true`, an empty trim report, and a concrete diagnostic, then returns exit code `2`. This keeps
resume and final Report aggregation on the normal stage-report path instead of failing with no Pack
report. Python-generated Pack fatal reports mirror the Rust report schema with
`trim_report.duplicate_assets=[]` and `delta_apply_verified=false`, even when the packer was never
started or exits without writing its own report, including an otherwise successful process exit.
Dry-run still prints the selected command and input paths when preflight has no diagnostics.

The Python stage calls the Rust `zircon_export_pack` binary with the export `--profile`, which runs
the `ZrPackTrimPlanner` before any pack bytes are accepted for publication. Missing dependencies and
duplicate trim inputs are fatal preflight diagnostics backed by structured `missing_dependencies` and
`duplicate_assets` report fields: the binary writes `<out>/stages/pack/report.json` with
`manifest=null`, zero asset/chunk counts, and the trim diagnostics, but it does not invoke
`ZrPackWriter`, write `<out>/stages/pack/assets.zrpack`, or emit a delta package. When trim preflight
has no fatal diagnostics, included assets pass to `ZrPackWriter`, the full pack is written, and the
report includes the same `profile` field used by other stages so downstream pipeline handoff can reject
mismatched reports when profile metadata is present. `--determinism-check` writes the pack in memory a
second time and fails the stage if the bytes differ. Writer errors are fatal.
`ZrPackWriter` applies the same safe normalized package-path boundary as CookAssets and final Report
before writing bytes: unsafe paths, padded paths, and backslash-separated paths fail at the writer
input gate instead of being serialized into the pack manifest. `ZrPackReader` applies the same
manifest boundary when loading full packs, so external or downloaded `.zrpack` files with unsafe,
unnormalized, duplicate, or non-sorted asset rows fail before chunk ranges are trusted. It also
validates decoded pack document chunk evidence: pack format version, unique sorted chunk hashes,
`total_size`, asset chunk references, asset byte sizes, and absence of unreferenced chunks must stay
consistent with writer output. After range checks, the reader recomputes each chunk payload's
`zrpack_content_hash` and rejects bytes whose physical content no longer matches the manifest hash.
Final Report aggregation treats a non-fatal Pack report as release evidence rather than an optional
hint. A successful Pack report must include `asset_manifest`, `pack`, `stage_output`, `asset_count`,
`chunk_count`, `deduplicated_assets`, `deterministic_double_run`, `trim_report`, and `manifest`;
missing any of those fields marks `Pack` fatal during final aggregation. Delta report fields remain
optional/nullable for full-pack-only exports, with the existing `delta_pack` and
`delta_apply_verified` gates still enforcing requested delta publication.
The Pack report wrapper is a closed schema and the count fields `asset_count`, `chunk_count`,
`delta_asset_count`, and `delta_chunk_count` must be non-negative integers before manifest length
reconciliation runs. This keeps hand-authored reports with negative audit counts on field-level
diagnostics instead of reducing them to later count-mismatch noise.
Those length checks also require schema-clean manifest rows: malformed `manifest.assets[]`,
`manifest.pack.chunks[]`, `delta_manifest.changed_assets[]`, or `delta_manifest.chunks[]` entries
stop at their own row diagnostics before count fields are compared.
The required Pack path fields `asset_manifest`, `pack`, and `stage_output` must also be non-empty
strings, so a hand-authored successful report cannot satisfy the required-field gate with whitespace
that later resolves to the current working directory or no usable artifact.
Non-fatal Pack reports also bind artifact files back to the embedded binary manifests. Final Report
opens `pack` as `ZRPK`, `delta_pack` as `ZRPD`, and `previous_pack` as `ZRPK`, validates the 24-byte
header, supported format version, manifest byte range, manifest-at-EOF boundary, and JSON object
shape, then compares the embedded manifest to the report's `manifest`, `delta_manifest`, and
`delta_manifest.base` respectively. This prevents a report from publishing one manifest while
pointing at another pack or delta file. The report-side delta comparison waits for the whole
`delta_manifest` to be schema-clean first, so field-level delta manifest errors are reported before
embedded-manifest drift is considered. Final Report then derives the contiguous payload extent from
those embedded chunk tables and requires the binary `manifest_offset` to equal the end of the last
declared chunk. Full packs and previous packs use `pack.chunks[]`; delta packs use `chunks[]`. Chunk
ranges must stay inside that payload extent, and `zrpack_content_hash(...)` is replayed over each
range. An extent gap, trailing bytes after the manifest, a range outside the physical artifact or
manifest boundary, or a hash mismatch marks the Pack stage fatal, so a hash-current report cannot
publish a `.zrpack` or `.zrpd` whose payload bytes were swapped or whose artifact carries undeclared
bytes before or after the embedded manifest.
For non-fatal Pack reports, `stage_output` must also resolve to the current Pack stage directory
derived from the loaded `<out>/stages/pack/report.json` path. This prevents an external or stale Pack
stage directory from being published as the current pipeline stage while the report body remains
otherwise schema-clean.
Because missing dependencies and duplicate trim inputs are Pack preflight failures, a non-fatal Pack
report must keep `trim_report.missing_dependencies` and `trim_report.duplicate_assets` empty after
those arrays are schema-clean. Malformed rows stop at their field diagnostics first; non-string
members of `trim_report.included_assets[]` and `trim_report.diagnostics[]` report the exact entry
index instead of collapsing the whole field back to a string-array diagnostic.
`test_report_stage_rejects_pack_trim_report_schema_before_preflight` covers padded duplicate and
missing-dependency paths so they do not also emit the non-fatal preflight empty-list diagnostic.
`pipeline_report_pack_stage_schema.py` owns the Pack stage wrapper,
`pipeline_report_pack_trim_schema.py` owns trim-report schema, preflight, and manifest consistency,
`pipeline_report_stage_location.py` owns report-path-derived stage output location checks shared by
Validate and Pack,
`pipeline_report_pack_manifest_schema.py` owns embedded pack manifest schema/count/dedup evidence,
and `pipeline_report_pack_delta_schema.py` owns delta manifest schema/count/asset-set/chunk
evidence. This keeps final Report aggregation behavior unchanged while keeping the Pack release
evidence checks split by responsibility.
For non-fatal reports that carry schema-clean `trim_report` and `manifest` objects, final Report also
compares `trim_report.included_assets` with `manifest.assets[].path`. The lists must match after
sorting, so a hand-authored Pack report cannot keep correct counts while claiming that a different
trimmed asset set produced the shipped pack manifest. Final Report also replays the CookAssets
reachability/filter rule against the staged manifest and compares the derived included and trimmed
audit sets to `trim_report.included_assets` and `trim_report.trimmed_assets`; it also derives missing
root/dependency rows using the Rust `ZrPackMissingDependency` shape and compares them to
`trim_report.missing_dependencies`, and derives duplicate CookAssets asset paths using the same
first-entry-wins rule as `ZrPackTrimPlanner::collect_assets`. It also regenerates the Rust trim
diagnostic strings, including duplicated asset rows, missing roots/dependencies, and each trimmed
asset reason. Trimmed audit reasons use the Rust `ZrPackTrimReason` JSON shape, including `Unreferenced`,
`{"AssetFilterMismatch": "<label>"}`, and `{"UnreferencedAndAssetFilterMismatch": "<label>"}`.
This prevents Pack and CookAssets from being made individually self-consistent while publishing a
package that omits a reachable filtered dependency, hides a missing dependency, or misreports why an
asset was trimmed; duplicate asset rows cannot be hidden behind a successful Pack report either.
Final Report derives the expected `deduplicated_assets` list from schema-clean
`manifest.assets[].chunk_hash` rows by sorting assets the same way as `ZrPackWriter` and recording
every later path that reuses an already-seen chunk hash. The reported `deduplicated_assets` list must
match that derived set after sorting, so duplicate-content audit evidence cannot drift from the
embedded pack manifest. The list is also schema-gated for empty or whitespace-only entries before the
manifest comparison runs; an empty list remains valid when no asset content was deduplicated, but a
blank path is never publishable release evidence.
`deduplicated_assets[]` now uses the same indexed entry-type gate as the Pack delta asset lists:
non-array values still report `pack report deduplicated_assets must be a string array`, while
non-string rows report `pack report deduplicated_assets[index] must be a string` before blank,
trimmed, safe-path, duplicate-path, or manifest-derived deduplication comparison logic runs.
Chunk tables are release evidence too. Schema-clean `manifest.pack.chunks`,
`delta_manifest.base.pack.chunks`, `delta_manifest.target.pack.chunks`, and
`delta_manifest.chunks` may not repeat the same 32-byte chunk hash and must already be sorted by
chunk hash, so count-correct reports cannot publish duplicate or non-deterministic rows for the
content-addressed pack or delta payload table. Pack documents must carry both required top-level
tables, `pack` and `assets`; the `pack` table must carry `version`, `total_size`, and `chunks`;
each chunk row must carry `hash`, `offset`, and `size`.
Pack document asset rows are also tied to those chunk tables: every schema-clean
`assets[].chunk_hash` in the outer manifest and in `delta_manifest.base` / `target` must reference a
hash present in that same document's `pack.chunks`, and each asset's `size` must equal the size of
the chunk it references. ZRPD `delta_manifest.changed_assets` rows follow the same size rule against
`delta_manifest.chunks`. Every `assets[].path` row in those pack documents and every
`delta_manifest.changed_assets[].path` row must be a non-empty safe relative asset path in
normalized forward-slash form; duplicate normalized asset paths are rejected before trim,
deduplication, or delta set comparison trusts the manifest. The `assets[]` rows must also stay
sorted by `path`, matching `ZrPackWriter`'s canonical manifest output before deduplication or delta
set comparison uses the row order. Every asset row must carry `path`, `chunk_hash`, and `size`.
Embedded ZRPD `removed_assets[]` rows use the same safe normalized path
identity and duplicate-path gate as changed/full pack asset rows. Report-level path arrays that
mirror or summarize pack identities follow the same rule: `deduplicated_assets[]`,
`delta_removed_assets[]`, `delta_reused_assets[]`, `trim_report.included_assets[]`,
`trim_report.duplicate_assets[]`, `trim_report.trimmed_assets[].path`, and
`trim_report.missing_dependencies[].owner` / `.dependency` must all be non-empty trimmed strings
before they are accepted as normalized safe relative asset paths. The same trimmed-string gate
applies to `manifest.assets[].path`, so hand-authored Pack evidence with padded package paths fails
at the field schema boundary instead of falling through to generic normalized-path or trim-set
mismatch diagnostics.
Delta publish evidence shares the same field-ordering rule: `delta_manifest.base.assets[].path`,
`delta_manifest.target.assets[].path`, `delta_manifest.changed_assets[].path`,
`delta_manifest.removed_assets[]`, `delta_removed_assets[]`, and `delta_reused_assets[]` must be
non-empty trimmed strings before delta asset-set, target-manifest, reused-asset, removed-asset, or
changed-asset reconciliation consumes them.
Those arrays also split field shape from entry type: a non-array field keeps the string-array
diagnostic, while a non-string entry reports `delta_removed_assets[index]`,
`delta_reused_assets[index]`, or `delta_manifest.removed_assets[index]` as the bad value. Final
Report only derives delta target/removed/reused/changed set semantics after the whole
`delta_manifest` passes this schema-clean gate.
For schema-clean pack documents, `pack.total_size` must equal the sum of `pack.chunks[].size`,
matching `ZrPackWriter`'s unique-chunk byte accounting. `pack.total_size`,
`pack.chunks[].offset`, `pack.chunks[].size`, and `assets[].size` must all be non-negative integers,
so negative byte layout rows fail at the field schema layer before derived size or offset comparisons.
Chunk offsets must also describe the writer's contiguous payload layout: full-pack document
`pack.chunks` and ZRPD `delta_manifest.chunks` both start after the 24-byte pack/delta header,
and each following chunk starts at the previous chunk's end.
Included asset sources are part of the same publication gate. If an included path is missing from
the cooked manifest, lacks a `source`, or the source file cannot be read, `zircon_export_pack`
records a diagnostic in the report and stops before `ZrPackWriter`; the report still preserves the
trim result for audit, but `manifest` stays `null` and no full or delta pack is written.

For M5-T2, Pack can also produce a delta package by passing `--previous-pack <old.zrpack>` and
`--delta-pack <delta.zrpd>` together. The Rust packer reads the old and newly written full pack,
computes the chunk-hash difference, writes only target chunks missing from the old pack into the
`ZRPD` delta file, and records `delta_manifest`, delta asset/chunk counts, removed assets, and reused
assets in the pack report. The delta reader rejects unsafe, unnormalized, duplicate, or non-sorted
asset identities in embedded base/target pack manifests, `changed_assets[]`, and `removed_assets[]`
before it trusts changed chunk ranges. It also re-derives removed assets, changed asset entries, and
the unique sorted delta chunk hash table from embedded base/target manifests, then recomputes changed
chunk payload hashes before exposing changed bytes. A downloaded or hand-written ZRPD cannot publish
shape-correct but semantically stale patch evidence or swap a changed chunk payload behind a valid
hash table. This is the byte-package/report layer only; runtime application of a NativeDynamic hot
update remains a later slice.
Final Report re-derives the delta asset sets from the embedded `delta_manifest`: `removed_assets`
must equal `base.assets[].path - target.assets[].path`, `changed_assets` must be the full target
asset entries whose `chunk_hash` is not present in the base pack chunks, and report-level
`delta_reused_assets` must be the target paths whose `chunk_hash` already exists in the base pack
chunks. Report-level `delta_removed_assets` must also mirror `delta_manifest.removed_assets`, and
`delta_manifest.chunks[].hash` must equal the unique chunk hashes referenced by those changed asset
entries. A publishable `delta_manifest` must carry `format_version`, `base`, `target`, `chunks`,
`changed_assets`, and `removed_assets`; partial delta manifests fail at the schema layer before
delta set derivation. The report-level `delta_removed_assets` and `delta_reused_assets` lists, plus embedded
`delta_manifest.removed_assets`, are schema-gated for non-string, empty, or whitespace-only path
entries before those semantic set comparisons run.
The full-pack manifest and delta target manifest are also tied together: when a non-fatal Pack
report carries both schema-clean `manifest` and `delta_manifest.target` objects, final Report compares
the pack version, chunk table, total size, and asset entries. The target manifest embedded in a
delta report must describe the same full pack as the outer `manifest`, so stale or hand-authored
delta evidence cannot point at a different target package while reusing the current pack report.
The delta publication evidence is paired: if a Pack report publishes `delta_pack`, it must also carry
the schema-clean `delta_manifest` that explains the delta contents; if it carries a `delta_manifest`,
it must also publish the `delta_pack` path. This blocks hand-authored reports that claim delta bytes
without the manifest evidence needed to audit them, or manifest evidence without a released delta
artifact. `test_report_stage_rejects_pack_delta_blank_delta_pack` keeps that artifact path non-empty
even when the field is present and typed as a string, and `test_optional_path_field_reports_blank_present_value`
applies the same rule to stage handoff helpers before PlatformBundle can inherit a blank delta path.
`test_report_stage_rejects_pack_delta_optional_path_blank_string` applies the same non-empty rule
as soon as optional Pack delta path fields are present, so an empty `delta_pack` or blank
`previous_pack` cannot be interpreted as "no delta" before publication-pairing diagnostics run.
`test_report_stage_rejects_pack_delta_unpaired_previous_pack` also rejects the inverse pairing hole:
a report cannot carry a non-empty `previous_pack` without a matching non-empty `delta_pack`, because
the old full-pack input only has release meaning when a delta artifact is actually published.
`test_report_stage_rejects_pack_delta_missing_previous_pack` also requires the paired `previous_pack`
path whenever `delta_pack` is published, matching the packer CLI contract that delta generation always
consumes an old full pack and writes the previous-pack path into `ExportPackReport`.
`test_report_stage_rejects_pack_delta_blank_previous_pack` keeps that base-pack identity usable by
rejecting empty or whitespace-only `previous_pack` values in the same delta publication branch.
The Python stage preflights that both delta arguments are present and non-empty before invoking the
packer, so a one-sided or empty delta request fails at Pack instead of silently becoming a full
pack-only run.

## PlatformBundle Stage

`PlatformBundle` currently creates:

```text
<out>/
  bundle/
    <profile>/
      assets.zrpack
      bundle.json
  stages/
    platform_bundle/
      report.json
```

When `--host-executable` is supplied, the executable is copied beside the pack. Without a host
executable, the stage writes an assets-only bundle directory but returns fatal status; this keeps the
M2 report honest until CompileHost can produce the actual runtime/editor executable.

PlatformBundle reads typed stage handoff reports for both main-pipeline/resume execution and
standalone `--stage platform_bundle` debugging. When no explicit `--host-executable` is supplied, a
matching non-fatal `<out>/stages/compile_host/report.json` provides the host input. When
`--pack-file` is not supplied, a matching non-fatal `<out>/stages/pack/report.json` provides the full
pack input, and its optional `delta_pack` field provides the delta package input. Explicit CLI
arguments still win. If the matching report exists but is fatal, belongs to another profile, or has a
missing/empty/non-string path field, PlatformBundle reports that typed handoff error instead of
falling back to stale default files.
Explicit PlatformBundle handoff inputs are also typed release evidence. `--host-executable`,
`--pack-file`, `--delta-pack`, and `--native-plugins-dir` must be non-empty strings when supplied;
empty and whitespace-only strings are rejected at the same explicit-argument gate.
Pipeline/resume defaulting preserves explicit empty values and lets PlatformBundle fail with a
parameter diagnostic instead of replacing them with stage report defaults.
Final Report aggregation now applies the same release-evidence rule to non-fatal PlatformBundle
stage reports. A successful report must include `bundle`, `host_executable`, `host_source`,
`host_source_origin`, `pack`, `pack_source`, `pack_source_origin`, `template_files`, and
`bundle_manifest`; missing fields make `PlatformBundle` fatal at the stage schema layer before
bundle manifest or payload hash checks run. Delta, NativeDynamic payload, template metadata, and
template-root resolution fields remain optional/nullable unless their corresponding export path or
strategy is active.
The required PlatformBundle string fields `bundle`, `host_executable`, `host_source`,
`host_source_origin`, `pack`, `pack_source`, `pack_source_origin`, and `bundle_manifest` must also
be non-empty after trimming whitespace, so a hand-authored successful report cannot satisfy the
schema with `" "` before path containment, manifest matching, or file hash diagnostics run.
Optional-but-present PlatformBundle string evidence follows the same rule: `delta_pack`,
`delta_pack_source`, `delta_pack_source_origin`, and `native_plugins` may be `null` when that output
is not active, but cannot be whitespace-only once published in a non-fatal stage report.
`test_report_rejects_platform_bundle_report_padded_string_field` also rejects leading or trailing
whitespace on non-empty PlatformBundle report strings, so `bundle`, bundled host/pack/delta paths,
their provenance fields, `native_plugins`, and `bundle_manifest` fail with field-level diagnostics
before path containment, origin matching, manifest comparison, or payload-existence checks consume
the value.
The NativeDynamic payload final-report entrypoint repeats the trimmed non-empty `native_plugins`
guard before payload schema or path resolution, so
`test_payload_diagnostics_rejects_blank_native_plugins_before_path_resolution` preserves the same
field-level diagnostic even when that helper is exercised directly.
When PlatformBundle copies a host executable, its stage report and final `bundle.json` also record
`host_source` and `host_source_origin`. CompileHost handoff sources use
`host_source_origin = "compile_host_report"` and must remain aligned with the CompileHost report;
explicit `--host-executable` sources use `argument`; template-provided host placeholders use
`template`.
Supplying an explicit `--pack-file` also prevents the pipeline from inheriting the Pack report's
optional `delta_pack`; callers must opt into a matching delta package with `--delta-pack`.

Each non-dry-run PlatformBundle execution recreates the current profile bundle directory before
validation and materialization. If the stage becomes fatal before or during materialization, the
profile bundle directory is removed and no final `bundle.json` or template-provided manifest is
written; callers should inspect only the stage report for that failed attempt.

M3-T1 adds optional `--template-dir <dir>` support. The directory must contain `template.toml`; when
valid, `paths.host_executable` can provide the host executable path so callers do not need to pass
`--host-executable` separately. M3-T2 adds `--template-root <dir>` for local template repositories:
when `--template-dir` is omitted, PlatformBundle scans direct child packages and selects the single
template matching the requested profile, target platform, engine version, and format version.
Template validation runs before copying bundle contents and records a `template` object in both
`bundle.json` and `<out>/stages/platform_bundle/report.json`. Template-root selection also records
`template_resolution` with candidates and diagnostics. A mismatch returns exit code `2` and skips
host/pack copying.
When `--target-platform` is omitted, template-root resolution may infer the target platform from the
current Validate report only after the shared stage metadata contract passes: the report must be the
`Validate` stage, non-fatal, carry string-array diagnostics, and match the requested profile. If
PlatformBundle already has handoff or strategy diagnostics, it skips template-root resolution
entirely so a fatal stage report cannot also advertise a stale template selection.

The current `template.toml` format is:

```toml
format_version = 1
template_id = "windows-x86_64-library_embed-debug"
engine_version = "0.1.0"
target_platform = "windows-x86_64"
host_kind = "desktop"
host_artifact = "placeholder"
resource_strategy = "filesystem_bundle"
plugin_strategy = "native_dynamic_allowed"
bundle_format = "directory"
compatible_profiles = ["windows-release"]
content_hash = "<sha256 over sorted file path + file sha256 rows>"

[paths]
host_executable = "bin/zircon_runtime.host-placeholder"

[bundle]
host_path = "ZirconRuntime"
pack_path = "data/assets.zrpack"
delta_pack_path = "patches/assets.delta.zrpd"
manifest_path = "zircon-export.json"

[[files]]
path = "bin/zircon_runtime.host-placeholder"
bundle_path = "ZirconRuntime"
purpose = "M3-T1 placeholder host path for template contract validation"
sha256 = "<file sha256>"
```

`format_version` is locked to `1`. `engine_version` defaults to `[workspace.package].version` from
the root `Cargo.toml` unless `--engine-version` is supplied. When PlatformBundle needs that fallback,
the workspace manifest path must exist as a regular file; a directory or unreadable root
`Cargo.toml` becomes a fatal PlatformBundle diagnostic before template-root selection or
template-dir validation continues. Target platform is taken from
`<out>/stages/validate/report.json` when available, or from `--target-platform`. The CLI also
keeps the input `template.toml` contract closed: unknown top-level keys, unknown `[paths]` keys,
unknown `[bundle]` keys, and unknown `[[files]]` row keys are fatal template diagnostics before
PlatformBundle can trust or copy the package. `compatible_profiles` must be a string array and
cannot contain empty, whitespace-only, or duplicate entries, so profile matching metadata cannot
carry inert or repeated sidecar rows.
`host_artifact` is also required and must be either `placeholder` or `precompiled`. The checked-in
M3 template packages deliberately publish `placeholder` because their host files are contract
fixtures; real CI-built runtime hosts must publish `precompiled` before release/Hub distribution
can treat them as runnable platform templates.
Duplicate `compatible_profiles[]` checks on both the template manifest and the PlatformBundle
template report only consume schema-clean profile strings, so padded entries remain trimmed-string
diagnostics instead of also producing duplicate-entry diagnostics. Template report profile-membership
checks use the same boundary, so malformed profile-list entries do not also report
`compatible_profiles does not include profile ...`. It verifies `paths.host_executable` and each
`[[files]].path` are safe relative paths: no absolute path, empty segment, `.`, or `..`. It then
checks that the declared host path stays inside the template directory, is present in `[[files]]`,
and matches its declared SHA-256 digest and aggregate `content_hash`. Every declared
`[[files]].path` must resolve to a regular file before hashing or bundle copy; a directory at that
path is reported as `is not a file` in the template or PlatformBundle diagnostics instead of
surfacing as a filesystem exception. `test_template_rejects_unknown_manifest_fields` locks this
input-side closed-schema gate, and `test_template_rejects_blank_compatible_profile_entries` plus
`test_template_rejects_duplicate_compatible_profile_entries` lock the profile-list quality gate.

M3-T2 extends the same contract with template-driven bundle layout. `[bundle]` can declare
`root`, `host_path`, `pack_path`, and `manifest_path`, and each `[[files]]` entry can declare the
destination `bundle_path`. All bundle paths must be relative and stay inside the profile bundle
directory. The checked-in Linux fixture materializes a directory bundle as
`ZirconRuntime`, `data/assets.zrpack`, and `zircon-export.json`; the macOS fixture materializes
`ZirconRuntime.app/Contents/MacOS/ZirconRuntime`,
`ZirconRuntime.app/Contents/Resources/assets.zrpack`,
`ZirconRuntime.app/Contents/Info.plist`, and
`ZirconRuntime.app/Contents/Resources/zircon-export.json`. The host files are still placeholders;
real runnable platform templates remain gated on CompileHost/CI artifacts.

When the trusted Validate report explicitly carries `profile_summary.strategies`, PlatformBundle
first applies the shared strategy-list hard gate, then requires that normalized list to include
either `library_embed` or `native_dynamic` before materializing a profile bundle. This keeps
SourceTemplate-only profiles and malformed or unsupported strategy metadata from publishing stale
host/pack artifacts through explicit `--host-executable` and `--pack-file` arguments, while
preserving direct debug runs that do not have a Validate report yet.
Explicit PlatformBundle entry and release input paths (`repo_root`, `--host-executable`,
`--pack-file`, `--delta-pack`, `--native-plugins-dir`, `--template-dir`, and `--template-root`) are
resolved through the stage diagnostics path; if path resolution fails, the stage writes a fatal
`PlatformBundle` report instead of aborting before `report.json` exists. The explicit `repo_root`
path uses the shared resolver from `path_resolve.py`, while empty/non-string release input arguments
remain on the existing argument-diagnostic path so they are not accidentally canonicalized as the
current working directory.

When `--native-plugins-dir` is supplied and resolves successfully, PlatformBundle copies that staged
NativeDynamic `plugins/` directory into `<bundle-root>/plugins`.
When the option is omitted, PlatformBundle reads
`<out>/stages/native_dynamic/report.json` for a non-fatal current-profile `plugins_dir`, so both main
pipeline execution and standalone `--stage platform_bundle` can keep NativeDynamic plugin packages
in the final bundle without a manual path. When that same NativeDynamic report contains
`content_hash` and
`file_manifest`, and a well-formed `materialized_packages` list, PlatformBundle preserves the
payload audit as `native_plugins_payload` in both `bundle.json` and the PlatformBundle stage report,
including the stage report path, source path, final bundle path, file count, file manifest, content
hash, package count, per-package loadable artifact lists, and stable `native_signing` /
`native_notarization` audit summaries when present. Those operation summaries intentionally carry
only stable header fields such as `enabled`, `profile`, `target_platform`, `allowed_platforms`,
`platform_allowed`, `fatal`, and `package_count`; the full per-artifact command/stdout/stderr/hash
evidence remains in the NativeDynamic stage report. If these operation summaries are present in the
NativeDynamic stage report, PlatformBundle requires them to be structurally valid before projecting
them into `native_plugins_payload`; enabled operation summaries must also report a `package_count`
matching the current `materialized_packages` count, while disabled placeholders may keep
`package_count = 0`. Before copying, PlatformBundle
recomputes the current staged `plugins/` directory hash and rejects the bundle if it no longer
matches the NativeDynamic report `content_hash`; malformed package audit rows also become stage
diagnostics instead of a partial payload summary.
The same stage-backed payload summary path applies the shared stage metadata contract before copy:
`stage`, `fatal`, `diagnostics`, and `profile` must be trustworthy before PlatformBundle can use a
NativeDynamic report as staged plugin release evidence. Existing current NativeDynamic report paths
must also be files before that summary path reads them; directory paths or unreadable paths become
PlatformBundle diagnostics instead of falling back to a manual-directory snapshot.
Final Report applies the same diagnostic stance to PlatformBundle release evidence: host, pack, delta,
and NativeDynamic payload source/output paths from CompileHost, Pack, PlatformBundle,
`native_plugins_payload`, and `bundle.json` are resolved through report diagnostics, so a stale or
hand-authored path that cannot be canonicalized marks the release report fatal instead of
interrupting aggregation.
The shared bundle-containment guard also treats a late `Path.resolve()` failure as outside the
bundle root; `test_path_is_relative_to_treats_resolve_error_as_outside` keeps those secondary
canonicalization failures from escaping as raw `OSError`.
The current output bundle expectation is diagnostic too:
`test_report_bundle_path_rejects_stage_path_resolve_error` keeps a PlatformBundle stage report path
that cannot be canonicalized from aborting final Report while deriving `<out>/bundle/<profile>`.
Template-file expected hash mapping follows the same rule:
`test_template_file_hashes_rejects_declared_path_resolve_error` keeps
`template.files[].path` canonicalization failures diagnostic while building the expected hash table.
The stage report `template_files[].source` lookup is also diagnostics-aware:
`test_template_file_expected_hash_rejects_source_resolve_error` keeps source canonicalization
failures from aborting expected-hash lookup.
`test_report_rejects_template_file_destination_resolve_error` applies the same rule to
`template_files[].destination`, so final Report records a destination resolve diagnostic before
checking existence, file type, or content hash.
The automatic inheritance path performs that report file/metadata check before extracting
`plugins_dir`, so a bad current NativeDynamic report keeps its concrete diagnostic instead of being
reduced to a generic missing-native-plugins message.
For explicit `--native-plugins-dir` directory snapshots that do not have a matching stage report,
package-level `native_dynamic_package.toml` entries must also be regular files when present.
Directory or unreadable package reports become PlatformBundle diagnostics before bundle
materialization, instead of silently falling back to the package directory name.
When a trusted Validate report explicitly carries `profile_summary.strategies`, PlatformBundle also
requires that list to include `native_dynamic` before accepting either a staged NativeDynamic
plugins directory or an explicit `--native-plugins-dir`. This blocks LibraryEmbed-only exports from
publishing native plugin payloads at the PlatformBundle stage instead of relying on the final Report
to reject the bundle later. Historical standalone/debug inputs that omit `strategies` entirely keep
their existing inspect behavior.
The same trusted Validate strategy list is also checked for malformed or unsupported values before
PlatformBundle materializes a bundle. A non-list strategy field, an explicitly empty list, or an
unknown strategy such as `future_export_path` makes the PlatformBundle stage fatal with the same
diagnostic text used by final Report aggregation, so standalone bundles cannot publish against
unrecognized export paths.

## Report Stage

`Report` is the final pipeline aggregation stage. It reads Validate first, then derives required
stage reports from Validate report `profile_summary.strategies`.

For a LibraryEmbed profile, it requires:

```text
<out>/stages/validate/report.json
<out>/stages/compile_host/report.json
<out>/stages/cook_assets/report.json
<out>/stages/pack/report.json
<out>/stages/platform_bundle/report.json
```

For a SourceTemplate profile, it requires:

```text
<out>/stages/validate/report.json
<out>/stages/source_template/report.json
```

For a NativeDynamic profile, it requires:

```text
<out>/stages/validate/report.json
<out>/stages/native_dynamic/report.json
<out>/stages/compile_host/report.json
<out>/stages/cook_assets/report.json
<out>/stages/pack/report.json
<out>/stages/platform_bundle/report.json
```

The stage writes both `<out>/stages/report/report.json` and the release-level `<out>/report.json`.
The aggregate report records missing stages, fatal stages, each source report path, each stage's
diagnostics, and the embedded raw stage report. Missing required reports, malformed JSON, profile
mismatch, or any stage with `fatal = true` makes the final report fatal and returns exit code `2`.
Existing stage report paths must resolve to files; directory paths or unreadable paths become final
Report diagnostics instead of filesystem exceptions. The same file gate applies when legacy
PlatformBundle payload inspection reloads a declared current NativeDynamic report for signing and
notarization comparison.
The JSON body `stage` field must also match the stage implied by its path, such as `CompileHost`
for `<out>/stages/compile_host/report.json`; mismatched or missing stage identities are fatal
release evidence. The same stage metadata gate requires `fatal` to be an explicit boolean, so a
missing or non-boolean value cannot be interpreted as a non-fatal stage report. `diagnostics` must
be a string array, and every entry must be non-empty and trimmed, so final reports and editor/Hub
readers can display stage diagnostics without accepting arbitrary JSON or padded release-audit text.
`source_template` and `native_dynamic` are read only when the Validate strategies request them. Stale
reports for non-selected strategies can remain under `<out>/stages/`, but they do not enter the
current pipeline report and cannot make the current export fatal.
When a trusted Validate report explicitly carries `profile_summary.strategies`, a non-fatal
PlatformBundle report may publish `native_plugins` or `native_plugins_payload` only if that strategy
list includes `native_dynamic`. LibraryEmbed-only reports with stale NativeDynamic payload evidence
become fatal and the top-level `native_plugins_payload` projection is suppressed. Historical
debug/report fixtures that omit the `strategies` field entirely keep the existing permissive
fallback so old hand-authored PlatformBundle reports remain inspectable.
Unknown, explicitly empty, or non-list Validate strategies are fatal release evidence and do not
fall back to the default LibraryEmbed stage set; this keeps misspelled, malformed, empty, or future
strategy names from producing a misleading bundle.
The same no-default rule applies when the Validate report exists but is not trustworthy because its
basic metadata is malformed, it is already fatal, or it belongs to another profile. In that case the
final Report records the Validate wrapper as fatal and requires only `validate`, so it exposes the
Validate problem without inventing missing LibraryEmbed stages. The default LibraryEmbed required
stage set is kept only for the debug/report case where no Validate report file exists at all.
For each non-fatal PlatformBundle report, the final Report stage now treats `bundle` as the current
output's owned profile bundle root. The field must be a non-empty string, must resolve to an
existing directory, and must match `<out>/bundle/<profile>` as derived from the current
PlatformBundle stage report before the reported `bundle_manifest` is allowed to resolve inside that
directory. The manifest path must then exist as a file; directories or unreadable paths become fatal
diagnostics instead of filesystem exceptions before the JSON consistency recheck runs. After the
manifest JSON object is loaded, `test_report_rejects_bundle_manifest_unknown_top_level_field` keeps
the manifest schema closed: every top-level key must be one of the PlatformBundle release-evidence
fields that final Report audits against the stage report, so unknown sidecar metadata cannot be
published as unvalidated bundle evidence. After the manifest is accepted, final release payload paths
in `host_executable`,
`pack`, `delta_pack`, `native_plugins`, and every `template_files[].destination` must also resolve
inside the same bundle root before payload-existence, template-file, NativeDynamic payload, and
source-hash rechecks run. Source/provenance fields such as `host_source` and `pack_source` may still
point outside the bundle because they identify upstream build inputs. These gates prevent a stage
report from pointing at an external but self-consistent profile bundle or publishing external files
as if they were owned bundle payload. Containment canonicalization failures stay on the
`... could not be resolved` diagnostic lane through
`test_report_rejects_payload_containment_path_resolve_error` and
`test_report_rejects_payload_containment_parent_resolve_error`; only successfully resolved paths
that fail the relative-to check are reported as outside the PlatformBundle bundle.

## Resume Flow

Omitting `--stage` runs the main export stage machine from `validate` through `report`. After
Validate succeeds, the runner reads `profile_summary.strategies` from
`<out>/stages/validate/report.json` and selects the remaining stages for the requested path. This is
the plan-level release command used by local export and CI orchestration:

```powershell
python -m tools.zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export
```

`--resume-from <stage>` now runs the main export stage machine from the selected stage through
`report`. Resume also uses the Validate report strategy list when available, so
`--resume-from source_template` on a SourceTemplate profile continues with `source_template,report`,
`--resume-from native_dynamic` on a NativeDynamic profile continues with
`native_dynamic,compile_host,cook_assets,pack,platform_bundle,report`,
while `--resume-from pack` on a LibraryEmbed profile continues with `pack,platform_bundle,report`.
The no-Validate-report fallback remains available for direct debug dry-runs, but only when the
Validate report is genuinely absent. If `<out>/stages/validate/report.json` exists but is malformed,
has invalid basic metadata, is fatal, belongs to another profile, is a directory, or cannot be read,
the main pipeline and resume selection do not fall back to the default LibraryEmbed stage set; they
run only `report`, letting the final Report stage expose the invalid Validate evidence.

If the Validate report is present and the requested resume stage is outside the selected strategy
set, the runner skips directly to `report`; stale or manually requested strategy stages are not
replayed for the current profile.

```text
validate -> [source_template] -> [native_dynamic] -> [compile_host -> cook_assets -> pack -> platform_bundle] -> report
```

This option is for pipeline recovery and cannot be combined with `--stage`, which remains the
single-stage debug and CI entry point. The runner stops at the first non-zero stage exit code and
does not synthesize later reports, so a failed `platform_bundle` resume does not accidentally write a
final `<out>/report.json`.

`source_template` can still be run as a standalone debug stage with `--stage source_template`, but it
is no longer excluded from the main pipeline when the selected profile requests the SourceTemplate
strategy.

## Command Surface

Useful commands:

```powershell
python -m tools.zircon_export --help
python -m tools.zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export --stage validate
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage validate --dry-run --offline --target-dir D:\cargo-targets\zircon-export-validate-cli-0614
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage compile_host --offline
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage source_template --offline
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage source_template --source-template-build --offline
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-build --offline
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline
python -m tools.zircon_export plugin build native_dynamic_fixture --form dist --platform windows-x86_64 --mode debug --out D:\zircon-export\plugins --target-dir D:\cargo-targets\zircon-plugin-native_dynamic_fixture
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-sign-command D:\tools\sign-native.exe --native-dynamic-sign-arg "{artifact}"
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-notarize-command D:\tools\notarize-native.exe --native-dynamic-notarize-arg "{artifact}" --native-dynamic-notarize-profile windows-attestation --native-dynamic-notarize-platform windows
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage cook_assets --asset-manifest D:\zircon-export\assets\assets.json
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage pack --determinism-check
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage pack --previous-pack D:\zircon-export\previous\assets.zrpack --delta-pack D:\zircon-export\stages\pack\assets.delta.zrpd
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --host-executable D:\zircon-export\stages\compile_host\zircon_runtime.exe
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --host-executable D:\zircon-export\stages\compile_host\zircon_runtime.exe --native-plugins-dir D:\zircon-export\stages\native_dynamic\plugins
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --pack-file D:\zircon-export\stages\pack\assets.zrpack --template-dir tools/zircon_export/export-templates\windows-x86_64-library_embed-debug --target-platform windows-x86_64
python -m tools.zircon_export --profile linux-release --out D:\zircon-export --stage platform_bundle --pack-file D:\zircon-export\stages\pack\assets.zrpack --template-root tools/zircon_export/export-templates --target-platform linux-x86_64
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --stage report
python -m tools.zircon_export --profile windows-release --out D:\zircon-export --resume-from pack
```

`--validator <path>` lets callers use a prebuilt `zircon_export_validate` executable and skip
`cargo run`. `--packer <path>` does the same for `zircon_export_pack`. `--asset-manifest <path>` is
the CookAssets source manifest and remains an explicit Pack input override when needed.
`--previous-pack` and `--delta-pack` enable M5-T2 delta package output for Pack and must be supplied
together.
`--template-dir <path>` makes PlatformBundle consume one export-template package.
`--template-root <path>` makes PlatformBundle resolve one matching package from a local template
repository when `--template-dir` is omitted. `--engine-version` and `--target-platform` can override
the values used for template compatibility checks.
`--native-plugins-dir <path>` copies a NativeDynamic stage `plugins/` directory into the final
PlatformBundle output; main/resume pipeline execution fills it from a non-fatal
`<out>/stages/native_dynamic/report.json` when present. If no matching NativeDynamic stage report is
available for an explicit directory, PlatformBundle still records a directory-level
`native_plugins_payload` snapshot with loader manifest, content hash, file manifest, and
package/loadable-artifact summary. That explicit-directory snapshot reports final bundle logical
paths under `plugins/...` even when the source directory has a different local name, and rejects
package-level
`native_dynamic_package.toml` paths that are directories or unreadable files. A matching malformed
or stale NativeDynamic report remains fatal instead of being silently replaced by a directory
snapshot.
`--source-template-build` makes the SourceTemplate stage execute the generated project's Cargo build
instead of only materializing files. `--native-dynamic-build` makes the NativeDynamic stage execute
its cdylib Cargo build plan and copy the built loadable artifacts into staged plugin packages;
without it, NativeDynamic only consumes existing package `native/` artifacts. Repeat
`--native-dynamic-build-feature` to pass Cargo features such as `abi_v2_only` into the native cdylib
build plan and execution command. Build feature arguments must be non-empty trimmed strings; blank,
padded, or non-string values fail before the stage can construct a Cargo `--features` argument.
`plugin build <id>` is the Plugins 13 per-plugin dist entry. It reads the plugin root
`plugin.toml` `[distribution]` section, builds the declared `dist_crate` with
`--no-default-features --features dist --locked` by default, and writes an isolated package
directory at `<out>/<id>/` containing the loadable library renamed to `<id>.dll|so|dylib`,
`plugin.toml`, optional `<id>.zrpack` assets from `[distribution].assets`, and
`native_dynamic_package.toml`, plus a generated `<id>.sig` hash sidecar. It also writes
`<out>/native_plugins.toml` so the isolated package output already has an ABI v3 loader manifest
for runtime-side discovery. Asset subpackages are built by the Rust `zircon_export_pack`
binary with `--determinism-check`; use `--packer <path>` to point at a prebuilt packer instead of
running it through Cargo. The command uses its own `--target-dir` and does not create profile
pipeline output under `<out>/stages`. Package reports use package-relative paths so two runs from
the same source can be compared byte-for-byte without out/target absolute path drift.
`--sign-command` enables an external signer for per-plugin dist build output, with
`--native-dynamic-sign-command` kept as the profile-stage spelling. Repeat `--sign-arg` or
`--native-dynamic-sign-arg` for signer arguments and use placeholders such as `{artifact}`,
`{package_id}`, and `{target_platform}` when the signer needs artifact-specific values. Add
`--sign-profile` / `--native-dynamic-sign-profile` to record and pass a profile label, and repeat
`--sign-platform` / `--native-dynamic-sign-platform` to restrict that profile to target-platform
prefixes before the external signer is launched. These signing command, argument, profile, and
platform values are schema-clean release inputs: they must be strings without leading/trailing
whitespace and cannot be blank, and invalid values stop before payload materialization or external
command launch. The `.sig` sidecar is still emitted without an external signer and records loadable
artifact bytes/hash evidence; with signing enabled it additionally records signer before/after hash
evidence before `native_dynamic_package.toml` and the loader manifest are sealed.
`--dry-run` prints the exact stage command or bundle inputs without creating stage output. Cargo
commands use `--locked` by default;
`--native-dynamic-notarize-command` enables an external notarization or platform post-processing
command after signing and before package reports/manifests are sealed; repeat
`--native-dynamic-notarize-arg` for arguments, use `--native-dynamic-notarize-profile` to record a
profile label, and repeat `--native-dynamic-notarize-platform` to gate target-platform prefixes
before the command is launched. The notarization command, arguments, profile, and platform gates use
the same non-empty trimmed-string rule and fail before notarizer launch when malformed.
`--no-locked` exists only for explicit lockfile work.
`--resume-from <stage>` replays the main pipeline from a persisted stage directory and is mutually
exclusive with `--stage`.

## Future Stages

The Python stage enum currently exposes `Validate`, `CompileHost`, `SourceTemplate`,
`NativeDynamic`, `CookAssets`, `Pack`, `PlatformBundle`, and `Report`; the resumable main pipeline
selects SourceTemplate, NativeDynamic, and LibraryEmbed stage groups from Validate strategies. Later
work should replace the CookAssets handoff with real importer-driven cooking, extend NativeDynamic
beyond optional cdylib build/copy plus external signer/notarization profile command execution into
platform-native certificate-store signing, real notary service/ticket workflows, runtime hot-update
end-to-end invocation, and the
linux/macos cross-platform real fixture matrix, and expand the final report with
launch-smoke/performance evidence without moving plan validation out of `zircon_runtime`.
Each stage should continue writing beneath
`<out>/stages/<stage>/` so failures are resumable and editor UI can stream a stable pipeline model.

## Test Coverage

M1 adds `validate_report_summarizes_profile_plan_and_fatal_state` to prove the shared runtime report
summarizes profile fields, plan crate links, and fatal state. Python smoke coverage checks module
compilation, `--help`, and a dry-run Validate command.

M2-T1 adds `feature_matrix_links_selected_plugins_only`, which verifies that a LibraryEmbed profile
projects selected plugins and selected feature crates into the CompileHost plan while trimming
unselected plugin and optional feature crates.

The Python CompileHost dry-run coverage verifies that the CLI consumes
`plan_summary.library_embed_compile_host`, rewrites `--target-dir` to
`<out>/stages/compile_host/target`, appends Cargo lock/offline flags, computes the expected host
executable path, and rejects profile mismatches before invoking Cargo. It now also includes
`test_compile_host_report_respects_target_dir_override`, which verifies a custom `--target-dir`
feeds both Cargo execution and the report `host_executable` handoff path. A real Cargo CompileHost
run is not claimed yet because current runtime/UI compile drift is still being tracked separately.
The behavior-preserving module split keeps the same public CLI dispatch surface while moving the
CompileHost subprocess and report-write implementation into `compile_host.py`; focused tests still
import the compatibility exports from `cli.py`, and subprocess mocks now patch
`tools.zircon_export.compile_host.subprocess.run`.
`test_compile_host_dry_run_rejects_invalid_validate_metadata` keeps malformed Validate metadata from
being consumed as a compile plan during dry-run or execution planning.
`test_compile_host_rejects_plan_without_binary` and
`test_compile_host_rejects_plan_without_cargo_profile` keep required host output selectors out of
the Cargo launch path when Validate hands over an incomplete CompileHost plan.
`test_compile_host_rejects_plan_with_empty_command` covers the corresponding command vector
boundary, while `test_compile_host_rejects_plan_with_blank_command_entry` keeps blank command
entries on the same fatal path.
`test_compile_host_rejects_plan_with_padded_command_entry` covers the trimmed-token boundary for
direct CompileHost execution before command rewrite and Cargo launch.
`test_compile_host_rejects_plan_with_padded_feature_entry` covers the same pre-launch normalized
feature evidence boundary for `app_features[]` and `runtime_features[]`.
`test_compile_host_rejects_plan_with_dangling_target_dir_option` keeps command option rewriting from
turning malformed `--target-dir` evidence into a valid launch command.
`test_compile_host_rejects_target_dir_option_with_option_value` keeps `--target-dir --release`
from being accepted as a plan option/value pair before rewrite.
`test_compile_host_dry_run_rejects_invalid_strategy_metadata` applies the same standalone stage hard
gate as final Report for non-list, empty, and unsupported `profile_summary.strategies` values before
any CompileHost command is printed.
`test_compile_host_dry_run_requires_host_strategy` keeps SourceTemplate-only Validate reports from
consuming stale LibraryEmbed/NativeDynamic host plan rows.

The Python SourceTemplate coverage verifies that the CLI consumes Validate report generated-file
contents, rewrites the generated project's `--manifest-path` and `--target-dir`, materializes
`<out>/stages/source_template/project`, rewrites local `zircon_*` path dependencies to the current
workspace root, and records a non-fatal report when build execution is intentionally skipped.
`test_source_template_rejects_plan_with_blank_command_entry` keeps blank command entries from
reaching SourceTemplate command rewrite or optional Cargo execution.
`test_source_template_rejects_plan_with_non_string_command_entry_before_array_shape` keeps
non-string command rows on the indexed Validate build-plan schema diagnostic path instead of also
emitting the broader SourceTemplate build-plan command fallback.
`test_source_template_rejects_plan_with_dangling_manifest_path_option` and
`test_source_template_rejects_plan_with_dangling_target_dir_option` keep SourceTemplate command
option rewriting from turning malformed Validate plan evidence into a valid build command.
`test_source_template_rejects_manifest_path_option_with_option_value` keeps option-looking
manifest-path values on the same fatal path.
`test_source_template_stage_rejects_generated_manifest_directory` keeps the generated
`project/Cargo.toml` dependency-rewrite step on the same file gate as later generated-file evidence:
a directory or unreadable manifest becomes a fatal SourceTemplate diagnostic and the owned
`project/` directory is removed before the report is written.
`test_source_template_stage_rejects_invalid_validate_metadata` keeps malformed Validate metadata
from being consumed as generated-project release input.
`test_source_template_stage_reports_successful_build_validation` and
`test_source_template_stage_reports_failed_build_validation` cover the structured
`build_validation` evidence for executed Cargo builds, including working directory, command, status,
and exit code.
`test_source_template_stage_marks_invalid_generated_file_fatal` verifies that invalid generated
paths fail the SourceTemplate report instead of producing an incomplete successful project.
The path-aware pipeline coverage adds `test_pipeline_from_validate_uses_source_template_profile_stages`,
`test_report_stage_uses_source_template_profile_requirements`, and
`test_report_stage_requires_source_template_for_source_template_profile`. Together they verify that
Validate strategies drive both execution order and final report requirements for SourceTemplate-only
profiles. `test_report_stage_ignores_stale_strategy_reports` keeps the final Report stage isolated
from stale `source_template` or `native_dynamic` reports when the current Validate report requests
only `library_embed`.

The Python NativeDynamic path coverage adds `test_native_dynamic_stage_writes_package_export_report`,
`test_native_dynamic_stage_materializes_package_and_loader_manifest`,
`test_native_dynamic_stage_reports_materialized_file_manifest`,
`test_native_dynamic_package_report_records_package_payload_hash`,
`test_native_dynamic_stage_removes_stale_unselected_packages`,
`test_native_dynamic_stage_filters_artifacts_by_target_platform`,
`test_native_dynamic_stage_requires_platform_loadable_artifact`,
`test_native_dynamic_stage_copies_macos_dsym_bundle`,
`test_native_dynamic_stage_reports_package_loadable_artifacts`,
`test_native_dynamic_payload_summary_keeps_loadable_artifact_audit`,
`test_native_dynamic_payload_summary_rejects_malformed_package_audit`,
`test_native_dynamic_payload_summary_rejects_loadable_artifact_not_in_manifest`,
`test_native_dynamic_stage_reports_native_cdylib_build_plan`,
`test_native_dynamic_build_plan_rejects_workspace_manifest_directory`,
`test_native_dynamic_build_plan_rejects_crate_manifest_directory`,
`test_native_dynamic_build_executes_plan_and_stages_cdylib`,
`test_native_dynamic_signs_loadable_artifact_before_manifest_hash`,
`test_native_dynamic_signing_rejects_before_hash_read_error`,
`test_native_dynamic_signing_rejects_after_hash_read_error`,
`test_native_dynamic_signing_failure_cleans_staged_payload`,
`test_native_dynamic_payload_summary_accepts_sanitized_package_directory`,
`test_native_dynamic_stage_removes_partial_package_on_artifact_filter_fatal`,
`test_native_dynamic_stage_removes_all_packages_when_any_package_fails`,
`test_native_dynamic_stage_rejects_inconsistent_package_paths`,
`test_native_dynamic_stage_rejects_inconsistent_package_report_path`,
`test_native_dynamic_stage_derives_missing_package_report_path`,
`test_native_dynamic_stage_accepts_sanitized_package_directory`,
`test_native_dynamic_stage_rejects_package_directory_id_mismatch`,
`test_native_dynamic_stage_rejects_duplicate_package_ids`,
`test_native_dynamic_stage_rejects_source_manifest_id_mismatch`,
`test_native_dynamic_stage_rejects_source_manifest_parse_error`,
`test_native_dynamic_stage_rejects_source_manifest_directory`,
`test_native_dynamic_stage_rejects_source_manifest_missing_id`,
`test_native_dynamic_stage_rejects_duplicate_recursive_package_sources`,
`test_native_dynamic_stage_rejects_non_v3_abi_version`,
`test_native_dynamic_stage_rejects_wrong_v3_descriptor_symbol`,
`test_native_dynamic_stage_rejects_unselected_package_export`,
`test_native_dynamic_stage_rejects_duplicate_selected_package_ids`,
`test_native_dynamic_stage_rejects_missing_selected_package_export`,
`test_native_dynamic_stage_rejects_invalid_validate_metadata`,
`test_native_dynamic_stage_reports_missing_package_source_fatal`,
`test_report_stage_requires_native_dynamic_for_native_dynamic_profile`, and
`test_pipeline_from_validate_uses_native_dynamic_profile_stages`. Together they verify that a
NativeDynamic profile runs
`NativeDynamic -> CompileHost -> CookAssets -> Pack -> PlatformBundle -> Report`, that final Report
requires the `native_dynamic` stage report plus the downstream bundle stage reports, that the stage
preserves the ABI v3 package export table from the Validate report, that package files and
`plugins/native_plugins.toml` are materialized, that staged files are reported with deterministic
paths, byte lengths, sha256 values, and a top-level
`content_hash`, that each package's loadable `.dll`/`.so`/`.dylib` files stay visible in
`materialized_packages[]` and payload summaries, that malformed package audit rows are rejected,
that claimed loadable artifact paths must exist in the stage `file_manifest`,
that the loadable-artifact prefix check follows sanitized output directories rather than raw ids,
that the report includes a cdylib Cargo build plan for selected package crates, that
schema-invalid `--native-dynamic-build-feature` values stop before Cargo feature joining, that
schema-clean repeated build features are normalized into the build plan and Cargo command, that
`test_native_dynamic_build_plan_respects_target_dir_override` keeps custom
`--target-dir` values aligned across the native build plan, command, and expected loadable path, that
`--native-dynamic-build` can execute that plan and stage the built loadable artifact into the package
before package payload reports and file manifests are written, that an explicit signer command can
mutate staged loadable artifacts before package payload reports and file manifests are sealed, that
the signing report records before/after hashes and command execution, that signing artifact hash
read failures before or after the external command become fatal diagnostics instead of filesystem
exceptions, that schema-invalid signing command/argument/profile/platform input stops with
field-level diagnostics before external command execution, and that signing failures clean the
staged payload atomically, that an explicit
notarization/post-processing command runs after
signing but before package payload reports and file manifests are sealed, that the notarization
report records before/after hashes and command execution, that schema-invalid notarization
command/argument/profile/platform input stops with field-level diagnostics before external command
execution, and that notarization platform mismatches clean the staged payload atomically,
that each `native_dynamic_package.toml` records its package-local payload files and hash, that stale
unselected package directories are removed before the new payload manifest is computed, that
platform-specific packages do not copy foreign-platform dynamic library/debug symbol artifacts, that
platform debug symbols cannot replace a loadable native library, that package-level stage reports
expose loadable artifact paths/counts for audit, that macOS `.dSYM`
debug symbol bundles are copied recursively beside `.dylib` artifacts and included in deterministic
file manifests, that packages failing artifact filtering are removed instead of remaining as partial
payload directories, that any
package failure clears the whole staged `plugins/` payload instead of
leaving partially successful packages publishable, that fatal cleanup is reflected in the stage report
with `payload_cleaned` and `cleanup_reason`, that missing Rust-side `package_report` fields are
derived before writing the loader manifest, that sanitized package directories are accepted, that
package directories which do not match their `package_id`-derived value are rejected, and that
inconsistent package `path`/`manifest`/`package_report` rows are rejected before writing the loader manifest,
that duplicate `package_id` entries are
rejected before writing the loader manifest, that direct source package manifest parse errors,
missing ids, and id mismatches are specific fatal diagnostics, that recursive package source search
rejects multiple `plugin.toml` matches for the same selected package id, that non-v3 ABI package
exports and mismatched ABI v3 contract names are rejected before materialization, that the selected package list and
export table must match exactly, that non-string package export ids stop at the package export
field type diagnostic before non-empty, uniqueness, or selected-package semantics, that duplicate selected package ids are rejected before
materialization, and that a missing selected package is fatal without leaving
`plugins/native_plugins.toml` behind.
`test_native_dynamic_reports_repo_root_resolve_error`,
`test_native_dynamic_reports_validate_report_resolve_error`, and
`test_native_dynamic_reports_native_plugin_root_resolve_error` keep NativeDynamic entry-path
canonicalization failures on the stage-report lane before Validate report loading or package source
materialization.
The Python NativeDynamic tests are split by responsibility: `test_native_dynamic.py` keeps
ABI/package selection, target-platform artifact and payload-summary coverage;
`test_native_dynamic_build_signing.py` keeps build-plan, cdylib execution, signing and
notarization/post-processing coverage; `native_dynamic_test_support.py` owns the focused fixtures
that would otherwise duplicate NativeDynamic package manifests, fake Cargo, signing, and
notarization scripts across both files.

The NativeDynamic M5-T1 coverage verifies that Validate report exposes
`native_dynamic_package_exports`, that generated loader manifests deserialize optional ABI v3
contract fields, and that materialized native package directories include
`native_dynamic_package.toml`. The scoped validator Cargo check passed under
`D:\cargo-targets\zircon-export-m5-native-dynamic-0614` with existing warning noise.
`native_dynamic_only_profile_carries_minimal_compile_host_plan` verifies that a NativeDynamic-only
profile carries a minimal CompileHost plan for the final host while keeping dynamic packages out of
linked runtime crates; the scoped `cargo check -p zircon_runtime --lib --no-default-features
--features core-min` passed under
`D:\cargo-targets\zircon-plugin-native-dynamic-host-plan-check-0615`, but the focused Rust test
timed out during lib-test compilation and is not claimed as passing.
`native_loader_loads_real_fixture_from_export_load_manifest_payload` constructs a stage-style
`plugins/native_plugins.toml` payload with the real `native_dynamic_fixture` cdylib, package
`plugin.toml`, and `native_dynamic_package.toml`, then loads it through
`NativePluginLoader.load_all_from_load_manifest(...)` and asserts the ABI v3 runtime/editor entry
reports. The scoped core-min Cargo check passed under
`D:\cargo-targets\zircon-plugin-native-dynamic-loader-manifest-check-0615`; the exact focused test
timed out after 904 seconds during lib-test execution, so this guard is type-checked but not claimed
as a passing focused runtime test.

The Python CookAssets coverage verifies that `--stage cook_assets` writes
`<out>/stages/cook_assets/assets.json` and `report.json`, preserves the cooked manifest shape, and
normalizes relative asset `source` paths before Pack reads the staged file. The Pack dry-run coverage
also verifies that omitting `--asset-manifest` selects the CookAssets default path.
`test_cook_assets_rejects_unknown_manifest_fields` keeps the explicit manifest as closed release
evidence: unknown top-level keys and unknown asset-row keys are fatal before CookAssets writes a
staged `assets.json`.
`test_cook_assets_rejects_manifest_missing_references` keeps explicit cooked manifests
self-contained: `roots[]` and `dependencies[]` must name declared `assets[].path` rows before
CookAssets writes the handoff.
`test_cook_assets_stage_rejects_explicit_manifest_source_directory` keeps explicit source rows on
the CookAssets pre-Pack file contract by rejecting a declared source that resolves to a directory.
`test_cook_assets_stage_orders_explicit_manifest_assets_and_dependencies` keeps explicit cooked
manifests on the same deterministic handoff path: staged `assets[]` are package-path sorted, and
declared `dependencies[]` are sorted and deduplicated before Pack consumes the file.
`test_cook_assets_stage_orders_explicit_manifest_roots` applies the same deterministic rule to the
entry root list so repeated authoring roots cannot affect Pack closure evidence.
`test_cook_assets_stage_orders_explicit_manifest_labels` applies the same deterministic rule to
declared labels so filter evidence is stable even when source manifests repeat labels or list them in
authoring order.
`test_cook_assets_normalizes_explicit_manifest_package_paths` keeps explicit manifest `roots[]`,
`assets[].path`, and `dependencies[]` on the same forward-slash package path form before staged
evidence is written. `test_cook_assets_rejects_duplicate_manifest_paths_after_normalization` keeps
two authoring spellings of the same package path from becoming separate CookAssets assets.
`test_cook_assets_normalizes_manifest_filter_and_labels` applies the same staging contract to
publish labels: top-level `asset_filter` and per-asset `labels[]` are trimmed before the report and
manifest are written, so label-based trim evidence is not split by authoring whitespace.
`test_cook_assets_normalizes_manifest_source_when_present` keeps optional source evidence on the
same trim boundary: a padded relative `source` resolves to the intended file before CookAssets writes
the absolute source path into staged `assets.json`.
The profile-filter handoff coverage verifies that main-pipeline and standalone CookAssets can receive
`profile_summary.asset_filter` from Validate report and that a manifest-declared `asset_filter`
still takes priority. `test_stage_cook_assets_uses_validate_report_asset_filter` covers the
standalone entry point.
`test_pipeline_cook_assets_rejects_invalid_validate_report_asset_filter` and
`test_stage_cook_assets_rejects_invalid_validate_report_asset_filter` keep non-string or empty
Validate filter fields from being treated as "no filter".
`test_stage_cook_assets_rejects_invalid_validate_metadata` keeps malformed Validate metadata from
being treated as an absent report by standalone CookAssets.
`test_cook_assets_project_fallback_records_direct_res_asset_references` locks the conservative
project fallback dependency scan: direct `res://...` strings in the default scene become cooked
assets and root-scene dependencies, with URI fragments removed before source lookup.
`test_cook_assets_project_fallback_records_recursive_direct_references` extends that fallback to the
temporary recursive direct-reference closure: a material referenced by the root scene can contribute
its own direct texture dependency without promoting that transitive texture into the root scene's
direct dependency list.
`test_cook_assets_project_fallback_treats_binary_reference_as_leaf` keeps binary assets in the
closure without trying to decode them as text; a referenced PNG is emitted as a leaf asset instead of
raising a UTF-8 decode error.
`test_cook_assets_project_fallback_rejects_missing_direct_reference` verifies those derived
dependencies still pass through the normal source existence gate before `assets.json` is emitted.
`test_cook_assets_project_fallback_rejects_unsafe_direct_reference` keeps unsafe direct `res://`
references on a precise `project asset reference ... does not resolve to a safe asset path`
diagnostic instead of labeling the scanned reference as the root `project default_scene`.
`test_cook_assets_project_fallback_rejects_default_scene_empty_path_segment` and
`test_cook_assets_project_fallback_rejects_direct_reference_empty_path_segment` keep the fallback
from collapsing empty `res://` path segments into valid package paths.
`test_cook_assets_rejects_empty_explicit_asset_filter` applies the same non-empty rule to direct
CookAssets inputs.
`test_cook_assets_reports_asset_manifest_resolve_error` and
`test_cook_assets_reports_project_manifest_resolve_error` keep explicit CookAssets entry path
failures on the stage-report lane before JSON/TOML parsing.
`test_pipeline_cook_assets_preserves_empty_explicit_asset_filter` verifies that pipeline defaulting
does not overwrite an explicit empty filter before CookAssets can reject it.
`test_cook_assets_dry_run_rejects_empty_explicit_asset_filter` keeps the dry-run preflight aligned
with the non-dry-run stage report path.
The shared stage-report handoff rules live in `tools/zircon_export/stage_handoff.py`, keeping
typed report-field parsing and diagnostics out of the command runner while preserving the same CLI
surface. Those helpers also reject reports whose JSON `stage` identity does not match the stage
directory being consumed, so a copied or hand-authored report body cannot masquerade as current
CompileHost, CookAssets, Pack, Validate, or NativeDynamic evidence by landing at the right path.
They also require `fatal` to be an explicit boolean before any path field can be consumed.
`diagnostics` is part of the same basic report contract and must be a string array.
`tools.zircon_export.tests.test_stage_handoff` directly covers the shared required-field,
optional-field, and Validate `asset_filter` diagnostics so future handoff gates do not have to rely
only on large pipeline integration tests.
`test_stage_cook_assets_requires_bundle_strategy` keeps SourceTemplate-only Validate reports from
consuming stale CookAssets inputs and writing `<out>/stages/cook_assets/assets.json`.
`test_stage_cook_assets_rejects_invalid_strategy_metadata` covers the shared bundle strategy helper
for non-list, empty, and unsupported `profile_summary.strategies` values before CookAssets writes a
staged manifest. The same helper feeds Pack and PlatformBundle bundle-strategy gates.
`test_pack_requires_bundle_strategy` applies the same bundle-strategy gate before Pack invokes the
Rust packer or prints a pack command.
The project fallback coverage adds `test_cook_assets_derives_project_default_scene_without_manifest`,
confirming that a `zircon-project.toml` with `default_scene = "res://..."` can generate the minimal
CookAssets staged manifest when no explicit cooked manifest is available.
`test_cook_assets_project_manifest_asset_manifest_records_dependency_closure` covers the stronger
project-declared manifest path: a binary default scene with no textual `res://` references can still
publish the recursive material/texture dependency closure through `asset_manifest =
"export/assets.json"`, with the report recording `source_asset_manifest` and preserving
`project_default_scene`.
`test_cook_assets_project_manifest_rejects_unsafe_asset_manifest_path` keeps unsafe project-level
manifest paths fatal before CookAssets can fall back to the old scene scan.
`test_cook_assets_project_fallback_orders_assets_and_dependencies_deterministically` locks the same
fallback to lexicographic `assets[]` and `dependencies[]` ordering before Pack consumes the handoff.
`test_cook_assets_reports_missing_project_default_scene_source` keeps missing fallback sources fatal
at CookAssets instead of letting them leak into Pack.
`test_cook_assets_rejects_asset_manifest_directory` and
`test_cook_assets_rejects_project_manifest_directory` keep directory inputs on the fatal
stage-report path before JSON/TOML parsing.
`test_report_stage_rejects_cook_assets_manifest_asset_filter_mismatch` locks the final Report
publication evidence so CookAssets report `asset_filter` cannot drift from the staged manifest's
filter value after the manifest hash and count evidence are otherwise correct.
`test_report_stage_rejects_cook_assets_manifest_shape_mismatch` covers hand-edited staged manifest
shape drift that can keep hash and count evidence coherent while corrupting roots, assets, or filter
types.
`test_report_stage_rejects_cook_assets_manifest_outside_stage_directory` keeps final Report from
publishing an external side manifest as CookAssets evidence even when the report hash and Pack
handoff are updated to the same file.
`test_report_stage_rejects_cook_assets_manifest_bad_source_path` keeps final Report from publishing
a hash/count-correct `assets.json` whose `source` rows are still relative or point at missing files.
`test_report_stage_rejects_cook_assets_manifest_non_deterministic_order` keeps final Report from
publishing a hash/count-correct `assets.json` whose `roots[]` order, `assets[]` order, per-asset
`dependencies[]` order, or per-asset `labels[]` order no longer matches CookAssets' deterministic
staging contract.
`test_report_stage_rejects_cook_assets_manifest_shape_mismatch` also keeps hand-written staged
manifests from publishing non-normalized `roots[]`, `assets[].path`, or `dependencies[]` values, and
rejects normalized duplicate asset paths, padded `asset_filter`, and padded `labels[]` before Pack
closure evidence can hide the drift; padded `source` fields fail at the same shape boundary before
final Report falls through to absolute-path or missing-file diagnostics. The same test now covers
unknown top-level and asset-row fields, so hand-authored staged manifests cannot smuggle release
metadata past the declared CookAssets schema.
`test_report_rejects_cook_assets_manifest_padded_package_path_field` narrows the package-path
boundary further: padded `roots[]`, `assets[].path`, and `dependencies[]` rows now fail with a
field-level non-empty trimmed string diagnostic before normalized-path or Pack trim-closure
diagnostics can mask the authoring error.
`test_report_rejects_cook_assets_manifest_padded_optional_or_label_string` applies the same
field-level wording to padded `asset_filter`, optional `assets[].source`, and `labels[]` entries,
and verifies final Report does not also fall through to generic trimmed-string, absolute-path, or
Pack trim-report diagnostics for those schema-invalid values.
It also rejects missing root/dependency references at staged-manifest shape time, while the
CookAssets/Pack handoff tests continue to verify Pack trim diagnostics for reports that claim
success over a malformed closure.
`test_report_stage_rejects_pack_trim_included_assets_outside_cook_assets_closure`
keeps final Report cross-stage evidence honest: a non-fatal Pack report whose included assets do not
match the CookAssets roots/dependencies/asset_filter closure is rejected even when the Pack manifest
and trim report are internally consistent.
The Pack handoff regression helper rewrites both `report.manifest` and the embedded `.zrpack`
manifest/payload when it changes trim evidence. That keeps Pack file-evidence schema clean so the
CookAssets-to-Pack closure diagnostics run on the intended cross-stage boundary instead of being
masked by `pack embedded manifest does not match manifest`.
`test_report_stage_rejects_pack_included_asset_missing_cook_assets_source` keeps that same included
asset set tied to Pack-readable source bytes by rejecting a non-fatal Pack report when an included
CookAssets asset row has no `source`.
`test_report_stage_rejects_pack_asset_manifest_drift_from_cook_assets_source` extends that byte
evidence to Pack manifest metadata: included asset `size` and `chunk_hash` must match the actual
CookAssets source bytes rather than a stale hand-written Pack manifest.
`test_report_stage_rejects_cook_assets_source_path_before_pack_source_byte_semantics` keeps that
byte evidence behind the CookAssets source-path gate, so relative or missing `assets[].source`
values do not also emit Pack source-byte read diagnostics.
`test_report_stage_rejects_pack_trimmed_assets_outside_cook_assets_closure` extends the same gate to
the trimmed audit list, including filter-mismatch reasons, so a report cannot publish the right
included set with a misleading trim report.
`test_report_stage_rejects_pack_success_with_cook_assets_missing_dependency` covers the matching
preflight hole: a CookAssets manifest that references a missing asset cannot be paired with a
non-fatal Pack report that claims `missing_dependencies=[]`.
`test_report_stage_rejects_pack_success_with_cook_assets_duplicate_asset` does the same for duplicate
asset paths: a hand-edited CookAssets manifest with repeated `assets[].path` cannot be paired with a
successful Pack report that claims `duplicate_assets=[]`.
`test_report_stage_rejects_pack_trim_diagnostics_outside_cook_assets_closure` keeps the textual trim
diagnostics aligned with the structured audit fields, so a report cannot publish correct
`trimmed_assets` while hiding the corresponding `trimmed asset ...` diagnostic lines.

M5-T2 adds writer-level `delta_pack_contains_only_changed_chunks` for the `ZRPD` chunk-diff package
and Python dry-run coverage `test_pack_delta_args_are_forwarded_to_packer` for `--previous-pack` /
`--delta-pack` pass-through. `test_pipeline_platform_bundle_uses_pack_report_delta_pack_path` extends
the handoff: a non-fatal Pack report `delta_pack` path is now defaulted into PlatformBundle, copied
beside the full `.zrpack`, and recorded in both PlatformBundle `report.json` and `bundle.json`.
`test_pack_rejects_unpaired_previous_pack` and `test_pack_rejects_empty_delta_pack_argument` keep
the Python Pack preflight from dropping malformed delta requests into ordinary full-pack output.
They also assert `command=<skipped>` and no partial `--previous-pack` command preview for invalid
delta inputs.
`test_pack_rejects_previous_pack_resolve_error` and `test_pack_rejects_delta_pack_resolve_error`
cover the next boundary: `previous_pack` and `delta_pack` are canonicalized before command
construction, and any `Path.resolve()` failure is reported as a Pack diagnostic while the dry-run
preview remains `command=<skipped>` with no partial delta arguments.
`test_template_delta_pack_path_controls_bundle_location` verifies that platform templates can route
the copied `.zrpd` through `[bundle].delta_pack_path`, matching the existing host/full-pack path
customization contract.
`test_template_rejects_blank_bundle_path_fields` keeps those `[bundle]` path controls explicit:
`root`, `host_path`, `pack_path`, `delta_pack_path`, and `manifest_path` may be omitted to use their
defaults, but if present they must be non-empty, whitespace-trimmed safe relative paths.
`test_template_rejects_duplicate_bundle_path_entries` applies the same output-side uniqueness to
source `template.toml` files: two distinct `[[files]]` rows cannot publish the same normalized
`bundle_path`, even if the aggregate `content_hash` has been recomputed. This prevents template
selection from accepting a package whose materialization would overwrite one declared template file
with another before final Report aggregation ever runs.
`test_template_rejects_invalid_file_purpose` keeps optional `[[files]].purpose` metadata typed:
when present it must be a non-blank string. The validator no longer coerces non-string TOML values
with `str(...)`, so template package metadata remains auditable instead of being silently rewritten
before it reaches the embedded template report.
`test_report_rejects_template_bundle_string_fields_blank` applies the same rule to hand-authored
PlatformBundle template reports: generated reports may keep empty-string default markers for unset
bundle overrides, but whitespace-only strings are rejected as malformed path evidence.
`test_report_rejects_template_file_required_string_blank` and
`test_report_rejects_template_copied_file_required_string_blank` keep template file evidence on the
same contract: template `files[]` path/bundle path/hash fields and copied `template_files[]`
source/destination paths must be non-empty strings before hash matching or path containment runs.
`test_checked_in_windows_template_routes_delta_pack_path` keeps the checked-in Windows export
template on that same contract, while the Linux and macOS fixtures now declare platform-specific
delta package locations as part of their template bundle metadata.
The runtime pack layer now also has `delta_pack_applies_to_base_pack` and
`delta_pack_rejects_wrong_base_manifest` coverage for the next lower-level hot-update primitive:
`ZrPackDeltaReader::apply_to_base` rebuilds the target full pack from a matching base pack plus
delta payload and refuses mismatched base manifests before any reconstructed bytes are accepted.
The `zircon_export_pack` binary uses that primitive as a writer self-check and reports
`delta_apply_verified`; requested delta output is fatal unless applying the written `.zrpd` to the
previous pack reconstructs the target `.zrpack` bytes.
`test_report_stage_rejects_unverified_delta_pack` keeps the final Report aggregator on the same
contract: a Pack report that contains `delta_pack` but does not carry `delta_apply_verified = true`
marks the pipeline report fatal, even if the Pack stage did not mark itself fatal.
`test_report_stage_rejects_invalid_pack_delta_pack_field` also keeps malformed `delta_pack` values
fatal at final Report aggregation instead of treating a non-string or empty field as "no delta".
`test_report_stage_rejects_platform_delta_without_pack_verification` links the final PlatformBundle
release evidence back to Pack: if PlatformBundle reports a bundled delta package, the same final
Report must contain a non-fatal Pack report with a non-empty `delta_pack` and
`delta_apply_verified = true`. PlatformBundle now records `host_source` / `host_source_origin`,
`pack_source` / `pack_source_origin`, and `delta_pack_source` / `delta_pack_source_origin` in its
stage report and final `bundle.json`.
The final Report stage also treats the PlatformBundle `bundle` root and `bundle_manifest` path as
required release evidence: `bundle` must be the current `<out>/bundle/<profile>` directory, and the
manifest file must exist there, parse as a JSON object, and match the PlatformBundle stage report for
profile, template metadata, bundled host/pack/delta paths, provenance fields, native plugin payload,
and copied template file entries. `test_report_stage_rejects_platform_bundle_without_manifest`,
`test_report_stage_rejects_platform_bundle_manifest_missing_file`,
`test_report_stage_rejects_platform_bundle_manifest_host_mismatch`, and
`test_report_rejects_platform_bundle_root_outside_current_output` keep report-only, stale, or
externally owned PlatformBundle evidence from being accepted as publishable output.
`test_report_stage_rejects_platform_bundle_missing_release_evidence_field` keeps the schema layer on
the same success shape, so missing `bundle`, host/pack provenance, `template_files`, or
`bundle_manifest` is attributed to `PlatformBundle` before downstream semantic checks can lose the
stage ownership of the failure.
`test_report_rejects_platform_bundle_manifest_padded_string_field` applies the same trimmed-string
gate to loaded `bundle.json` evidence: `profile`, host/pack/delta path and provenance fields, and
`native_plugins` must already be non-empty trimmed strings before stage-report comparison and final
payload validation use them.
The same gate verifies final bundle files for the report's `host_executable`, `pack`, and
`delta_pack` fields; `test_report_stage_rejects_missing_platform_bundle_host_output`,
`test_report_stage_rejects_missing_platform_bundle_pack_output`, and
`test_report_stage_rejects_missing_platform_bundle_delta_output` keep a matching report/manifest
pair from passing after the actual bundle payload is deleted or was never materialized.
Those outputs are also compared against their provenance inputs: `host_executable` must match
`host_source`, `pack` must match `pack_source`, and `delta_pack` must match `delta_pack_source` by
SHA-256, and the source side must still exist as a file when present. `test_report_rejects_platform_host_output_hash_mismatch`,
`test_report_rejects_platform_pack_output_hash_mismatch`, and
`test_report_rejects_platform_delta_output_hash_mismatch` keep copied bundle payloads from being
accepted after post-copy mutation; `test_report_rejects_missing_platform_host_source_file`,
`test_report_rejects_missing_platform_pack_source_file`, and
`test_report_rejects_missing_platform_delta_source_file` keep source evidence from disappearing
after PlatformBundle.
Template auxiliary files copied from export templates are rechecked as final bundle outputs too.
When `template_files[]` is present, final Report requires each destination to exist as a file and
match the SHA-256 declared by the corresponding `template.files[]` entry, except for a template host
placeholder that is intentionally overwritten by `host_executable` and is already covered by the
host output gate. `test_report_rejects_missing_template_file_output`,
`test_report_rejects_template_file_hash_mismatch`, and
`test_report_accepts_current_template_file_output` keep template metadata such as macOS
`Info.plist` from drifting after PlatformBundle.
For NativeDynamic bundles, final Report also requires a non-null
`native_plugins_payload.stage_report` to resolve to the current output's
`stages/native_dynamic/report.json`; when that stage-backed field is present, the payload `source`
must be the sibling `plugins` directory from the same NativeDynamic stage report. It also requires
`native_plugins_payload.bundle_path` to resolve to the report's current `native_plugins` directory,
and `native_plugins_payload.loader_manifest` to resolve to the current
`bundle/plugins/native_plugins.toml` file before the top-level payload is exposed.
That final loader manifest is parsed as TOML, `plugins` must be an array of plugin tables with
non-empty string `id` fields, and the resulting `[[plugins]].id` sequence must match
`native_plugins_payload.materialized_packages[].package_id`; malformed TOML or malformed plugin
shape is fatal.
For stage-backed payloads, final Report also requires every normalized
`native_plugins_payload.materialized_packages[]` entry to carry `package_report` before package
path diagnostics run. This keeps final PlatformBundle evidence from downgrading a current
NativeDynamic stage package to a loadable-artifact-only summary that cannot prove its generated
`native_dynamic_package.toml`. The same stage-backed path compares the final payload package id
sequence to the current NativeDynamic report `materialized_packages[].package_id` sequence before
loader or package TOML evidence can re-label the copied bundle packages. It also compares final
`content_hash`, `file_count`, and `file_manifest` with the current NativeDynamic report after
package-report content diagnostics have run, so precise package TOML errors are not hidden by a
coarser stage-copy mismatch.
If a final loader manifest row explicitly carries `path`, `manifest`, `package_report`, or
`[plugins.abi]`, final Report derives the expected `plugins/<package>` values from the final
`bundle/plugins/` root and the projected `materialized_packages[].destination/package_report`
fields, then compares present ABI fields against the fixed ABI v3 contract. Any row-level field
drift is fatal before the top-level payload is exposed, while minimal historical rows that only
carry `id` remain accepted.
When final Report is inspecting legacy Validate output that did not require the NativeDynamic stage
directly, `test_current_output_stage_report_path_reports_plugins_dir_resolve_error` keeps the
expected current-output report path derivation diagnostic if `native_plugins` cannot be
canonicalized.
If a NativeDynamic stage wrapper is already fatal, PlatformBundle no longer consumes its
`native_plugins_payload.stage_report` reference as trusted stage-backed evidence. The existing
NativeDynamic field or location diagnostic remains the source of truth, and final Report avoids
projecting secondary operation-audit or materialized-package fallback diagnostics from a corrupted
stage report. `test_report_does_not_use_fatal_native_dynamic_stage_report_for_payload` locks this
handoff boundary.
For stage-backed payloads, final Report loads the current NativeDynamic stage report and requires
stable `native_signing` and `native_notarization` operation summaries to match the stage report
after normalization, including the case where the PlatformBundle payload omits a summary that is
present in the NativeDynamic report. If the current NativeDynamic report carries a malformed
operation summary, final Report treats that stage evidence as fatal instead of normalizing it away.
Enabled stage operation summaries must also report a `package_count` matching the normalized
`native_plugins_payload.materialized_packages` length; disabled placeholders may keep
`package_count = 0`. A non-fatal NativeDynamic stage report cannot carry a fatal operation summary,
and an enabled operation summary cannot disallow the report target platform; either condition is
treated as corrupted stage evidence. For enabled operation summaries, `platform_allowed` is also
recomputed from `target_platform` and `allowed_platforms`, so hand-authored reports cannot mark an
unsupported platform as allowed.
Explicit directory snapshots without a stage-backed report cannot carry signing or notarization
summaries because there is no stage operation audit to prove them.
It then requires every materialized package
destination to stay under that directory and every package report path to stay inside its package
directory as an existing file. The package-path diagnostic helper resolves that `native_plugins`
root through report diagnostics as well, so a late canonicalization failure is reported instead of
interrupting final Report aggregation. Package `destination` and `package_report` canonicalization
failures use the same `... could not be resolved` diagnostic lane; only successfully resolved paths
that fail containment checks are reported as outside the expected directory. The loadable-artifact
manifest matcher also accepts the final Report diagnostics list, so
`test_loadable_artifact_manifest_match_reports_destination_resolve_error` records destination
canonicalization failures before the broader `loadable_artifacts are not present` summary. It then recomputes the current `bundle/plugins` file manifest and
content hash before exposing the top-level `native_plugins_payload`. A stale or missing plugin
payload, malformed package summary, wrong stage-report path, wrong stage-backed source path, wrong
bundle path, stage-backed package-id drift, stage-backed file-manifest drift, missing, malformed, shape-invalid, mismatched, package-id-drifted, or row-field-drifted loader manifest, spoofed operation audit, external package
destination, external package report, missing package report, stage-backed package-report omission, or `package_count` mismatch makes the pipeline report fatal and suppresses that top-level payload
projection; the stage-level evidence remains under `stages[]` for diagnosis. The focused checks are `test_report_rejects_stale_native_plugins_payload_hash`,
`test_report_rejects_missing_native_plugins_payload_directory`,
`test_report_rejects_native_plugins_payload_package_count_mismatch`,
`test_report_rejects_native_plugins_payload_for_library_embed_strategy`,
`test_report_rejects_native_plugins_payload_stage_report_mismatch`,
`test_report_rejects_native_plugins_payload_source_mismatch_for_stage_payload`,
`test_report_rejects_native_plugins_payload_bundle_path_mismatch`, and
`test_report_rejects_native_plugins_payload_missing_loader_manifest`,
`test_report_rejects_native_plugins_payload_loader_manifest_mismatch`, and
`test_report_rejects_native_plugins_payload_loader_manifest_package_mismatch`,
`test_report_rejects_native_plugins_payload_malformed_loader_manifest`, and
`test_report_rejects_native_plugins_payload_loader_manifest_missing_plugins_table`,
`test_report_rejects_native_plugins_payload_loader_manifest_bad_plugin_id`, and
`test_report_rejects_native_plugins_payload_loader_manifest_path_mismatch`,
`test_report_rejects_native_plugins_payload_loader_manifest_manifest_mismatch`,
`test_report_rejects_native_plugins_payload_loader_manifest_package_report_mismatch`, and
`test_report_rejects_native_plugins_payload_loader_manifest_abi_mismatch`, and
`test_report_rejects_native_plugins_payload_package_destination_outside_plugins`,
`test_report_rejects_native_plugins_payload_package_report_outside_package`,
`test_report_rejects_missing_native_plugins_payload_package_report`,
`test_report_rejects_stage_backed_native_plugins_payload_missing_package_report`,
`test_report_rejects_stage_backed_native_plugins_payload_package_id_drift`,
`test_report_rejects_stage_backed_native_plugins_payload_file_manifest_drift`,
`test_report_rejects_legacy_native_dynamic_stage_report_schema_before_payload_semantics`,
`test_report_rejects_legacy_native_dynamic_operation_audit_schema_before_payload_semantics`,
`test_package_path_diagnostics_rejects_plugins_root_resolve_error`,
`test_package_path_diagnostics_rejects_destination_resolve_error`,
`test_package_path_diagnostics_rejects_package_report_resolve_error`, and
`test_report_rejects_spoofed_native_plugins_payload_signing_audit`,
`test_report_rejects_spoofed_native_plugins_payload_notarization_audit`, and
`test_report_accepts_current_native_plugins_payload`.
`test_report_stage_rejects_platform_host_without_source` and
`test_report_stage_rejects_platform_host_source_mismatch` keep CompileHost-report host handoff tied
to CompileHost stage evidence, while `test_report_stage_allows_platform_argument_host_source`
preserves explicit `--host-executable` as valid manual provenance.
`test_report_stage_rejects_platform_pack_without_source` and
`test_report_stage_rejects_platform_pack_source_mismatch` keep Pack-report handoff full packs tied
to Pack stage evidence, while explicit `--pack-file` inputs remain valid manual provenance.
`test_report_stage_rejects_platform_delta_without_source` and
`test_report_stage_rejects_platform_delta_source_mismatch` keep delta source paths present and
aligned with the Pack report's verified `delta_pack`, because PlatformBundle does not independently
prove that a delta can apply to its base pack.
`cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-native-dynamic-0614` passed with existing warning noise. The
focused lib-test command for the delta test timed out during lib-test compilation before running the
target test.

The Pack profile handoff coverage adds `test_pack_command_forwards_profile_to_packer`, confirming
that the Python stage supplies `--profile <name>` to the Rust packer. The scoped packer check under
`D:\cargo-targets\zircon-export-pack-profile-0615` confirms the Rust `PackArgs` parser and
`ExportPackReport.profile` field compile with the real binary target. A real Pack smoke using the
same target directory confirms that the Python CLI forwards `windows-release` into the Rust packer
and that `<out>/stages/pack/report.json` writes `profile=windows-release`, `fatal=false`, one asset,
and one chunk.
The Rust binary coverage adds `run_rejects_missing_dependency_without_writing_pack`, confirming that
a missing dependency trim report returns exit code `2`, keeps `manifest=null`, reports zero assets and
chunks, records the missing owner/dependency pair, and leaves no `assets.zrpack` behind.
`run_rejects_duplicate_trim_input_without_writing_pack` applies the same no-output gate to duplicate
trim input and verifies that `trim_report.duplicate_assets` contains the rejected package path.

The handoff coverage adds `test_pipeline_pack_uses_cook_assets_report_manifest` and
`test_stage_pack_uses_cook_assets_report_manifest`, confirming that both resume/main-pipeline Pack
execution and standalone `--stage pack` consume CookAssets' `cooked_asset_manifest` report field
when the user did not supply `--asset-manifest`.
`test_pipeline_pack_rejects_invalid_cook_assets_report_manifest_field` keeps malformed CookAssets
report fields from feeding Pack through the default manifest fallback.
`test_stage_pack_rejects_empty_explicit_asset_manifest` and
`test_pipeline_pack_preserves_empty_explicit_asset_manifest` keep empty explicit Pack manifest
inputs from being replaced by CookAssets report defaults or by the fixed CookAssets output path.
`test_pack_rejects_empty_explicit_pack_file` applies the same non-empty rule to Pack's explicit
output path so an empty `--pack-file` cannot silently become the default stage pack.
`test_pack_rejects_blank_explicit_path_arguments` and
`test_pack_rejects_padded_explicit_path_arguments` extend that entry contract to whitespace-only
and padded values for `--asset-manifest`, `--pack-file`, `--previous-pack`, `--delta-pack`,
`--packer`, and `--target-dir`: each explicit path argument must be a trimmed non-empty string
before Pack builds a command or falls back to CookAssets-derived defaults.

`test_pack_reports_missing_asset_manifest_before_packer` confirms Pack writes a fatal stage report
and does not invoke the packer when the cooked manifest is missing. This locks the failure path used
by resume, final Report aggregation, and the editor wizard.
`test_pack_reports_failed_packer_without_stage_report` covers the wrapper fallback after a launched
packer exits non-zero without creating `<out>/stages/pack/report.json`: Python writes a fatal Pack
report with the intended asset manifest, pack path, zero asset/chunk counts, and a diagnostic naming
the missing report instead of returning a bare exit code.
`test_pack_reports_successful_packer_without_stage_report` covers the same report gate when a launched
packer exits with code `0` but still omits `<out>/stages/pack/report.json`: Python now writes the fatal
Pack report and returns exit code `2`, because stage success requires a concrete Pack report for resume
and final Report aggregation.
`test_pack_preflight_failure_report_matches_pack_schema` keeps Python-generated Pack fatal reports
aligned with the Rust report shape by asserting the empty `trim_report.duplicate_assets` list and
`delta_apply_verified=false`.
`test_report_stage_rejects_pack_missing_release_evidence_field` keeps final Report aggregation on the
successful Pack report shape: non-fatal Pack reports that omit `asset_manifest`, `pack`,
`stage_output`, `asset_count`, `chunk_count`, `deduplicated_assets`, `deterministic_double_run`,
`trim_report`, or `manifest` now make `Pack` fatal instead of publishing a partial evidence chain.
`test_report_stage_rejects_pack_stage_output_outside_current_stage` keeps that `stage_output`
evidence bound to the loaded Pack report path, so a non-fatal report cannot point at an external
stage directory while being attributed to the current pipeline output.
`test_pack_rejects_repo_root_resolve_error` covers the Pack entry boundary: explicit engine roots
are canonicalized after the Pack report path is known, and failures keep command construction on
`command=<skipped>` with a `Pack repo_root ... could not be resolved` diagnostic.
`test_pack_rejects_asset_manifest_resolve_error` and `test_pack_rejects_pack_file_resolve_error`
cover explicit Pack path canonicalization before command construction: `asset_manifest` and
`pack_file` `Path.resolve()` failures are reported as diagnostics, the printed path becomes
`<invalid>`, and the packer command is skipped instead of aborting before a Pack report can be
emitted.
`test_pack_rejects_packer_resolve_error` and `test_pack_rejects_target_dir_resolve_error` extend
the same preflight boundary to command tool paths: explicit `packer` binaries and Cargo
`target_dir` overrides are resolved before `pack_command(...)`, and failures keep the Pack stage on
`command=<skipped>` with a field-specific diagnostic.

The Python Report coverage verifies that `--stage report` aggregates all required upstream stage
reports into `<out>/report.json` and `<out>/stages/report/report.json`, includes
`source_template` when present, allows it to be absent for non-SourceTemplate releases, and marks a
missing required upstream stage fatal with a concrete diagnostic.
Final Report orchestration stays in `tools/zircon_export/pipeline_report.py`; stage report
allow-lists and typed schema diagnostics live in
`tools/zircon_export/pipeline_report_stage_schema.py`, so the aggregation path can stay focused on
stage selection, handoff loading, and top-level report assembly.
Basic stage report metadata regressions now live in
`tools/zircon_export/tests/test_pipeline_report_stage_metadata.py`, keeping
`test_pipeline_report_stage.py` focused on cross-stage aggregation, strategy selection, and
provenance checks.
Validate report nested-schema regressions are split into
`test_pipeline_report_validate_schema.py`,
`test_pipeline_report_validate_diagnostics_schema.py`,
`test_pipeline_report_validate_plan_vector_schema.py`,
`test_pipeline_report_validate_native_dynamic_schema.py`, and
`test_pipeline_report_validate_runtime_availability_schema.py`, keeping the base metadata module
focused on stage wrapper contracts and non-Validate stage report schemas.
`test_source_template_stage_requires_source_template_strategy` keeps the SourceTemplate stage itself
from consuming stale generated-project plan rows when the trusted Validate strategy list explicitly
selects only LibraryEmbed. `test_source_template_stage_rejects_invalid_strategy_metadata` applies
the same standalone stage hard gate as final Report for non-list, empty, and unsupported
`profile_summary.strategies` values before any generated project is materialized.
For SourceTemplate profiles, final Report also rechecks the generated project evidence instead of
trusting the stage JSON alone: `validate_report` must point at the current Validate stage report,
`project` must resolve to the current SourceTemplate stage's `project` directory, `generated_files`
must be a list of project-relative paths, and every declared file must still exist as a normal file
under that project. Each generated file row
must carry an integer `size` and a lowercase 64-character hex
`sha256`; final Report recomputes the current file bytes and rejects stale generated projects whose
contents changed after the SourceTemplate stage report was written. Duplicate paths in the
SourceTemplate report are fatal.
When the current Validate report selects the SourceTemplate strategy, final Report also compares the
SourceTemplate report file list with `plan_summary.generated_files`, rejecting both missing planned
generated files and unplanned extra generated files.
That Validate generated-file plan must itself be well-formed: it has to be a list of object entries
with non-empty string paths, paths must be project-relative and stay inside the generated project,
and duplicate planned paths are fatal.
The same final Report path also verifies that the current Validate report still contains
`plan_summary.source_template_build` with a non-empty string-array `command`, non-empty
`manifest_path`, and non-empty `target_dir`, so the SourceTemplate stage report cannot stand alone
without its originating build plan. Command entries must be non-blank, and planned
`--manifest-path`/`--target-dir` values cannot be missing or another option. The Validate build-plan
`manifest_path` must also be relative to the generated project and stay inside it; absolute or
`..`-escaped manifest paths are not trusted release evidence. Its `target_dir` must resolve to the
current SourceTemplate stage `target` directory.
`test_report_rejects_missing_source_template_generated_file`,
`test_report_rejects_source_template_without_project`,
`test_report_rejects_source_template_project_resolve_error`,
`test_report_rejects_source_template_stage_path_resolve_error`,
`test_report_rejects_source_template_project_outside_stage_dir`,
`test_report_rejects_source_template_generated_file_outside_project`,
`test_report_rejects_source_template_generated_file_path_resolve_error`, and
`test_report_rejects_source_template_generated_file_directory` keep missing, escaped, or non-file
generated project reports fatal before a SourceTemplate release is accepted;
the project path itself is resolved through the same diagnostics boundary, so late
`Path.resolve()` failures become `SourceTemplate report project ... could not be resolved`.
The SourceTemplate stage report path used to infer the expected generated project directory also
uses that boundary and reports `SourceTemplate stage report path ... could not be resolved`.
Generated file path canonicalization failures report `SourceTemplate generated file path ... could
not be resolved` before existence, file-type, and hash checks run.
`test_report_rejects_source_template_generated_file_hash_mismatch` keeps post-stage mutations fatal.
`test_report_rejects_source_template_generated_file_missing_size`,
`test_report_rejects_source_template_generated_file_missing_sha256`, and
`test_report_rejects_source_template_generated_file_malformed_content_evidence` keep reports with
missing or malformed content evidence from being accepted.
`test_report_rejects_duplicate_source_template_report_generated_file_path` keeps duplicate
SourceTemplate report file rows fatal.
`test_report_rejects_source_template_generated_file_unknown_field` also closes those report rows:
each SourceTemplate `generated_files[]` entry may only contain `path`, `purpose`, `size`, and
`sha256`, so sidecar fields cannot be carried through the final Report after path/hash checks pass.
`test_report_rejects_source_template_generated_file_missing_field` requires those four row fields
as release evidence before path, file, size, and hash reconciliation can be accepted.
`test_report_rejects_source_template_generated_file_blank_path` keeps SourceTemplate stage-report
generated-file paths on a trimmed non-empty path diagnostic before file existence and hash checks
run.
`test_report_rejects_source_template_generated_file_blank_purpose` keeps stage-report
generated-file purpose metadata trimmed non-empty, matching the row's required release-evidence
contract.
`test_report_rejects_source_template_generated_file_missing_from_plan` and
`test_report_rejects_source_template_unplanned_generated_file` keep the final report aligned with
the Validate-generated project plan. `test_report_rejects_invalid_source_template_validate_generated_files`,
`test_report_rejects_non_object_source_template_validate_generated_file`, and
`test_report_rejects_duplicate_source_template_validate_generated_file_path` keep malformed Validate
plans from being treated as trusted release input.
`test_report_rejects_source_template_validate_generated_file_unknown_field` closes Validate
`generated_files[]` plan rows: each planned generated file may only contain `path`, `purpose`, and
`contents` before it can drive final Report SourceTemplate generated-file reconciliation.
`test_report_rejects_source_template_validate_generated_file_missing_field` requires all three
Validate plan row fields before generated project reconciliation can trust the plan.
`test_report_rejects_source_template_validate_generated_file_blank_path` keeps planned generated
file paths on a trimmed non-empty path diagnostic before later path boundary checks run.
`test_report_rejects_source_template_validate_generated_file_blank_purpose` applies the same
trimmed non-empty rule to Validate generated-file purpose metadata before reconciliation.
`test_report_rejects_absolute_source_template_validate_generated_file_path` and
`test_report_rejects_escaped_source_template_validate_generated_file_path` keep unsafe Validate
planned paths fatal.
`test_report_rejects_source_template_validate_report_path_mismatch` keeps stale SourceTemplate reports
from pointing at a different Validate stage report.
`test_report_rejects_source_template_validate_report_resolve_error` keeps `validate_report` path
resolution failures on the same final Report diagnostic path instead of surfacing filesystem errors.
SourceTemplate final Report now closes stage-report schema at the loader boundary:
`test_report_rejects_source_template_unknown_top_level_field` rejects any field outside
`stage`, `profile`, `fatal`, `diagnostics`, `validate_report`, `project`, `generated_files`,
`command`, `build_executed`, and `build_validation` before the generated project evidence can be
accepted. The same loader-stage schema gate validates `generated_files[]`, `build_validation`, and
the SourceTemplate-specific Validate plan evidence (`source_template_build` and `generated_files[]`).
Bad SourceTemplate stage evidence now marks `SourceTemplate` in `fatal_stages`; bad SourceTemplate
Validate plan evidence marks `Validate` in `fatal_stages`, instead of relying only on later semantic
diagnostics after the stage attribution point has passed. SourceTemplate stage-report schema helpers
live in `tools/zircon_export/pipeline_report_source_template_stage_schema.py`; Validate-side
SourceTemplate plan schema helpers live in
`tools/zircon_export/pipeline_report_source_template_validate_schema.py`; SourceTemplate path,
content, hash, and build-status semantics stay in
`tools/zircon_export/pipeline_report_source_template.py`.
The two schema helpers reuse `pipeline_report_schema_primitives.py` only for ordinary
bool/integer/string/object diagnostics; SourceTemplate-specific non-empty string/string-array,
sha256, `exit_code` nullable-integer, path, and command-shape rules remain local to the
SourceTemplate helpers. `pipeline_report_source_template_string_array_schema.py` carries the shared
SourceTemplate command-array entry-type gate so stage-report commands, nested build-validation
commands, and Validate `source_template_build.command[]` use the same field-shape/entry-type split.
The Validate-side schema helper owns both the object-level `plan_summary.source_template_build`
gate, its required internal build-plan field gates, and the list-level
`plan_summary.generated_files` gate, plus the row-level generated-file allow-list.
Both stage-report and Validate generated-file `path` fields are trimmed before the non-empty check,
matching the SourceTemplate build-plan string evidence gate.
Both generated-file `path` and `purpose` strings must be trimmed before final Report compares file
sets or standalone SourceTemplate materializes the generated project; padded rows stop at the
schema diagnostic instead of producing rewritten Cargo commands or indirect missing-file reports.
Coverage lives in `tools/zircon_export/tests/test_pipeline_report_source_template_schema.py` so the
already-large SourceTemplate behavior test file does not gain another responsibility.
Final Report also validates SourceTemplate `build_validation`: the field must be an object with
typed requested/executed/status/exit-code/working-dir/command values, and `failed` or `blocked`
statuses are not publishable even if the SourceTemplate stage report is otherwise marked non-fatal.
`test_report_rejects_source_template_build_validation_unknown_field` closes this nested object
schema; only `requested`, `executed`, `status`, `exit_code`, `working_dir`, and `command` are
accepted.
It also cross-checks that `build_validation.command` matches the top-level SourceTemplate `command`,
that `build_validation.executed` matches the legacy `build_executed` flag, and that a requested
build is not reported as `skipped`.
The top-level SourceTemplate `command` must be a non-empty string array, include `--manifest-path`
pointing at the current generated `project/Cargo.toml`, and include `--target-dir` pointing at the
current SourceTemplate stage `target` directory; a command that targets another manifest or target
directory is not publishable evidence. Both controlled options must have actual path values, not
another command option, and duplicate controlled options are rejected before path provenance checks.
The nested build evidence must remain semantically publishable as well: its command must be a
non-empty string array, include the same controlled options, pass the same direct option-value
diagnostics, and independently bind `--manifest-path` to the current generated `project/Cargo.toml`
and `--target-dir` to the current SourceTemplate stage `target` directory. The working directory
must resolve to the generated `project`, skipped builds keep `exit_code=null`, and executed builds
must have been requested.
For all three SourceTemplate command arrays, non-array, empty, or blank-entry values keep the
existing `must be a non-empty string array` diagnostic, while non-string entries now report
`<label>[index] must be a string` before trimmed, option-value, manifest-path, target-dir, or
command-match semantics run.
SourceTemplate `build_validation.stdout_lines[]` and `stderr_lines[]` keep the same schema-clean
entry-type contract for captured Cargo logs: non-list fields still report `must be a string array`,
while non-string log rows report `<field>[index] must be a string` and do not also emit the broad
whole-field string-array fallback.
`test_report_rejects_failed_source_template_build_validation` and
`test_report_rejects_malformed_source_template_build_validation` cover the shape/status gates;
`test_report_rejects_source_template_build_validation_command_mismatch`,
`test_report_rejects_source_template_build_validation_execution_mismatch`, and
`test_report_rejects_requested_source_template_build_validation_skip` cover the consistency gates.
`test_report_rejects_source_template_command_manifest_path_outside_project` covers the top-level
command manifest provenance gate.
`test_report_rejects_source_template_command_manifest_path_resolve_error` keeps command
`--manifest-path` canonicalization failures fatal as final Report diagnostics before provenance
comparison.
`test_report_rejects_source_template_report_command_dangling_manifest_path` keeps missing
`--manifest-path` values on the shared command option diagnostic path.
`test_report_rejects_source_template_report_non_string_command_entry_before_array_shape` and
`test_report_rejects_source_template_build_validation_non_string_command_entry_before_array_shape`
keep malformed command entries from collapsing into broad command-array diagnostics.
`test_report_rejects_source_template_report_command_option_value` keeps option-looking
`--manifest-path` values on the same direct command-shape diagnostic path.
`test_report_rejects_source_template_report_command_duplicate_manifest_path` rejects ambiguous
duplicate top-level `--manifest-path` entries instead of trusting the first one only.
`test_report_rejects_source_template_report_command_dangling_target_dir` and
`test_report_rejects_source_template_report_command_target_dir_option_value` apply the same direct
command-shape gate when a top-level SourceTemplate report command carries `--target-dir`.
`test_report_rejects_source_template_report_command_target_dir_mismatch` keeps that optional
`--target-dir` value tied to the current SourceTemplate stage `target` directory when it is present.
`test_report_rejects_source_template_report_command_duplicate_target_dir` confirms duplicate
top-level `--target-dir` entries are rejected through the same shared command diagnostic path.
`test_report_rejects_source_template_build_validation_command_dangling_manifest_path` keeps nested
`build_validation.command --manifest-path` failures on the direct build-validation command
diagnostic path instead of relying only on the broader command mismatch gate.
`test_report_rejects_source_template_build_validation_command_missing_target_dir` keeps the nested
build command on the same direct command-schema path when target-dir evidence is omitted.
`test_report_rejects_source_template_build_validation_command_manifest_mismatch` and
`test_report_rejects_source_template_build_validation_command_target_dir_mismatch` keep nested
build command provenance tied directly to the generated project and stage target, even when the
top-level command mismatch gate would also make the report fatal.
`test_report_rejects_missing_source_template_validate_build_plan`,
`test_report_rejects_malformed_source_template_validate_build_plan_command`,
`test_report_rejects_blank_source_template_validate_build_plan_command_entry`,
`test_report_rejects_source_template_validate_build_plan_missing_required_field`,
`test_report_rejects_source_template_validate_build_plan_blank_required_string_field`,
`test_report_rejects_source_template_validate_build_plan_padded_required_string_field`,
`test_report_rejects_source_template_validate_build_plan_option_value`,
`test_report_rejects_malformed_source_template_validate_build_plan_manifest_path`, and
`test_report_rejects_malformed_source_template_validate_build_plan_target_dir` cover the Validate
build-plan provenance and shape gates.
`test_report_rejects_source_template_validate_build_plan_unknown_field` closes the Validate
`source_template_build` object schema as well: only `manifest_path`, `target_dir`,
`cargo_profile`, `release`, and `command` are accepted as plan evidence, and all five fields are
required on a non-fatal SourceTemplate Validate build plan. The three string fields must be
trimmed non-empty strings, not whitespace placeholders.
`test_report_rejects_source_template_validate_build_plan_non_string_command_entry_before_array_shape`
keeps the Validate handoff command on the same index-level entry-type diagnostic path as stage
report command evidence.
`test_report_rejects_absolute_source_template_validate_build_plan_manifest_path` and
`test_report_rejects_escaped_source_template_validate_build_plan_manifest_path` cover the Validate
build-plan manifest path boundary gate.
`test_report_rejects_source_template_validate_build_plan_target_dir_mismatch` covers the Validate
build-plan target directory provenance gate.
`test_report_rejects_source_template_validate_build_plan_target_dir_resolve_error` keeps
`target_dir` canonicalization failures on the same diagnostic path before the provenance mismatch
comparison runs.
The Validate build-plan final Report regressions now live in
`tools/zircon_export/tests/test_pipeline_report_source_template_validate_build_plan.py`, keeping
the broader SourceTemplate behavior module focused on generated files, project paths, and
profile/stage selection.
`test_report_rejects_empty_source_template_build_validation_command`,
`test_report_rejects_blank_source_template_build_validation_command_entry`,
`test_report_rejects_blank_source_template_report_command_entry`,
`test_report_rejects_source_template_build_validation_working_dir_mismatch`,
`test_report_rejects_source_template_build_validation_working_dir_resolve_error`,
`test_report_rejects_skipped_source_template_build_validation_exit_code`, and
`test_report_rejects_unrequested_source_template_build_validation_execution` cover these semantic
gates.
These SourceTemplate final Report regressions live in
`tools/zircon_export/tests/test_pipeline_report_source_template.py`; stage-schema and
Validate-plan-schema attribution coverage lives in
`tools/zircon_export/tests/test_pipeline_report_source_template_schema.py`; build-validation-only
SourceTemplate final Report gates live in
`tools/zircon_export/tests/test_pipeline_report_source_template_build_validation.py`.
`tools/zircon_export/tests/test_pipeline_report_source_template_command_schema.py` keeps focused
top-level and nested SourceTemplate command schema/provenance regressions from growing the
already-large behavior files.
`tools/zircon_export/tests/test_source_template_plan_string_schema.py` keeps standalone
SourceTemplate pre-materialization string-evidence regressions separate from the larger
SourceTemplate project materialization suite.
`tools/zircon_export/tests/test_source_template_plan_generated_file_schema.py` keeps standalone
generated-file plan row regressions on the same small-module path.
The generic `test_pipeline_report_stage.py` keeps cross-stage aggregation, strategy fallback,
malformed strategy, and NativeDynamic projection coverage; basic metadata gates are split into
`test_pipeline_report_stage_metadata.py`.
Successful non-fatal SourceTemplate stage reports are now a required release-evidence shape:
`project`, `validate_report`, `generated_files`, `command`, `build_executed`,
`build_validation`, `project_cleaned`, and nullable `cleanup_reason` must be present before
final Report follows generated-project evidence. `cleanup_reason = null` is the successful
no-cleanup path, while non-empty strings record fatal cleanup reasons emitted by
`run_source_template(...)`. Missing any required success or cleanup audit field marks
`SourceTemplate` fatal during stage schema loading.
The scalar stage strings `project` and `validate_report`, nullable `cleanup_reason`, and nested
`build_validation.status` / `build_validation.working_dir` must be trimmed before final Report
performs path, status, or build-validation semantic checks.
`test_report_stage_rejects_source_template_missing_release_evidence_field` now also covers missing
`project_cleaned` and `cleanup_reason`, keeping hand-authored stage reports from dropping cleanup
audit evidence before final Report accepts the generated-project output.
`test_report_stage_rejects_source_template_blank_required_string_field` keeps `project` and
`validate_report` on trimmed non-empty string diagnostics, so whitespace-only path evidence cannot
fall through to later generated-project or Validate-report path checks.
`test_report_stage_rejects_source_template_blank_cleanup_reason` keeps cleanup audit evidence
explicit as either `null` or a trimmed non-empty string, rejecting whitespace-only cleanup reasons
before final Report accepts the stage as successful.
SourceTemplate build-validation audit evidence is also explicit: even skipped builds must publish
`exit_code`, `stdout_lines`, and `stderr_lines` so final Report can distinguish "not run" from
"field omitted." `test_report_rejects_source_template_build_validation_missing_audit_field` keeps
those nested fields required without promoting the whole stage report to loader-level fatal before
command, target-dir, and working-dir diagnostics can run.
`test_report_rejects_source_template_build_validation_blank_required_string` keeps
`build_validation.status` and `build_validation.working_dir` on trimmed non-empty string
diagnostics before enum/status or path provenance checks run.
`test_report_stage_ignores_profile_mismatch_validate_strategies` keeps a wrong-profile Validate
report from selecting the current profile's required stages: the mismatched report remains fatal
evidence in `stages[]`, but its stale `profile_summary.strategies` are not used to aggregate
strategy-specific stage reports.
`test_report_stage_rejects_unknown_validate_strategy_without_defaulting` and
`test_pipeline_from_validate_rejects_unknown_strategy_without_defaulting` keep unsupported Validate
strategies fatal without defaulting the final Report or main pipeline back to LibraryEmbed.
`test_report_stage_rejects_empty_validate_strategies_without_defaulting` and
`test_pipeline_from_validate_rejects_empty_strategies_without_defaulting` apply the same hard gate
when `profile_summary.strategies` is present but empty.
`test_report_stage_rejects_invalid_validate_strategies_without_defaulting` and
`test_pipeline_from_validate_rejects_invalid_strategies_without_defaulting` cover malformed
non-list strategy fields.
All stage reports consumed by the final Report must now carry a matching `stage` identity, a
boolean `fatal`, a string-array `diagnostics`, and a string `profile`.
`test_report_stage_rejects_stage_identity_mismatch` keeps a report whose body says `Pack` but lives
at `stages/compile_host/report.json` fatal,
`test_report_stage_rejects_stage_report_without_boolean_fatal` keeps missing or malformed fatal
state from becoming non-fatal evidence,
`test_report_stage_rejects_stage_report_without_string_diagnostics` keeps malformed diagnostic
payloads out of final report display state, while
`test_report_stage_rejects_stage_report_without_profile` keeps old no-profile stage reports fatal,
so per-stage evidence cannot silently attach to whichever profile requested aggregation.
For NativeDynamic releases, `test_report_stage_projects_native_dynamic_release_audit` also keeps the
final pipeline report projecting PlatformBundle's stable `native_plugins_payload` summary at the
top level, so Hub/editor readers do not have to parse the nested PlatformBundle stage entry to find
payload hash, package count, signing, or notarization state.

The Python resume coverage verifies that `--resume-from pack` dry-runs Pack, PlatformBundle, and
Report without replaying earlier stages, that explicit `--stage` and `--resume-from` are rejected
together, that a fatal PlatformBundle resume stops before writing the final pipeline report, and
that `test_resume_from_ignores_stage_outside_validated_strategy` skips stale strategy stages when
the Validate report says the current profile does not request them.
`test_pipeline_from_validate_rejects_invalid_validate_metadata_without_defaulting` and
`test_resume_from_invalid_validate_metadata_does_not_use_fallback_stages` keep existing but
untrusted Validate reports from being treated like a missing report and defaulting to LibraryEmbed
execution. `test_resume_from_validate_report_directory_does_not_use_fallback_stages` applies the
same no-default rule when the Validate report path exists as a directory, so recovery from `pack` or
later stages cannot silently publish stale bundle outputs after a corrupted Validate handoff path.
The same coverage now locks the default command surface: omitting `--stage` dispatches to the main
pipeline starting at Validate instead of silently running only the Validate stage.
It also verifies that PlatformBundle consumes CompileHost's staged `host_executable` report field,
Pack's `pack` and optional `delta_pack` report fields, and NativeDynamic's `plugins_dir` report field
in both pipeline/resume execution and standalone `--stage platform_bundle` runs. Custom stage output
paths therefore survive direct reruns, resume execution, and hot-update artifact handoff instead of
being stranded in the producing stage.
`tools.zircon_export.tests.test_platform_bundle_inputs` covers the inverse explicit-argument
boundary: `test_stage_rejects_empty_explicit_handoff_inputs` and
`test_pipeline_preserves_empty_explicit_handoff_inputs` keep empty host, pack, delta-pack, and
native-plugin directory arguments from being replaced by stage report defaults.
`test_pipeline_explicit_pack_file_does_not_inherit_report_delta` keeps a user-supplied full pack
from accidentally inheriting a stale optional delta package from the Pack stage report. That manual
full-pack path now writes `pack_source` and `pack_source_origin = "argument"` in both the
PlatformBundle stage report and final `bundle.json`, so the release audit records the caller's
source without requiring it to match the current Pack stage report.
When a current-profile CompileHost report exists, `host_executable` is treated as typed handoff
evidence rather than an optional hint: `test_pipeline_platform_bundle_rejects_invalid_compile_host_report_host_field`
keeps a non-string host field from being downgraded to the generic "host executable not supplied"
diagnostic. `test_pipeline_platform_bundle_rejects_compile_host_report_host_resolve_error` keeps a
path canonicalization failure on the same typed handoff diagnostic path. The successful CompileHost
handoff path now records the original CompileHost output as `host_source` with
`host_source_origin = "compile_host_report"` in both PlatformBundle outputs.
Pack handoff is now a hard cutover to profiled reports: `test_pipeline_platform_bundle_ignores_pack_report_without_profile`
keeps legacy Pack reports that omit `profile` from feeding PlatformBundle, so stale pack artifacts
from another export cannot be promoted into the current profile's bundle.
When a current-profile Pack report exists, its `pack` field is also mandatory release evidence:
`test_pipeline_platform_bundle_rejects_invalid_pack_report_pack_field` keeps a non-string or empty
`pack` field from falling back to a stale default `<out>/stages/pack/assets.zrpack` file.
The Pack report `delta_pack` field remains optional, but if a matching Pack report contains it,
PlatformBundle treats it as typed evidence too: `test_pipeline_platform_bundle_rejects_invalid_pack_report_delta_pack_field`
keeps malformed delta handoff from being silently dropped into a full-pack-only bundle.
If the Pack wrapper is already fatal, final Report treats that Pack report as non-publishable
PlatformBundle evidence: `pack_source_origin = "pack_report"` no longer produces a secondary
missing-pack-evidence diagnostic, and a bundled `delta_pack` no longer produces a secondary missing
verified-delta-evidence diagnostic. `test_report_does_not_use_fatal_pack_stage_report_for_pack_source`
keeps the Pack field-level schema diagnostic as the source of truth.
`test_platform_bundle_failure_cleans_previous_profile_bundle` covers repeat execution after a
successful bundle: when a later PlatformBundle run fails, stale host, pack, and final bundle
manifest files from the previous run are removed instead of remaining publishable beside a fatal
stage report. The missing-template-match branch also asserts that pre-materialization fatal paths
leave no empty profile bundle directory and no `bundle_manifest` path in the stage report.

M2-T4 adds `deterministic_pack_double_run_byte_identical` for the writer-level byte guarantee and a
real CLI Pack smoke with `--determinism-check`. The smoke passed on 2026-06-14 with
`fatal=false`, two included assets, one trimmed unused/editor-only asset, and a successful
`deterministic pack double-run byte comparison passed` diagnostic. PlatformBundle has a stage report
and directory layout, but startup-to-first-frame validation remains deferred until CompileHost and
CookAssets can feed it real host and cooked asset outputs.

M3-T1 adds `tools.zircon_export.tests.test_templates`. The plan-named
`template_version_mismatch_rejected` check mutates a copied template manifest to
`format_version = 999` and asserts a fatal validation report. The valid-template check proves the
checked-in `tools/zircon_export/export-templates/windows-x86_64-library_embed-debug/template.toml` resolves its declared
host path and computed content hash. `test_template_rejects_aliasing_file_and_host_paths` mutates a
copied template to use `bin/./zircon_runtime.host-placeholder` for both host and file paths, then
recomputes the aggregate content hash; validation must still fail so path aliases cannot become
part of a version-locked template package. `test_template_rejects_declared_directory_file` replaces
the declared host file with a directory and keeps that malformed template on the fatal report path
before SHA-256 hashing or PlatformBundle copy begins. Template manifest reads have the same file
gate: `test_template_rejects_manifest_directory` replaces `template.toml` with a directory and
expects a fatal `export template manifest ... is not a file` diagnostic before TOML parsing.

M3-T2 layout coverage in the same Python test module checks the signed-in
`linux-x86_64-library_embed-debug` and `macos-aarch64-library_embed-debug` fixtures. It verifies the
Linux directory output paths and the macOS `.app/Contents` output paths, including template-file copy
for `Info.plist`. These are layout tests only, not executable launch tests.

M3-T2 template-root coverage verifies that `--template-root tools/zircon_export/export-templates --target-platform
linux-x86_64` resolves the checked-in Linux template, records `template_resolution`, materializes the
Linux directory layout, and returns a fatal report when no compatible profile/platform template is
found. `test_template_root_skips_invalid_matching_template_candidate` keeps template-root resolution
from letting a corrupted template package shadow a valid package for the same profile/platform:
matching candidates are fully validated before selection, invalid matches are recorded under
`template_resolution.skipped_candidates`, and only valid candidates participate in the duplicate
candidate check. `test_template_root_skips_matching_candidate_with_blank_profile_entry` locks that
the same skipped-candidate audit also carries `compatible_profiles` blank-entry diagnostics for a
candidate that otherwise matches profile/platform. `test_template_root_skips_malformed_template_manifest` applies the same audit path
to a child template whose `template.toml` cannot be parsed; a malformed package is visible in
`skipped_candidates` but does not block a separate valid template from being selected.
`test_template_root_skips_manifest_directory_candidate` applies the same skipped-candidate audit to
a child template whose `template.toml` path is a directory, so template-root selection never lets an
OS read error from one malformed package block a valid package for the same profile/platform.
`test_template_root_rejects_workspace_manifest_directory` covers the separate engine-version
fallback boundary: if `--engine-version` is omitted and `--repo-root/Cargo.toml` is a directory,
PlatformBundle reports `workspace manifest ... is not a file`, leaves `template_resolution = null`,
and removes the profile bundle instead of raising a filesystem exception.
`test_template_root_ignores_target_platform_from_wrong_profile_validate_report` keeps
PlatformBundle from using a wrong-profile Validate report's `profile_summary.target_platform` during
automatic template-root selection; the fatal report keeps the Validate profile mismatch diagnostic
and leaves `template_resolution = null` rather than emitting stale template evidence.

The PlatformBundle/NativeDynamic handoff tests now live in
`test_platform_bundle_native_dynamic.py`, while stage operation-audit consistency tests live in
`test_platform_bundle_native_dynamic_operation_audit.py`, and final bundle loader-manifest payload
evidence lives in `test_platform_bundle_native_payload_loader_manifest.py`; `test_templates.py`
stays limited to export-template validation, template-root selection, and checked-in platform
layout materialization.
PlatformBundle NativeDynamic handoff coverage adds
`test_platform_bundle_copies_native_dynamic_plugins_dir` and
`test_pipeline_platform_bundle_uses_native_dynamic_report_plugins`. Together they verify that an
explicit `--native-plugins-dir` is copied into the final bundle `plugins/` directory and that
pipeline execution can default that input from a non-fatal NativeDynamic stage report.
PlatformBundle also preflights the host executable, full pack, and optional delta pack as file
inputs before copying any template or release payload data. Directory paths now produce fatal stage
diagnostics such as `is not a file`, write `report.json`, and remove the partially created bundle
directory instead of surfacing a Python copy exception. The regression coverage is
`test_platform_bundle_rejects_host_directory_input`,
`test_platform_bundle_rejects_pack_directory_input`, and
`test_platform_bundle_rejects_delta_pack_directory_input`.
The explicit-directory path now also records a directory-level `native_plugins_payload` with the
current loader manifest path, content hash, file manifest, package count, and loadable artifact
audit in both the PlatformBundle stage report and `bundle.json`. After PlatformBundle copies the
plugins payload, the final payload's `loader_manifest` and `materialized_packages` summary rewrite
package `destination` and `package_report` paths to the final `bundle/plugins/...` locations while
keeping `source` and `stage_report` as upstream provenance.
`test_platform_bundle_records_loader_manifest_in_native_plugins_payload` verifies the generated
stage report and bundle manifest carry the final `bundle/plugins/native_plugins.toml` path. When a
final Report later sees that bundle loader manifest with a missing `plugins` array, a bad plugin
`id`, a different `[[plugins]].id` sequence, malformed TOML, explicit `path`/`manifest`/
`package_report` values that no longer match the final bundle package projection, or explicit
`[plugins.abi]` values that no longer match ABI v3,
`test_loader_manifest_helpers_reject_blank_path_before_resolution` first keeps direct helper calls
from resolving whitespace-only `loader_manifest` values as the current working directory; both the
path-binding helper and package-binding helper return the same field-level non-empty diagnostic.
`test_report_rejects_native_plugins_payload_loader_manifest_missing_plugins_table`,
`test_report_rejects_native_plugins_payload_loader_manifest_bad_plugin_id`,
`test_report_rejects_native_plugins_payload_loader_manifest_package_mismatch`, and
`test_report_rejects_native_plugins_payload_malformed_loader_manifest`,
`test_report_rejects_native_plugins_payload_loader_manifest_path_mismatch`,
`test_report_rejects_native_plugins_payload_loader_manifest_manifest_mismatch`, and
`test_report_rejects_native_plugins_payload_loader_manifest_package_report_mismatch`, and
`test_report_rejects_native_plugins_payload_loader_manifest_abi_mismatch` keep the
top-level payload projection suppressed.
When a NativeDynamic payload is present, PlatformBundle recreates the final
`bundle/plugins/` directory immediately before copying that payload, so template-provided files under
the same directory cannot remain as untracked release plugins; template file report entries targeting
that removed directory are also dropped so `template_files` only describes files still present in the
final bundle. `test_platform_bundle_rejects_malformed_native_dynamic_report` keeps a matching
malformed NativeDynamic report fatal so damaged stage evidence cannot be bypassed by an explicit
directory argument, including malformed base stage metadata such as non-boolean `fatal` or
non-array `diagnostics`. `test_platform_bundle_rejects_native_dynamic_report_directory` applies the
same rule when the matching current NativeDynamic `report.json` path is a directory.
`test_platform_bundle_explicit_native_dir_uses_bundle_plugin_paths` verifies that a manual source
directory such as `manual-native-payload/` still produces `plugins/...` payload paths and matching
loadable artifact paths, so hashes and release audit metadata describe the final bundle layout
rather than a caller's temporary directory name.
`test_platform_bundle_explicit_native_dir_rejects_payload_rewrite_resolve_error` keeps that rewrite
fatal when a package `destination` or `package_report` cannot be canonicalized relative to the
explicit source directory, preventing source-local paths from leaking into `bundle.json`.
`test_platform_bundle_native_plugins_replaces_template_plugins_dir` verifies that template files
copied into `plugins/` are removed before the NativeDynamic payload is copied, keeping the final
plugin directory and `template_files` report aligned with `native_plugins_payload.file_manifest`.
`test_platform_bundle_rejects_template_plugins_filter_resolve_error` keeps that replacement fatal
when a stale template `plugins/...` destination or the replaced `plugins/` directory cannot be
canonicalized during `template_files` filtering, preventing deleted template plugin records from
surviving into stage reports or `bundle.json`.
`test_pipeline_platform_bundle_preserves_native_dynamic_payload_hash` extends the handoff so the
final PlatformBundle report and `bundle.json` retain the staged NativeDynamic payload hash, file
manifest metadata, per-package loadable artifact audit, and stable signing/notarization operation
summaries. `test_pipeline_platform_bundle_rejects_native_payload_destination_summary_resolve_error`
keeps that stage-backed payload summary from publishing when a materialized package destination
cannot be canonicalized during the pre-copy loadable-artifact audit; PlatformBundle reports the
underlying `NativeDynamic payload ... destination ... could not be resolved` diagnostic and clears
the partial bundle. `test_pipeline_platform_bundle_rejects_stale_native_dynamic_payload_hash` keeps
a mutated staged plugin payload fatal before PlatformBundle copies it into the final bundle.
`test_pipeline_platform_bundle_rejects_malformed_native_dynamic_signing_audit`,
`test_pipeline_platform_bundle_rejects_malformed_native_dynamic_notarization_audit`, and
`test_pipeline_platform_bundle_rejects_native_dynamic_signing_package_count_mismatch` keep damaged
or count-inconsistent operation summaries from becoming bundle-level release audit metadata, while
`test_pipeline_platform_bundle_accepts_disabled_native_dynamic_signing_placeholder` preserves the
legitimate disabled-operation placeholder emitted by NativeDynamic when no signer is configured.
`test_pipeline_platform_bundle_rejects_fatal_native_dynamic_signing_audit` and
`test_pipeline_platform_bundle_rejects_disallowed_native_dynamic_signing_platform` keep
non-fatal NativeDynamic reports from handing PlatformBundle a fatal or target-disallowed signing
audit. `test_pipeline_platform_bundle_rejects_spoofed_native_dynamic_signing_platform_allowed`
keeps PlatformBundle from accepting a spoofed `platform_allowed=true` value when the target platform
does not match `allowed_platforms`.
`test_report_rejects_missing_native_plugins_payload_signing_audit` and
`test_report_rejects_missing_native_plugins_payload_notarization_audit` keep final Report from
projecting a stage-backed payload that drops operation summaries present in the current
NativeDynamic report. `test_report_rejects_malformed_native_dynamic_report_signing_audit` and
`test_report_rejects_malformed_native_dynamic_report_notarization_audit` keep malformed stage
operation summaries fatal even if the PlatformBundle payload itself omitted those fields.
`test_report_rejects_native_dynamic_report_signing_package_count_mismatch` and
`test_report_accepts_disabled_native_dynamic_report_signing_placeholder` lock the final Report
version of the enabled-package-count rule while preserving legitimate disabled operation
placeholders. `test_report_rejects_fatal_native_dynamic_report_signing_audit` and
`test_report_rejects_disallowed_native_dynamic_report_signing_platform` apply the same
fatal/platform gate at final Report aggregation time for old or hand-authored PlatformBundle
reports. `test_report_rejects_spoofed_native_dynamic_report_signing_platform_allowed` applies the
computed platform gate during final Report aggregation too.
`test_pipeline_platform_bundle_rejects_profile_mismatch_native_dynamic_report` keeps an explicit
handoff of `<out>/stages/native_dynamic/plugins` from bypassing a same-directory NativeDynamic
report that belongs to another profile; only genuinely independent manual plugin directories may
fall back to directory-level payload snapshots.
`test_pipeline_platform_bundle_rejects_inherited_native_dynamic_report_directory` keeps the
automatic NativeDynamic handoff path from swallowing a directory report path into a generic missing
payload diagnostic.
`test_pipeline_platform_bundle_rejects_invalid_native_dynamic_metadata` applies the same rule to
malformed NativeDynamic report metadata before PlatformBundle copies staged plugin payloads.
`test_platform_bundle_rejects_native_dynamic_package_report_directory` keeps explicit directory
snapshots from accepting a package-level `native_dynamic_package.toml` path that is not a file.
`test_report_rejects_native_plugins_payload_unknown_top_level_field` keeps the PlatformBundle
`native_plugins_payload` object on a closed-schema release-evidence path: top-level keys must be the
audited payload fields (`bundle_path`, loader manifest, content/file/package summaries, source/stage
handoff, package list, and signing/notarization audits), so unaudited sidecar metadata cannot be
projected into the final pipeline report.
`test_report_rejects_native_plugins_payload_materialized_package_unknown_field` closes each
`native_plugins_payload.materialized_packages[]` entry as well: package rows may contain only
`package_id`, `destination`, optional `source`/`package_report`, and loadable-artifact count/list
evidence, so row-local sidecar metadata cannot be dropped by materialized-package normalization.
`test_report_rejects_native_plugins_payload_file_manifest_unknown_field` closes the top-level
`native_plugins_payload.file_manifest[]` entries too. Bundle payload file rows may contain only
`path`, `bytes`, and `sha256`, so sidecar fields on final bundle file evidence are rejected before
the payload is projected into the final report.
`test_report_rejects_native_plugins_payload_operation_audit_unknown_field` closes the stable
`native_signing` and `native_notarization` summaries carried inside `native_plugins_payload`: each
summary may contain only `enabled`, `profile`, `target_platform`, `allowed_platforms`,
`platform_allowed`, `fatal`, and `package_count`, so command-level or platform-service sidecar data
cannot be projected into the top-level release payload.
`test_report_rejects_native_plugins_payload_file_manifest_non_object_array`,
`test_report_rejects_native_plugins_payload_file_manifest_field_types`,
`test_report_rejects_native_plugins_payload_file_manifest_missing_required_field`,
`test_report_rejects_native_plugins_payload_materialized_packages_non_object_array`,
`test_report_rejects_native_plugins_payload_materialized_package_field_types`,
`test_report_rejects_native_plugins_payload_materialized_package_missing_required_field`,
`test_report_rejects_native_plugins_payload_operation_audit_non_object`,
`test_report_rejects_native_plugins_payload_operation_audit_field_types`, and
`test_report_rejects_native_plugins_payload_operation_audit_missing_required_field` close the typed shape of
that same release payload. `file_manifest[]` and `materialized_packages[]` must be object arrays,
file rows must keep `path`/`sha256` strings and `bytes` integers, materialized package rows must
keep string path/id fields, integer loadable-artifact counts, and string-array loadable artifacts,
and signing/notarization summaries must keep their string/boolean/integer/string-array field types
before final Report trusts the NativeDynamic payload. Stable audit summaries must also carry
`enabled`, `allowed_platforms`, `platform_allowed`, `fatal`, and `package_count`; `profile` and
`target_platform` remain nullable/optional metadata fields. These NativeDynamic payload schema rules
live in `pipeline_report_native_dynamic_payload_schema.py`; signing/notarization operation-audit
schema rules live in `pipeline_report_native_dynamic_operation_audit_schema.py`; bundled
`native_dynamic_package.toml` top-level, `[abi]`, `[payload]`, and package-local
`[[payload.files]]` schema diagnostics live in
`pipeline_report_native_dynamic_package_report_schema.py`.
`pipeline_report_native_dynamic_payload_stage_report.py` owns current NativeDynamic stage-report
comparisons used by final PlatformBundle payload diagnostics. `pipeline_report_native_dynamic_payload.py`
keeps the path containment, hash, package-report, and bundle payload consistency checks and imports
those focused schema/stage-report modules.
Shared table/sequence row dispatch and unknown-field attribution live in
`pipeline_report_schema_table.py`, which imports the common bool/integer/string/string-array/object
and object-array diagnostic primitives from `pipeline_report_schema_primitives.py`.
`test_report_rejects_native_plugins_payload_content_hash_non_string_without_semantic_fallback`
locks the handoff between those layers: wrong-typed top-level payload fields stop at dotted schema
diagnostics such as `native_plugins_payload.content_hash must be a string` and do not continue into
non-empty/hash semantic checks that assume the field type is already trustworthy.
`test_report_rejects_native_plugins_payload_padded_top_level_string` lives in
`test_pipeline_report_native_dynamic_payload_top_level_trimmed_schema.py` and keeps top-level
`bundle_path`, `content_hash`, `loader_manifest`, `source`, and `stage_report` normalized before
path resolution, hash shape, bundle/stage matching, loader-manifest reads, or payload projection.
`test_report_rejects_native_plugins_payload_file_manifest_padded_path` and
`test_report_rejects_native_plugins_payload_file_manifest_padded_sha256` live in
`test_pipeline_report_native_dynamic_payload_file_manifest_trimmed_schema.py` and keep top-level
payload `file_manifest[].path` / `sha256` rows trimmed before bundle/stage manifest matching or
SHA-256 shape checks consume them.
`test_report_rejects_native_plugins_payload_materialized_package_padded_loadable_artifact` lives in
`test_pipeline_report_native_dynamic_payload_materialized_trimmed_schema.py` and keeps
`materialized_packages[].loadable_artifacts[]` rows trimmed before payload membership checks report
broader "loadable_artifacts are not present" diagnostics.
`test_report_rejects_native_plugins_payload_materialized_package_padded_string_field` uses the same
trimmed schema test module to keep materialized package `package_id`, `destination`,
`package_report`, and `source` values normalized before package-id, path containment,
package-report, or stage-backed drift checks consume them.
The operation-audit branch follows the same rule. `test_report_rejects_native_plugins_payload_operation_audit_field_types`
now asserts that wrong-typed `native_signing`/`native_notarization` summary fields report their
field-level boolean/string/integer/string-array schema diagnostics without also emitting the broad
`native_signing is malformed` fallback. `test_report_rejects_native_plugins_payload_operation_audit_missing_required_field`
extends that gate to incomplete stable summaries, so missing `enabled`, `allowed_platforms`,
`platform_allowed`, `fatal`, or `package_count` is reported as a direct typed field diagnostic rather
than broad malformed payload evidence. `test_report_stage_rejects_native_dynamic_operation_audit_package_missing_required_field`
locks the full-stage audit package row gate: missing `package_id`, `artifact_count`, or `artifacts`
marks the NativeDynamic stage fatal before package-level operation evidence can be trusted.
`test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_required_field` extends
the same gate into operation artifact rows: missing `artifact`, `package_relative_artifact`,
`stdout`, `stderr`, or `command` is rejected while nullable hash/exit-code failure evidence remains
allowed.
The nested list evidence is also gated this way. `file_manifest[]` and `materialized_packages[]`
schema helpers stay in `pipeline_report_native_dynamic_payload_schema.py`, while their shared row
dispatch lives in `pipeline_report_schema_table.py`; bundled
package-report schema lives in `pipeline_report_native_dynamic_package_report_schema.py` and reuses
the same file-manifest row contract for package-local payload files. Final Report only calls the
normalized manifest/package fallback after the matching object-array schema has no shape or
field-type diagnostics. The file-manifest and materialized-package schema tests now assert
that object-array and row-field type errors do not also emit `file_manifest is malformed` or
`materialized_packages are malformed`; the file-manifest test also covers missing `path`, `sha256`,
or `bytes` fields so incomplete rows are not accepted as publishable NativeDynamic payload evidence.
The materialized-package test mirrors that rule for the package audit itself: `package_id`,
`destination`, `loadable_artifact_count`, and `loadable_artifacts` are required release evidence,
while `source` and `package_report` remain optional typed fields at the reusable payload-schema
layer for legacy/manual directory payloads. The NativeDynamic stage payload gate tightens the current
generated stage shape and requires both fields before a non-fatal stage report can be trusted.
`test_report_rejects_native_plugins_package_report_unknown_top_level_field` applies the same
closed-schema rule inside each bundled `native_dynamic_package.toml`: top-level TOML keys must be
one of the package-report fields audited by final Report (`format_version`, identity/locator fields,
`[payload]`, and `[abi]`), so package-level sidecar metadata cannot hide behind only an outer payload
hash mismatch.
`test_report_rejects_native_plugins_package_report_format_version_non_integer` closes the typed
shape of that top-level package report. `format_version`, when present, must be an integer before
final Report compares it to the supported package-report format version, so string values do not
fall through as misleading unsupported-version diagnostics; the test also asserts that
`format_version 1 is not supported; expected 1` is not emitted for the wrong-typed string case.
`test_report_rejects_native_plugins_payload_package_report_missing_required_field` extends that
gate to the generated package-report header: missing `format_version`, `package_id`, `directory`,
`path`, `manifest`, `[abi]`, or `[payload]` now emits a direct typed diagnostic instead of being
accepted as legacy or discovered only through later payload drift.
`test_report_rejects_native_plugins_package_report_top_level_blank_strings` tightens the same
header for generated reports whose required locator strings are present but whitespace-only:
`package_id`, `directory`, `path`, and `manifest` now report direct
`package_report.<field> must be a non-empty string` diagnostics and no longer fall through to
identity/path mismatch diagnostics.
`test_report_rejects_native_plugins_package_report_padded_top_level_string` extends that gate to
non-empty strings with leading/trailing whitespace, requiring `package_id`, `directory`, `path`, and
`manifest` to be trimmed before package identity, safe-relative path, or materialized package
location checks consume them.
`test_report_rejects_native_plugins_package_report_abi_string_field_types` closes the typed shape
of the generated package-report `[abi]` contract table. ABI v3 contract fields must be strings
before final Report checks them against `NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS`, so numeric or
boolean TOML values report direct `abi.<field> must be a string` schema diagnostics instead of
being conflated with missing/empty-string contract failures.
`test_report_rejects_native_plugins_package_report_abi_string_field_blank` applies the field-level
non-empty gate to ABI v3 string contracts as well: whitespace-only ABI contract values report
`abi.<field> must be a non-empty string` before constant-value comparison.
`test_report_rejects_native_plugins_package_report_abi_padded_string` applies the same normalization
rule to padded ABI v3 string contracts, so constant-value comparison does not hide an unnormalized
package report row.
`test_report_rejects_native_plugins_payload_package_report_abi_missing_required_field` applies the
same required-field rule to `[abi]`: `abi_version` and every ABI v3 string contract field must be
present before final Report compares the package report against the expected ABI v3 constants.
`test_report_rejects_native_plugins_package_report_payload_unknown_field` and
`test_report_rejects_native_plugins_package_report_abi_unknown_field` close the generated
package-report `[payload]` and `[abi]` tables too. `[payload]` may contain only `file_count`,
`content_hash`, and `[[payload.files]]`, and `[abi]` may contain only `abi_version` plus the shared
ABI v3 string contract fields; unknown table-local keys become direct package-report diagnostics
instead of only surfacing as outer payload hash drift.
`test_report_rejects_native_plugins_package_report_payload_content_hash_non_string` closes the typed
shape of the generated package-report `[payload]` table itself. `content_hash` must be a string and
`file_count` must be an integer before final Report compares package-local payload bytes against the
current package directory; wrong-typed hash values now report `payload.content_hash must be a string`
instead of falling into the missing/empty string semantic check.
`test_report_rejects_native_plugins_package_report_payload_content_hash_blank` covers the string
case after typing: whitespace-only `payload.content_hash` now reports
`payload.content_hash must be a non-empty string` before package payload hash comparison.
`test_report_rejects_native_plugins_package_report_payload_padded_content_hash` rejects a padded
package-local payload hash before SHA-256 shape validation or package payload comparison.
`test_report_rejects_native_plugins_package_report_payload_content_hash_malformed` adds the hash
shape boundary after that non-empty gate: generated package reports must publish a SHA-256 hex
digest before final Report compares the digest against the current package payload bytes.
`test_report_rejects_native_plugins_package_report_payload_file_count_negative` adds the matching
integer range gate for generated package reports: `[payload].file_count` must be non-negative before
final Report treats it as package-local payload count evidence.
`test_report_rejects_native_plugins_payload_package_report_payload_missing_required_field` extends
that typed gate to missing generated `[payload]` header fields: `file_count` and `content_hash` must
both be present before package-local payload evidence can be compared.
`test_report_rejects_native_plugins_package_report_payload_file_unknown_field` extends the same
closed-schema rule to generated `[[payload.files]]` entries: each file row may contain only `path`,
`bytes`, and `sha256`, so file-level sidecar fields are reported as direct package-report
diagnostics before the final bundle audit falls back to outer payload hash drift.
`test_report_rejects_native_plugins_package_report_payload_files_non_object_array` and
`test_report_rejects_native_plugins_package_report_payload_file_duplicate_path` keep
`[[payload.files]]` itself trustworthy: file rows must be TOML object entries, and their `path`
values must be unique before final Report compares the generated package report to current package
bytes.
`test_report_rejects_native_plugins_package_report_payload_file_field_types` close the typed shape
of those generated payload file rows. `[payload].files` must be an object array, every row must be a
TOML table, `path` and `sha256` must be strings, and `bytes` must be an integer before final Report
uses package-report payload file evidence for content-hash and file-manifest comparison. The same
tests now also assert that shape/type failures do not fall through to `payload files are malformed`;
that fallback is reserved for schema-clean payload file evidence that still cannot normalize.
`test_report_rejects_native_plugins_package_report_payload_file_blank_strings` applies the same
field-level non-empty rule to `[[payload.files]]` identity/hash strings: whitespace-only `path` or
`sha256` now reports `payload files[0].<field> must be a non-empty string` before package payload
file-manifest comparison.
`test_report_rejects_native_plugins_package_report_payload_file_padded_string` extends that
file-row gate to non-empty padded `path` and `sha256` values, keeping package-local file manifest
evidence normalized before safe-relative, SHA-256, duplicate, and payload comparison checks run.
`test_report_rejects_native_plugins_package_report_top_level_unsafe_relative_paths` and
`test_report_rejects_native_plugins_package_report_payload_file_unsafe_path` apply the shared export
safe-relative path contract to bundled package reports. Top-level locator fields `directory`,
`path`, and `manifest`, plus each `[[payload.files]].path`, must be non-absolute relative paths
without empty, `.`, or `..` segments before final Report compares them against materialized package
locations or package-local file manifests.
`test_report_rejects_native_plugins_package_report_payload_file_negative_bytes` applies the same
range contract to package-local file rows: `[[payload.files]].bytes` must be non-negative before
file-manifest evidence can participate in current package payload comparison.
`test_report_rejects_native_plugins_package_report_payload_file_malformed_sha256` gives
`[[payload.files]].sha256` the same SHA-256 hex shape gate before package payload file-manifest
comparison.
`test_report_rejects_native_plugins_payload_package_report_payload_file_missing_required_field`
extends the same required-field gate to bundled `native_dynamic_package.toml` payload rows, so
partial package-local payload file evidence cannot be treated as a hash-only package report.
`test_report_rejects_native_plugins_payload_package_report_package_id_mismatch` keeps final Report
from accepting a bundled package report whose TOML `package_id` no longer matches the corresponding
`native_plugins_payload.materialized_packages[]` entry; the direct package-report diagnostic is
emitted before callers have to infer identity drift from payload hash/file-manifest mismatches.
`test_report_rejects_native_plugins_payload_package_report_payload_count_mismatch` extends that
package-report content audit to the generated `[payload]` table: when present, its `file_count`,
`content_hash`, and optional `[[payload.files]]` evidence must match the current package directory
contents excluding `native_dynamic_package.toml` itself.
`test_report_rejects_native_plugins_payload_package_report_directory_mismatch` covers the package
locator fields in the same final Report pass: a production package report that declares `directory`
must name the current package directory relative to the bundle `plugins/` root.
`test_report_rejects_native_plugins_payload_package_report_path_mismatch` extends the locator audit
to `path` and `manifest`, which must match `plugins/<directory>` and
`plugins/<directory>/plugin.toml` whenever those production fields are present.
`test_report_rejects_native_plugins_payload_package_report_format_version_mismatch` keeps the same
package-report content pass version-aware: `format_version` must be present and must be `1`,
matching the current NativeDynamic package report generator.
`test_report_rejects_native_plugins_payload_package_report_abi_version_mismatch` applies the same
final Report audit to a production `[abi]` table: when present, it must describe ABI v3 and all
shared `NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS` string contracts.
`test_report_rejects_native_plugins_payload_loader_manifest_bad_abi_table` applies the shared
loader-manifest shape gate to the final bundle: if a loader row carries `abi`, it must be a TOML
table before final Report compares nested ABI v3 fields.
`test_report_rejects_native_plugins_payload_loader_manifest_unknown_abi_field` keeps that loader
ABI table closed to the shared ABI v3 contract, so bundle-local sidecar keys are reported before
the top-level `native_plugins_payload` is projected.
`test_report_rejects_native_plugins_payload_loader_manifest_unknown_plugin_field` closes the
loader row itself: final bundle `[[plugins]]` entries may only carry `id`, `path`, `manifest`,
`package_report`, and `abi`.
`test_report_rejects_native_plugins_payload_loader_manifest_string_field_type` keeps optional
loader path fields typed: if `path`, `manifest`, or `package_report` appears, it must be a string
before final Report compares the field to the materialized package projection.
`test_report_rejects_native_plugins_payload_loader_manifest_blank_string_fields` applies the same
trim-non-empty rule to `[[plugins]].id`, `path`, `manifest`, and `package_report`, so blank loader
rows fail as field-level evidence instead of later package-id or row-field drift.
`test_report_rejects_native_plugins_payload_loader_manifest_abi_field_types` applies the same typed
gate inside `[plugins.abi]`: `abi_version` must be an integer and ABI v3 contract fields must be
strings before final Report compares them to fixed NativeDynamic ABI constants.
`test_report_rejects_native_plugins_payload_loader_manifest_abi_blank_strings` then keeps those ABI
v3 string contracts trimmed and non-empty, so blank ABI evidence is reported before constant
mismatch diagnostics.
`test_report_rejects_native_plugins_payload_loader_manifest_abi_missing_required_field` also treats
an explicit loader ABI table as complete-by-contract: every ABI v3 field must be present once the
table exists.
`test_report_rejects_native_plugins_payload_loader_manifest_unknown_top_level_field` closes the
loader document root too: top-level TOML keys other than `plugins` are rejected before the final
`native_plugins_payload` projection.
`test_report_rejects_stage_backed_native_plugins_payload_loader_manifest_missing_row_field` and
`test_report_rejects_stage_backed_native_plugins_payload_loader_manifest_missing_abi_table` tighten
the stage-backed final bundle path: once `native_plugins_payload.stage_report` matches the current
NativeDynamic report, the copied final `bundle/plugins/native_plugins.toml` row must still carry
`path`, `manifest`, `package_report`, and the ABI v3 table instead of downgrading to an id-only
loader row.
`test_report_rejects_stage_backed_native_plugins_payload_missing_package_report` applies the same
stage-backed rule one level earlier: a copied final payload backed by the current NativeDynamic
stage must retain each materialized package `package_report`, so final Report can validate the
package TOML before projecting the top-level `native_plugins_payload`.
`test_report_rejects_stage_backed_native_plugins_payload_package_id_drift` keeps that copied final
payload tied to the stage report package identity even if the final loader manifest, package TOML,
file manifest, and content hash are all updated to agree with the forged final payload.
`test_report_rejects_stage_backed_native_plugins_payload_file_manifest_drift` extends the same
stage-backed evidence to copied plugin bytes: even if the final package report, file manifest, and
content hash are self-consistent after a bundle-side artifact mutation, they must still match the
current NativeDynamic stage report before final Report projects `native_plugins_payload`.
`test_report_rejects_legacy_native_dynamic_stage_report_schema_before_payload_semantics` covers the
legacy Validate reports that do not declare `profile_summary.strategies` but still carry a
stage-backed PlatformBundle NativeDynamic payload. In that compatibility path, final Report now
reuses the NativeDynamic `file_manifest[]` and `materialized_packages[]` shape schema before copied
payload comparisons run, so padded or otherwise schema-invalid stage evidence stops at the
NativeDynamic field diagnostic instead of producing secondary copied-payload file-manifest or
package-id drift noise.
`test_report_rejects_legacy_native_dynamic_operation_audit_schema_before_payload_semantics` applies
the same ordering to the legacy signing/notarization audit comparison. A stage-backed copied payload
cannot silently publish over malformed NativeDynamic audit evidence; the NativeDynamic stage audit
schema reports first, and broad `NativeDynamic report <audit> is malformed` fallback diagnostics are
not emitted for already schema-invalid audit tables.
`platform_bundle_report_test_support.py` owns the shared fixture that builds those final
PlatformBundle reports, their stage-backed NativeDynamic reports, bundle manifests, payload file
manifests, and package TOML files; `test_pipeline_report_platform_bundle.py` now stays focused on
report behavior instead of exporting fixture helpers to adjacent tests.
`test_native_dynamic_stage_requires_native_dynamic_strategy` keeps the NativeDynamic stage itself
from consuming stale ABI v3 package export plan rows when the trusted Validate strategy list
explicitly selects only LibraryEmbed. `test_native_dynamic_stage_rejects_invalid_strategy_metadata`
applies the same standalone stage hard gate as final Report for non-list, empty, and unsupported
`profile_summary.strategies` values before any NativeDynamic package payload is materialized.
When the current Validate report selects the `native_dynamic` strategy, PlatformBundle now treats a
missing current-profile native plugin payload as its own fatal input error instead of publishing a
bundle without plugins and waiting for the final Report stage to fail.
If that Validate report exists but its strategy evidence has malformed base metadata, standalone
PlatformBundle reports the Validate metadata error instead of treating the profile as a
non-NativeDynamic bundle.
That metadata gate still runs when callers pass an explicit `--native-plugins-dir`, so a manual
plugin directory cannot bypass corrupted Validate strategy evidence in the current output root.
`test_pipeline_platform_bundle_requires_native_dynamic_payload_for_native_dynamic_profile` keeps a
wrong-profile NativeDynamic report from producing a pluginless PlatformBundle artifact under a
NativeDynamic profile.
`test_platform_bundle_rejects_invalid_validate_metadata_for_strategy` keeps malformed Validate
metadata from bypassing the NativeDynamic required-payload gate during standalone PlatformBundle
runs.
`test_platform_bundle_explicit_native_dir_rejects_invalid_validate_metadata` keeps explicit
native-plugin directory inputs on the same corrupted-Validate fatal path.
`test_platform_bundle_requires_bundle_strategy` keeps SourceTemplate-only Validate reports from
publishing a PlatformBundle with stale explicit host/pack inputs.
`test_platform_bundle_explicit_native_dir_requires_native_dynamic_strategy` and
`test_platform_bundle_staged_native_plugins_require_native_dynamic_strategy` keep both manual and
stage-inherited native plugin payloads from being accepted by PlatformBundle when the trusted
Validate strategy list explicitly selects only LibraryEmbed.
`test_platform_bundle_rejects_unknown_validate_strategy`,
`test_platform_bundle_rejects_empty_validate_strategies`, and
`test_platform_bundle_rejects_invalid_validate_strategies` keep standalone PlatformBundle aligned
with final Report's hard gate for unsupported, empty, or malformed Validate strategy lists.
PlatformBundle emits those hard-gate diagnostics once per failed evidence item; the stage-specific
bundle strategy check is the owner when it already reports malformed `profile_summary.strategies`,
so the generic final-Report-style strategy diagnostics are not appended a second time.
CookAssets, Pack, and PlatformBundle all use the shared bundle strategy diagnostic list, so a
Validate report containing multiple unsupported strategies produces one fatal diagnostic per
unsupported strategy, matching final Report output instead of truncating to the first unknown value.
The strategy identity contract is centralized in `export_strategy_contract.py`: supported strategy
names, legacy CamelCase aliases, and hyphen/space normalization are shared by Validate schema and
stage handoffs. `stage_handoff.py` owns empty/non-list checks, unsupported-strategy de-duplication,
and diagnostic rendering for the final Report stage and standalone stage handoffs.
Pipeline stage selection imports those shared strategy helpers directly; only the ordered stage-key
projection remains in `pipeline_report.py` because it is also used by final Report aggregation.
Standalone SourceTemplate, NativeDynamic, CompileHost, and PlatformBundle strategy gates use the
same direct import path. `native_dynamic_payload_allowed(...)` is centralized there as well, so
final Report is no longer the incidental owner of stage-local strategy validation helpers.
The same handoff helpers now treat existing stage `report.json` paths as files before reading them.
Required handoff diagnostics return `... is not a file` for directory paths, while optional/silent
handoff lookups return no inherited value instead of surfacing OS read errors.
CompileHost, SourceTemplate, and NativeDynamic also use that shared loader when they directly read
the Validate report, so explicit `--validate-report` paths and default `<out>/stages/validate/report.json`
paths cannot be directories or unreadable files.
PlatformBundle's standalone Validate strategy helpers now use the same trusted loader as well:
an existing non-file or unreadable Validate report keeps `native_dynamic` payload authorization
closed and surfaces the concrete Validate report diagnostic instead of being treated as missing
strategy evidence.
Final Report also treats `host_executable` and `pack` as required release evidence for every
non-fatal PlatformBundle report. A bundle manifest that omits either field, even if the
`bundle.json` mirrors that omission, is fatal because Hub/editor readers cannot verify a
publishable host executable plus zrpack payload from it.
`test_compile_host_dry_run_rejects_validate_report_directory`,
`test_source_template_stage_rejects_validate_report_directory`, and
`test_native_dynamic_stage_rejects_validate_report_directory` keep direct Validate report reads on
the same file gate as stage handoffs.
`test_validate_strategy_helpers_reject_report_directory` locks the PlatformBundle strategy helper
behavior for directory Validate reports: strategy membership returns false, NativeDynamic payload
authorization returns false, and the helper diagnostic remains `Validate report ... is not a file`.
`test_required_path_field_reports_stage_report_directory` and
`test_optional_path_field_ignores_stage_report_directory` lock the standalone handoff behavior for
directory stage reports: required handoffs produce a stage diagnostic, optional lookups stay empty.
`test_required_path_field_reports_path_resolve_error` and
`test_optional_path_field_reports_path_resolve_error` keep shared path-field handoff on the
diagnostic path when `Path.resolve()` fails: the value helper returns no path, and the matching
required or optional diagnostic helper reports `{Stage} report field ... could not be resolved`
instead of surfacing `OSError`.
Stage report top-level `diagnostics[]` entries must be meaningful strings on both final Report and
standalone handoff paths. `test_report_stage_rejects_blank_stage_diagnostic_entry` and
`test_handoff_rejects_blank_stage_diagnostic_entry` keep empty or whitespace-only diagnostic rows
from acting as release evidence. `test_report_stage_rejects_padded_stage_diagnostic_entry` and
`test_handoff_rejects_padded_stage_diagnostic_entry` also reject non-empty rows with leading or
trailing whitespace before final Report aggregation or stage handoff can consume them. Those
regressions live in
`tools/zircon_export/tests/test_pipeline_report_stage_metadata_diagnostics_schema.py` instead of
growing the already-large generic metadata suite.
The same shared handoff helper now separates field shape from entry type for stage diagnostics:
non-list `diagnostics` still reports `{stage} report diagnostics must be a string array`, while
non-string rows report `{stage} report diagnostics[index] must be a string` before blank/trimmed
diagnostic semantics or downstream stage path handoff logic can consume them. The final Report
loader and the standalone required/optional path helpers use the same indexed entry-type gate.
`test_report_stage_rejects_stage_report_directory` keeps a required stage `report.json` directory
from surfacing as an OS read error during aggregation, and
`test_report_rejects_legacy_native_plugins_payload_stage_report_directory` applies the same rule to
legacy NativeDynamic report reloads.
`test_report_rejects_bundle_manifest_directory` keeps a bundle-local directory path from reaching
the manifest JSON reader; final Report now reports `bundle_manifest ... is not a file` instead of
surfacing an OS read error.
`test_report_stage_rejects_validate_unknown_top_level_field` applies the same closed-schema rule to
the Validate report that chooses the remaining pipeline stages. A non-fatal `validate` report may
only carry stage metadata, project/profile discovery, diagnostics, profile summary, and plan
summary; unknown sidecar fields make the wrapper stage fatal before Report derives the stage
requirements.
`test_report_stage_rejects_validate_profile_summary_unknown_field` closes the nested
`profile_summary` object that drives strategy, platform, mode, plugin, feature, and asset-filter
selection. Unknown keys make the Validate wrapper fatal before any downstream stage treats the
profile summary as trusted strategy evidence.
`test_report_stage_rejects_validate_profile_summary_missing_required_field` also keeps the Rust
`ExportValidateProfileSummary` required field set complete: `name`, `target_mode`,
`target_platform`, `build_mode`, `selected_plugins`, and `features` must be present with their
typed schema shapes before final Report uses the profile summary as release-plan evidence.
`test_report_stage_rejects_validate_profile_summary_string_fields_non_string`,
`test_report_stage_rejects_validate_selected_plugins_non_string_array`, and
`test_report_stage_rejects_validate_selected_plugin_ids_invalid` close the core typed
profile fields and selected plugin id shape inside that object. `name`, `target_mode`,
`target_platform`, `build_mode`, and `asset_filter` must be strings, while `selected_plugins`
must be a string array whose entries are valid project plugin package ids before final Report
accepts the profile summary as release-plan evidence. Non-array `selected_plugins` values still
report the table-level string-array diagnostic, but non-string entries now report
`validate report profile_summary.selected_plugins[index] must be a string` before plugin-id or
duplicate-selection semantics run.
`test_report_stage_rejects_validate_profile_summary_asset_filter_not_trimmed` keeps the optional
Validate `profile_summary.asset_filter` on the same non-empty trimmed contract as the CookAssets
handoff path. Empty or padded filter labels now fail during final Report Validate schema wrapping
instead of waiting for a later CookAssets handoff consumer to catch the stale profile filter.
`test_report_stage_rejects_validate_profile_summary_name_not_trimmed` aligns
`profile_summary.name` with the Rust export profile validation rule: profile names must be
non-empty and trimmed before the final Report accepts the Validate profile summary as release-plan
evidence.
`test_report_stage_rejects_validate_profile_summary_enum_fields_unknown` and
`test_report_stage_rejects_validate_profile_summary_enum_fields_not_trimmed` bind the serialized
profile enum fields to the Rust export contracts. `build_mode` must be `debug` or `release`,
`target_mode` must be `client_runtime`, `server_runtime`, or `editor_host`, and
`target_platform` must use the supported export target names or the desktop platform aliases
accepted by `ExportTargetPlatform`. Empty, padded, or unknown values now fail at Validate schema
time with `Validate` in `fatal_stages`, before downstream stages derive build mode, target mode, or
platform-specific NativeDynamic behavior from stale metadata.
`test_report_stage_rejects_validate_strategies_non_string_array` applies the same string-array
schema to `profile_summary.strategies`, which directly chooses the stage set final Report will
load. Non-array strategy values still report the table-level string-array diagnostic, while
non-string entries now report `profile_summary.strategies[index] must be a string` and mark
`Validate` in `fatal_stages` before later unsupported-strategy normalization runs.
`test_report_stage_rejects_validate_profile_strategies_empty_as_schema` gives an explicitly empty
strategy list the same Validate-stage ownership: empty `profile_summary.strategies` now marks the
Validate wrapper fatal before final Report falls back to generic strategy requirement handling.
`test_report_stage_rejects_validate_profile_strategies_unknown_as_schema` and
`test_report_stage_rejects_validate_profile_strategies_not_trimmed` apply the same ownership to
individual strategy values. Unknown strategy names still use the shared `unsupported export
strategy ...` diagnostic, but now mark `Validate` in `fatal_stages`; empty or padded strategy
strings fail as non-empty trimmed export strategy schema before normalization aliases are applied.
`test_report_stage_rejects_validate_profile_strategies_duplicate` keeps normalized strategy rows
unique after alias normalization, so a report cannot publish both `library_embed` and
`LibraryEmbed` as separate strategy evidence after Rust would have diagnosed and deduplicated the
profile.
`test_report_stage_rejects_validate_profile_features_non_object`,
`test_report_stage_rejects_validate_profile_feature_list_non_string_array`, and
`test_report_stage_rejects_validate_profile_feature_ids_not_trimmed` close the selected feature
matrix shape. `profile_summary.features` must be an object whose values are string arrays, and
owner plugin ids plus selected feature ids must be non-empty trimmed strings, matching the Rust
`BTreeMap<String, Vec<String>>` shape before final Report accepts it as profile feature-selection
evidence. Non-string feature rows report `profile_summary.features.<plugin>[index] must be a string`
instead of collapsing the owner list to a table-level string-array failure.
`test_report_stage_rejects_validate_profile_feature_owner_plugin_ids_invalid` also routes
the owner keys through the same project plugin package id schema as `selected_plugins`, so uppercase
owners, hyphenated package names, and repeated underscore tokens fail at Validate schema time rather
than being treated as trustworthy feature-selection owners.
`test_report_stage_rejects_validate_profile_feature_ids_invalid` closes the feature ids themselves
against the project manifest namespace contract: each feature id must use `owner.feature` dot form,
must avoid empty segments, may contain only lowercase ASCII letters, digits, underscores, and dots,
and must be prefixed by its owner plugin id.
`test_report_stage_rejects_validate_top_level_field_types` closes the remaining Validate
stage-report top-level typed fields before any strategy or plan-summary derivation runs.
`profile_found` must be boolean, `project_manifest` and `stage_output` must be strings, and
`fatal_diagnostics` must be a string array. Non-string diagnostic entries now report
`fatal_diagnostics[index] must be a string`; wrong types still mark `Validate` in `fatal_stages`
instead of passing through as harmless sidecar metadata.
`test_report_stage_rejects_validate_blank_fatal_diagnostic_entry` keeps those fatal diagnostics
meaningful too: empty or whitespace-only entries are rejected before plan-time fatal evidence can be
accepted. `test_report_stage_rejects_validate_padded_fatal_diagnostic_entry` also rejects non-empty
fatal diagnostic rows with leading or trailing whitespace before strategy selection, required-stage
derivation, or plan-summary checks can treat the row as trusted release evidence. Those focused
regressions live in
`tools/zircon_export/tests/test_pipeline_report_validate_diagnostics_schema.py` instead of growing
the broad Validate schema suite.
`test_report_stage_rejects_validate_summary_objects_non_object` closes the two summary containers
themselves. `profile_summary` and `plan_summary` must be objects before final Report derives the
required stage set or inspects build-plan evidence; malformed containers now stop at `Validate`
instead of causing fallback stage requirements and unrelated missing-stage diagnostics.
`test_report_stage_rejects_validate_plan_summary_unknown_field` closes the nested `plan_summary`
object that carries linked crates, NativeDynamic package exports, SourceTemplate generated files,
LibraryEmbed compile plans, and runtime plugin availability. Unknown plan-summary keys make Validate
fatal before any stage consumes plan evidence.
`test_report_stage_rejects_validate_plan_summary_missing_required_field` keeps the Rust
`ExportValidatePlanSummary` required field set complete on non-fatal Validate reports:
`enabled_runtime_plugins`, `linked_runtime_crates`, `native_dynamic_packages`, `generated_files`,
and `runtime_plugin_availability` must be present with their typed schema shapes before final
Report treats the Validate plan as release evidence.
Validate wrapper schema and `plan_summary` dispatch now live in
`tools/zircon_export/pipeline_report_validate_stage_schema.py`; the generic
`pipeline_report_stage_schema.py` only registers the Validate stage fields and delegates the
wrapper-specific schema work to that module.
Shared bool/integer/string/object/object-array primitive diagnostics used by the remaining injected
NativeDynamic and Pack schema paths live in
`tools/zircon_export/pipeline_report_schema_primitives.py`, so the stage dispatcher only wires
those helpers into downstream schema modules. Shared table/sequence string-array diagnostics live
behind `tools/zircon_export/pipeline_report_schema_table.py` and now reuse the indexed helper from
`tools/zircon_export/pipeline_report_validate_string_array_schema.py`: non-array fields keep
`<field> must be a string array`, while non-string entries inside a list report
`<field>[index] must be a string`.
CompileHost and CookAssets stage schema helpers also import those shared primitives for their local
typed field checks, keeping stage-specific files focused on field ownership and label selection.
Validate wrapper schema and PlatformBundle report/bundle-manifest schema now use the same shared
primitive helpers for their local bool/string/string-array/object/object-array checks. The
stage-specific modules still own field allow-lists, nested schema dispatch, and diagnostic labels;
`pipeline_report_schema_primitives.py` owns only the common typed-shape predicates.
Validate child schema modules follow the same split. CompileHost plan, linked runtime crate,
profile summary, and runtime availability helpers import the shared primitive predicates while
keeping their enum/id/path/availability semantics in their local modules.
`test_report_stage_rejects_validate_enabled_runtime_plugins_non_string_array`,
`test_report_stage_rejects_validate_enabled_runtime_plugin_ids_invalid`,
`test_report_stage_rejects_validate_linked_runtime_crates_non_string_array`, and
`test_report_stage_rejects_validate_native_dynamic_packages_non_string_array` close the top-level
`plan_summary` plugin/package vectors. `enabled_runtime_plugins`, `linked_runtime_crates`, and
`native_dynamic_packages` must be string arrays; non-string entries now report
`validate report plan_summary.<field>[index] must be a string` before plugin-id, runtime-crate,
NativeDynamic package-id, duplicate, or later handoff semantics consume the vector. Schema-clean
`enabled_runtime_plugins` entries must also be valid project plugin package ids, and
`native_dynamic_packages` entries must be non-empty trimmed NativeDynamic package ids before final
Report accepts them as selected runtime plugin, linked crate, or NativeDynamic package evidence.
`test_report_stage_rejects_validate_linked_runtime_crate_names_invalid` also validates
`linked_runtime_crates[]` entries against the project runtime crate naming contract before final
Report trusts linked crate evidence.
`test_report_stage_rejects_validate_plan_vector_duplicate_entry` keeps all three top-level plan
vectors unique after their entry identity schema passes, so a Validate report cannot publish the
same selected runtime plugin, linked runtime crate, or NativeDynamic package twice as release
evidence.
The identifier helpers live in `pipeline_report_validate_identifier_schema.py`, keeping stage
report orchestration separate from reusable id/token diagnostics.
The top-level plan-vector dispatch for `enabled_runtime_plugins`, `linked_runtime_crates`, and
`native_dynamic_packages` lives in
`tools/zircon_export/pipeline_report_validate_plan_vector_schema.py`, so the final Report stage
dispatcher only routes `plan_summary` rather than owning every vector's identity contract.
Focused plan-vector regressions live in
`tools/zircon_export/tests/test_pipeline_report_validate_plan_vector_schema.py`; the broad
Validate schema test module no longer owns the vector type/id cases.
`test_report_stage_rejects_validate_compile_host_plan_non_object` closes the
`plan_summary.library_embed_compile_host` container before any host build-plan fields are trusted.
The value must be an object; malformed containers now stop at `Validate` instead of falling back to
LibraryEmbed stage requirements and unrelated missing-stage diagnostics.
`test_report_stage_rejects_validate_compile_host_plan_unknown_field` closes the nested
`plan_summary.library_embed_compile_host` object. Unknown compile-plan keys make Validate fatal
before CompileHost or editor command planners trust the host package, binary, feature matrix,
runtime crate links, target directory, or cargo command.
The CompileHost plan schema helper lives in
`tools/zircon_export/pipeline_report_validate_compile_host_schema.py`; the final Report stage
dispatcher routes `plan_summary.library_embed_compile_host` into that helper and keeps only the
outer Validate wrapper orchestration.
`test_report_stage_rejects_validate_compile_host_plan_string_fields_non_string`,
`test_report_stage_rejects_validate_compile_host_plan_release_non_bool`,
`test_report_stage_rejects_validate_compile_host_plan_string_array_fields_non_string_array`, and
`test_report_stage_rejects_validate_compile_host_linked_crate_non_object` close the typed shape of
that CompileHost plan. Package, binary, manifest path, target directory, and cargo profile must be
strings; `release` must be boolean; feature/plugin/command vectors must be string arrays;
`expected_runtime_plugins[]` entries must be valid project plugin package ids; and
`linked_runtime_crates[]` must contain objects before final Report trusts the LibraryEmbed host
build plan. The plugin-id vector shares the indexed entry-type diagnostic
`validate report plan_summary.library_embed_compile_host.expected_runtime_plugins[index] must be a string`;
`app_features[]`, `runtime_features[]`, and `command[]` now follow the same entry-type boundary:
non-array fields keep the table-level string-array shape gate, and non-string rows report
`validate report plan_summary.library_embed_compile_host.<field>[index] must be a string`.
`test_report_stage_rejects_validate_compile_host_blank_string_array_entry` keeps
`app_features[]`, `runtime_features[]`, and `command[]` from accepting empty or whitespace-only
entries after the string-array shape gate passes, so the published LibraryEmbed CompileHost plan
cannot smuggle blank Cargo feature or command tokens into the final report.
`test_report_stage_rejects_validate_compile_host_linked_crate_unknown_field` applies that same
closed-schema rule to `library_embed_compile_host.linked_runtime_crates[]`. Each linked crate row
may only carry crate name, path, registration kind, and provider package id, and
`test_report_stage_rejects_validate_compile_host_linked_crate_missing_field` requires all four
fields before final Report accepts it as compile-plan linkage evidence.
`test_report_stage_rejects_validate_compile_host_duplicate_linked_crate_name` keeps schema-clean
linked crate rows unique by `crate_name`, so a Validate CompileHost plan cannot publish two rows
for the same runtime crate.
`test_report_stage_rejects_validate_compile_host_linked_crate_string_fields_non_string`,
`test_report_stage_rejects_validate_compile_host_linked_crate_names_invalid`,
`test_report_stage_rejects_validate_compile_host_linked_crate_provider_ids_invalid`, and
`test_report_stage_rejects_validate_compile_host_linked_crate_registration_kind_invalid` also
check those linked crate row values in the dedicated CompileHost linkage schema module.
`test_report_stage_rejects_validate_compile_host_linked_crate_path_invalid` keeps each Validate
`linked_runtime_crates[].path` as a non-empty, trimmed safe relative path, so the LibraryEmbed
CompileHost plan cannot publish absolute paths or parent-directory escapes as runtime crate
provenance.
That module is
`tools/zircon_export/pipeline_report_validate_compile_host_linkage_schema.py`; the final Report
stage dispatcher only routes the nested linked-crate vector into it. Both CompileHost plan and
linked-crate schema modules import shared primitive diagnostics for their local bool/string and
object-array gates; they still own the LibraryEmbed linkage field allow-lists and runtime crate
identity rules.
`crate_name`, `path`, `registration_kind`, and `provider_package_id` must be strings; `crate_name`
must satisfy the project runtime crate naming contract, `provider_package_id` must be a valid
project plugin package id, and `registration_kind` must be `runtime_plugin` before final Report
accepts runtime plugin crate linkage.
`test_report_stage_rejects_validate_native_dynamic_export_unknown_field` and
`test_report_stage_rejects_validate_native_dynamic_abi_unknown_field` close the Validate
`plan_summary.native_dynamic_package_exports[]` table before final Report trusts NativeDynamic ABI
v3 package publishing evidence. Each package export row may only carry package identity, output
directory/path, manifest/package-report paths, and its `abi` object; each ABI object may only carry
the fixed ABI v3 contract fields.
`test_report_stage_rejects_validate_native_dynamic_export_non_object`,
`test_report_stage_rejects_validate_native_dynamic_export_string_fields_non_string`,
`test_report_stage_rejects_validate_native_dynamic_export_abi_non_object`,
`test_report_stage_rejects_validate_native_dynamic_abi_version_non_integer`, and
`test_report_stage_rejects_validate_native_dynamic_abi_string_fields_non_string` close the typed
shape of that table. Package export rows must be objects, package identity/path fields must be
strings, `abi` must be an object, `abi.abi_version` must be an integer, and ABI contract fields
must be strings before final Report accepts the NativeDynamic ABI v3 publishing plan.
`test_report_stage_rejects_validate_native_dynamic_export_path_contract_mismatch` and
`test_report_stage_rejects_validate_native_dynamic_abi_v3_contract_mismatch` also enforce the
semantic ABI/path contract in the same loader-stage schema path. The `package_id` must be a
non-empty trimmed NativeDynamic package id, `directory` must match the shared sanitization rule
used by NativeDynamic materialization, `path`, `manifest`, and `package_report` must point to the
derived package locations, `abi.abi_version` must be `3`, and every ABI contract string must be
non-empty, trimmed, and equal to the fixed v3 constants before final Report trusts the package
export plan. The shared package-export diagnostics live in
`pipeline_report_native_dynamic_package_export_schema.py`, while `native_dynamic_contract.py`
owns the common `native_dynamic_package_directory(...)` sanitization helper used by both the
NativeDynamic stage and final Report schema checks.
Those schema checks are intentionally ordered before contract checks: an ABI version must be an
integer before it can be compared to `3`, and ABI string contracts must be trimmed and non-empty
before they are compared to the fixed v3 constants. Stage payload reconciliation now ignores
schema-invalid package export rows when comparing materialized packages or loader-manifest ABI
handoff, so malformed release evidence fails on field-level diagnostics instead of follow-on drift.
The same package-export schema module also owns the Validate plan table wrapper for
`plan_summary.native_dynamic_package_exports`, including list-level and non-object-row diagnostics;
`pipeline_report_stage_schema.py` only routes that plan field into the helper. The package
export/ABI field shape checks import the shared string/integer/object primitives from
`pipeline_report_schema_primitives.py`; the wrapper keeps its historical top-level list diagnostic
so existing release reports still say `must be a list` for a non-list
`native_dynamic_package_exports` value.
`test_report_stage_rejects_validate_runtime_availability_unknown_field` and
`test_report_stage_rejects_validate_runtime_availability_entry_unknown_field` close
the runtime availability object and entry schema before final Report trusts plugin availability
evidence. The runtime availability schema helper owns the top-level object gate, required bucket
set, per-bucket list checks, and per-entry closed schema so the generic stage dispatcher only routes
`plan_summary.runtime_plugin_availability` into the helper when present.
`plan_summary.runtime_plugin_availability`. The availability report may only carry the Rust
runtime availability buckets, and each bucket entry may only carry plugin identity, runtime id,
required flag, maturity, and reason. All five entry fields are required release evidence because
they are serialized from Rust `RuntimePluginAvailabilityEntry`.
The runtime availability schema helper lives in
`tools/zircon_export/pipeline_report_validate_runtime_availability_schema.py`, keeping the final
Report stage dispatcher focused on wrapper dispatch and generic stage-report field gates. It imports
the shared bool/string primitive predicates for entry fields and keeps bucket membership, runtime id,
maturity, reason trim, and required-entry semantics locally.
`test_report_stage_rejects_validate_runtime_availability_non_object`,
`test_report_stage_rejects_validate_runtime_availability_bucket_non_array`,
`test_report_stage_rejects_validate_runtime_availability_entry_non_object`,
`test_report_stage_rejects_validate_runtime_availability_entry_string_fields_non_string`, and
`test_report_stage_rejects_validate_runtime_availability_required_non_bool` close the typed shape of
the same availability report. The report itself must be an object, each bucket must be an array,
each entry must be an object, `id`/`runtime_id`/`maturity`/`reason` must be strings, and `required`
must be boolean before final Report trusts runtime plugin availability evidence.
Missing any of those five entry fields produces the same typed field diagnostic as a wrong-shaped
value, covered by
`test_report_stage_rejects_validate_runtime_availability_entry_missing_required_field`.
`test_report_stage_rejects_validate_runtime_availability_missing_bucket`,
`test_report_stage_rejects_validate_runtime_availability_plugin_ids_invalid`,
`test_report_stage_rejects_validate_runtime_availability_runtime_ids_invalid`,
`test_report_stage_rejects_validate_runtime_availability_maturity_invalid`,
`test_report_stage_rejects_validate_runtime_availability_reason_not_trimmed`, and
`test_report_stage_rejects_validate_runtime_availability_missing_required_false` close the
semantic availability contract. All eight Rust availability buckets must be present, entry `id`
must satisfy the project plugin id contract, `runtime_id` must be a known runtime plugin id,
`maturity` must be one of the serialized `PluginMaturity` values, `reason` must be non-empty and
trimmed, and `missing_required[]` entries must carry `required = true`.
`test_report_stage_rejects_validate_runtime_availability_id_runtime_mismatch` also rejects entries
whose project plugin `id` and serialized `runtime_id` are each valid but do not name the same
runtime plugin.
Runtime availability category semantics are likewise checked after per-entry identity validation:
primary categories are exclusive, `missing_required` may only overlay a blocked required entry, and
duplicate `missing_required` rows are rejected. The focused coverage is
`test_report_stage_rejects_validate_runtime_availability_duplicate_plugin_bucket`,
`test_report_stage_allows_validate_runtime_availability_missing_required_overlay`, and
`test_report_stage_rejects_validate_runtime_availability_missing_required_without_blocked_entry`.
`test_report_stage_rejects_compile_host_unknown_top_level_field` keeps final Report's trusted
CompileHost handoff on a closed schema too. A non-fatal `compile_host` stage report may only carry
`stage`, `profile`, `fatal`, `diagnostics`, `command`, `link_plan`, `host_executable`,
`exit_code`, `stdout_lines`, and `stderr_lines`; unknown sidecar fields make the wrapper stage
fatal before PlatformBundle can trust the host path.
`test_compile_host_report_preserves_library_embed_link_plan` keeps the execution report connected
to the validated LibraryEmbed feature/link matrix. When CompileHost consumes
`plan_summary.library_embed_compile_host`, the stage report writes `link_plan.app_features`,
`link_plan.runtime_features`, `link_plan.expected_runtime_plugins`, and
`link_plan.linked_runtime_crates` beside the Cargo command/host evidence so final Report readers can
audit which profile-selected app/runtime features and runtime crates the host build used.
`test_report_stage_accepts_compile_host_link_plan`,
`test_report_stage_rejects_compile_host_link_plan_unknown_field`,
`test_report_stage_rejects_compile_host_link_plan_invalid_shape`, and
`test_report_stage_rejects_compile_host_link_plan_missing_evidence_field` close that nested object
in the final Report schema. `link_plan` must be an object; app/runtime feature vectors must be
present string arrays; expected runtime plugins must be a present string array of valid project
plugin ids; linked runtime crates must be a present object array and reuse the same linked-crate
schema as the Validate CompileHost plan with a CompileHost-report diagnostic label.
Non-string entries inside `link_plan.app_features[]` and `link_plan.runtime_features[]` now fail as
`compile_host report link_plan.<field>[index] must be a string`, while non-array fields keep the
broader string-array diagnostic. This keeps malformed feature rows from falling through into blank,
trimmed, duplicate, or Validate/CompileHost link-plan mismatch semantics.
`test_report_stage_rejects_compile_host_link_plan_blank_feature_entry` also requires the published
`link_plan.app_features[]` and `link_plan.runtime_features[]` entries to be non-blank, matching the
Validate CompileHost plan quality gate before final Report trusts CompileHost's feature/link
evidence.
That reused schema also requires every CompileHost `link_plan.linked_runtime_crates[]` row to carry
`crate_name`, `path`, `provider_package_id`, and `registration_kind`;
`test_report_stage_rejects_compile_host_linked_crate_missing_field` covers the CompileHost label.
`test_report_stage_rejects_compile_host_duplicate_linked_crate_name` applies the same duplicate
`crate_name` gate to CompileHost execution evidence, so a hand-written `link_plan` cannot claim two
rows for one runtime crate after Validate published a unique table.
`test_report_stage_rejects_compile_host_linked_crate_path_invalid` applies the same safe-relative
path rule to execution reports, so a hand-written CompileHost report cannot hide an unsafe crate
path behind a broad link-plan mismatch diagnostic.
`test_report_stage_rejects_compile_host_command_non_string_array`,
`test_report_stage_rejects_compile_host_log_line_non_string_array`,
`test_report_stage_rejects_compile_host_missing_log_line_array`,
`test_report_stage_rejects_compile_host_missing_release_evidence_field`,
`test_report_stage_rejects_compile_host_host_executable_non_string`, and
`test_report_stage_rejects_compile_host_exit_code_non_integer` close the typed shape of that report.
`command`, `stdout_lines`, and `stderr_lines` must be string arrays, `host_executable` must be a
string, and `exit_code` must be an integer before final Report accepts CompileHost release
evidence. `command`, `host_executable`, `exit_code`, and the two log arrays are required on
non-fatal reports, so pre-audit or hand-authored success-shaped CompileHost reports cannot bypass
the command, host, exit-code, or Cargo output evidence contract by omitting them.
`test_report_stage_rejects_compile_host_command_entry_non_string_before_array_shape` and
`test_report_stage_rejects_compile_host_log_line_entry_non_string_before_array_shape` pin the
entry-type boundary: malformed command or log rows report `compile_host report <field>[index] must
be a string`, but a field that is not an array still reports `compile_host report <field> must be a
string array`.
`test_report_stage_rejects_cook_assets_unknown_top_level_field` closes the CookAssets handoff on
the same shared loader path. A non-fatal `cook_assets` report may only carry its stage metadata,
source/project manifest provenance, project fallback summary, cooked manifest path, asset/root
counts, staged manifest SHA-256, and asset filter; unknown sidecar fields make the wrapper stage
fatal before Pack can trust `cooked_asset_manifest`.
`test_report_stage_rejects_cook_assets_string_fields_non_string`,
`test_report_stage_rejects_cook_assets_blank_or_padded_required_string`,
`test_report_stage_rejects_cook_assets_blank_or_padded_optional_string`,
`test_report_stage_rejects_cook_assets_count_fields_non_integer`, and
`test_report_stage_rejects_cook_assets_generated_from_project_non_bool`, and
`test_report_stage_rejects_cook_assets_missing_release_evidence_field` close the typed shape of the
same handoff report. Manifest/default-scene/asset-filter fields must be strings, asset/root counts
must be integers, and `generated_from_project` must be boolean before final Report accepts CookAssets
release evidence. `cooked_asset_manifest`, `cooked_asset_manifest_sha256`, `asset_count`,
`root_count`, and `generated_from_project` are required on non-fatal reports, while
source/project/default-scene and asset-filter provenance may stay `null` when CookAssets did not
consume those optional inputs. Blank or padded string evidence is rejected by the CookAssets stage
schema before aggregate manifest, hash, path, or Pack handoff checks run.
`test_report_stage_rejects_cook_assets_manifest_hash_mismatch` also
locks the aggregate report against stale or hand-edited CookAssets evidence by comparing the reported
hash with the staged manifest bytes, and
`test_report_stage_rejects_cook_assets_manifest_outside_stage_directory` keeps that manifest bound
to the current `<out>/stages/cook_assets/assets.json` rather than an external side file.
`test_report_stage_rejects_pack_asset_manifest_not_from_cook_assets` requires the final Pack input
to resolve back to the same staged CookAssets manifest.
`test_report_stage_rejects_cook_assets_manifest_count_mismatch` keeps `asset_count` and `root_count`
bound to the actual staged manifest array lengths.
`test_report_stage_rejects_cook_assets_manifest_bad_source_path` keeps staged asset `source` rows
absolute and file-backed before final Report trusts CookAssets as release evidence.
`test_report_stage_rejects_pack_included_asset_missing_cook_assets_source` keeps Pack success
evidence from including a CookAssets asset row that would make the real packer report
`included asset ... is missing source`.
`test_report_stage_rejects_pack_asset_manifest_drift_from_cook_assets_source` keeps Pack manifest
`size` and `chunk_hash` tied to the actual CookAssets source bytes for included assets.
`test_report_stage_rejects_native_dynamic_unknown_top_level_field` closes the NativeDynamic stage
report before PlatformBundle can consume the staged plugins directory and payload audit. A non-fatal
`native_dynamic` report may only carry stage metadata, validate/plugin-root provenance, target and
artifact summaries, staged plugin payload evidence, build/signing/notarization audits, and cleanup
state; unknown sidecar fields make the wrapper stage fatal.
`test_report_stage_rejects_native_dynamic_string_fields_non_string`,
`test_report_stage_rejects_native_dynamic_string_array_fields_non_string_array`,
`test_report_stage_rejects_native_dynamic_string_array_fields_blank_entry`,
`test_report_stage_rejects_native_dynamic_count_fields_non_integer`,
`test_report_stage_rejects_native_dynamic_negative_count_fields`,
`test_report_stage_rejects_native_dynamic_bool_fields_non_bool`,
`test_report_stage_rejects_native_dynamic_object_fields_non_object`, and
`test_report_stage_rejects_native_dynamic_object_array_fields_non_object_array` close the typed
shape of the same report. NativeDynamic path/provenance fields must be strings when present,
selected package and artifact extension fields must be string arrays with no blank entries,
`package_count` must be a non-negative integer, `payload_cleaned` must be boolean,
build/signing/notarization audit summaries must be objects, and manifest/materialized/package-export
evidence must be object arrays before final Report accepts NativeDynamic release evidence.
`test_report_stage_rejects_native_dynamic_missing_release_evidence_field` keeps the same
successful-stage contract closed: non-fatal NativeDynamic reports must include `plugins_dir`,
`loader_manifest`, `content_hash`, `file_manifest`, `materialized_packages`, and `package_exports`,
so final Report does not publish a staged plugin payload that omitted the accepted ABI export table.
`test_report_stage_rejects_native_dynamic_stage_output_outside_current_stage` keeps an explicitly
reported `stage_output` bound to the current `<out>/stages/native_dynamic` directory derived from
the loaded stage report path. The shared `pipeline_report_stage_location.py` gate now covers
NativeDynamic, Validate, and Pack without trusting self-reported output directories.
`test_report_stage_rejects_native_dynamic_package_export_unknown_field`,
`test_report_stage_rejects_native_dynamic_package_export_abi_unknown_field`,
`test_report_stage_rejects_native_dynamic_package_export_field_types`, and
`test_report_stage_rejects_native_dynamic_package_export_abi_field_types` then close the
`package_exports[]` row and nested ABI shape on the NativeDynamic stage report itself. These tests
live with the rest of the NativeDynamic stage report schema coverage in
`test_pipeline_report_native_dynamic_stage_schema.py` and reuse the Validate package export schema
through a label-aware helper, so the same ABI v3 publishing contract protects both Validate plan
evidence and the later staged NativeDynamic handoff.
`test_report_stage_rejects_native_dynamic_package_export_abi_shape_before_contract` keeps the
ordering explicit: schema-invalid ABI values stop at integer or trimmed-string diagnostics and are
not reused for ABI constant, materialized package, or loader-manifest drift checks.
`test_report_stage_rejects_native_dynamic_package_export_missing_required_field` and
`test_report_stage_rejects_validate_native_dynamic_export_missing_required_field` keep that shared
schema from accepting partial package-export rows: `package_id`, `directory`, `path`, `manifest`,
`package_report`, and `abi` must all be present before final Report treats either the NativeDynamic
stage table or Validate handoff table as complete release evidence.
`test_report_stage_rejects_validate_native_dynamic_missing_package_exports` closes the Validate
table-level case: when Validate reports the `native_dynamic` strategy or non-empty
`plan_summary.native_dynamic_packages`, `plan_summary.native_dynamic_package_exports` itself is
required as a list before the pipeline can proceed to NativeDynamic publication.
`test_report_stage_rejects_native_dynamic_package_export_abi_missing_required_field` and
`test_report_stage_rejects_validate_native_dynamic_abi_missing_required_field` apply the same rule
inside `abi`: every ABI v3 integer/string contract field must be present, so an empty or partial ABI
table cannot satisfy package export publishing evidence.
`test_pipeline_report_native_dynamic_stage_build_schema.py` owns the
`native_build_plan` / `native_build_execution` nested schema coverage and reuses
`native_dynamic_stage_schema_test_support.py` for the shared stage-report fixture/assertion path:
`test_report_stage_rejects_native_dynamic_build_plan_unknown_field`,
`test_report_stage_rejects_native_dynamic_build_plan_missing_release_evidence_field`,
`test_report_stage_rejects_native_dynamic_build_plan_empty_required_string_release_evidence_field`,
`test_report_stage_rejects_native_dynamic_build_plan_field_types`,
`test_report_stage_rejects_native_dynamic_build_plan_negative_count_fields`,
`test_report_stage_rejects_native_dynamic_build_plan_blank_build_feature_entry`,
`test_report_stage_rejects_native_dynamic_build_plan_profile_release_contract`,
`test_report_stage_rejects_native_dynamic_build_plan_expected_artifact_mismatch`,
`test_report_stage_rejects_native_dynamic_build_plan_blank_diagnostic_entry`,
`test_report_stage_rejects_native_dynamic_build_plan_package_unknown_field`,
`test_report_stage_rejects_native_dynamic_build_plan_package_missing_required_field`,
`test_report_stage_rejects_native_dynamic_build_plan_package_empty_required_string_field`,
`test_report_stage_rejects_native_dynamic_build_plan_package_empty_command`,
`test_report_stage_rejects_native_dynamic_build_plan_package_blank_feature_entry`,
`test_report_stage_rejects_native_dynamic_build_plan_package_field_types`,
`test_report_stage_rejects_native_dynamic_build_execution_unknown_field`,
`test_report_stage_rejects_native_dynamic_build_execution_missing_release_evidence_field`,
`test_report_stage_rejects_native_dynamic_build_execution_field_types`,
`test_report_stage_rejects_native_dynamic_build_execution_negative_count_fields`,
`test_report_stage_rejects_native_dynamic_build_execution_disabled_non_empty_table`,
`test_report_stage_rejects_native_dynamic_build_execution_fatal_success_report_mismatch`,
`test_report_stage_rejects_native_dynamic_build_execution_skipped_success_report_mismatch`,
`test_report_stage_rejects_native_dynamic_build_execution_blank_diagnostic_entry`,
`test_report_stage_rejects_native_dynamic_build_execution_package_unknown_field`,
`test_report_stage_rejects_native_dynamic_build_execution_package_missing_required_field`,
`test_report_stage_rejects_native_dynamic_build_execution_package_empty_required_string_field`,
`test_report_stage_rejects_native_dynamic_build_execution_package_empty_command`,
`test_report_stage_rejects_native_dynamic_build_execution_package_blank_copied_sidecar_entry`,
`test_report_stage_rejects_native_dynamic_build_execution_package_padded_copied_sidecar_entry`, and
`test_report_stage_rejects_native_dynamic_build_execution_package_field_types` close the Cargo
build plan and build execution evidence before final Report can trust it. The implementation lives
in `pipeline_report_native_dynamic_stage_schema.py`, keeping the shared stage schema dispatcher
focused on cross-stage wiring. Both nested `package_count` headers are non-negative integer evidence
before the later package-list consistency checks compare them with their `packages[]` arrays.
`test_report_stage_rejects_native_dynamic_operation_audit_missing_stage_evidence_field`
locks the sibling operation-audit header gate: `native_signing` / `native_notarization` stage
objects must carry `diagnostics` and `packages` even when the operation is disabled and the package
table is empty.
`test_report_stage_rejects_native_dynamic_operation_audit_blank_allowed_platform_entry` keeps
operation audit platform filters aligned with CLI normalization: empty arrays still mean all
platforms, but empty or whitespace-only entries are malformed release evidence.
`test_report_stage_rejects_native_dynamic_operation_audit_padded_allowed_platform_entry` extends
that gate to non-empty padded entries and prevents malformed filter rows from falling through to a
noisier `platform_allowed` mismatch diagnostic.
`test_report_stage_rejects_native_dynamic_operation_audit_artifact_empty_command` keeps signed or
notarized artifact rows from publishing an empty or blank-entry command vector.
`test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_execution_evidence_field`
in `test_pipeline_report_native_dynamic_operation_audit_schema.py` locks the artifact execution
gate: enabled operation audit artifact rows must carry `exit_code`, `before_sha256`, and
`after_sha256`, not just the expanded command and captured streams.
`test_report_stage_rejects_native_dynamic_operation_audit_unsafe_relative_artifact` keeps operation
artifact `package_relative_artifact` evidence on the same safe-relative path contract as staged
loadable artifacts, so `../` escapes fail at the field-level schema boundary before artifact-set
comparison.
`test_report_stage_rejects_native_dynamic_operation_audit_empty_required_identity_string` keeps
operation package identity, artifact identity, and before/after hash evidence non-empty while
leaving captured stdout/stderr on their existing empty-string semantics.
`test_report_stage_rejects_native_dynamic_operation_audit_padded_required_identity_string` applies
the same normalization rule to non-empty package/artifact/hash values, so later package-set,
artifact-set, safe-relative, or SHA-256 checks do not have to explain padded release evidence.
`test_report_stage_rejects_native_dynamic_operation_audit_invalid_hash_string` keeps operation
artifact `before_sha256` and `after_sha256` evidence on the same SHA-256 hex contract as other
NativeDynamic content hashes.
`test_report_stage_rejects_native_dynamic_operation_audit_blank_target_platform` keeps operation
audit target platform evidence non-empty before platform_allowed recomputation or stable summary
projection can trust it.
`test_report_stage_rejects_native_dynamic_operation_audit_padded_summary_string` keeps
`target_platform` and present `profile` values trimmed, and prevents a padded target platform from
falling through to a noisier `platform_allowed` mismatch diagnostic.
`test_report_stage_rejects_native_dynamic_operation_audit_duplicate_allowed_platform` rejects
duplicate `allowed_platforms[]` entries so platform-gate evidence remains a deterministic set.
`test_report_stage_rejects_native_dynamic_operation_audit_negative_counts` keeps stage
`package_count` and package-row `artifact_count` non-negative, and
`test_report_rejects_native_plugins_payload_operation_audit_negative_package_count` applies the
same stable summary rule to PlatformBundle `native_plugins_payload` operation audit summaries.
`test_report_stage_rejects_native_dynamic_operation_audit_blank_diagnostic_entry` keeps the
signing/notarization `diagnostics[]` evidence meaningful: empty or whitespace-only rows make the
NativeDynamic stage fatal before a stable payload summary can be trusted.
`test_report_stage_rejects_native_dynamic_operation_audit_non_string_diagnostic_entry_before_array_shape`
keeps non-string signing/notarization diagnostic rows at
`native_dynamic report <audit>.diagnostics[index] must be a string` instead of collapsing them into
the broader whole-field string-array diagnostic.
`test_report_stage_rejects_native_dynamic_operation_audit_padded_diagnostic_entry` keeps the same
reason rows normalized: non-empty diagnostics with leading or trailing whitespace now fail at the
operation-audit schema boundary instead of passing through as publishable audit evidence.
`test_report_stage_rejects_pack_unknown_top_level_field` closes the Pack handoff on the same shared
loader path. A non-fatal `pack` report may only carry its stage metadata, asset/pack paths,
trim/determinism summaries, manifest counts, and explicit delta audit fields; unknown sidecar fields
make the wrapper stage fatal before PlatformBundle can trust `pack` or `delta_pack`.
`test_report_stage_rejects_pack_string_fields_non_string`,
`test_report_stage_rejects_pack_count_fields_non_integer`,
`test_report_stage_rejects_pack_string_array_fields_non_string_array`,
`test_report_stage_rejects_pack_bool_fields_non_bool`, and
`test_report_stage_rejects_pack_object_fields_non_object` close the typed shape of the same report.
Pack path fields must be strings when present, count fields must be integers, asset-list fields must
be string arrays, delta/determinism flags must be booleans, and manifest/trim objects must be objects
before final Report accepts Pack release evidence.
Pack nested release evidence is also typed before final Report trusts it.
`test_report_stage_rejects_pack_manifest_unknown_field`,
`test_report_stage_rejects_pack_manifest_nested_field_types`,
`test_report_stage_rejects_pack_delta_manifest_unknown_field`, and
`test_report_stage_rejects_pack_delta_manifest_field_types` close `manifest`, `delta_manifest`,
pack/chunk rows, asset rows, byte-hash arrays, and delta base/target manifests against the Rust
`zircon_export_pack` report shape.
`test_report_stage_rejects_pack_manifest_version_mismatch` and
`test_report_stage_rejects_pack_delta_manifest_version_mismatch` bind `manifest.pack.version`,
delta base/target `pack.version`, and `delta_manifest.format_version` to the currently supported
zrpack format version `1` before final Report trusts full-pack or ZRPD release evidence.
`test_report_stage_rejects_pack_manifest_count_mismatch` also
binds `asset_count` to `manifest.assets[]` length and `chunk_count` to `manifest.pack.chunks[]`
length before the report is accepted as publishable Pack evidence.
`test_report_stage_rejects_pack_manifest_count_schema_before_length_semantics` keeps that count
comparison behind row-level manifest schema, so malformed assets or chunks do not add length
mismatch noise.
`test_report_stage_rejects_pack_asset_chunk_reference_after_schema_clean` applies the same
schema-clean ordering to asset chunk references: `assets[].chunk_hash` is checked against the
document chunk table only after both asset rows and chunk rows are valid enough to compare.
`test_report_stage_rejects_pack_delta_manifest_count_mismatch` applies the same rule to delta
reports by binding `delta_asset_count` to `delta_manifest.changed_assets[]` length and
`delta_chunk_count` to `delta_manifest.chunks[]` length.
`test_report_stage_rejects_delta_count_schema_before_length_semantics` keeps that comparison
behind row-level delta schema: malformed `changed_assets[]` or `chunks[]` entries stop at their
field diagnostics before count mismatch diagnostics consume the same invalid rows.
`test_report_stage_rejects_delta_changed_asset_schema_before_set_semantics` keeps changed-asset
set derivation on the same boundary: a malformed `changed_assets[]` row is not used for changed
path, target-entry, or changed-chunk comparisons.
`test_report_stage_rejects_delta_chunk_schema_before_changed_chunk_semantics` mirrors that for
delta payload chunks, keeping malformed `chunks[]` rows out of changed-chunk hash comparison until
their own schema is clean.
`test_report_stage_rejects_delta_manifest_schema_before_embedded_match` keeps `.zrpd` embedded
manifest comparison behind the same boundary: a schema-invalid report `delta_manifest` stops at its
field diagnostics before `delta_pack embedded manifest does not match delta_manifest` can add
secondary noise.
`test_report_stage_rejects_pack_delta_missing_report_audit_field` makes those report-level delta
audit fields explicit release evidence whenever a Pack report publishes a `delta_pack`: the report
must include `delta_asset_count`, `delta_chunk_count`, `delta_removed_assets`, and
`delta_reused_assets` alongside the paired `delta_manifest`.
`test_report_stage_rejects_pack_delta_optional_path_blank_string` keeps optional delta path fields
honest even before the full delta publication branch is active: present `delta_pack` and
`previous_pack` values must be non-empty strings instead of relying on later pairing or handoff
checks to notice blank paths.
`test_report_stage_rejects_pack_delta_unpaired_previous_pack` then makes `previous_pack` part of
the same publication pair: it is only valid release evidence when a non-empty `delta_pack` is also
present.
`test_report_stage_rejects_pack_delta_unverified_apply` makes the writer self-check explicit at
the same Pack schema boundary: a non-fatal report that publishes `delta_pack` and `delta_manifest`
must also include `delta_apply_verified = true`; a missing flag fails boolean schema validation and
`false` fails with a field-level "must be true when delta_pack is published" diagnostic before
cross-stage PlatformBundle delta handoff checks run.
`test_report_stage_rejects_pack_delta_publication_padded_path_before_pairing` keeps the same
ordering for padded delta publication paths: `delta_pack` and `previous_pack` must pass trimmed
path-string schema before publication pairing, apply-verified, or PlatformBundle delta handoff
checks can consume them.
`test_report_stage_rejects_invalid_pack_delta_path_before_cross_stage_delta` extends that guard to
non-string `delta_pack` values: once the Pack stage report is fatal from schema validation,
PlatformBundle delta verification does not re-read it and add secondary non-empty/apply-verified
handoff diagnostics.
`test_report_stage_rejects_pack_path_field_padded_string` and
`test_report_stage_rejects_pack_delta_path_field_padded_string` keep path string evidence
canonical before path resolution: `asset_manifest`, `pack`, `stage_output`, `delta_pack`, and
`previous_pack` must be trimmed non-empty strings when present, so padded paths fail with a
field-level Pack schema diagnostic instead of drifting into missing-file or stage-output mismatch
diagnostics.
`test_report_stage_rejects_pack_manifest_asset_path_padded_string` also keeps embedded manifest
and duplicate-asset evidence behind the same schema boundary: Pack only recomputes
`deduplicated_assets` and compares `.zrpack` embedded manifests against report manifests when the
manifest document, chunk rows, and asset rows are schema-clean. A padded `manifest.assets[].path`
therefore stops at `must be a non-empty trimmed string` instead of adding
`pack embedded manifest does not match manifest` or `deduplicated_assets does not match ...`
noise.
`test_report_stage_rejects_pack_deduplicated_asset_path_padded_string` applies the same rule to the
reported `deduplicated_assets[]` array itself: padded duplicate-path evidence is rejected as a
field-level package path schema error before manifest duplicate-chunk comparison runs.
`test_report_stage_rejects_pack_required_file_missing`, `test_report_stage_rejects_pack_required_file_directory`,
`test_report_stage_rejects_pack_required_file_empty`, `test_report_stage_rejects_pack_delta_file_missing`, and
`test_report_stage_rejects_pack_delta_file_empty` keep Pack path release evidence tied to files: non-fatal
`asset_manifest` and `pack` must exist as non-empty files, and present `delta_pack` or
`previous_pack` values must also resolve to non-empty files before final Report trusts artifacts.
`test_report_stage_rejects_pack_file_invalid_header`,
`test_report_stage_rejects_pack_file_embedded_manifest_mismatch`,
`test_report_stage_rejects_delta_file_embedded_manifest_mismatch`, and
`test_report_stage_rejects_previous_pack_embedded_manifest_mismatch` keep those artifact files bound
to their binary `ZRPK` / `ZRPD` embedded manifests before PlatformBundle can inherit Pack outputs.
Payload extent, range, and hash diagnostics now wait for the embedded manifest chunk rows to be
schema-clean first. `test_report_stage_rejects_pack_file_chunk_schema_before_payload_semantics`
keeps a negative `manifest.pack.chunks[0].size` at the field diagnostic instead of also emitting
`manifest offset`, payload-range, or payload-hash secondary noise.
`test_report_stage_rejects_pack_file_payload_hash_mismatch`,
`test_report_stage_rejects_delta_file_payload_hash_mismatch`, and
`test_report_stage_rejects_previous_pack_payload_hash_mismatch` extend that evidence to physical
chunk bytes by corrupting payload data while keeping the embedded manifest and report manifest current.
`test_report_stage_rejects_pack_file_payload_manifest_gap`,
`test_report_stage_rejects_delta_file_payload_manifest_gap`, and
`test_report_stage_rejects_previous_pack_payload_manifest_gap` keep `manifest_offset` tied to the
declared payload extent so final Report rejects unreported bytes between the last chunk and the
embedded manifest.
`test_report_stage_rejects_pack_file_manifest_trailing_bytes`,
`test_report_stage_rejects_delta_file_manifest_trailing_bytes`, and
`test_report_stage_rejects_previous_pack_manifest_trailing_bytes` reject the symmetric case where
the embedded manifest is valid but the artifact carries trailing bytes after it.
`test_report_stage_rejects_pack_chunk_hash_malformed_before_chunk_semantics` keeps chunk-row
schema ahead of derived pack semantics: malformed `manifest.pack.chunks[].hash` rows stop at the
32-byte integer-array diagnostic before `total_size`, expected chunk-offset, embedded manifest, or
trim/manifest closure checks consume the same row.
`test_report_stage_rejects_pack_chunk_or_asset_size_schema_before_size_semantics` applies that
same schema-clean rule to size math: negative chunk offsets, negative chunk sizes, and negative
asset sizes fail their field schema before Pack compares `assets[].size` with `pack.chunks[]`
sizes.
`test_report_stage_rejects_pack_manifest_duplicate_chunk_hash` and
`test_report_stage_rejects_pack_delta_manifest_duplicate_chunk_hash` reject repeated chunk hashes in
full-pack, delta base/target, and delta payload chunk tables before accepting content-addressed pack
evidence.
`test_report_stage_rejects_pack_manifest_unsorted_chunk_hash` and
`test_report_stage_rejects_pack_delta_manifest_unsorted_chunk_hash` keep those same chunk tables in
deterministic hash order, matching the writer's normalized manifest layout.
`test_report_stage_rejects_pack_manifest_asset_missing_chunk_hash` and
`test_report_stage_rejects_pack_delta_manifest_asset_missing_chunk_hash` reject pack document assets
whose `chunk_hash` is absent from their own `pack.chunks` table.
`test_report_stage_rejects_pack_manifest_asset_chunk_size_mismatch`,
`test_report_stage_rejects_pack_delta_manifest_asset_chunk_size_mismatch`, and
`test_report_stage_rejects_pack_delta_manifest_payload_asset_chunk_size_mismatch` reject asset rows
whose declared `size` drifts from the referenced full-pack or ZRPD payload chunk size.
`test_report_stage_rejects_delta_asset_negative_size_before_entry_semantics` keeps delta changed
asset matching behind the same schema-clean row contract: negative `delta_manifest.target.assets[]`
or `delta_manifest.changed_assets[]` sizes fail their field schema before changed asset rows are
compared with target manifest entries.
`test_report_stage_rejects_pack_manifest_total_size_mismatch` and
`test_report_stage_rejects_pack_delta_manifest_total_size_mismatch` bind each pack document's
`pack.total_size` to the sum of its chunk row sizes.
`test_report_stage_rejects_pack_manifest_chunk_offset_gap`,
`test_report_stage_rejects_pack_delta_manifest_chunk_offset_gap`, and
`test_report_stage_rejects_pack_delta_manifest_payload_chunk_offset_gap` reject full-pack,
delta base/target, and ZRPD payload chunk tables whose offsets do not form the contiguous payload
byte range written after the 24-byte pack/delta header.
`test_report_stage_rejects_pack_delta_asset_set_mismatch` closes the delta asset-set semantics:
`delta_manifest.removed_assets`, report `delta_removed_assets`, report `delta_reused_assets`, and
`delta_manifest.changed_assets` must match the base/target path and chunk-hash differences embedded
in the delta manifest.
`test_report_stage_rejects_pack_delta_changed_asset_manifest_mismatch` tightens the changed payload
itself: changed asset rows must match the full target manifest asset entries, and delta chunk hashes
must match the changed asset chunk hashes exactly.
`test_report_stage_rejects_pack_delta_target_manifest_mismatch` then binds that delta target back to
the outer Pack `manifest`, so the report cannot publish one full-pack manifest while embedding a
different target package in `delta_manifest.target`.
`test_report_stage_rejects_pack_delta_manifest_publication_mismatch` binds the delta artifact to its
manifest evidence: `delta_pack` and `delta_manifest` must appear together in a non-fatal Pack report.
`test_report_stage_rejects_pack_trim_report_manifest_asset_mismatch` then binds
`trim_report.included_assets` to the embedded `manifest.assets[].path` set, preventing count-correct
reports from drifting away from the actual packed asset identities.
`test_report_stage_rejects_pack_trim_report_unresolved_preflight` rejects non-fatal Pack reports
that still carry `trim_report.duplicate_assets` or `trim_report.missing_dependencies`, matching the
Rust preflight rule that those lists are fatal before pack bytes are written.
`test_report_stage_rejects_pack_required_path_blank_string` keeps the required Pack path evidence
usable by rejecting blank `asset_manifest`, `pack`, and `stage_output` strings before path-location
or stage-handoff diagnostics run.
`test_report_stage_rejects_pack_stage_output_outside_current_stage` rejects Pack reports whose
`stage_output` resolves outside the current `<out>/stages/pack` directory derived from the loaded
report path.
`test_report_stage_rejects_pack_deduplicated_assets_manifest_mismatch` derives the expected
`deduplicated_assets` set from repeated `manifest.assets[].chunk_hash` values, matching the writer's
path-sorted first-owner rule before accepting duplicate-content audit evidence.
`test_report_stage_rejects_pack_deduplicated_assets_blank_entry` keeps the field-level schema gate
separate from that manifest comparison by rejecting blank `deduplicated_assets[]` entries directly.
The aggregate `test_report_stage_rejects_pack_string_array_fields_non_string_array` also locks the
entry-type boundary for `deduplicated_assets[]`, `delta_removed_assets[]`, and
`delta_reused_assets[]`: malformed rows now produce indexed `must be a string` diagnostics and do
not fall back to whole-field string-array diagnostics.
`test_report_stage_rejects_pack_delta_path_array_blank_entry` applies the same field-level path
array gate to `delta_removed_assets`, `delta_reused_assets`, and
`delta_manifest.removed_assets`, so blank delta audit rows cannot hide behind later set mismatch
diagnostics.
`test_report_stage_rejects_pack_manifest_asset_empty_path` and
`test_report_stage_rejects_pack_delta_manifest_asset_empty_path` keep pack manifest asset identities
non-empty across the outer `manifest.assets[]`, `delta_manifest.base.assets[]`,
`delta_manifest.target.assets[]`, and `delta_manifest.changed_assets[]` rows.
`test_report_stage_rejects_pack_manifest_asset_path_shape`,
`test_report_stage_rejects_pack_delta_manifest_asset_path_shape`, and
`test_report_stage_rejects_pack_delta_removed_asset_path_shape` extend that identity gate to safe
relative normalized asset paths and duplicate normalized path rejection across full pack manifests,
delta base/target manifests, delta changed assets, and embedded delta removed assets.
`test_report_stage_rejects_pack_deduplicated_assets_path_shape`,
`test_report_stage_rejects_pack_delta_path_array_shape`, and
`test_report_stage_rejects_pack_trim_report_path_shape` apply the same safe normalized path rule to
report-level deduplication, delta audit, and trim audit path arrays before later manifest/set
reconciliation can hide malformed path evidence behind generic mismatch diagnostics.
`test_report_stage_rejects_pack_manifest_asset_path_padded_string` and
`test_report_stage_rejects_pack_trim_report_path_padded_string` keep padded package paths on the
field-level non-empty trimmed string schema path before safe-path, normalized-path, duplicate-path,
or trim/manifest consistency diagnostics run.
The same focused Pack path-string schema module covers delta evidence with
`test_report_stage_rejects_pack_delta_manifest_path_padded_string` and
`test_report_stage_rejects_pack_delta_audit_path_padded_string`, keeping padded delta
`assets[].path`, `changed_assets[].path`, `removed_assets[]`, and report-level delta audit arrays
from degrading into later delta set mismatch diagnostics.
`test_report_stage_rejects_pack_manifest_negative_layout_numbers` keeps the same pack-document
schema helper from accepting negative byte counts or offsets for `pack.total_size`,
`pack.chunks[].offset`, `pack.chunks[].size`, and `assets[].size`.
`test_report_stage_rejects_pack_manifest_missing_required_field` makes the nested Pack manifest
release-evidence fields mandatory: `pack`, `assets`, `pack.version`, `pack.total_size`,
`pack.chunks`, chunk `hash`/`offset`/`size`, and asset `path`/`chunk_hash`/`size`.
`test_report_stage_rejects_pack_delta_manifest_missing_required_field` does the same for the
top-level ZRPD manifest fields: `format_version`, `base`, `target`, `chunks`, `changed_assets`, and
`removed_assets`.
`test_report_stage_rejects_pack_trim_report_unknown_fields` and
`test_report_stage_rejects_pack_trim_report_field_types` close `trim_report`, trimmed-asset rows,
missing-dependency rows, duplicate/included asset arrays, and diagnostics arrays. The trim-report
string arrays split non-array fields from non-string entries, and
`test_report_stage_rejects_pack_trim_report_string_array_non_string_entry_before_array_shape`
keeps those diagnostics pinned to the exact bad array index. These checks live
across the Pack schema modules: stage wrapper checks stay in `pipeline_report_pack_stage_schema.py`,
trim-report checks stay in `pipeline_report_pack_trim_schema.py`,
pack manifest/count/dedup checks stay in `pipeline_report_pack_manifest_schema.py`, and delta
manifest/count/asset-set/chunk checks stay in `pipeline_report_pack_delta_schema.py`. The shared
`pipeline_report_stage_schema.py` remains only the cross-stage dispatcher.
Common Pack manifest, chunk, asset, and delta-manifest fixtures now live in
`tools/zircon_export/tests/pack_test_support.py`, while Pack schema report staging and assertion
helpers live in `tools/zircon_export/tests/pack_schema_test_support.py`.
`test_pipeline_report_pack_stage_schema.py` now owns outer Pack report and manifest regressions;
`test_pipeline_report_pack_trim_report_schema.py` owns trim-report schema, preflight, and
manifest-consistency regressions;
`test_pipeline_report_pack_delta_top_level_schema.py` owns delta manifest top-level closed-schema,
required-field, typed-field, and path-array blank-entry regressions; `test_pipeline_report_pack_delta_schema.py`
owns embedded pack-document, delta asset-set, target-manifest, changed-chunk, and publication-pairing
semantic regressions. `test_report_stage_rejects_pack_trim_report_blank_diagnostic_entry` keeps
trim-report diagnostic evidence on the Pack schema path instead of letting blank rows fall through
to later trim consistency diagnostics.
`test_report_rejects_bundle_manifest_unknown_top_level_field` keeps loaded bundle manifests on the
closed-schema path: an extra key such as `unsigned_sidecar` marks the final Report fatal before Hub
or editor readers can treat unaudited manifest metadata as release evidence.
`test_report_rejects_bundle_manifest_string_fields_non_string`,
`test_report_rejects_bundle_manifest_object_fields_non_object`, and
`test_report_rejects_bundle_manifest_template_files_non_object_array` close the typed shape of that
loaded manifest. The manifest `profile`, host/pack/native plugin path, and provenance fields must
be strings when present, `template_resolution`, `template`, and `native_plugins_payload` must be
objects when present, and `template_files` must be an object array before final Report compares the
manifest to the PlatformBundle stage report or exposes it as release evidence.
`test_report_rejects_bundle_manifest_required_path_blank_string` keeps the same top-level
`bundle.json` string evidence from accepting empty or whitespace-only values: `profile`,
host/pack path fields, and source-origin fields must be non-empty after trimming before final Report
compares the manifest to the stage report.
`test_report_rejects_bundle_manifest_native_plugins_payload_nested_schema`,
`test_report_rejects_bundle_manifest_template_nested_schema`,
`test_report_rejects_bundle_manifest_template_resolution_nested_schema`, and
`test_report_rejects_bundle_manifest_template_files_nested_schema` apply the same typed schema to
the nested release evidence loaded from `bundle.json`. The embedded NativeDynamic payload,
template report, template-root resolution report, and copied template file rows now reuse the same
schema helpers as their stage-report counterparts with `PlatformBundle bundle_manifest ...` labels.
If that published manifest evidence has an unknown field or wrong type, final Report emits the
manifest-local diagnostic and skips the broad `does not match stage report` comparison noise for
that invalid manifest.
`test_report_rejects_platform_bundle_unknown_top_level_field` applies the same closed-schema rule
to the non-fatal `PlatformBundle` stage report itself before final Report follows the report's
bundle manifest, host, pack, template, or NativeDynamic payload pointers.
`test_report_rejects_platform_bundle_string_fields_non_string`,
`test_report_rejects_platform_bundle_object_fields_non_object`, and
`test_report_rejects_platform_bundle_template_files_non_object_array` close the typed shape of that
same stage report. PlatformBundle bundle/host/pack/native plugin path and provenance fields must be
strings when present, `template_resolution`, `template`, and `native_plugins_payload` must be
objects when present, and `template_files` must be an object array before final Report trusts bundle
manifest, template hash, or NativeDynamic payload evidence from the report.
`test_report_rejects_platform_bundle_report_template_nested_schema_before_manifest_compare` and
`test_report_rejects_platform_bundle_report_native_payload_schema_before_manifest_compare` extend
that rule into the stage report's nested release evidence. `template`, `template_resolution`,
`template_files`, and `native_plugins_payload` now run through the same nested schema helpers as
loaded `bundle.json`, but with `PlatformBundle report ...` labels. If the stage report evidence is
already malformed, final Report stops before following the bundle manifest and does not add broad
`bundle_manifest ... does not match stage report` diagnostics for the same broken value.
`test_report_stage_rejects_platform_bundle_unknown_top_level_field` and
`test_report_stage_rejects_platform_bundle_nested_template_schema` also register that PlatformBundle
report schema with the shared stage-report loader. A bad PlatformBundle report now marks
`PlatformBundle` in `fatal_stages` at load time, matching the Pack, NativeDynamic, and SourceTemplate
schema attribution paths instead of relying only on later PlatformBundle manifest or semantic
diagnostics.
`test_report_stage_rejects_platform_bundle_required_path_blank_string` keeps the same stage-loader
schema attribution for blank required PlatformBundle report strings: `bundle`, host/pack
source/provenance fields, and `bundle_manifest` are rejected as non-empty release evidence before
final Report follows those paths or compares the loaded `bundle.json`.
`test_report_stage_rejects_platform_bundle_optional_path_blank_string` extends that rule to
optional report strings that become release evidence when present, including bundled delta paths,
delta source provenance, and the NativeDynamic plugins directory.
Those PlatformBundle stage-report and loaded `bundle.json` schema rules now live in
`pipeline_report_platform_bundle_schema.py`; `pipeline_report_platform_bundle.py` keeps the final
Report orchestration, manifest loading, path containment, hash, and NativeDynamic payload
consistency checks. Its local primitive checks are imported from
`pipeline_report_schema_primitives.py`, while nested template/payload schema ownership stays in the
PlatformBundle-specific helper modules.
`pipeline_report_platform_bundle_template.py` follows the same split: copied-template file matching,
path resolution, and on-disk SHA-256 checks stay in the evidence helper, while
`pipeline_report_platform_bundle_template_schema.py` owns the embedded template report schema,
copied `template_files[]` row schema, and the field-level hash/format-version gates.
`pipeline_report_platform_bundle_template_manifest_schema.py` owns the parsed `template.manifest`
path, TOML, closed-schema, and manifest/report identity gates, so the main template schema module
can keep delegating source-template manifest semantics instead of accumulating another long
diagnostic block. The main template schema imports only
`template_report_manifest_path_diagnostics(...)` from that module; manifest TOML reads, manifest
bundle defaults, and manifest/report identity helpers stay out of the embedded template report
schema file.
`pipeline_report_platform_bundle_template_resolution_schema.py` owns the template-root resolution
field shape, required/null rules, candidate/skipped-candidate row schemas, and the aggregate
diagnostic entry point. `pipeline_report_platform_bundle_template_resolution_semantics.py` owns the
selected-candidate, fatal/no-match/root-failure, candidate identity/profile/bundle-format, skipped
diagnostic, and template-root containment semantics. Shared schema primitives still own the basic
bool/integer/string/object/object-array diagnostic text, while the index-aware string-array helper
is reused for PlatformBundle entry checks.
`pipeline_report_platform_bundle_template_schema_helpers.py` now carries the local field/table
diagnostic helpers for that area: unknown-field scans, typed table/sequence checks, non-empty string
checks, trimmed-string checks, string-array entry checks, SHA-256 shape checks,
safe-relative path checks, enum checks, and duplicate `template.files[].path` detection.
Its string-array helper now distinguishes non-array fields from non-string entries, so
`template.diagnostics[]`, `template_resolution.diagnostics[]`,
`template_resolution.candidates[].compatible_profiles[]`, and
`template_resolution.skipped_candidates[].diagnostics[]` publish `<field>[index] must be a string`
before final Report runs higher-level template semantics.
The main template schema module stays focused on the report
sections and template-specific semantic gates such as content-hash, identity, and profile
membership. The helper ownership is singular: sequence object-shape diagnostics also live in the
helper module, so the main schema module does not shadow field-level helper behavior after the
split.
`test_report_rejects_template_report_unknown_field`,
`test_report_rejects_template_report_file_unknown_field`, and
`test_report_rejects_template_file_unknown_field` apply the same closed-schema rule to
`PlatformBundle` template release evidence. The embedded `template` report, its `template.files[]`
manifest rows, and copied `template_files[]` rows all have explicit allow-lists, so sidecar metadata
cannot ride through the final Report alongside template hash verification.
`test_report_rejects_template_report_string_fields_non_string`,
`test_report_rejects_template_report_count_fields_non_integer`,
`test_report_rejects_template_report_bool_fields_non_bool`,
`test_report_rejects_template_report_string_array_fields_non_string_array`,
`test_report_rejects_template_report_object_fields_non_object`,
`test_report_rejects_template_report_files_non_object_array`,
`test_report_rejects_template_bundle_string_fields_non_string`,
`test_report_rejects_template_file_string_fields_non_string`, and
`test_report_rejects_template_copied_file_string_fields_non_string` close the typed shape of the
same template evidence. Template report metadata must use string/count/boolean/string-array fields
as declared, `template.bundle` and `template.files` must have object shapes, and copied
`template_files[]` source/destination rows must be strings before final Report performs template
hash matching or bundle file checks.
`test_report_rejects_template_report_string_field_blank` also keeps present template report string
metadata from being empty or whitespace-only: `bundle_format`, content hashes, engine/profile/target
identities, host/template paths, and strategy fields must be non-empty after trimming before final
Report exposes the embedded template report as release evidence. Fields that are genuinely
unproduced may still be omitted or set to `None`; a hand-authored `" "` value is rejected at the
schema layer instead of relying on later hash or path diagnostics.
`test_report_rejects_template_report_engine_version_mismatch` and
`test_report_rejects_template_report_target_platform_mismatch` preserve the identity checks that
`template.toml` validation already performed before embedding the template report. When both actual
and expected fields are present, `engine_version` must match `expected_engine_version`, and
`target_platform` must match `expected_target_platform` after the same platform alias normalization
used by template validation. This keeps hand-authored final reports from swapping expected release
identity while leaving genuinely unproduced fields as schema-optional evidence.
`test_report_rejects_template_report_manifest_path_mismatch` ties the embedded `manifest` evidence
back to its `template_dir`: when both strings are present, final Report resolves them and requires
`manifest` to point at `template_dir/template.toml`. A report can no longer point template metadata
at an arbitrary neighboring TOML file while keeping the rest of the template evidence self-consistent.
`test_report_rejects_template_report_manifest_missing_file` also keeps that canonical manifest path
bound to current disk state. Once a report publishes `template.manifest`, final Report requires the
resolved `template_dir/template.toml` to exist as a file, matching the source template validator's
manifest existence gate.
`test_report_rejects_template_report_manifest_invalid_toml` adds the matching parse gate. The
canonical manifest file is opened and parsed as TOML before the embedded template report is trusted,
so a stale or hand-authored report cannot point at an unreadable or syntactically invalid
`template.toml` and still publish release evidence.
`test_report_rejects_template_report_manifest_unknown_fields` keeps that parsed manifest on the
same closed schema accepted by source template validation. After final Report parses
`template.manifest`, unknown top-level keys, `[paths]` keys, `[bundle]` keys, or `[[files]]` row
keys are fatal, so a hand-authored report cannot point at sidecar template metadata that
`validate_export_template(...)` would have rejected.
`test_report_rejects_template_report_manifest_shape_mismatch` applies the same source-template
table and array shape boundaries to parsed manifest evidence: malformed `[paths]`, `[bundle]`, or
`[[files]]` structures are fatal before final Report uses manifest identity, host path, bundle
layout, or file-row bindings.
`test_report_rejects_template_report_manifest_scalar_field_schema` closes the remaining source
template scalar gate in final Report: parsed manifests must publish non-empty `template_id`,
engine/target identity, host/resource/plugin/bundle strategy fields, and a SHA-256 `content_hash`;
strategy fields also stay inside the source validator's allowed-value sets before manifest/report
identity comparisons are trusted.
`test_report_rejects_template_report_manifest_compatible_profiles_schema` keeps the parsed
`compatible_profiles` value aligned with source validation as an optional string array with no blank
entries and no duplicate profile entries, before final Report compares it against embedded
`template.compatible_profiles`.
Manifest identity regression fixtures now build schema-clean parsed manifests with both `[paths]`
and `[[files]]` sections before changing only the target identity field, so stricter
source-template gates do not mask the specific manifest/report drift under test.
`test_report_rejects_template_report_manifest_paths_host_schema` keeps parsed `[paths]` evidence on
the source-template host path contract: `[paths]` must exist, `host_executable` must be a non-empty
string, and the path must remain safely relative before final Report resolves it under
`template_dir`.
`test_report_rejects_template_report_manifest_bundle_field_schema` applies the same optional
`[bundle]` path-field contract after the parsed manifest passes scalar and path checks: any present
bundle path field must be a string, cannot be blank, and must remain safe relative before final
Report compares it with embedded `template.bundle`.
`test_report_rejects_template_report_manifest_file_row_schema` brings parsed `[[files]]` row
shape into the same final Report boundary. Each manifest row must publish a non-empty safe relative
`path`, a SHA-256 `sha256`, an optional non-blank safe relative `bundle_path`, and an optional
string `purpose` before final Report performs row count, bundle path, hash, or purpose identity
checks against embedded `template.files[]`.
`test_report_rejects_template_report_manifest_missing_file_rows` keeps parsed template manifests
from publishing no file rows. Missing `files` and explicit root-level `files = []` now fail as
`template.manifest must declare at least one [[files]] entry`, matching
`validate_export_template(...)` before any manifest/report row-count identity comparison can mask
the source-template error.
`test_report_rejects_template_report_manifest_file_row_duplicates` applies the source-template
uniqueness rule to parsed manifest rows before row-count identity gates run. Final Report now
normalizes manifest `[[files]].path` and defaulted `bundle_path` values, then rejects duplicate
template source files or duplicate bundle output destinations with the same diagnostics used by
`validate_export_template(...)`.
`test_report_rejects_template_report_manifest_format_version_mismatch` keeps that parsed manifest
on the supported template schema version: `format_version` must be an integer and must equal the
current `EXPORT_TEMPLATE_FORMAT_VERSION`. This prevents a report from embedding current-looking
template evidence while the referenced `template.toml` belongs to another template schema version.
`test_report_rejects_template_report_manifest_template_id_mismatch` then binds parsed manifest
identity back to the embedded report: when both `template.toml` and the embedded `template` report
publish non-empty `template_id` values, they must match. This prevents a final Report from pointing
at a different template package while preserving self-consistent embedded fields.
`test_report_rejects_template_report_manifest_engine_version_mismatch` applies the same binding to
`engine_version`, so a report cannot point at a manifest for a different engine build while the
embedded template report continues to publish another engine identity.
`test_report_rejects_template_report_manifest_target_platform_mismatch` then applies the platform
side of the same manifest/report identity binding with `normalize_target_platform(...)`, so alias
forms such as `windows` remain equivalent to `windows-x86_64`, but a report cannot point at a
Linux template manifest while publishing a Windows embedded template target.
`test_report_rejects_template_report_manifest_strategy_field_mismatch` extends the same
manifest/report binding to the template strategy fields that drive bundle materialization:
`host_kind`, `resource_strategy`, `plugin_strategy`, and `bundle_format`. Those fields already pass
the embedded report allowed-value gate, but final Report now also rejects a report that points at a
manifest with different host, resource, plugin, or bundle packaging semantics.
`test_report_rejects_template_report_manifest_content_hash_mismatch` also binds the parsed
`template.toml` `content_hash` to the embedded report's `template.content_hash` when both are
published. This complements the existing computed hash check over embedded `template.files[]`: a
hand-authored report cannot keep its embedded file rows and computed hash self-consistent while
pointing at a manifest that declares a different template content hash.
`test_report_rejects_template_report_manifest_compatible_profiles_mismatch` performs the same
binding for `compatible_profiles[]` once both manifest and embedded report publish valid string
arrays. The source validator copies this list from `template.toml` into the template report, so
final Report rejects profile-list drift even when both lists independently contain the current
profile.
`test_report_rejects_template_report_manifest_host_executable_mismatch` completes that
manifest/report binding for `[paths].host_executable`. Final Report normalizes the manifest's
relative host path against `template_dir` and compares it with the embedded
`template.host_executable`, so a report cannot point at one manifest host slot while publishing a
different embedded host, even when both files exist and both are listed in `template.files[]`.
`test_report_rejects_template_report_manifest_bundle_field_mismatch` applies the same final Report
binding to template bundle layout fields. Final Report default-fills the parsed manifest `[bundle]`
table with the same `root`, `host_path`, `pack_path`, `delta_pack_path`, and `manifest_path`
defaults used by `validate_export_template(...)`, then compares the clean string values with
embedded `template.bundle`. A hand-authored report can no longer point at a manifest that routes the
host, pack, delta, manifest, or bundle root differently while keeping embedded bundle evidence
self-consistent.
`test_report_rejects_template_report_manifest_file_bundle_path_mismatch` then binds parsed
manifest `[[files]]` rows back to embedded `template.files[]`. Final Report compares schema-clean
file rows by index after applying the same path and bundle-path normalization used by
`validate_export_template(...)`, including the default `bundle_path = path` behavior and optional
`purpose` default. This prevents a hand-authored report from keeping embedded file rows, hashes, and
content hash self-consistent while pointing at a manifest whose template file would materialize to a
different bundle destination.
`test_report_rejects_template_report_host_executable_not_declared_file` preserves the generated
template rule that `paths.host_executable` must also appear in `[[files]]`. Final Report resolves
the embedded `host_executable` relative to `template_dir` and requires that relative path to be one
of `template.files[].path`, so an unlisted executable cannot be published as template host evidence.
`test_report_rejects_template_report_missing_success_evidence_field` keeps successful embedded
template reports complete before any semantic checks trust them. When `template.fatal` is false,
the final Report requires the full generated evidence shape: template identity, format and expected
format versions, engine and expected engine, target and expected target platform, profile,
host/resource/plugin/bundle strategy fields, manifest and host executable paths, compatible
profiles, both content hashes, diagnostics, bundle table, and `files[]`. Missing fields and
explicit `null` values produce field-level `PlatformBundle report template.<field> must be ...`
diagnostics; fatal template-validation reports still use their diagnostic-bearing failure shape.
Those successful template report string fields must also be non-empty trimmed strings before later
enum, hash, manifest-identity, profile-membership, or path-closure checks run.
`test_report_rejects_template_report_padded_required_string_field` keeps hand-authored embedded
template reports from publishing padded identity, path, hash, or profile evidence and relying on
later semantic checks to catch the drift indirectly.
`test_report_rejects_template_report_host_executable_missing_file` also keeps that embedded host
path tied to the current template directory contents. A hand-authored report cannot publish a
declared `template.host_executable` whose file has disappeared from disk, matching the source
`validate_export_template(...)` rule that rejects missing template host files.
`test_report_rejects_template_report_enum_field_unknown_value` keeps the embedded report on the
same allowed-value sets as `template.toml`: `host_kind`, `resource_strategy`, `plugin_strategy`, and
`bundle_format` must be one of the current export-template enum values after passing the string and
non-empty gates. A final Report can no longer publish an unknown host kind or packaging strategy as
opaque text.
`test_report_rejects_template_report_missing_profile_membership` keeps non-empty
`compatible_profiles[]` tied to the embedded `profile` field. Empty profile lists still mean the
template is unrestricted, but once the list is present and non-empty it must contain the report
profile, matching the `template.toml` validation rule used during template selection.
`test_report_rejects_template_report_padded_compatible_profile_entry` applies the same
trimmed-string rule to embedded profile-list entries before membership or manifest/report identity
checks consume them.
`test_report_rejects_template_report_duplicate_compatible_profile_entry` keeps the embedded
template report profile list unique before profile membership or manifest/report identity checks
consume it, so duplicated profile evidence is rejected as a field-level schema issue.
`test_report_rejects_template_report_hash_field_malformed` narrows the two top-level template hash
fields after that non-empty string gate: `content_hash` and `computed_content_hash` must be SHA-256
hex strings whenever they are present. This mirrors `template.toml` validation and prevents a
hand-authored PlatformBundle report from publishing arbitrary text as template content hash
evidence.
`test_report_rejects_template_report_content_hash_mismatch` then recomputes the template content
hash from the embedded `template.files[]` rows and rejects both `content_hash` and
`computed_content_hash` when either valid digest does not match that derived value. The recompute
waits for the file rows to be object-shaped and to carry valid `path`, `bundle_path`, and `sha256`
strings, so malformed file-row diagnostics remain the primary error when the lower-level evidence is
already broken.
`test_report_rejects_template_file_padded_sha256` applies the trimmed-string gate to embedded
`template.files[].sha256` rows before digest-shape and source-file hash checks run, so padded
per-file digests fail as field evidence instead of degrading into only a SHA-256 shape diagnostic.
`test_report_rejects_template_file_malformed_sha256` gives the same field-level hash-shape rule to
each `template.files[].sha256` row. A malformed per-file digest now fails as
`PlatformBundle report template.files[0].sha256 must be a SHA-256 hex digest` before copied-file
hash comparison can turn it into a broader destination mismatch.
`test_report_rejects_template_file_source_hash_mismatch` then re-reads each declared
`template.files[].path` under `template.template_dir` and compares the current bytes to the embedded
`sha256`. This mirrors source `template.toml` validation after the report is handoff material: a
hand-authored final Report cannot keep a self-consistent `content_hash` while the actual template
source file has been changed or removed.
`test_report_rejects_template_file_unsafe_relative_path` also keeps `template.files[].path` and
`template.files[].bundle_path` on the same safe-relative path contract as `template.toml`: absolute
paths, empty segments, `.` segments, or `..` escapes are rejected at schema time before template file
lookup, content-hash recompute, or bundle copy verification can reinterpret them.
`test_report_rejects_template_file_padded_required_path_field` also requires those embedded path
fields to already be trimmed strings, `test_report_rejects_template_file_padded_sha256` applies the
same rule to embedded per-file hashes, and
`test_report_rejects_template_file_purpose_padded_when_present` applies the same present-when-trimmed
rule to optional `template.files[].purpose`. The embedded template file semantic checks now consume
only schema-clean rows: content-hash recomputation, source-file hash reads, and host membership skip
rows whose `path`, `bundle_path`, or `sha256` evidence is already untrimmed, unsafe, or malformed.
These field-level diagnostics run before content-hash, host membership, manifest identity, or
copied-file closure can report broader drift.
`test_report_rejects_template_file_duplicate_path` preserves the same one-row-per-template-source
contract after the template has been embedded in the PlatformBundle report. The final Report schema
normalizes `template.files[].path` before comparing rows and rejects duplicates even when a
hand-authored report also recalculates `content_hash`, so duplicated source-file evidence cannot be
hidden behind an otherwise self-consistent template hash.
`test_report_rejects_template_file_duplicate_bundle_path` applies the same uniqueness rule to
`template.files[].bundle_path`, the final bundle output location. This closes the hand-authored
report case where two different template sources publish one bundle-relative destination and then
hide the overwrite behind a recalculated `content_hash`.
`test_report_rejects_template_file_purpose_blank_when_present` mirrors the source `template.toml`
metadata rule after embedding: `template.files[].purpose` is optional, but when a final Report
publishes it, the value must not be whitespace-only. This keeps hand-authored reports from
reintroducing template file metadata that the template validator would have rejected.
`test_report_rejects_copied_template_file_destination_mismatch` binds copied template-file evidence
back to the declared bundle layout. When `template_files[].source` matches a `template.files[]` row,
the copied `destination` must equal the current PlatformBundle `bundle` plus optional
`template.bundle.root` plus that row's `bundle_path`. This keeps a report from moving a valid
template source file to a different bundle-relative path while preserving the expected SHA-256.
`test_report_rejects_duplicate_copied_template_file_entry` keeps copied template-file rows unique.
The generated PlatformBundle stage appends at most one `template_files[]` row per copied
source/destination pair, so final Report now rejects duplicate copied rows before treating repeated
evidence as additional release output.
`test_report_rejects_template_copied_file_padded_required_string` keeps copied
`template_files[].source` and `template_files[].destination` trimmed before hash matching, bundle
containment, destination existence, or copied-file closure diagnostics run.
`test_report_rejects_template_bundle_unsafe_relative_path` applies the same path-traversal gate to
embedded `template.bundle` output override fields. `root = "."` and generated empty-string default
markers remain valid because they are the current template report representation for default bundle
layout, but non-empty absolute paths, empty segments, or `..` escapes now fail as direct
`template.bundle.<field> must be a safe relative path` diagnostics before final Report accepts the
bundle layout evidence.
`test_report_rejects_template_report_format_version_mismatch` keeps the same top-level template
report from publishing an unsupported template schema version after it has left `template.toml`
validation. Both `expected_format_version` and `format_version` must equal the current
`EXPORT_TEMPLATE_FORMAT_VERSION`, so a hand-authored embedded template report cannot advertise
`999` as accepted release evidence merely because the field is an integer.
`test_report_rejects_template_report_non_fatal_with_diagnostics` keeps optional embedded template
state fields coherent when they are present. If a report publishes `template.fatal = false`, its
`template.diagnostics[]` evidence must be empty; generated warnings or failures belong either in a
fatal template validation report or in the surrounding stage diagnostics, not in a non-fatal embedded
template report.
`test_report_rejects_template_report_padded_diagnostic_entry` keeps fatal embedded template
diagnostic rows trimmed too. A fatal report still needs concrete `template.diagnostics[]` evidence,
but each row must already be a non-empty trimmed string before the final Report accepts it as the
reason for trusting the fatal template-validation state.
`test_report_rejects_template_report_fatal_without_diagnostics` applies the inverse optional-state
rule: if a report publishes `template.fatal = true`, its `template.diagnostics[]` evidence must
exist and include at least one non-empty diagnostic string. This mirrors
`validate_export_template(...)`, where fatal template validation paths set `fatal` only after
appending concrete failure evidence.
`test_report_rejects_template_bundle_unknown_field` keeps the same gate on the embedded
`template.bundle` object; only `root`, `manifest_path`, `host_path`, `pack_path`, and
`delta_pack_path` are accepted.
`test_report_rejects_template_bundle_padded_path_field` keeps every non-empty embedded bundle
override field trimmed before safe-relative and bundle-layout checks run; generated empty-string
default markers and `root = "."` remain valid report values.
`test_report_rejects_template_report_manifest_padded_scalar_field` applies the same canonical
string-evidence rule to the source `template.toml` file that final Report reloads through
`template.manifest`: top-level `template_id`, `engine_version`, `target_platform`, `host_kind`,
`resource_strategy`, `plugin_strategy`, `bundle_format`, and `content_hash` must already be
non-empty trimmed strings before SHA-256, allowed-value, target-platform alias, or
manifest/report identity checks consume them.
`test_report_rejects_template_report_manifest_padded_compatible_profile_entry` closes the
same source-manifest path for profile lists: `compatible_profiles[]` entries read from
`template.toml` must be trimmed before uniqueness, manifest/report identity, or profile
membership checks consume them.
`test_report_rejects_template_report_manifest_padded_path_field` keeps the same source
`template.toml` boundary for path-like and descriptive fields. `[paths].host_executable`,
present `[bundle]` path overrides, `[[files]].path`, `[[files]].bundle_path`, and present
`[[files]].purpose` must be non-empty trimmed strings before safe-relative checks,
bundle/file identity, host-executable identity, or duplicate bundle-path checks consume them.
`test_report_rejects_template_report_manifest_padded_file_sha256` extends that source-manifest
boundary to `[[files]].sha256`: reloaded file digests must be trimmed before SHA-256 shape,
file identity, or outer PlatformBundle fatal-stage summarization consumes them.
That trimmed-source-manifest coverage lives in
`test_pipeline_report_platform_bundle_template_manifest_trimmed_schema.py`, so the general
manifest schema test file stays below the large-file warning threshold.
`test_report_rejects_template_resolution_padded_string_field`,
`test_report_rejects_template_resolution_candidate_padded_string_field`,
`test_report_rejects_template_resolution_candidate_padded_profile_entry`, and
`test_report_rejects_template_resolution_skipped_candidate_padded_template_dir` keep
template-root selection evidence canonical before semantic checks consume it. Top-level
`template_resolution` string fields, accepted-candidate identity/path/bundle-format fields,
candidate `compatible_profiles[]` entries, and skipped-candidate `template_dir` values must be
non-empty trimmed strings before profile membership, identity, bundle-format,
selected-candidate, template-root containment, direct-child, or embedded-template binding
diagnostics run.
`test_report_rejects_template_resolution_unknown_field` closes the template-root selection report
as well: `template_resolution`, `candidates[]`, and `skipped_candidates[]` each reject unknown
fields before final Report accepts them from matching `bundle.json` and stage-report evidence.
`test_report_rejects_template_resolution_selected_candidate_mismatch` keeps successful
template-root resolution tied to the selected candidate row. When `template_resolution.fatal` is
false, the top-level `template_dir` must match exactly one `candidates[].template_dir`; otherwise a
hand-authored report could point the selected template at a directory that was never part of the
candidate list.
`test_report_rejects_template_resolution_template_dir_mismatch` and
`test_report_rejects_bundle_manifest_template_resolution_template_dir_mismatch` bind that selected
template directory to the embedded template evidence as well. For non-fatal template resolution,
final Report resolves `template_resolution.template_dir` and `template.template_dir` in both the
stage report and `bundle.json`, and rejects any mismatch before accepting the release package as
publishable.
`test_report_rejects_template_resolution_duplicate_candidate_template_dir`,
`test_report_rejects_template_resolution_duplicate_skipped_candidate_template_dir`, and
`test_report_rejects_template_resolution_candidate_also_skipped` keep that directory evidence
unique across the full resolution audit. Final Report resolves and normalizes every non-empty
`candidates[].template_dir` and `skipped_candidates[].template_dir`, rejects duplicate candidate
rows, duplicate skipped rows, and any template directory that appears both as accepted and skipped
evidence. This prevents hand-authored resolution reports from double-counting one template package
or publishing contradictory accepted/skipped status for the same directory.
`test_report_rejects_template_resolution_non_fatal_multiple_candidates` also preserves the
selection cardinality rule from template-root resolution: a non-fatal resolution report must contain
exactly one candidate. Multiple matching candidates are fatal at generation time and cannot be
retrofit into a successful final Report by choosing one top-level `template_dir`.
`test_report_rejects_template_resolution_fatal_selected_template_dir` keeps the inverse state
coherent as well: a fatal template-root resolution cannot publish a non-empty top-level
`template_dir`, because generation only fills the selected template when exactly one valid
candidate was accepted.
`test_report_rejects_template_resolution_fatal_single_candidate` keeps the candidate count coherent
for fatal reports too: a resolution with exactly one valid candidate is successful at generation
time, so final Report rejects `fatal=true` evidence that still publishes one candidate while
clearing selected `template_dir`.
`test_report_rejects_template_resolution_fatal_multiple_candidates_without_multiple_match_diagnostic`
keeps the multi-candidate fatal branch explainable: when fatal resolution publishes multiple valid
candidate rows, top-level diagnostics must include the generated
`multiple export templates matched profile=...` message instead of an unrelated generic failure.
`test_report_rejects_template_resolution_fatal_multiple_candidates_wrong_profile_diagnostic`
also binds that generated message to the current `template_resolution.profile`, so a multi-match
diagnostic copied from another profile cannot explain this profile's fatal candidate set.
`test_report_rejects_template_resolution_fatal_multiple_candidates_incomplete_candidate_diagnostic`
keeps the same diagnostic tied to the actual candidate set: the generated multi-match line must list
every object-shaped `candidates[].template_dir` value published in the fatal resolution report.
`test_report_rejects_template_resolution_fatal_multiple_candidates_with_no_match_diagnostic`
keeps that fatal branch mutually exclusive with zero-candidate failures: a report with multiple valid
candidate rows cannot also publish generated root-failure or no-match diagnostics.
`test_report_rejects_template_resolution_fatal_no_candidates_without_failure_diagnostic` keeps the
zero-candidate fatal branch on the same path: root failures must still describe `export template root
...`, and no-match failures must still describe `no export template under ... matched ...`.
`test_report_rejects_template_resolution_fatal_no_candidates_with_multiple_match_diagnostic` applies
the inverse rule: a report with no object-shaped candidate rows cannot also publish the generated
multi-match diagnostic.
`test_report_rejects_template_resolution_fatal_no_candidates_with_root_failure_and_no_match` keeps
the two zero-candidate failure families mutually exclusive too: generation returns immediately for
template-root failures, while no-match diagnostics only appear after a readable template root was
scanned.
`test_report_rejects_template_resolution_fatal_no_candidates_wrong_profile_no_match_diagnostic`
also binds the no-match line to the current `template_resolution.profile`, so a missing-template
diagnostic copied from another profile cannot explain this profile's zero-candidate fatal state.
`test_report_rejects_template_resolution_fatal_no_candidates_wrong_identity_no_match_diagnostic`
does the same for expected target platform and engine identity when those fields are known:
`target_platform=<expected_target_platform>` and `engine_version=<expected_engine_version>` must be
present in the generated no-match line.
`test_report_rejects_template_resolution_fatal_no_candidates_wrong_unresolved_identity_marker`
covers the generated marker values when those expected identities are absent: no-match diagnostics
must use `target_platform=<any>` and `engine_version=<unresolved>` instead of silently borrowing a
concrete target or engine from another report.
`test_report_rejects_template_resolution_fatal_no_candidates_wrong_root_no_match_diagnostic`
binds that no-match line to the current `template_resolution.template_root`, so an otherwise valid
zero-candidate diagnostic from another template repository cannot be reused.
`test_report_rejects_template_resolution_root_failure_with_skipped_candidates` keeps root failures
from publishing scan evidence that could not have been generated: if the root path itself failed
before discovery, `candidates[]` and `skipped_candidates[]` must both stay empty.
`test_report_rejects_template_resolution_root_failure_wrong_root_diagnostic` binds root-failure
diagnostics to the current `template_resolution.template_root`, matching the generated
`export template root <root> ...` failure text.
`test_report_rejects_template_resolution_fatal_without_diagnostics` also keeps fatal resolution
evidence explainable: when `fatal` is true, the top-level `diagnostics[]` must contain at least one
non-empty diagnostic string, matching the generation path where root errors, missing matches, and
duplicate matches all write a reason.
`test_report_rejects_template_resolution_non_fatal_with_diagnostics` keeps the successful state
clean: when `fatal` is false, top-level diagnostics must be empty, and per-template rejection
reasons stay in `skipped_candidates[].diagnostics[]`.
`test_report_rejects_template_resolution_padded_diagnostic_entry` applies the same trimmed-row
requirement to fatal `template_resolution.diagnostics[]`, and
`test_report_rejects_template_resolution_skipped_candidate_padded_diagnostic_entry` applies it to
`skipped_candidates[].diagnostics[]`. Template-root selection evidence cannot publish padded failure
reasons before final Report trusts the selected or rejected template rows.
`test_report_rejects_template_resolution_non_fatal_null_expected_identity` keeps successful
template-root resolution tied to the current export identity as well. When `fatal` is false,
`expected_engine_version` and `expected_target_platform` must be non-empty strings, so a
hand-authored report cannot null out the expected identity fields and bypass candidate
engine-version or target-platform matching.
`test_report_rejects_bundle_manifest_template_resolution_null_expected_identity` confirms the same
shared schema gate runs for `bundle.json.template_resolution`, producing the bundle-manifest label
before whole-field stage-report comparison can hide the exact missing expected identity.
`test_report_rejects_template_resolution_non_fatal_without_template_dir` keeps the successful
state selected: non-fatal template-root resolution must publish a non-empty top-level
`template_dir` before final Report checks that it matches exactly one candidate row.
`test_report_rejects_template_resolution_profile_mismatch` binds the resolution profile back to
the enclosing PlatformBundle stage report profile before candidate membership checks are trusted.
This prevents a hand-authored `template_resolution` from changing `profile` and
`candidates[].compatible_profiles` together while the surrounding release bundle still belongs to
the requested profile.
`test_report_rejects_bundle_manifest_template_resolution_profile_mismatch` applies the same
profile binding inside `bundle.json` before final Report falls back to whole-field stage-report
comparison, so bundle manifests get a direct field-level diagnostic when nested
`template_resolution.profile` drifts from their own top-level `profile`.
`test_report_rejects_template_resolution_candidate_missing_profile_membership` keeps each
candidate's non-empty `compatible_profiles[]` tied to the resolution `profile`, mirroring the
template-root matcher before a candidate can be considered selectable.
`test_report_rejects_template_resolution_candidate_identity_mismatch` applies the same matcher
identity checks to candidate rows: candidate `engine_version` must equal
`expected_engine_version`, and candidate `target_platform` must match `expected_target_platform`
after platform alias normalization.
`test_report_rejects_template_resolution_candidate_bundle_format_unknown` keeps candidate
`bundle_format` values on the same export-template allowed set as embedded template reports.
`test_report_rejects_template_resolution_candidate_host_artifact_unknown` keeps candidate
`host_artifact` values on the same placeholder/precompiled allowed set as `template.toml`.
`test_report_rejects_template_resolution_candidate_missing_required_field` and
`test_report_rejects_template_resolution_skipped_candidate_missing_required_field` keep candidate
rows complete: accepted candidates must publish template directory/id, engine, target platform,
compatible-profile list, host artifact status, and bundle format, while skipped candidates must publish both their
directory and diagnostic list. Missing fields and explicit `null` values both fail the same
required-field gate, so a report cannot satisfy candidate-row completeness with placeholders.
`test_report_rejects_template_resolution_missing_required_field` keeps the top-level
template-root selection report complete as generated evidence. Reports must carry
`template_root`, `profile`, expected engine/target platform, `fatal`, `diagnostics`, candidate and
skipped-candidate lists, and the selected `template_dir` key; fatal reports may still set
`template_dir` to null, but cannot omit the field.
`test_report_rejects_template_resolution_required_field_null` keeps generated non-null top-level
evidence from being replaced with placeholders: `template_root`, `profile`, `fatal`,
`diagnostics`, `candidates`, and `skipped_candidates` must be real values. The expected engine and
target fields may remain null when generation could not derive those identities, and fatal reports
may still publish `template_dir = null`.
`test_report_rejects_template_resolution_candidate_dir_outside_root` and
`test_report_rejects_template_resolution_skipped_candidate_dir_outside_root` keep candidate
directory evidence under the reported `template_root`. The generation path only discovers
candidate and skipped template packages from that root, so final Report now rejects a hand-authored
selection report whose candidate or skipped-candidate `template_dir` resolves outside the template
root.
`test_report_rejects_template_resolution_candidate_dir_not_direct_child_of_root` and
`test_report_rejects_template_resolution_skipped_candidate_dir_not_direct_child_of_root` tighten the
same rule to the exact template-root discovery shape: generated resolution only scans direct child
packages via `*/template.toml`, so final Report rejects nested candidate or skipped-candidate
directories even when they still resolve inside the template root.
`test_report_rejects_template_resolution_skipped_candidate_without_diagnostics` keeps skipped rows
actionable: every skipped candidate must include at least one non-empty diagnostic string explaining
why that template package was not selectable.
`test_report_rejects_template_resolution_string_fields_non_string`,
`test_report_rejects_template_resolution_bool_fields_non_bool`,
`test_report_rejects_template_resolution_string_array_fields_non_string_array`,
`test_report_rejects_template_resolution_candidate_entries_non_object`,
`test_report_rejects_template_resolution_candidate_string_fields_non_string`,
`test_report_rejects_template_resolution_candidate_string_arrays_non_string_array`, and
`test_report_rejects_template_resolution_skipped_candidate_fields_non_string` close the typed
shape of that selection report. Top-level resolution fields must be strings, booleans, or
string arrays as declared; `candidates[]` and `skipped_candidates[]` must contain objects; and
candidate/skipped-candidate rows must keep their string and string-array fields typed before
final Report trusts template-root evidence from a PlatformBundle report or `bundle.json`.
`test_pipeline_report_platform_bundle_template_resolution_string_array_schema.py` keeps malformed
string-array entries pinned to index-level diagnostics while preserving the broader field-level
diagnostic for non-array fields.
`test_report_rejects_template_resolution_candidate_blank_profile_entry` and
`test_report_rejects_template_resolution_candidate_duplicate_profile_entry` keep
`candidates[].compatible_profiles` free of empty, whitespace-only, or duplicate entries, matching
the input-side `template.toml` profile-list gate.
`test_report_rejects_template_resolution_string_field_blank`,
`test_report_rejects_template_resolution_candidate_string_field_blank`, and
`test_report_rejects_template_resolution_skipped_candidate_string_field_blank` apply the same
non-empty string rule to top-level resolution identities, candidate rows, and skipped-candidate
`template_dir` evidence before report consumers trust template-root selection output.
`test_source_template_stage_rejects_generated_file_read_error` keeps SourceTemplate stage
materialization on the diagnostic path when a generated file passes existence/type checks but fails
during stage size/hash calculation; the stage reports `SourceTemplate generated file ... could not
be read` and clears the owned generated project instead of surfacing the filesystem exception.
`test_source_template_stage_rejects_generated_file_write_error` applies the same failure policy while
materializing Validate-planned generated files: parent-directory creation or file writes that fail
become fatal SourceTemplate diagnostics, the stage report is still written, and the owned generated
project is cleared.
`test_source_template_stage_rejects_stale_project_cleanup_error` and
`test_source_template_stage_reports_final_project_cleanup_error` keep generated-project cleanup
itself on that reportable path: stale `project/` deletion failures and fatal cleanup failures are
recorded in `cleanup_reason`/`project_cleaned` without losing the original materialization
diagnostic.
`test_source_template_stage_rejects_project_root_create_error` covers the generated project root
creation boundary: if `project/` cannot be created before writing planned files, SourceTemplate
records `SourceTemplate generated project ... could not be created`, clears generated file evidence,
and still emits the stage report.
`test_source_template_stage_rejects_generated_file_path_resolve_error` covers generated project
child-path canonicalization. `resolve_project_child(...)` now records either a project-root resolve
diagnostic or a `{kind} ... could not be resolved` diagnostic, marks the stage fatal through the
existing materialization gate, and clears the generated project instead of surfacing `OSError`.
`test_source_template_stage_rejects_dependency_path_resolve_error` covers the generated
`Cargo.toml` dependency rewrite step. If a `zircon_* = { path = "..." }` dependency cannot be
canonicalized against the selected repo root, SourceTemplate records
`SourceTemplate dependency ... could not be resolved`, leaves the original manifest text in place for
diagnostic context, and clears the generated project through the same fatal path.
`test_source_template_rejects_repo_root_resolve_error`,
`test_source_template_rejects_validate_report_resolve_error`, and
`test_source_template_rejects_target_dir_resolve_error` cover the stage entry and command planning
boundary: explicit `repo_root`, `validate_report`, and `target_dir` values are resolved through
SourceTemplate diagnostics before report loading or command preview, and dry-run failures print
field-specific diagnostics plus `command=<skipped>`.
`test_cook_assets_stage_rejects_cooked_manifest_write_error` applies the write gate to CookAssets:
if `<out>/stages/cook_assets/assets.json` cannot be written after manifest validation, the stage
returns fatal diagnostics while still emitting `report.json`.
`test_cook_assets_rejects_asset_source_path_resolve_error` and
`test_cook_assets_rejects_project_default_scene_source_resolve_error` cover CookAssets source-path
canonicalization for explicit asset manifests and the project `default_scene` fallback. Both paths
now record `asset source for ... could not be resolved`, return fatal CookAssets JSON, and skip
`assets.json` output instead of surfacing `OSError`.
`test_cook_assets_rejects_non_string_manifest_string_array_entry_before_array_shape` keeps source
asset manifest array field shape distinct from entry type. Non-array `roots`, `dependencies`, or
`labels` fields still report the whole-field string-array diagnostic, but non-string entries inside
those arrays now report the indexed `entry <index> must be a string` diagnostic before reference
closure, path normalization, or label filtering can consume the malformed row.
`test_cook_assets_stage_reports_report_write_error_to_stdout` and
`test_report_stage_records_pipeline_report_write_error_in_stage_report` cover the shared
`report_io.py` boundary: Python stages no longer surface `OSError` when `report.json` or the final
pipeline report cannot be written, and the final Report stage preserves a readable stage report when
only the top-level `<out>/report.json` write fails.
`test_report_stage_removes_stale_stage_report_when_rewrite_fails` covers the follow-up failure path:
if `<out>/stages/report/report.json` is written first, `<out>/report.json` then fails, and the stage
report rewrite also fails, the stale pre-diagnostic stage report is removed and the stdout payload
contains both write diagnostics.
`test_report_stage_writes_pipeline_report_when_stage_dir_create_fails` covers the final Report
stage directory boundary: if `<out>/stages/report/` cannot be created, final Report records a typed
directory diagnostic in the top-level `<out>/report.json` and prints the same fatal JSON to stdout.
`test_pack_preflight_reports_stage_directory_create_error_to_stdout` applies the same stage
directory policy to Pack preflight: when `<out>/stages/pack/` cannot be created, Pack prints its
normal fatal preflight JSON with a typed `Pack stage directory ... could not be created` diagnostic
instead of surfacing `OSError`.
`test_compile_host_reports_stage_directory_create_error_to_stdout` covers the same rule for
CompileHost: a failed `<out>/stages/compile_host/` create emits a fatal CompileHost JSON payload to
stdout, preserving Validate report diagnostics and the planned command/host fields where available.
`test_cook_assets_reports_stage_directory_create_error_to_stdout` applies the same boundary to
CookAssets: a failed `<out>/stages/cook_assets/` create emits fatal CookAssets JSON to stdout with
the source/project/cooked-manifest fields still populated for caller diagnostics.
`test_source_template_reports_stage_directory_create_error_to_stdout` covers the outer
SourceTemplate stage directory: if `<out>/stages/source_template/` cannot be created, the stage
prints fatal SourceTemplate JSON with validate/project/command/build fields instead of surfacing
`OSError`.
`test_native_dynamic_reports_stage_directory_create_error_to_stdout` covers the outer
NativeDynamic stage directory with the same rule: failed `<out>/stages/native_dynamic/` creation
prints fatal NativeDynamic JSON with Validate payload, package selection, build/signing/notarization
audit shells, and a typed directory diagnostic.
`test_platform_bundle_reports_stage_directory_create_error_to_stdout` covers PlatformBundle's
outer stage directory as well: failed `<out>/stages/platform_bundle/` creation prints fatal
PlatformBundle JSON with input diagnostics and a typed directory diagnostic instead of surfacing
`OSError`.
`test_validate_reports_stage_directory_create_error_to_stdout` closes the Python Validate wrapper
side of the same boundary: if `<out>/stages/validate/` cannot be created before launching the Rust
validator binary, the wrapper prints a fatal Validate JSON payload with the planned command and
directory diagnostic instead of raising.
`test_validate_rejects_repo_root_resolve_error` and `test_validate_rejects_project_resolve_error`
cover the Validate entry boundary: explicit engine roots and project manifests are canonicalized
after the Validate report path is known, and failures now produce `command=<skipped>` plus
field-specific diagnostics instead of surfacing `OSError`.
`test_validate_rejects_validator_resolve_error` and `test_validate_rejects_target_dir_resolve_error`
cover the later command-construction boundary: explicit validator binaries and Cargo `target_dir`
overrides are canonicalized before `validate_command(...)`, and failures follow the same skipped
command diagnostics path.
`test_validate_reports_validator_launch_error` and `test_pack_reports_packer_launch_error` cover
external binary launch failures. If an explicit validator or packer executable cannot start, the
Python wrapper writes the corresponding stage report with a typed `... command ... could not start`
diagnostic instead of surfacing `FileNotFoundError`/`OSError`.
`test_validate_reports_successful_validator_without_stage_report` covers the adjacent success-shaped
failure: a launched validator that exits with code `0` but omits
`<out>/stages/validate/report.json` now produces a fatal Validate report and returns exit code `2`.
`test_compile_host_reports_cargo_launch_error` and `test_source_template_reports_cargo_launch_error`
close the same boundary for Python-launched Cargo commands. CompileHost writes a fatal stage report
with `CompileHost cargo command ... could not start`, exit code `2`, and empty `stdout_lines` /
`stderr_lines`; launched CompileHost builds record the captured Cargo streams in those arrays.
SourceTemplate records the failed build validation attempt with
`SourceTemplate cargo build command ... could not start` and keeps `exit_code` unset because the
process never launched. The same SourceTemplate report keeps `stdout_lines` and `stderr_lines`
empty in this no-process case, while successful or failed launched builds record the captured Cargo
streams for final Report and editor-side audit views.
`test_report_rejects_source_template_report_command_missing_target_dir` and
`test_report_rejects_source_template_build_validation_command_missing_target_dir` keep SourceTemplate
final Report build evidence bound to the generated stage target directory. Non-fatal SourceTemplate
reports now require both the top-level `command` and nested `build_validation.command` to carry
`--target-dir`; report-command diagnostics accumulate with manifest-path diagnostics instead of
short-circuiting, so hand-authored reports cannot omit the isolated target while hiding manifest
drift.
`test_report_command_reports_manifest_and_target_option_errors` keeps that aggregation behavior
explicit when `--manifest-path` and `--target-dir` are both malformed in the same top-level
SourceTemplate report command.
`test_report_rejects_source_template_generated_file_read_error` keeps SourceTemplate final Report
aggregation on the same diagnostic path when a generated file passes existence/type checks but
fails during content hashing; final Report reports `SourceTemplate generated file ... could not be read`
instead of surfacing the filesystem exception.
`test_report_rejects_platform_host_output_read_error`,
`test_report_rejects_platform_host_source_read_error`, and
`test_report_rejects_template_file_read_error` apply the same rule to PlatformBundle release
evidence hashing. The host/pack/delta output/source comparisons and template file sha256 checks now
emit field-specific `PlatformBundle report ... could not be read` diagnostics instead of surfacing
filesystem exceptions.
`test_platform_bundle_rejects_host_copy_error` keeps PlatformBundle bundle materialization on the
same diagnostic path when release files cannot be copied. Template files, host executables, pack
files, and delta pack files now share `copy_platform_bundle_file(...)`, so copy or destination
parent-directory failures make the PlatformBundle report fatal and clear the partial bundle.
`test_platform_bundle_rejects_template_copy_source_resolve_error` covers the copy preflight
canonicalization used to skip a template host placeholder that will be overwritten by the real host;
template source or host path `Path.resolve()` failures now emit `could not be resolved during bundle
copy` diagnostics and clear the partial bundle instead of aborting report emission.
`test_platform_bundle_rejects_bundle_output_path_resolve_error` extends the same rule to
template-declared bundle destinations. When `[bundle].host_path`, `[bundle].pack_path`,
`[bundle].delta_pack_path`, `[bundle].manifest_path`, or `[[files]].bundle_path` cannot be
canonicalized under the bundle root, PlatformBundle records a `bundle path ... could not be
resolved` diagnostic, marks the stage fatal, and skips release file copies.
`test_platform_bundle_rejects_bundle_manifest_write_error` covers the final bundle manifest seal:
if `bundle.json` cannot be created or written, PlatformBundle records a fatal diagnostic, deletes
the partial profile bundle, and still writes the stage report.
`test_template_rejects_declared_file_read_error` applies the same policy at template validation
time: a `template.toml [[files]]` entry that resolves to a regular file but fails during SHA-256
calculation makes the template report fatal with `template file ... could not be read` instead of
aborting the PlatformBundle/template-root flow.
`test_template_rejects_declared_file_path_resolve_error` covers the preceding child-path
canonicalization step: if a declared template file path cannot be resolved, template validation now
records `template path ... could not be resolved` and returns a fatal template report instead of
raising out of PlatformBundle.
Template directory canonicalization follows the same contract.
`test_validate_template_rejects_template_directory_resolve_error` records an
`export template directory ... could not be resolved` fatal template report when a direct
`template_dir` cannot be resolved, and
`test_template_resolution_rejects_candidate_directory_resolve_error` keeps template-root discovery
diagnostic by adding the failed candidate to `skipped_candidates[]` instead of raising while
resolving the candidate directory.
NativeDynamic payload manifests now follow the same file-read contract. Stage package-report
generation, stage-level payload hashing, explicit directory payload summaries, and final Report
bundle payload revalidation all route native artifact/resource byte reads through diagnostics-aware
manifest helpers. `test_native_dynamic_stage_rejects_package_payload_read_error` keeps stage
materialization fatal and cleanup-driven when a package file cannot be hashed, while
`test_report_rejects_native_plugins_payload_file_read_error` keeps final Report from projecting a
NativeDynamic payload when the bundled plugin bytes cannot be re-read. Directory traversal for those
payload manifests is diagnostic as well: `test_native_dynamic_stage_rejects_package_payload_listing_error`
keeps `rglob()` failures fatal, cleanup-driven, and reportable as `NativeDynamic payload directory ...
could not be listed`.
Explicit directory payload snapshots also treat their source path as release evidence:
`test_platform_bundle_rejects_native_plugins_payload_source_resolve_error` keeps a selected
`--native-plugins-dir` from being copied when the payload summary cannot canonicalize that source
directory, recording `NativeDynamic payload source ... could not be resolved` instead of publishing
`native_plugins_payload = null` as a successful bundle.
The lower-level payload manifest helpers use the same diagnostics-aware path resolver for stage
roots, bundle `plugins/` roots, package payload directories, and loadable-artifact scans.
`test_native_dynamic_payload_bundle_manifest_rejects_source_resolve_error` keeps bundle payload
manifest recomputation from surfacing a raw `OSError` when the `plugins/` root cannot be resolved.
`test_native_dynamic_stage_rejects_package_artifact_copy_error` applies the same fatal cleanup
policy during NativeDynamic package materialization. Native artifact/debug artifact files, resource
directories, and package `plugin.toml` are copied through diagnostics-aware helpers; copy failures
remove the partial package and then clear the whole staged `plugins/` payload.
NativeDynamic cleanup itself is also reportable: `test_native_dynamic_stage_rejects_stale_plugins_cleanup_error`,
`test_native_dynamic_stage_reports_partial_package_cleanup_error`, and
`test_native_dynamic_stage_reports_final_payload_cleanup_error` cover stale payload removal, partial
package removal, and final payload cleanup failures without losing the stage report.
`test_native_dynamic_build_rejects_staged_cdylib_copy_error` applies that policy to explicit
`--native-dynamic-build` execution. After Cargo succeeds, the built loadable artifact and adjacent
debug sidecars are copied into the materialized package through diagnostics-aware staging code; a
copy failure marks `native_build_execution` fatal and clears the staged payload instead of surfacing
the filesystem exception.
`test_platform_bundle_rejects_native_plugins_copy_error` protects the PlatformBundle handoff side
of the same payload. When an explicit or inherited NativeDynamic `plugins/` directory is copied into
the final profile bundle, directory creation/listing, stale destination removal, and recursive file
copies all report typed PlatformBundle diagnostics and clear the partial profile bundle on failure.
`test_platform_bundle_rejects_stale_bundle_cleanup_error` and
`test_platform_bundle_reports_failed_bundle_cleanup_after_copy_error` keep cleanup failures reportable:
failed deletion of stale or partial profile bundles is recorded as a PlatformBundle fatal diagnostic
without hiding the copy/write diagnostic that caused cleanup to run.
`test_platform_bundle_rejects_bundle_root_create_error` applies the same reportable failure policy
to the materialization root itself: if the profile bundle root cannot be created or re-confirmed
inside `materialize_platform_bundle(...)`, the stage writes a typed fatal diagnostic instead of
surfacing `OSError`.
`test_native_dynamic_stage_rejects_package_report_write_error` keeps the package payload report
seal on that same path: if `native_dynamic_package.toml` cannot be written after payload hashing,
the stage records a typed diagnostic and clears the staged payload instead of publishing a package
without its package report.
`test_native_dynamic_stage_rejects_loader_manifest_write_error` protects the stage-level dynamic
plugin entrypoint. If `plugins/native_plugins.toml` cannot be created or written, the stage records
`NativeDynamic loader manifest ... could not be written`, skips final payload hashing, and clears
the staged plugin payload.
NativeDynamic signing and notarization operation audits use the same diagnostic policy for loadable
artifact hashes. `test_native_dynamic_signing_rejects_before_hash_read_error` covers a read failure
before the external command starts, and `test_native_dynamic_signing_rejects_after_hash_read_error`
covers a read failure after a successful signer mutation; both paths keep the artifact audit
structured, mark the operation fatal, and clean the staged payload.
The matching source-origin fields are release evidence too: `host_source_origin`,
`pack_source_origin`, and `delta_pack_source_origin` must be explicit whenever their
corresponding output path is present. The final Report stage no longer infers those origins for
old or hand-authored PlatformBundle reports.
For NativeDynamic payloads, `native_plugins_payload.stage_report` is stage handoff evidence when
the current final Report has loaded a non-fatal NativeDynamic stage. In that case the payload must
point back to the current `<out>/stages/native_dynamic/report.json`, and `source` must point to the
same report directory's `plugins/` payload. Historical Validate reports that omit
`profile_summary.strategies` may still be inspected when the PlatformBundle payload itself carries
a real current-output `stage_report`, but only after the non-fatal Validate report has carried the
required `profile_summary` object; final Report rejects external report paths, then loads the
declared current report before comparing signing and notarization audit summaries. Pure manual
`--native-plugins-dir` directory snapshots keep `stage_report = null` and remain distinct from
stage-backed payloads.
That legacy inspection path still treats the declared NativeDynamic report as stage evidence: the
loaded report must identify the `NativeDynamic` stage, be non-fatal, carry string-array
`diagnostics`, and match the PlatformBundle report profile before its signing/notarization summaries
are trusted.
Successful non-fatal NativeDynamic stage reports are now a required release-evidence shape:
`plugins_dir`, `loader_manifest`, `content_hash`, `file_manifest`, `materialized_packages`,
`stage_output`, `validate_report`, `target_platform`, `artifact_extensions`,
`native_plugin_root`, `native_dynamic_packages`, `package_exports`, `package_count`, `native_build_plan`,
`native_build_execution`, `native_signing`, and `native_notarization` must all be present before
final Report trusts the stage as publishable. Missing any of those fields marks `NativeDynamic` fatal
during stage schema loading, before PlatformBundle projects a `native_plugins_payload`.
`test_report_stage_rejects_native_dynamic_blank_required_string_release_evidence_field` also keeps
those required string fields trimmed and non-empty, so whitespace-only `content_hash`, path/location
fields, `target_platform`, or `validate_report` fail as field-level evidence instead of being
resolved as filesystem paths, compared as hashes, or silently accepted.
`stage_output` is also bound to the loaded report's current stage directory, `validate_report`
records the Validate report that drove stage materialization, `target_platform` and
`artifact_extensions[]` record the platform strategy and loadable-artifact suffix set used for staged
artifacts and operation audits, `native_plugin_root` records the source package root used during
materialization, and the `native_dynamic_packages[]` table is the stage's own selected-package
evidence and is later reconciled with Validate handoff and materialized package ids; `package_count`
is likewise required and then compared with the current `materialized_packages[]` length.
`artifact_extensions[]` and `native_dynamic_packages[]` may be empty only when the surrounding stage
evidence makes that meaningful, but neither array may contain blank entries. The
build/sign/notarization audit objects may still report `enabled = false` with
`package_count = 0`; the release gate requires the audit boundary itself to be present, while
package-id closure is enforced only for enabled/non-empty audit package tables.
The NativeDynamic build audit schema is split by responsibility: the stage schema keeps only the
Report-stage dispatch, `pipeline_report_native_dynamic_build_audit_schema.py` re-exports the build
audit surface, `pipeline_report_native_dynamic_build_audit_common.py` owns shared count,
diagnostics, uniqueness, and primitive-table helpers, and the build-plan/build-execution modules own
their package-row contracts. The legacy build-schema test module remains a compatibility entry point;
the actual coverage is split into build-plan and build-execution test modules with shared report
fixtures in `native_dynamic_stage_schema_test_support.py`.
Inside the required `native_build_plan` object, successful reports must include the plan header
fields `workspace_manifest`, `target_dir`, `cargo_profile`, `release`, `build_features`,
`package_count`, `diagnostics`, `packages`, and `fatal`; a half-written plan object is fatal even
when its package rows are otherwise well typed.
`cargo_profile` is limited to `debug` or `release`, and `release` must agree with that profile
before final Report treats the build-plan header as Cargo mode evidence.
The same plan header is bound back to every package row: `workspace_manifest`, `target_dir`,
`cargo_profile`, and `release` must match the row fields with the same names, and
`build_features[]` must match the row `features[]`.
Each package row's `expected_loadable_artifact` must also resolve to the dynamic library path
derived from `target_dir`, `cargo_profile`, `crate_name`, and the enclosing `target_platform`,
with slash direction normalized before comparison.
`test_report_stage_rejects_native_dynamic_build_plan_blank_required_string_release_evidence_field`
keeps the three string header fields trimmed and non-empty before Report trusts the recorded Cargo
workspace, target directory, or profile evidence. Each `packages[]` row must also carry its full
command evidence row, including package id, crate, manifests, target dir, profile, expected loadable
artifact, release flag, feature list, and Cargo command array.
`test_report_stage_rejects_native_dynamic_build_plan_package_blank_required_string_field` applies
the same trimmed non-empty rule to package-row identity, path, profile, and expected-artifact strings
before package-id closure or command evidence is trusted. The string entries in that row must be
trimmed and non-empty; `build_features[]` and package-row `features[]` reject non-string entries at
the indexed field diagnostic before header, uniqueness, or command semantics are derived. Feature
arrays may be empty but schema-clean entries must be trimmed, non-empty, and unique, diagnostics rows cannot be blank, and the command array must be non-empty
with no blank entries. `test_report_stage_rejects_native_dynamic_build_plan_padded_build_feature_entry`
and `test_report_stage_rejects_native_dynamic_build_plan_package_padded_feature_entry` keep the
top-level `build_features[]` and package-row `features[]` evidence from being accepted only because
plan, package, and command values carry matching whitespace.
Inside the required `native_build_execution` object, successful reports must include `enabled`,
`fatal`, `skipped`, `diagnostics`, `package_count`, and `packages`; disabled execution is
represented by an explicit empty audit table, not by omitting the audit header or the skipped-state
flag. The NativeDynamic generator writes `skipped=false` for the default non-executing audit, for
fatal build-plan diagnostics, and for completed build execution reports; it only flips the flag to
`true` for an explicitly skipped build-execution branch such as prior materialization diagnostics.
`test_report_stage_rejects_native_dynamic_build_execution_missing_release_evidence_field` keeps all
six header fields required, while `test_native_dynamic_stage_reports_native_cdylib_build_plan` and
`test_native_dynamic_build_executes_plan_and_stages_cdylib` cover the generated non-executing and
executing success reports. The optional `skip_reason` is tied to the same explicit state:
`test_report_stage_rejects_native_dynamic_build_execution_skip_reason_non_skipped_mismatch` rejects
a reason when `skipped=false`, and
`test_report_stage_rejects_native_dynamic_build_execution_skipped_without_reason` requires a
trimmed reason when `skipped=true`. If `enabled=false`, `package_count` must be `0` and `packages`
must be empty before final Report accepts the skipped Cargo-build audit boundary. Non-empty
execution package rows must also carry package id, crate, command, exit code, stdout/stderr capture,
expected loadable artifact,
copied loadable artifact, and copied sidecar evidence. The identity and artifact path strings in
those rows must be trimmed and non-empty, while stdout/stderr capture strings may be empty.
`exit_code` is always a non-negative integer; successful execution evidence additionally requires
`exit_code=0`, while fatal execution may report a positive Cargo process failure code.
`test_report_stage_rejects_native_dynamic_build_execution_fatal_success_report_mismatch` keeps
successful stage reports from carrying `native_build_execution.fatal=true`; failed Cargo build
evidence must make the enclosing NativeDynamic report fatal too.
`test_report_stage_rejects_native_dynamic_build_execution_fatal_without_diagnostics` applies the
same explainability rule to build execution: a fatal execution audit must include at least one
non-empty diagnostic instead of relying only on the outer stage fatal status.
`test_report_stage_rejects_native_dynamic_build_execution_skipped_success_report_mismatch` applies
the same rule to `skipped=true`, so skipped Cargo-build evidence cannot be published inside a
successful NativeDynamic report.
`test_report_stage_rejects_native_dynamic_build_execution_package_blank_required_string_field` keeps
that execution-row rule on package identity, crate, expected artifact, and copied artifact evidence
before package-id closure or copy evidence can be trusted. The command array must be non-empty with
no blank entries, and copied sidecar arrays may be empty but reject non-string entries at
`copied_sidecars[index] must be a string` before path checks or duplicate detection. Schema-clean
sidecar entries cannot contain blank or duplicate entries, and any present sidecar entry must already be a non-empty trimmed string. Copied loadable-artifact
and copied-sidecar paths are also schema-checked as safe relative bundle paths before the later
materialized-artifact/file-manifest closure runs, so absolute paths,
`.`/`..` segments, or Windows drive prefixes cannot be published as build execution copy evidence.
After the path is safe-relative, it must also stay inside the package's own
`plugins/<package_id>/` directory; cross-package copied loadables or sidecars are rejected at the
build-execution schema boundary before the later materialized-artifact comparison reports a broader
payload mismatch.
Execution diagnostics rows cannot be blank. If `skip_reason` is present, it must also be trimmed and
non-empty; `test_report_stage_rejects_native_dynamic_build_execution_blank_skip_reason` prevents a
hand-authored report from publishing a whitespace-only execution skip reason.
Inside the required `native_signing` and `native_notarization` objects, successful reports must
also include `diagnostics` and `packages`; disabled operation audits may use an empty package table,
but the stage evidence boundary itself must stay explicit. `allowed_platforms` may be empty, but its
entries cannot be blank or padded. Non-string `allowed_platforms[]` entries are also field-level
evidence now: NativeDynamic stage reports and PlatformBundle `native_plugins_payload` both report
`<audit>.allowed_platforms[index] must be a string` before duplicate checks or `platform_allowed`
semantics run. Operation diagnostic rows cannot be blank. Optional `profile` is accepted only when
it is trimmed and non-empty, while `target_platform` remains required. Artifact command arrays must
be non-empty and cannot contain blank entries; non-string command entries report
`artifacts[index].command[index] must be a string` before the command is treated as a generic
string-array failure.
When a NativeDynamic report carries `stage_output`, it must resolve to the same
`<out>/stages/native_dynamic` directory as the loaded `report.json` parent; external or stale stage
directories mark `NativeDynamic` fatal before any bundle payload is projected.
The same loaded-report path is also the authority for NativeDynamic `plugins_dir`: a successful
stage report must point at that current directory's `plugins/` payload, not at another self-consistent
external plugin tree. `test_report_stage_rejects_native_dynamic_plugins_dir_outside_current_stage`
keeps final Report from publishing NativeDynamic package/file/hash evidence that was generated for a
different stage directory.
`loader_manifest` is bound to the same current payload root and must resolve to
`<out>/stages/native_dynamic/plugins/native_plugins.toml`. `test_report_stage_rejects_native_dynamic_loader_manifest_outside_current_stage`
prevents a report from pairing current staged plugin bytes with a stale or external dynamic-plugin
load manifest.
Final Report also parses the current loader manifest before trusting a non-fatal NativeDynamic
stage. `test_report_stage_rejects_native_dynamic_loader_manifest_missing_plugins_table` and
`test_report_stage_rejects_native_dynamic_loader_manifest_bad_plugin_id` require `plugins` to be an
array of plugin tables with non-empty string `id` fields, while
`test_report_stage_rejects_native_dynamic_loader_manifest_bad_abi_table` requires any optional
loader-row `abi` value to be a TOML table, and
`test_report_stage_rejects_native_dynamic_loader_manifest_unknown_abi_field` keeps that table
closed to the accepted ABI v3 contract; `test_report_stage_rejects_native_dynamic_loader_manifest_unknown_plugin_field`
also rejects sidecar keys on the loader row itself before package id comparison; and
`test_report_stage_rejects_native_dynamic_loader_manifest_string_field_type` rejects non-string
`path`, `manifest`, or `package_report` values before row-field drift checks; and
`test_report_stage_rejects_native_dynamic_loader_manifest_abi_field_types` applies integer/string
type diagnostics to loader-row ABI fields before ABI contract mismatch checks. The shared loader
manifest ABI parser also rejects whitespace-only ABI v3 string contracts before those mismatch
checks; and
`test_report_stage_rejects_native_dynamic_loader_manifest_abi_missing_required_field` requires the
same table to carry every ABI v3 contract field once it is present; and
`test_report_stage_rejects_native_dynamic_loader_manifest_unknown_top_level_field` rejects
document-level sidecar tables such as `[metadata]` before package evidence is trusted; and
`test_report_stage_rejects_native_dynamic_loader_manifest_missing_row_field` plus
`test_report_stage_rejects_native_dynamic_loader_manifest_missing_abi_table` require current
NativeDynamic stage loader rows to carry the generated `path`, `manifest`, `package_report`, and
ABI v3 table whenever the stage report publishes `package_exports[]`;
`test_report_stage_rejects_native_dynamic_loader_manifest_package_mismatch` then requires the
resulting `[[plugins]].id` sequence to match the package ids in `materialized_packages[]`, and
`test_report_stage_rejects_native_dynamic_malformed_loader_manifest` marks the NativeDynamic stage
fatal when `native_plugins.toml` is no longer valid TOML. This keeps a hash-updated but semantically
stale load manifest from telling runtime loaders to open a different package set than the payload
summary publishes.
Loader manifest row strings are schema evidence before they are row-match evidence:
`[[plugins]].id`, `path`, `manifest`, and `package_report` must be non-empty trimmed strings in both
NativeDynamic stage reports and PlatformBundle payload checks. Padded row values stop at
`plugins[0].<field> must be a non-empty trimmed string` before plugin-id sequence or row-field
comparisons run.
Final Report also compares the current TOML row against the accepted package export row. The
preferred source is the NativeDynamic stage report's required `package_exports[]`; the Validate
`plan_summary.native_dynamic_package_exports[]` table remains the plan handoff used for cross-stage
drift diagnostics. For NativeDynamic stage evidence this is a required generated load-entry shape;
stage-backed PlatformBundle final payloads use the same required shape once
`native_plugins_payload.stage_report` matches the current NativeDynamic report. Non-stage-backed
PlatformBundle final payloads keep the historical compatibility rule where final bundle loader rows
are compared when those fields are explicitly present. The comparison covers `path`,
`manifest`, `package_report`, `abi.abi_version`, and each ABI v3 string contract field that appears
in both places. `test_report_stage_rejects_native_dynamic_loader_manifest_path_mismatch` and
`test_report_stage_rejects_native_dynamic_loader_manifest_abi_mismatch` keep a hash-updated
`native_plugins.toml` from redirecting runtime load paths or ABI descriptor contracts while the
stage report and Validate handoff still publish the expected package export table. The ABI
comparison only runs after the shared parser has proven the optional `abi` value is a table, and
extra ABI keys are reported as unsupported instead of being ignored.
NativeDynamic stage final Report also opens each current materialized package's
`native_dynamic_package.toml` before the payload is considered releasable. Each
`materialized_packages[]` row must explicitly declare the source package directory as non-empty
string `source` and the generated package report as non-empty string `package_report`; the stage gate
then requires the declared source path to resolve to an existing directory under `native_plugin_root`, requires that source
directory to contain parseable `plugin.toml` with a non-empty string `id` matching the current
materialized package id, requires the declared report path to be the package-local generated file,
and reuses the shared package-report content diagnostics for `format_version`, `package_id`,
`directory`, `path`, `manifest`, `[abi]`, and `[payload]` so stage-local evidence is rejected before
PlatformBundle copies it.
`test_report_stage_rejects_native_dynamic_missing_materialized_package_source` keeps a hash-current
stage report from dropping original plugin-package provenance while retaining staged bytes and
package-report evidence. `test_report_stage_rejects_native_dynamic_package_source_outside_plugin_root`
then prevents a report from claiming a source directory that exists but is not owned by the
reported `native_plugin_root`. `test_report_stage_rejects_native_dynamic_package_source_missing_manifest`
aligns final Report with the production source discovery rule by rejecting source directories that
no longer contain `plugin.toml`. `test_report_stage_rejects_native_dynamic_package_source_manifest_id_mismatch`,
`test_report_stage_rejects_native_dynamic_package_source_manifest_parse_error`, and
`test_report_stage_rejects_native_dynamic_package_source_manifest_missing_id` extend the same source
provenance gate into TOML parsing and package identity, so a hash-current stage report cannot claim
that `materialized_packages[0].package_id = "animation"` was sourced from a manifest whose `id`
belongs to another package or is not valid release evidence. Source manifest `id` values are also
trimmed release evidence: a padded but otherwise matching `id` stops at
`source manifest id must be a non-empty trimmed string` before package-id equality is derived.
Non-string source manifest ids stop at `source manifest id must be a string`, keeping malformed TOML
identity evidence separate from missing-id evidence.
The production NativeDynamic stage applies the same rule while discovering direct source packages:
`test_native_dynamic_stage_rejects_padded_source_manifest_id_before_package_match` keeps a padded
direct `plugin.toml` id from degrading into a selected-package mismatch diagnostic.
`test_native_dynamic_stage_rejects_non_string_source_manifest_id_before_missing_id` and
`test_report_stage_rejects_native_dynamic_package_source_manifest_non_string_id_before_missing_id`
cover the execution and final Report type gates for `id = 42`.
`test_native_dynamic_stage_rejects_padded_recursive_source_manifest_id_before_missing_manifest`
extends that contract to recursive source discovery, so nested schema-invalid manifests are not
masked by the generic missing-manifest fallback.
`test_native_dynamic_stage_rejects_recursive_source_manifest_parse_error_before_missing_manifest`
keeps the same direct diagnostic for nested parse errors.
The focused NativeDynamic stage report tests are split by responsibility: loader manifest row/schema
coverage stays in `test_pipeline_report_native_dynamic_stage_payload.py`, while source,
package-report, loadable-artifact, and package-export materialized package coverage lives in
`test_pipeline_report_native_dynamic_stage_materialized_packages.py`. Shared stage report and TOML
fixture helpers live in `native_dynamic_stage_report_test_support.py`, keeping future evidence gates
from growing the loader-focused module again.
`test_report_stage_rejects_native_dynamic_missing_materialized_package_report` keeps a hash-current
stage report from silently falling back to an implicit default package report path, while
`test_report_stage_rejects_native_dynamic_empty_materialized_package_source` and
`test_report_stage_rejects_native_dynamic_empty_materialized_package_report` keep empty string fields
from satisfying the current generated-stage shape or resolving the package report to the process
working directory.
`test_report_stage_rejects_native_dynamic_package_report_id_mismatch` and
`test_report_stage_rejects_native_dynamic_package_report_payload_hash_mismatch` cover reports whose
outer `file_manifest` and `content_hash` were recomputed after the package TOML drifted, preventing a
hash-updated NativeDynamic stage from publishing a forged package identity or stale package-local
payload digest.
`pipeline_report_native_dynamic_stage_payload.py` then recomputes the current stage `plugins/`
file manifest and content hash before final Report trusts the NativeDynamic stage wrapper.
That recomputation is gated by the shared `pipeline_report_native_dynamic_report_hash_schema.py`
helper, so a non-SHA-256 `content_hash` is schema evidence rather than payload drift evidence.
It also reuses `native_dynamic_file_manifest_schema_diagnostics(...)` before comparing the reported
`file_manifest[]` with the current `plugins/` directory or deriving loadable-artifact presence.
`test_report_stage_rejects_native_dynamic_schema_before_payload_semantics` covers a padded
`file_manifest[0].path`, which must stop at the field-level trimmed-string diagnostic instead of
falling through to current-directory mismatch noise.
`test_report_rejects_native_plugins_payload_materialized_package_padded_duplicate_id_before_uniqueness`
and
`test_report_rejects_native_plugins_payload_materialized_package_padded_duplicate_loadable_artifact_before_uniqueness`
apply the same schema-clean order to PlatformBundle payload uniqueness evidence: padded duplicate
package ids and loadable-artifact rows are rejected as string-shape evidence, not as trustworthy
duplicates.
`test_report_stage_rejects_native_dynamic_stale_file_manifest` keeps a stale report
`file_manifest`/`content_hash` from surviving after the staged plugin bytes have changed.
The same stage-payload gate reuses the materialized-package membership check:
`test_report_stage_rejects_native_dynamic_package_destination_outside_plugins_dir` locks the final
Report behavior that rejects a NativeDynamic stage report whose `materialized_packages[].destination`
points outside the current staged `plugins/` directory.
`test_report_stage_rejects_native_dynamic_package_count_mismatch` also requires
`package_count` to equal the number of `materialized_packages[]` rows, so the stage report cannot
inflate or under-report the package audit count while keeping the payload list itself valid.
`test_report_stage_rejects_native_dynamic_package_selection_mismatch` requires
`native_dynamic_packages` to match the package ids actually materialized in
`materialized_packages[]`; final Report no longer accepts a stage report that names one selected
package set while publishing another.
The same check is tied back to the current Validate stage report:
`test_report_stage_rejects_native_dynamic_package_selection_validate_mismatch` rejects a successful
NativeDynamic stage whose materialized package ids still describe an older selection while
Validate's `plan_summary.native_dynamic_packages` names the package set requested for this run.
`test_report_stage_rejects_native_dynamic_package_export_mismatch` applies the same rule to
`package_exports[]`: even when an ABI v3 package export row is internally self-consistent, its
package id sequence must describe the packages actually staged under `materialized_packages[]`.
`test_report_stage_rejects_native_dynamic_package_export_path_mismatch` closes the field-level
variant of the same handoff: final Report derives the expected package-export `directory`, `path`,
`manifest`, and `package_report` values from the current materialized package destination, then
rejects a stage report that refreshes its file manifest, content hash, and package-local
`native_dynamic_package.toml` while leaving `package_exports[]` pointed at a stale runtime package
path.
`test_report_stage_rejects_native_dynamic_package_export_validate_mismatch` also compares
`materialized_packages[]` against Validate's `plan_summary.native_dynamic_package_exports[].package_id`
rows so a stale NativeDynamic stage cannot publish an old ABI export package set after Validate has
accepted a different one.
`test_report_stage_rejects_native_dynamic_build_plan_package_mismatch` extends that package-id
closure to `native_build_plan.packages[]`, so the Cargo build plan recorded in a successful
NativeDynamic report cannot describe a different package set from the staged payload.
`test_report_stage_rejects_native_dynamic_build_plan_package_header_mismatch` also requires the
build-plan header and package rows to describe the same Cargo workspace, target directory, profile,
release mode, and feature set, so package command evidence cannot stay self-consistent while drifting
away from the plan header.
`test_report_stage_rejects_native_dynamic_build_plan_expected_artifact_mismatch` closes the planned
loadable artifact path to the same target directory, Cargo profile, crate name, and target platform,
so a forged library filename cannot ride along with otherwise valid Cargo command evidence.
The same final Report build-plan command gate rejects feature overrides (`--all-features`,
`--no-default-features`), target broadening (`--all-targets`, `--bins`, `--examples`, `--tests`,
`--benches`, `--lib`), package broadening (`--workspace`, `--all`, `--exclude`), and profile
override (`--profile`) flags. The
`native_build_plan.packages[].features`, `crate_name`, and `cargo_profile`/`release` fields own those
choices, so a command row cannot widen the planned NativeDynamic Cargo build while still matching the
rest of the package audit. Wrapper policy flags such as `--locked` and `--offline` remain owned by
the NativeDynamic command generator and are not rejected by this final Report gate.
The standalone NativeDynamic stage applies the same schema-first rule before deriving that Cargo
profile: a present Validate `profile_summary.build_mode` must be a non-empty trimmed known export
build mode before the stage emits native build package commands. Padded or unknown build-mode
evidence now makes the build plan fatal and leaves its package command table empty instead of
silently falling back to `debug`; clean casing such as `Release` is still normalized to the release
Cargo profile.
The same stage also treats source plugin `plugin.toml` module declarations as schema evidence before
looking up cdylib workspace members: a present `[[modules]].crate_name` must be a non-empty trimmed
string before it is matched against the `zircon_plugins/Cargo.toml` crate index. Malformed module
crate names now stop at that field diagnostic instead of degrading into a broader "declares no
cdylib crate" build-plan error.
Non-string module crate names stop at `modules[index].crate_name must be a string`, preserving the
type error instead of folding it into the empty-string diagnostic.
Non-object `modules[]` rows stop at `modules[index] must be an object`, so malformed rows cannot be
silently skipped and then reported as a missing cdylib crate.
Workspace member paths follow the same rule. `zircon_plugins/Cargo.toml` `workspace.members[]` rows
must be non-empty trimmed safe relative strings before NativeDynamic resolves member `Cargo.toml`
files, so padded or escaping member paths cannot trigger filesystem lookup noise or contaminate the
cdylib crate index.
Member crate manifests must also publish a clean `[package].name` before the crate can join that
index. A blank or padded package name remains a manifest schema diagnostic and does not fall through
to the source plugin's broader no-cdylib-crate diagnostic.
The `[lib].crate-type[]` evidence is checked before the same index membership step: `[lib]` must be
an object, `crate-type` must be an array, and each declared crate type must be a non-empty trimmed
string before `cdylib` membership is considered.
`test_report_stage_rejects_native_dynamic_build_plan_command_feature_broadening`,
`test_report_stage_rejects_native_dynamic_build_plan_command_no_default_features_override`,
`test_report_stage_rejects_native_dynamic_build_plan_command_target_broadening`,
`test_report_stage_rejects_native_dynamic_build_plan_command_package_broadening`, and
`test_report_stage_rejects_native_dynamic_build_plan_command_profile_override` cover those four
command-boundary classes.
`test_report_stage_rejects_native_dynamic_build_execution_package_mismatch` applies the same
package-id closure to `native_build_execution.packages[]`; executed build/copy evidence must describe
the packages that were actually materialized.
`test_report_stage_rejects_native_dynamic_build_execution_command_plan_mismatch` locks the existing
execution-command handoff: each executed package command must still match the corresponding
`native_build_plan.packages[]` command after package-id lookup, so a resume or hand-written report
cannot swap the Cargo invocation while keeping package metadata stable.
`test_report_stage_rejects_native_dynamic_build_execution_disabled_non_empty_table` keeps disabled
execution as a pure empty audit boundary, rejecting package counts or package rows when
`enabled=false`.
`test_report_stage_rejects_native_dynamic_build_execution_fatal_success_report_mismatch` binds the
nested build-execution fatal flag back to the enclosing NativeDynamic report success state.
`test_report_stage_rejects_native_dynamic_build_execution_skipped_success_report_mismatch` closes the
same status boundary for skipped build execution evidence.
`test_report_stage_rejects_native_dynamic_build_plan_fatal_enabled_execution_mismatch` closes the
cross-table state boundary: when `native_build_execution.enabled=true` and the enclosing
NativeDynamic report is successful, `native_build_plan.fatal` must be `false`; disabled or
diagnostic-only build plans keep their existing failure reporting semantics.
`test_report_stage_rejects_native_dynamic_signing_package_mismatch` and
`test_report_stage_rejects_native_dynamic_notarization_package_mismatch` apply the same package-id
closure to operation audits. Signing and notarization reports cannot name a different package set
from the staged NativeDynamic payload.
The operation-audit closure also validates package-level artifact evidence: signing/notarization
`packages[].artifacts[].package_relative_artifact` must match the staged package's current
`materialized_packages[].loadable_artifacts` after stripping that package's `plugins/<package>/`
prefix, and each audit `artifact_count` must match its `artifacts[]` length.
`test_report_stage_rejects_native_dynamic_operation_audit_duplicate_package_relative_artifact`
keeps each package audit's `package_relative_artifact` rows unique before final Report trusts the
artifact closure.
`test_report_stage_rejects_native_dynamic_operation_audit_blank_profile` keeps optional
signing/notarization `profile` evidence trimmed and non-empty before final Report publishes an audit
profile alongside platform allowance evidence.
`test_report_stage_rejects_native_dynamic_operation_audit_artifact_path_mismatch` also requires each
audit artifact path to equal `plugins/<package_id>/<package_relative_artifact>`, so an audit row
cannot point at one bundle file while satisfying the staged loadable-artifact closure with another.
`test_report_stage_rejects_native_dynamic_signing_artifact_mismatch`,
`test_report_stage_rejects_native_dynamic_notarization_artifact_mismatch`, and
`test_report_stage_rejects_native_dynamic_signing_artifact_count_mismatch` keep final Report from
publishing a successful signing/notarization audit for a forged, stale, or non-loadable file path
while the package id itself still matches the staged payload.
`test_report_stage_rejects_native_dynamic_unreported_loadable_artifact` closes the inverse
loadable-artifact audit gap for the NativeDynamic stage. A package that contains an additional
current `.dll`, `.so`, or `.dylib` file must list it in `materialized_packages[].loadable_artifacts`;
otherwise final Report marks the NativeDynamic stage fatal even when the outer file manifest,
content hash, and package-local `native_dynamic_package.toml` were refreshed after the extra
artifact was added.
The final Report stage now mirrors that PlatformBundle `native_plugins_payload` into the top-level
pipeline report only when the PlatformBundle stage is non-fatal, while the full per-stage evidence
remains available under `stages[]`. `test_report_stage_does_not_project_fatal_platform_bundle_payload`
keeps failed bundles from exposing a top-level NativeDynamic payload that Hub/editor readers might
mistake for a consumable release audit. `test_report_stage_does_not_project_profile_mismatch_platform_bundle_payload`
applies the same guard when a PlatformBundle report belongs to another profile; profile mismatches
mark the stage wrapper fatal before any top-level release payload is projected.
The top-level pipeline report also publishes an `export_plan` summary for UI and Hub readers that
need the selected plan without re-parsing the nested Validate report. `export_plan.strategies`
contains the normalized strategies accepted for this profile, `required_stages` records the exact
Report-stage execution closure, `completed_stages` records the loaded stage reports, and
`unsupported_strategies` preserves same-profile unsupported Validate strategy values even when
Validate is already fatal and cannot drive execution. `test_report_stage_aggregates_stage_reports`
keeps the successful SourceTemplate + LibraryEmbed projection stable, while
`test_report_stage_rejects_unknown_validate_strategy_without_defaulting` keeps unsupported strategy
summaries from falling back to the default LibraryEmbed stage closure.
For LibraryEmbed host builds, final Report also requires CompileHost's `link_plan` and cross-checks
it against the same Validate plan summary before it treats the stage set as publishable, so a stale
or incomplete CompileHost report cannot claim a different runtime plugin link matrix under a
successful shape. The same final Report pass binds CompileHost report `command` evidence back to
Validate `plan_summary.library_embed_compile_host`: package, binary, feature list, release/debug
flag, and target directory must match the validated host plan. The command must not include
`--all-features`; `app_features` is the explicit LibraryEmbed feature-selection vector, so an
all-feature Cargo build is treated as feature broadening rather than a publishable host plan. The target-directory comparison
accepts either the Validate relative `target_dir` token or the resolved absolute path under the
current output root, matching CompileHost's execution-time `--target-dir` rewrite. Successful
CompileHost reports must also keep `host_executable` inside the current output root; final Report
rejects external host paths before publishing the LibraryEmbed bundle evidence. When the command
uses the production absolute `--target-dir`, `host_executable` must live under that target
directory's Validate `cargo_profile` subdirectory and its filename must match the validated binary
name, with the Windows `.exe` suffix accepted. The path must already exist and be a file before
the final report can publish the CompileHost stage as release evidence. Non-fatal CompileHost
reports also require `exit_code = 0`; a non-zero Cargo result must remain fatal and cannot be
published as successful host-build evidence.
Validate's `library_embed_compile_host.cargo_profile` is limited to `debug` or `release`, and its
`release` boolean must agree with that profile before the final Report trusts the host build plan.
The same plan object must explicitly carry the full CompileHost field set; missing package,
command, release, linkage, feature, or target fields are final Report schema failures instead of
implicit opt-outs from later consistency checks.
Its string fields must also be non-empty trimmed strings before later command and link-plan
provenance checks run, so blank or padded package, binary, manifest, target directory, and profile
values fail at the Validate plan schema boundary.
The same CompileHost plan boundary now distinguishes field shape from entry type for
`command[]`, `app_features[]`, and `runtime_features[]` in both standalone CompileHost execution
and final Report's nested Validate plan audit: non-array fields still report `must be a string
array`, while malformed entries report `CompileHost plan <field>[index] must be a string` or
`validate report plan_summary.library_embed_compile_host.<field>[index] must be a string` before
command identity, feature matching, blank/trimmed, or duplicate-entry checks consume the array.
`linked_runtime_crates[]` rows use the same shape gate in Validate final Report evidence,
CompileHost stage `link_plan` evidence, and standalone CompileHost pre-launch plan consumption:
`crate_name`, `path`, `provider_package_id`, and `registration_kind` must be non-empty trimmed
strings before safe-path, runtime-crate-name, provider-id, `runtime_plugin` enum, duplicate-name,
or Validate/CompileHost link-plan identity checks consume the row. `test_report_stage_rejects_validate_compile_host_linked_crate_path_invalid`,
`test_report_stage_rejects_compile_host_linked_crate_path_invalid`,
`test_report_stage_rejects_validate_compile_host_linked_crate_registration_kind_invalid`,
`test_report_stage_rejects_compile_host_linked_crate_registration_kind_padded`, and
`test_compile_host_rejects_plan_with_padded_linked_crate_field` keep the final Report and direct
CompileHost execution boundaries aligned.
The Validate plan's `manifest_path` and `target_dir` fields must also be safe relative paths:
absolute paths, empty path segments, `.`, and `..` escapes are rejected before final Report compares
the command row or publishes the host-build plan as release evidence.
The command identity gate also checks both leading tokens: `command[0]` must be `cargo` and
`command[1]` must be `build`, so a non-Cargo executable cannot publish a shape-compatible
`build` command as CompileHost plan evidence.
`test_report_stage_rejects_validate_compile_host_command_feature_mismatch` covers both ordinary
`--features` drift and the `--all-features` broadening flag at final Report time, while
`test_compile_host_rejects_plan_command_metadata_mismatch` covers the same `--all-features`
rejection before the standalone CompileHost stage can launch Cargo from a polluted Validate report.
The same command provenance path rejects Cargo target broadening flags such as `--all-targets`,
`--bins`, `--examples`, `--tests`, `--benches`, and `--lib`, because the CompileHost plan's
`binary` field owns the single host target. `test_report_stage_rejects_validate_compile_host_command_target_broadening`
and `test_compile_host_rejects_plan_command_target_broadening` cover that final Report and
execution-time boundary.
It also rejects package-selection broadening flags such as `--workspace`, `--all`, and `--exclude`.
The CompileHost plan's `package` field owns the single package selection, so workspace-level or
exclude-list Cargo builds cannot be published as evidence for the planned LibraryEmbed host.
`test_report_stage_rejects_validate_compile_host_command_package_broadening` and
`test_compile_host_rejects_plan_command_package_broadening` keep that rule aligned across final
Report aggregation and standalone CompileHost execution.
`--profile` is forbidden for the same reason: the plan's `cargo_profile` and `release` fields own
profile selection, and the command may only express that choice through the debug/release flag
contract. `test_report_stage_rejects_validate_compile_host_command_profile_override` and
`test_compile_host_rejects_plan_command_profile_override` cover a custom profile override before it
can be reported or executed as LibraryEmbed host-build evidence.
Editor-side stdout consumers treat final Report JSON object/key/string lines as summary payload and
only promote values inside a `diagnostics` array to stage diagnostics, so plan fields such as
`export_plan.unsupported_strategies` cannot be mistaken for terminal errors.
The editor ReportBody projection also reads the same final Report stdout JSON to surface
`export_plan.strategies`, `required_stages`, `completed_stages`, and `unsupported_strategies` as
stable `report.export_plan.*` entries beside the pipeline report path. Empty unsupported strategy
sets render as `none`; non-empty sets are flagged as a danger entry so users see unsupported export
paths without opening the raw JSON report.
When the top-level final Report also exposes `native_plugins_payload`, the editor ReportBody uses
that same stdout JSON to surface `bundle_path`, `package_count`, `file_count`, `content_hash`, and
`materialized_packages[].package_id` as stable `report.native_plugins_payload.*` entries. Missing
payloads or malformed/missing fields simply omit those UI rows; the editor does not re-validate or
re-parse nested PlatformBundle stage evidence.
The desktop export plugin's report view descriptors expose the same stable UI rows through
`summary_entry_keys`: SourceTemplate and LibraryEmbed list the pipeline report and `export_plan`
rows, while NativeDynamic additionally lists the `native_plugins_payload` rows.

After M2-T1, the Validate binary still passes:

```powershell
cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
```

Focused `zircon_runtime` lib tests remain blocked by unrelated UI test compile drift, so the
CompileHost matrix tests are written but not fully executed through Cargo. The NativeDynamic-only
minimal-host Rust test also timed out during lib-test compilation; the production `core-min` library
check for the touched export build-plan code passes.

The real NativeDynamic fixture smoke used the checked-in `native_dynamic_fixture` package:

```powershell
python -m tools.zircon_export --profile native-dynamic-fixture-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-smoke-0615\out --stage native_dynamic --native-dynamic-build --offline --pretty
```

It returned `fatal=false`, built one cdylib package with Cargo exit code `0`, and staged the
fixture `.dll` plus `.pdb` sidecar into the NativeDynamic payload.

The ABI v2 fixture feature smoke used the same package with the `abi_v2_only` Cargo feature:

```powershell
python -m tools.zircon_export --profile native-dynamic-fixture-v2-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-v2-smoke-0615\out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
```

It returned `fatal=false`, wrote `native_build_plan.build_features = ["abi_v2_only"]`, built the
cdylib with Cargo exit code `0`, and staged the fixture `.dll` plus `.pdb` sidecar. This proves the
feature-matrix hook and ABI v2 fixture build path, not runtime loading of the fallback ABI.

The same checked-in fixture also has release-mode stage smokes:

```powershell
python -m tools.zircon_export --profile native-dynamic-fixture-release-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-release-smoke-0615\out --stage native_dynamic --native-dynamic-build --offline --pretty
python -m tools.zircon_export --profile native-dynamic-fixture-release-v2-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-release-v2-smoke-0615\out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
```

Both returned `fatal=false`, wrote `cargo_profile = "release"`, completed Cargo with exit code `0`,
and staged the fixture `.dll` plus `.pdb` sidecar. Taken together, the local Windows fixture matrix
now covers debug/release and default/`abi_v2_only` build variants. Linux/macOS cross-platform
fixture execution and runtime load/startup evidence remain pending.

The M1 testing stage passed the validator binary check and a real CLI Validate run:

```powershell
cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
python -m tools.zircon_export --profile windows-release --project D:\zircon-export-m1-smoke\project\zircon-project.toml --out D:\zircon-export-m1-smoke\run-valid --stage validate --offline --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
```

The real smoke wrote `<out>/stages/validate/report.json`, returned non-fatal JSON, and confirmed the
report contained selected linked crates such as `zircon_plugin_rendering_runtime` and
`zircon_plugin_net_http_runtime`. Focused runtime lib-test execution is still blocked by unrelated
UI test compile drift in `table_pointer_routes.rs`, so no `cargo test -p zircon_runtime --lib`
pass is claimed yet.

The M2 Pack smoke used:

```powershell
python -m tools.zircon_export --profile windows-release --out D:\zircon-export-m2-smoke --stage pack --asset-manifest D:\zircon-export-m2-smoke\assets\assets.json --determinism-check --offline --target-dir D:\cargo-targets\zircon-export-m1-validate-0614 --pretty
```

It wrote `<out>/stages/pack/assets.zrpack` and `<out>/stages/pack/report.json`. The report returned
`fatal=false`, included `scenes/main.zscene` and `textures/hero.png`, trimmed
`textures/unused.png`, and set `deterministic_double_run=true`.

The CookAssets project fallback smoke used:

```powershell
python -m tools.zircon_export --profile windows-release --project D:\zircon-export-cook-project-smoke\project\zircon-project.toml --out D:\zircon-export-cook-project-smoke\out --stage cook_assets --asset-filter shipping --pretty
```

It returned `fatal=false`, `generated_from_project=true`, and one staged root
`scenes/main.scene.toml`. The staged manifest stores the resolved source path under the project
`assets/` directory and labels the entry `shipping` because the temporary fallback received
`--asset-filter shipping`.

The Pack profile smoke used:

```powershell
python -m tools.zircon_export --profile windows-release --out D:\zircon-export-pack-profile-smoke\out --stage pack --asset-manifest D:\zircon-export-pack-profile-smoke\assets\assets.json --target-dir D:\cargo-targets\zircon-export-pack-profile-0615 --offline --pretty
```

It returned `fatal=false`, wrote `profile=windows-release` in the Pack report, and packed one asset
into one chunk through the real `zircon_export_pack` binary.

The Pack missing-manifest smoke used the default CookAssets handoff path without creating
`<out>/stages/cook_assets/assets.json`:

```powershell
python -m tools.zircon_export --profile windows-release --out D:\zircon-export-pack-missing-manifest-smoke --stage pack --pretty
```

It returned exit code `2`, wrote `<out>/stages/pack/report.json`, and recorded `fatal=true`,
`profile=windows-release`, and a diagnostic telling the caller to run CookAssets first or pass
`--asset-manifest`.

The M3-T1 template smoke used a placeholder pack and the checked-in template package:

```powershell
python -m tools.zircon_export --profile windows-release --out D:\zircon-export-m3-template-smoke --stage platform_bundle --pack-file D:\zircon-export-m3-template-smoke\inputs\assets.zrpack --template-dir E:\Git\ZirconEngine\tools\zircon_export\tools/zircon_export/export-templates\windows-x86_64-library_embed-debug --target-platform windows-x86_64
```

It returned `fatal=false`, copied the template-declared host placeholder and pack into
`bundle/windows-release`, and wrote a report containing the validated template manifest, file hash,
and computed aggregate `content_hash`. A second smoke with `format_version = 999` returned exit code
`2`, recorded `template format_version 999 is not supported; expected 1`, and skipped bundle copy.
`template.toml` top-level identity fields are now schema evidence rather than forgiving display
text: `template_id`, `engine_version`, `target_platform`, `host_kind`, `host_artifact`,
`resource_strategy`, `plugin_strategy`, `bundle_format`, and `content_hash` must be non-empty trimmed strings before
allowed-value, platform, version, or hash comparisons run. `test_template_rejects_padded_top_level_string_fields`
keeps hand-authored templates from publishing padded identities that are silently repaired by the
reader.
The checked-in platform template manifests now mark their host files as
`host_artifact = "placeholder"` and the report schema rejects any value outside
`placeholder | precompiled`; this is provenance evidence only, not a claim that the current
fixtures are real precompiled hosts.
`compatible_profiles[]` entries are also schema evidence: each present entry must be a non-empty
trimmed string before profile inclusion checks run. `test_template_rejects_padded_compatible_profile_entries`
keeps padded profile ids from collapsing into an indirect profile mismatch or being accepted beside
a valid profile entry.
Standalone template-root validation now uses the same schema-clean gate as final Report template
report validation: blank, padded, or duplicated `compatible_profiles[]` rows prevent profile
membership checks from running, so malformed profile-list evidence does not also emit
`template compatible_profiles does not include requested profile ...`.
Non-string entries are also field-level evidence now: standalone template validation reports
`template.toml field compatible_profiles[index] must be a string`, and embedded PlatformBundle
template manifests report `template.manifest field compatible_profiles[index] must be a string`,
before profile membership or duplicate checks consume the list. The PlatformBundle stage report
summary follows the same rule for `template.compatible_profiles[]`, so a generated or hand-edited
stage report cannot hide a malformed row behind one generic string-array diagnostic.
The same index-level rule now covers embedded template diagnostics and template-root resolution
arrays: `template.diagnostics[]`, `template_resolution.diagnostics[]`,
`template_resolution.candidates[].compatible_profiles[]`, and
`template_resolution.skipped_candidates[].diagnostics[]` report `<field>[index] must be a string`
instead of collapsing malformed rows into a broad string-array error.
Top-level template identity fields use the same rule. `engine_version`, `target_platform`,
`host_kind`, `host_artifact`, `resource_strategy`, `plugin_strategy`, `bundle_format`, and `content_hash` must be
non-empty trimmed strings before allowed-value, version, platform, or digest/content-hash semantics
run; `test_template_rejects_padded_top_level_string_before_allowed_value_semantics` covers that
schema-before-semantics boundary.
Template path fields use the same source-evidence rule before safe-relative normalization:
`[paths].host_executable`, `[bundle].root`, `[bundle].host_path`, `[bundle].pack_path`,
`[bundle].delta_pack_path`, `[bundle].manifest_path`, `[[files]].path`, and
`[[files]].bundle_path` must already be non-empty trimmed strings. `test_template_rejects_padded_path_fields`
keeps hand-authored template paths from being accepted only because the reader stripped leading or
trailing whitespace.
`[[files]].sha256` follows the same evidence ordering before hash-shape validation:
`test_template_rejects_padded_file_sha256` keeps a padded digest from degrading into only
`must declare a SHA-256 hex digest` plus secondary file-list or host-membership diagnostics.
`[[files]].bundle_path` also short-circuits before output-path uniqueness: if the declared bundle
path is blank, padded, or unsafe, the validator reports that field and does not use the source file
path as fallback uniqueness evidence. `test_template_rejects_padded_bundle_path_before_bundle_path_uniqueness`
locks that schema-clean ordering.
Optional `[[files]].purpose` remains absent-friendly, but when a template declares it the value must
be a non-empty trimmed string. `test_template_rejects_invalid_file_purpose` keeps padded file
metadata from being copied into PlatformBundle template reports.

The M3-T2 template-root smoke used the checked-in template repository:

```powershell
python -m tools.zircon_export --profile linux-release --out D:\zircon-export-template-root-smoke --stage platform_bundle --pack-file D:\zircon-export-template-root-smoke\inputs\assets.zrpack --template-root E:\Git\ZirconEngine\tools\zircon_export\tools/zircon_export/export-templates --target-platform linux-x86_64
```

It returned `fatal=false`, resolved `linux-x86_64-library_embed-debug`, wrote
`template_resolution`, and materialized `bundle/linux-release/ZirconRuntime`,
`bundle/linux-release/data/assets.zrpack`, and `bundle/linux-release/zircon-export.json`.
