---
related_code:
  - tools/zircon_export/__init__.py
  - tools/zircon_export/__main__.py
  - tools/zircon_export/cli.py
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
  - tools/zircon_export/pipeline_report_stage_location.py
  - tools/zircon_export/pipeline_report_stage_schema.py
  - tools/zircon_export/pipeline_report_schema_primitives.py
  - tools/zircon_export/pipeline_report_compile_host_stage_schema.py
  - tools/zircon_export/pipeline_report_cook_assets_stage_schema.py
  - tools/zircon_export/pipeline_report_validate_stage_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/pipeline_report_pack_stage_schema.py
  - tools/zircon_export/pipeline_report_pack_manifest_schema.py
  - tools/zircon_export/pipeline_report_pack_delta_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle.py
  - tools/zircon_export/pipeline_report_platform_bundle_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template.py
  - tools/zircon_export/pipeline_report_source_template.py
  - tools/zircon_export/pipeline_report_source_template_stage_schema.py
  - tools/zircon_export/pipeline_report_source_template_validate_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_linkage_schema.py
  - tools/zircon_export/pipeline_report_validate_plan_vector_schema.py
  - tools/zircon_export/pipeline_report_validate_runtime_availability_schema.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/platform_bundle.py
  - tools/zircon_export/report_io.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/stage_handoff.py
  - tools/zircon_export/subprocess_output.py
  - tools/zircon_export/tests/native_dynamic_test_support.py
  - tools/zircon_export/tests/export_test_support.py
  - tools/zircon_export/tests/pack_schema_test_support.py
  - tools/zircon_export/tests/pack_test_support.py
  - tools/zircon_export/tests/test_command_plan.py
  - tools/zircon_export/tests/test_compile_host_output_gate.py
  - tools/zircon_export/tests/test_compile_host_source_template.py
  - tools/zircon_export/tests/test_source_template_project_root_errors.py
  - tools/zircon_export/tests/test_cook_assets_pack_stage.py
  - tools/zircon_export/tests/test_cook_assets_project_fallback.py
  - tools/zircon_export/tests/test_native_dynamic_build_signing.py
  - tools/zircon_export/tests/test_native_dynamic_copy_file_errors.py
  - tools/zircon_export/tests/test_native_dynamic_signing_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_payload_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_path_resolve_errors.py
  - tools/zircon_export/tests/test_native_dynamic_stage.py
  - tools/zircon_export/tests/test_pipeline_report_source_template.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_build_validation.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_manifest_evidence.py
  - tools/zircon_export/tests/test_pipeline_report_cook_assets_pack_handoff.py
  - tools/zircon_export/tests/test_pipeline_report_stage.py
  - tools/zircon_export/tests/test_pipeline_report_stage_location.py
  - tools/zircon_export/tests/test_pipeline_report_stage_metadata.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_delta_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_stage_schema.py
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
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle.py
  - tools/zircon_export/tests/test_report_write_errors.py
  - tools/zircon_export/tests/test_pipeline_resume_flow.py
  - tools/zircon_export/tests/test_stage_directory_errors.py
  - tools/zircon_export/tests/test_subprocess_launch_errors.py
  - tools/zircon_export/tests/test_stage_handoff.py
  - tools/zircon_export/tests/test_templates.py
  - tools/zircon_export/tests/test_native_dynamic.py
  - export-templates/windows-x86_64-library_embed-debug/template.toml
  - export-templates/linux-x86_64-library_embed-debug/template.toml
  - export-templates/macos-aarch64-library_embed-debug/template.toml
  - zircon_export/__init__.py
  - zircon_export/__main__.py
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
  - tools/zircon_export/pipeline_report_stage_location.py
  - tools/zircon_export/pipeline_report_stage_schema.py
  - tools/zircon_export/pipeline_report_schema_primitives.py
  - tools/zircon_export/pipeline_report_compile_host_stage_schema.py
  - tools/zircon_export/pipeline_report_cook_assets_stage_schema.py
  - tools/zircon_export/pipeline_report_validate_stage_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/pipeline_report_pack_stage_schema.py
  - tools/zircon_export/pipeline_report_pack_manifest_schema.py
  - tools/zircon_export/pipeline_report_pack_delta_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle.py
  - tools/zircon_export/pipeline_report_platform_bundle_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template.py
  - tools/zircon_export/pipeline_report_source_template.py
  - tools/zircon_export/pipeline_report_source_template_stage_schema.py
  - tools/zircon_export/pipeline_report_source_template_validate_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_schema.py
  - tools/zircon_export/pipeline_report_validate_compile_host_linkage_schema.py
  - tools/zircon_export/pipeline_report_validate_plan_vector_schema.py
  - tools/zircon_export/pipeline_report_validate_runtime_availability_schema.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/platform_bundle.py
  - tools/zircon_export/report_io.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/tests/native_dynamic_test_support.py
  - tools/zircon_export/tests/export_test_support.py
  - tools/zircon_export/tests/pack_schema_test_support.py
  - tools/zircon_export/tests/pack_test_support.py
  - tools/zircon_export/tests/test_command_plan.py
  - tools/zircon_export/tests/test_compile_host_output_gate.py
  - tools/zircon_export/tests/test_compile_host_source_template.py
  - tools/zircon_export/tests/test_source_template_project_root_errors.py
  - tools/zircon_export/tests/test_cook_assets_pack_stage.py
  - tools/zircon_export/tests/test_native_dynamic_build_signing.py
  - tools/zircon_export/tests/test_native_dynamic_copy_file_errors.py
  - tools/zircon_export/tests/test_native_dynamic_signing_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_payload_file_reads.py
  - tools/zircon_export/tests/test_native_dynamic_stage.py
  - tools/zircon_export/tests/test_platform_bundle_native_dynamic.py
  - tools/zircon_export/tests/test_platform_bundle_native_dynamic_operation_audit.py
  - tools/zircon_export/tests/test_platform_bundle_native_payload_loader_manifest.py
  - tools/zircon_export/tests/test_platform_bundle_delta.py
  - tools/zircon_export/tests/test_platform_bundle_inputs.py
  - tools/zircon_export/tests/test_platform_bundle_path_resolve_errors.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_file_reads.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle_manifest_schema.py
  - tools/zircon_export/tests/test_pipeline_report_platform_bundle.py
  - tools/zircon_export/tests/test_report_write_errors.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_payload.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_package_report_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_schema.py
  - tools/zircon_export/tests/test_pipeline_report_source_template.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_build_validation.py
  - tools/zircon_export/tests/test_pipeline_report_source_template_schema.py
  - tools/zircon_export/tests/test_pipeline_report_stage.py
  - tools/zircon_export/tests/test_pipeline_report_stage_location.py
  - tools/zircon_export/tests/test_pipeline_report_stage_metadata.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_delta_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_stage_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_native_dynamic_schema.py
  - tools/zircon_export/tests/test_pipeline_report_validate_runtime_availability_schema.py
  - tools/zircon_export/tests/test_pipeline_resume_flow.py
  - tools/zircon_export/tests/test_stage_directory_errors.py
  - tools/zircon_export/tests/test_subprocess_launch_errors.py
  - tools/zircon_export/tests/test_templates.py
  - tools/zircon_export/tests/test_native_dynamic.py
  - export-templates/windows-x86_64-library_embed-debug/template.toml
  - export-templates/windows-x86_64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - export-templates/linux-x86_64-library_embed-debug/template.toml
  - export-templates/linux-x86_64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - export-templates/macos-aarch64-library_embed-debug/template.toml
  - export-templates/macos-aarch64-library_embed-debug/bin/zircon_runtime.host-placeholder
  - export-templates/macos-aarch64-library_embed-debug/platform/macos/Info.plist
  - zircon_export/__main__.py
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
  - python -m py_compile tools/zircon_export/__init__.py tools/zircon_export/__main__.py tools/zircon_export/cli.py tools/zircon_export/compile_host.py tools/zircon_export/cook_assets.py tools/zircon_export/export_template.py tools/zircon_export/native_build.py tools/zircon_export/native_dynamic.py tools/zircon_export/native_dynamic_plan.py tools/zircon_export/native_signing.py tools/zircon_export/pipeline_report.py tools/zircon_export/pipeline_report_platform_bundle.py tools/zircon_export/pipeline_stages.py tools/zircon_export/platform_bundle.py tools/zircon_export/source_template.py zircon_export/__init__.py zircon_export/__main__.py tools/zircon_export/tests/export_test_support.py tools/zircon_export/tests/test_templates.py tools/zircon_export/tests/test_compile_host_source_template.py tools/zircon_export/tests/test_cook_assets_pack_stage.py tools/zircon_export/tests/test_native_dynamic_stage.py tools/zircon_export/tests/test_pipeline_report_stage.py tools/zircon_export/tests/test_pipeline_resume_flow.py tools/zircon_export/tests/test_native_dynamic.py tools/zircon_export/tests/test_platform_bundle_delta.py
  - python -m zircon_export --help
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
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload tools.zircon_export.tests.test_pipeline_report_stage_location tools.zircon_export.tests.test_platform_bundle_native_payload_loader_manifest tools.zircon_export.tests.test_pipeline_report_platform_bundle_manifest_schema tools.zircon_export.tests.test_pipeline_report_platform_bundle tools.zircon_export.tests.test_platform_bundle_native_dynamic
  - python -m unittest discover tools.zircon_export.tests
  - python -m unittest tools.zircon_export.tests.test_pipeline_report_native_dynamic_stage_payload
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
  - python -m zircon_export --profile windows-release --out D:\zircon-export-platform-manifest-boundary-smoke-0615\out --stage report --pretty
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
  - python -m zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --resume-from native_dynamic --dry-run
  - python -m zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --stage native_dynamic
  - python -m zircon_export --profile windows-release --out D:/zircon-export-native-dynamic-path-smoke --stage report --pretty
  - python -m zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --resume-from native_dynamic --dry-run
  - python -m zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --stage native_dynamic
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (file_manifest/content_hash smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (native_dynamic_package.toml payload smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (stale package cleanup smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (target-platform artifact filtering smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (loadable artifact gate smoke)
  - python -m zircon_export --profile macos-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (macOS dSYM bundle copy smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (partial package cleanup smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (fatal stage atomic cleanup smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (inconsistent package path gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (inconsistent package_report gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (derived package_report gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (package directory/package_id gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate package_id gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (source manifest id mismatch smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (source manifest parse error smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate recursive source manifest smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (ABI version gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (ABI v3 contract value gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (selection/export consistency gate smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (duplicate selected package_id smoke)
  - python -m zircon_export --profile windows-release --repo-root <temp>/repo --out <temp>/out --stage native_dynamic (fatal package materialization leaves no loader manifest smoke)
  - python -m zircon_export --profile native-dynamic-fixture-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-smoke-0615/out --stage native_dynamic --native-dynamic-build --offline --pretty
  - python -m zircon_export --profile native-dynamic-fixture-v2-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-v2-smoke-0615/out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
  - python -m zircon_export --profile native-dynamic-fixture-release-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-release-smoke-0615/out --stage native_dynamic --native-dynamic-build --offline --pretty
  - python -m zircon_export --profile native-dynamic-fixture-release-v2-smoke --repo-root E:/Git/ZirconEngine --out D:/zircon-native-dynamic-real-fixture-release-v2-smoke-0615/out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
  - python -m zircon_export --profile windows-release --repo-root D:/zircon-export-native-dynamic-materialize-smoke/repo --out D:/zircon-export-native-dynamic-materialize-smoke/out --stage report --pretty
  - python -m zircon_export --profile windows-release --out D:/zircon-export-platform-native-plugins-smoke/out --resume-from platform_bundle
  - python -m zircon_export --profile windows-release --out <temp>/out --resume-from platform_bundle (NativeDynamic payload hash smoke)
  - python -m zircon_export --profile windows-release --out <temp>/out --resume-from platform_bundle (stale NativeDynamic payload hash smoke, expected exit code 2)
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
  - python -m zircon_export --profile windows-release --out D:/zircon-export-source-template-path-smoke --resume-from source_template --dry-run
  - test_cook_assets_reports_asset_manifest_resolve_error
  - test_cook_assets_reports_project_manifest_resolve_error
  - test_pack_reports_missing_asset_manifest_before_packer
  - test_pack_reports_failed_packer_without_stage_report
  - test_pack_rejects_repo_root_resolve_error
  - test_pack_rejects_asset_manifest_resolve_error
  - test_pack_rejects_pack_file_resolve_error
  - test_pack_rejects_packer_resolve_error
  - test_pack_rejects_target_dir_resolve_error
  - python -m zircon_export --profile windows-release --out D:/zircon-export-pack-missing-manifest-smoke --stage pack --pretty (expected exit code 2)
  - python -m zircon_export --profile windows-release --out D:/zircon-export-resume-smoke --resume-from pack --dry-run
  - python -m zircon_export --profile windows-release --project zircon-project.toml --out D:/zircon-export-m1-smoke --stage validate --dry-run --offline --target-dir D:/cargo-targets/zircon-export-validate-cli-0614
  - python -m zircon_export --profile windows-release --out D:/zircon-export-compile-host-dryrun --stage compile_host --dry-run --offline
  - test_compile_host_reports_target_dir_resolve_error
  - python -m zircon_export --profile windows-release --out D:/zircon-export-source-template-dryrun --stage source_template --dry-run --offline
  - python -m zircon_export --profile windows-release --out D:/zircon-export-m2-smoke --stage pack --asset-manifest D:/zircon-export-m2-smoke/assets/assets.json --determinism-check --offline --target-dir D:/cargo-targets/zircon-export-m1-validate-0614
  - cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-export-pack-profile-0615 --message-format short --color never
  - cargo test -p zircon_runtime --bin zircon_export_pack run_rejects_missing_dependency_without_writing_pack
  - cargo test -p zircon_runtime --bin zircon_export_pack run_rejects_duplicate_trim_input_without_writing_pack
  - cargo test -p zircon_runtime --bin zircon_export_pack run_reports_missing_asset_source_without_writing_pack
  - python -m zircon_export --profile windows-release --out D:/zircon-export-pack-profile-smoke/out --stage pack --asset-manifest D:/zircon-export-pack-profile-smoke/assets/assets.json --target-dir D:/cargo-targets/zircon-export-pack-profile-0615 --offline --pretty
  - python -m zircon_export --profile windows-release --project D:/zircon-export-cook-project-smoke/project/zircon-project.toml --out D:/zircon-export-cook-project-smoke/out --stage cook_assets --asset-filter shipping --pretty
  - python -m zircon_export --profile windows-release --out D:/zircon-export-m3-template-smoke --stage platform_bundle --pack-file D:/zircon-export-m3-template-smoke/inputs/assets.zrpack --template-dir export-templates/windows-x86_64-library_embed-debug --target-platform windows-x86_64
  - python -m zircon_export --profile linux-release --out D:/zircon-export-template-root-smoke --stage platform_bundle --pack-file D:/zircon-export-template-root-smoke/inputs/assets.zrpack --template-root export-templates --target-platform linux-x86_64
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
  - test_cook_assets_derives_project_default_scene_without_manifest
  - test_cook_assets_project_fallback_records_direct_res_asset_references
  - test_cook_assets_project_fallback_records_recursive_direct_references
  - test_cook_assets_project_fallback_treats_binary_reference_as_leaf
  - test_cook_assets_project_fallback_orders_assets_and_dependencies_deterministically
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

`python -m zircon_export` is the staged export pipeline entry point for project-level release
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

`zircon_export/__main__.py` is a thin top-level wrapper so the plan command works directly from the
repository root:

```powershell
python -m zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export
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

Successful non-fatal Validate reports are now a required release-evidence shape:
`project_manifest`, `stage_output`, `profile_found`, `fatal_diagnostics`, `profile_summary`, and
`plan_summary` must be present before final Report trusts the stage as publishable. Historical
standalone/debug reports may still omit `profile_summary.strategies` for strategy inspection
compatibility, but the `profile_summary` object itself is required on non-fatal Validate reports.
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
`test_compile_host_rejects_plan_with_empty_command` requires the planned Cargo command to be a
non-empty string array, so CompileHost never launches a command made only from appended flags such as
`--locked` and `--target-dir`.
`test_compile_host_rejects_plan_with_blank_command_entry` applies the same rule to each command
element, rejecting blank entries before wrapper flags are appended.
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
stdout/stderr arrays. Inside `link_plan`, `app_features`, `runtime_features`,
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
separate from the missing-evidence gate.

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
empty, or blank-entry build commands in `plan_summary.source_template_build`, invalid or escaping generated-file
paths, a build `manifest_path` that escapes the generated project, generated-file rows without
contents, a missing generated `Cargo.toml`, a generated `Cargo.toml` path that is not a regular
file, an unreadable/unwritable generated `Cargo.toml`, or missing rewritten local `zircon_*`
dependency paths.
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
`stdout_lines` and `stderr_lines`; final Report requires both fields to be string arrays before it
accepts the build validation evidence. This keeps generated-project build failures auditable in
the stage report instead of reducing them to an exit code and one diagnostic.

When Validate report `profile_summary.strategies` contains `source_template`, the main pipeline now
includes this stage. A SourceTemplate-only profile runs `Validate -> SourceTemplate -> Report`; a
hybrid SourceTemplate + LibraryEmbed profile runs SourceTemplate first and then the LibraryEmbed
host/assets/bundle stages. This keeps the first-class `python -m zircon_export --profile <name>`
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

The stage report records the Validate report path, `native_dynamic_packages`, the full
`native_dynamic_package_exports` table, `package_count`, `loader_manifest`, each materialized
package source/destination/report path, each package's stage-relative `loadable_artifacts` plus
`loadable_artifact_count`, and a `native_build_plan`. The build plan reads the selected source
package `plugin.toml` module crate names, matches them against `cdylib` members declared in
`<repo-root>/zircon_plugins/Cargo.toml`, derives the Cargo profile from Validate
`profile_summary.build_mode`, and records the target directory, exact `cargo build` command
(`--manifest-path`, `-p`, `--target-dir`, lock/offline/release flags), and platform-specific
expected loadable artifact path for every matched package. The default target directory remains
`<out>/stages/native_dynamic/target`, but an explicit `--target-dir` feeds the native cdylib build
plan, execution command, and expected loadable artifact path. Repeated
`--native-dynamic-build-feature <feature>` values are normalized, deduplicated, recorded in
`native_build_plan.build_features` and each package plan's `features`, and appended to the Cargo
command as `--features <comma-separated features>`.
When PlatformBundle reuses a stage-backed NativeDynamic payload summary, the stage report
`plugins_dir` and the caller's current `plugins_dir` are canonicalized separately, so
`test_native_dynamic_payload_summary_rejects_reported_plugins_dir_resolve_error` and
`test_native_dynamic_payload_summary_rejects_current_plugins_dir_resolve_error` preserve which side
failed instead of collapsing both cases into a generic report-field diagnostic.
Final Report now treats `native_build_plan` and `native_build_execution` as typed NativeDynamic
stage evidence instead of opaque objects. The stage loader closes build-plan top-level fields,
package command rows, build-execution top-level fields, and per-package execution rows before
PlatformBundle or Report trusts build commands, expected artifacts, stdout/stderr, copied sidecars,
or package counts. Wrong-typed entries produce diagnostics such as
`native_dynamic report native_build_plan.packages[0].command must be a string array` and
`native_dynamic report native_build_execution.packages[0].exit_code must be an integer`.
For non-fatal stage reports, `native_build_plan` must also carry the complete plan header:
`workspace_manifest`, `target_dir`, `cargo_profile`, `release`, `build_features`, `package_count`,
`diagnostics`, `packages`, and `fatal`. Missing any of those header fields marks `NativeDynamic`
fatal before the package command rows are trusted.
`native_build_execution` follows the same release-evidence rule for its execution header:
`enabled`, `fatal`, `diagnostics`, `package_count`, and `packages` must be present even when
execution is disabled and the package table is empty.

By default this plan is deliberately non-executing: missing workspace metadata is reported inside
`native_build_plan.diagnostics`, while package materialization still consumes existing artifacts
under each package's `native/` directory. Passing `--native-dynamic-build` turns the plan into an
execution gate. In that mode the stage may materialize package metadata/resources before source
`native/` artifacts exist, runs each planned Cargo command from `--repo-root`, copies the expected
`.dll`/`.so`/`.dylib` into the staged package `native/` directory, copies adjacent `.pdb`/`.dbg` or
`.dSYM` sidecars when present, then writes `native_dynamic_package.toml` after the built artifact is
part of the package payload. The stage report records this as `native_build_execution` with per
package command output, exit code, expected artifact, copied artifact, copied sidecars, and fatal
diagnostics.
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
boolean/string-array/integer diagnostics before normalization.
For non-fatal NativeDynamic stage reports, the stage-only operation audit fields are release
evidence: missing `native_signing.diagnostics`, `native_signing.packages`,
`native_notarization.diagnostics`, or `native_notarization.packages` marks the NativeDynamic stage
fatal before PlatformBundle can project the stable audit summary.
For full NativeDynamic stage audit objects, each `packages[]` row must also carry string
`package_id`, integer `artifact_count`, and object-array `artifacts` evidence before final Report
accepts the operation audit shape. Each artifact row must carry the full execution evidence emitted
by a completed signer/notarizer command: string `artifact`, string `package_relative_artifact`,
string `stdout`, string `stderr`, integer `exit_code`, string `before_sha256`, string
`after_sha256`, and string-array `command`. Missing execution fields make the NativeDynamic stage
fatal before PlatformBundle can project the stable summary, so a non-fatal operation audit cannot
claim a signed or notarized artifact without the command result and before/after bytes evidence.

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
has invalid cdylib plan metadata in build mode, fails Cargo execution, or does not produce its
expected loadable artifact, or fails the configured signing/notarization command, the stage writes a fatal report,
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
based on the source manifest directory, requires declared sources to resolve to regular files, sorts
explicit manifest `assets[]` by package path, sorts and deduplicates `roots[]`, `dependencies[]`,
and `labels[]`, and writes:

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
like missing referenced files.
Explicit cooked manifests use the same normalized ordering before they are staged: `assets[]` is
sorted by package path, while `roots[]`, `dependencies[]`, and `labels[]` are sorted and deduplicated
after the manifest passes the basic shape checks. Duplicate asset paths remain a fatal manifest-shape diagnostic before
Pack sees the handoff.
Explicit asset `source` rows that resolve to directories or other non-file paths are fatal at
CookAssets time, so Pack only receives source rows that can represent asset bytes.
The explicit asset manifest and fallback project manifest are checked as files before parsing;
directory paths or unreadable inputs become CookAssets diagnostics and no cooked `assets.json` is
written.
They are also resolved through CookAssets diagnostics before parsing. If explicit
`--asset-manifest` or `--project` path canonicalization fails, the stage writes a fatal report with
the failed field set to `null` and does not emit a cooked manifest.

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
When supplied directly, `--asset-filter` must also be a non-empty string; an empty value is fatal
instead of being treated as "no filter." Dry-run uses the same preflight and prints the same
diagnostic. Pipeline/resume execution preserves that explicit empty value and does not replace it
with a Validate report default, so the same hard gate applies through every CookAssets entry point.
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
evidence even when the CookAssets report, hash, and Pack handoff are edited consistently. The same
final-report helper also checks the Pack handoff: a non-fatal `pack` report `asset_manifest` must
resolve to the same file as the current non-fatal CookAssets `cooked_asset_manifest`, so Pack cannot
silently switch to a different manifest after CookAssets. It also parses the staged manifest JSON
and checks that report `asset_count` equals `assets[]` length, `root_count` equals `roots[]` length,
and report `asset_filter` equals staged manifest `asset_filter`, so hand-edited report counts and
filter provenance cannot drift from the published CookAssets input. Final Report also validates the
staged manifest shape itself: `roots` must be a string array, `asset_filter` must be a string when
present, `assets` must be an object array, and each asset row keeps `path`, `source`,
`dependencies`, and `labels` on the same typed contract as CookAssets execution. When an asset row
carries `source`, final Report also requires the staged value to be an absolute path that still
points to a regular file, so hand-edited or stale manifests cannot claim Pack-readable byte
provenance. It then keeps the staged handoff on CookAssets' deterministic output contract:
`roots[]` and `assets[]` must remain sorted, and each `dependencies[]` and `labels[]` list must
remain sorted and deduplicated. The same cross-stage helper
derives CookAssets' expected trim evidence from `roots`, transitive
`dependencies`, optional `asset_filter`, and per-asset `labels`, then requires a non-fatal Pack
report's `trim_report.included_assets`, `trim_report.trimmed_assets`, and
`trim_report.missing_dependencies`, `trim_report.duplicate_assets`, and `trim_report.diagnostics`
to match that closure. Every path in `trim_report.included_assets` must also map to a CookAssets
manifest asset row with a non-empty `source`, matching the Rust packer's byte-input contract that
included assets without sources fail before pack bytes are written. For those included assets,
`tools/zircon_export/pipeline_report_cook_assets_source_bytes.py` reuses the Rust writer's four-seed
FNV1a content hash algorithm and compares Pack `manifest.assets[].size` plus `chunk_hash` against
the actual CookAssets source file bytes, so a hand-authored Pack success report cannot claim stale
chunk metadata for different source contents.
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
An explicit `--asset-manifest` value is release input evidence and must be a non-empty string. An
empty explicit value is preserved through pipeline/resume defaulting, reports `asset_manifest=null`
in Pack's fatal preflight report, and skips command construction instead of previewing a default
manifest path.
The Pack output path follows the same rule: an explicit `--pack-file` must be non-empty, and an
empty value reports `pack=null` plus `command=<skipped>` instead of falling back to
`<out>/stages/pack/assets.zrpack`.
Incremental output uses the same hard preflight: `--previous-pack` and `--delta-pack` must be
supplied together as non-empty strings. If either delta argument is malformed, dry-run prints the
diagnostic and `command=<skipped>` instead of previewing a partial packer command.

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
Final Report aggregation treats a non-fatal Pack report as release evidence rather than an optional
hint. A successful Pack report must include `asset_manifest`, `pack`, `stage_output`, `asset_count`,
`chunk_count`, `deduplicated_assets`, `deterministic_double_run`, `trim_report`, and `manifest`;
missing any of those fields marks `Pack` fatal during final aggregation. Delta report fields remain
optional/nullable for full-pack-only exports, with the existing `delta_pack` and
`delta_apply_verified` gates still enforcing requested delta publication.
For non-fatal Pack reports, `stage_output` must also resolve to the current Pack stage directory
derived from the loaded `<out>/stages/pack/report.json` path. This prevents an external or stale Pack
stage directory from being published as the current pipeline stage while the report body remains
otherwise schema-clean.
Because missing dependencies and duplicate trim inputs are Pack preflight failures, a non-fatal Pack
report must keep `trim_report.missing_dependencies` and `trim_report.duplicate_assets` empty.
`pipeline_report_pack_stage_schema.py` owns the Pack stage wrapper and trim-report gate,
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
embedded pack manifest.
Chunk tables are release evidence too. Schema-clean `manifest.pack.chunks`,
`delta_manifest.base.pack.chunks`, `delta_manifest.target.pack.chunks`, and
`delta_manifest.chunks` may not repeat the same 32-byte chunk hash and must already be sorted by
chunk hash, so count-correct reports cannot publish duplicate or non-deterministic rows for the
content-addressed pack or delta payload table.
Pack document asset rows are also tied to those chunk tables: every schema-clean
`assets[].chunk_hash` in the outer manifest and in `delta_manifest.base` / `target` must reference a
hash present in that same document's `pack.chunks`, and each asset's `size` must equal the size of
the chunk it references. ZRPD `delta_manifest.changed_assets` rows follow the same size rule against
`delta_manifest.chunks`.
For schema-clean pack documents, `pack.total_size` must equal the sum of `pack.chunks[].size`,
matching `ZrPackWriter`'s unique-chunk byte accounting.
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
assets in the pack report. This is the byte-package/report layer only; runtime application of a
NativeDynamic hot update remains a later slice.
Final Report re-derives the delta asset sets from the embedded `delta_manifest`: `removed_assets`
must equal `base.assets[].path - target.assets[].path`, `changed_assets` must be the full target
asset entries whose `chunk_hash` is not present in the base pack chunks, and report-level
`delta_reused_assets` must be the target paths whose `chunk_hash` already exists in the base pack
chunks. Report-level `delta_removed_assets` must also mirror `delta_manifest.removed_assets`, and
`delta_manifest.chunks[].hash` must equal the unique chunk hashes referenced by those changed asset
entries.
The full-pack manifest and delta target manifest are also tied together: when a non-fatal Pack
report carries both schema-clean `manifest` and `delta_manifest.target` objects, final Report compares
the pack version, chunk table, total size, and asset entries. The target manifest embedded in a
delta report must describe the same full pack as the outer `manifest`, so stale or hand-authored
delta evidence cannot point at a different target package while reusing the current pack report.
The delta publication evidence is paired: if a Pack report publishes `delta_pack`, it must also carry
the schema-clean `delta_manifest` that explains the delta contents; if it carries a `delta_manifest`,
it must also publish the `delta_pack` path. This blocks hand-authored reports that claim delta bytes
without the manifest evidence needed to audit them, or manifest evidence without a released delta
artifact.
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
pipeline/resume defaulting preserves explicit empty values and lets PlatformBundle fail with a
parameter diagnostic instead of replacing them with stage report defaults.
Final Report aggregation now applies the same release-evidence rule to non-fatal PlatformBundle
stage reports. A successful report must include `bundle`, `host_executable`, `host_source`,
`host_source_origin`, `pack`, `pack_source`, `pack_source_origin`, `template_files`, and
`bundle_manifest`; missing fields make `PlatformBundle` fatal at the stage schema layer before
bundle manifest or payload hash checks run. Delta, NativeDynamic payload, template metadata, and
template-root resolution fields remain optional/nullable unless their corresponding export path or
strategy is active.
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
verifies `paths.host_executable` and each `[[files]].path` are safe relative paths: no absolute
path, empty segment, `.`, or `..`. It then checks that the declared host path stays inside the
template directory, is present in `[[files]]`, and matches its declared SHA-256 digest and
aggregate `content_hash`. Every declared `[[files]].path` must resolve to a regular file before
hashing or bundle copy; a directory at that path is reported as `is not a file` in the template or
PlatformBundle diagnostics instead of surfacing as a filesystem exception.

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
be a string array so final reports and editor/Hub readers can display stage diagnostics without
accepting arbitrary JSON as release-audit text.
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
python -m zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export
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
python -m zircon_export --help
python -m zircon_export --profile windows-release --project zircon-project.toml --out D:\zircon-export --stage validate
python -m zircon_export --profile windows-release --out D:\zircon-export --stage validate --dry-run --offline --target-dir D:\cargo-targets\zircon-export-validate-cli-0614
python -m zircon_export --profile windows-release --out D:\zircon-export --stage compile_host --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage source_template --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage source_template --source-template-build --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-build --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-sign-command D:\tools\sign-native.exe --native-dynamic-sign-arg "{artifact}"
python -m zircon_export --profile windows-release --out D:\zircon-export --stage native_dynamic --native-dynamic-notarize-command D:\tools\notarize-native.exe --native-dynamic-notarize-arg "{artifact}" --native-dynamic-notarize-profile windows-attestation --native-dynamic-notarize-platform windows
python -m zircon_export --profile windows-release --out D:\zircon-export --stage cook_assets --asset-manifest D:\zircon-export\assets\assets.json
python -m zircon_export --profile windows-release --out D:\zircon-export --stage pack --determinism-check
python -m zircon_export --profile windows-release --out D:\zircon-export --stage pack --previous-pack D:\zircon-export\previous\assets.zrpack --delta-pack D:\zircon-export\stages\pack\assets.delta.zrpd
python -m zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --host-executable D:\zircon-export\stages\compile_host\zircon_runtime.exe
python -m zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --host-executable D:\zircon-export\stages\compile_host\zircon_runtime.exe --native-plugins-dir D:\zircon-export\stages\native_dynamic\plugins
python -m zircon_export --profile windows-release --out D:\zircon-export --stage platform_bundle --pack-file D:\zircon-export\stages\pack\assets.zrpack --template-dir export-templates\windows-x86_64-library_embed-debug --target-platform windows-x86_64
python -m zircon_export --profile linux-release --out D:\zircon-export --stage platform_bundle --pack-file D:\zircon-export\stages\pack\assets.zrpack --template-root export-templates --target-platform linux-x86_64
python -m zircon_export --profile windows-release --out D:\zircon-export --stage report
python -m zircon_export --profile windows-release --out D:\zircon-export --resume-from pack
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
build plan and execution command.
`--native-dynamic-sign-command` enables an external signer for staged NativeDynamic loadable artifacts;
repeat `--native-dynamic-sign-arg` for signer arguments and use placeholders such as `{artifact}` and
`{target_platform}` when the signer needs artifact-specific values. Add
`--native-dynamic-sign-profile` to record and pass a profile label, and repeat
`--native-dynamic-sign-platform` to restrict that profile to target-platform prefixes before the
external signer is launched. `--dry-run` prints the exact stage command or bundle inputs without
creating stage output. Cargo commands use `--locked` by default;
`--native-dynamic-notarize-command` enables an external notarization or platform post-processing
command after signing and before package reports/manifests are sealed; repeat
`--native-dynamic-notarize-arg` for arguments, use `--native-dynamic-notarize-profile` to record a
profile label, and repeat `--native-dynamic-notarize-platform` to gate target-platform prefixes
before the command is launched.
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
repeated `--native-dynamic-build-feature` values are normalized into the build plan and Cargo
command, that `test_native_dynamic_build_plan_respects_target_dir_override` keeps custom
`--target-dir` values aligned across the native build plan, command, and expected loadable path, that
`--native-dynamic-build` can execute that plan and stage the built loadable artifact into the package
before package payload reports and file manifests are written, that an explicit signer command can
mutate staged loadable artifacts before package payload reports and file manifests are sealed, that
the signing report records before/after hashes and command execution, that signing artifact hash
read failures before or after the external command become fatal diagnostics instead of filesystem
exceptions, and that signing failures clean the staged payload atomically, that an explicit
notarization/post-processing command runs after
signing but before package payload reports and file manifests are sealed, that the notarization
report records before/after hashes and command execution, and that notarization platform mismatches
clean the staged payload atomically,
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
export table must match exactly, that duplicate selected package ids are rejected before
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
`test_report_stage_rejects_pack_trim_included_assets_outside_cook_assets_closure`
keeps final Report cross-stage evidence honest: a non-fatal Pack report whose included assets do not
match the CookAssets roots/dependencies/asset_filter closure is rejected even when the Pack manifest
and trim report are internally consistent.
`test_report_stage_rejects_pack_included_asset_missing_cook_assets_source` keeps that same included
asset set tied to Pack-readable source bytes by rejecting a non-fatal Pack report when an included
CookAssets asset row has no `source`.
`test_report_stage_rejects_pack_asset_manifest_drift_from_cook_assets_source` extends that byte
evidence to Pack manifest metadata: included asset `size` and `chunk_hash` must match the actual
CookAssets source bytes rather than a stale hand-written Pack manifest.
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
If a final loader manifest row explicitly carries `path`, `manifest`, or `package_report`, final
Report derives the expected `plugins/<package>` values from the final `bundle/plugins/` root and the
projected `materialized_packages[].destination/package_report` fields. Any row-level field drift is
fatal before the top-level payload is exposed, while minimal historical rows that only carry `id`
remain accepted.
When final Report is inspecting legacy Validate output that did not require the NativeDynamic stage
directly, `test_current_output_stage_report_path_reports_plugins_dir_resolve_error` keeps the
expected current-output report path derivation diagnostic if `native_plugins` cannot be
canonicalized.
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
bundle path, missing, malformed, shape-invalid, mismatched, package-id-drifted, or row-field-drifted loader manifest, spoofed operation audit, external package
destination, external or missing package report, or `package_count` mismatch makes the pipeline report fatal and suppresses that top-level payload
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
`test_report_rejects_native_plugins_payload_package_destination_outside_plugins`,
`test_report_rejects_native_plugins_payload_package_report_outside_package`,
`test_report_rejects_missing_native_plugins_payload_package_report`,
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
SourceTemplate helpers.
The Validate-side schema helper owns both the object-level `plan_summary.source_template_build`
gate and the list-level `plan_summary.generated_files` gate, plus the row-level generated-file
allow-list, so the generic stage dispatcher only routes each SourceTemplate plan field when it is
present.
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
The top-level SourceTemplate `command` must be a non-empty string array and include
`--manifest-path` pointing at the current generated `project/Cargo.toml`; a command that targets
another manifest is not publishable evidence. Its `--manifest-path` value must also be present and
must be an actual path value, not another command option.
If the top-level command includes `--target-dir`, that option follows the same value-shape rule;
it remains optional in historical reports but cannot be dangling, use another option as its value,
or point outside the current SourceTemplate stage target directory.
The nested build evidence must remain semantically publishable as well: its command must be a
non-empty string array, any `--manifest-path` or `--target-dir` entries must pass the same direct
option-value diagnostic as the top-level command, the working directory must resolve to the
generated `project`, skipped builds keep `exit_code=null`, and executed builds must have been
requested.
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
`test_report_rejects_missing_source_template_validate_build_plan`,
`test_report_rejects_malformed_source_template_validate_build_plan_command`,
`test_report_rejects_blank_source_template_validate_build_plan_command_entry`,
`test_report_rejects_source_template_validate_build_plan_option_value`,
`test_report_rejects_malformed_source_template_validate_build_plan_manifest_path`, and
`test_report_rejects_malformed_source_template_validate_build_plan_target_dir` cover the Validate
build-plan provenance and shape gates.
`test_report_rejects_source_template_validate_build_plan_unknown_field` closes the Validate
`source_template_build` object schema as well: only `manifest_path`, `target_dir`,
`cargo_profile`, `release`, and `command` are accepted as plan evidence.
`test_report_rejects_absolute_source_template_validate_build_plan_manifest_path` and
`test_report_rejects_escaped_source_template_validate_build_plan_manifest_path` cover the Validate
build-plan manifest path boundary gate.
`test_report_rejects_source_template_validate_build_plan_target_dir_mismatch` covers the Validate
build-plan target directory provenance gate.
`test_report_rejects_source_template_validate_build_plan_target_dir_resolve_error` keeps
`target_dir` canonicalization failures on the same diagnostic path before the provenance mismatch
comparison runs.
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
`tools/zircon_export/tests/test_pipeline_report_source_template_build_validation.py`. The generic
`test_pipeline_report_stage.py` keeps cross-stage aggregation, strategy fallback, malformed strategy,
and NativeDynamic projection coverage; basic metadata gates are split into
`test_pipeline_report_stage_metadata.py`.
Successful non-fatal SourceTemplate stage reports are now a required release-evidence shape:
`project`, `validate_report`, `generated_files`, `command`, `build_executed`, and
`build_validation` must be present before final Report follows generated-project evidence.
The stage schema also accepts the real cleanup audit fields `project_cleaned` and
nullable `cleanup_reason`, so reports emitted by `run_source_template(...)` are not rejected as
unknown-field evidence. Missing any required success field marks `SourceTemplate` fatal during
stage schema loading.
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
checked-in `export-templates/windows-x86_64-library_embed-debug/template.toml` resolves its declared
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

M3-T2 template-root coverage verifies that `--template-root export-templates --target-platform
linux-x86_64` resolves the checked-in Linux template, records `template_resolution`, materializes the
Linux directory layout, and returns a fatal report when no compatible profile/platform template is
found. `test_template_root_skips_invalid_matching_template_candidate` keeps template-root resolution
from letting a corrupted template package shadow a valid package for the same profile/platform:
matching candidates are fully validated before selection, invalid matches are recorded under
`template_resolution.skipped_candidates`, and only valid candidates participate in the duplicate
candidate check. `test_template_root_skips_malformed_template_manifest` applies the same audit path
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
`id`, a different `[[plugins]].id` sequence, malformed TOML, or explicit `path`/`manifest`/
`package_report` values that no longer match the final bundle package projection,
`test_report_rejects_native_plugins_payload_loader_manifest_missing_plugins_table`,
`test_report_rejects_native_plugins_payload_loader_manifest_bad_plugin_id`,
`test_report_rejects_native_plugins_payload_loader_manifest_package_mismatch`, and
`test_report_rejects_native_plugins_payload_malformed_loader_manifest`,
`test_report_rejects_native_plugins_payload_loader_manifest_path_mismatch`,
`test_report_rejects_native_plugins_payload_loader_manifest_manifest_mismatch`, and
`test_report_rejects_native_plugins_payload_loader_manifest_package_report_mismatch` keep the
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
`target_platform` remain nullable/optional metadata fields. These NativeDynamic payload schema rules live
in `pipeline_report_native_dynamic_payload_schema.py`, while bundled `native_dynamic_package.toml`
top-level, `[abi]`, `[payload]`, and package-local `[[payload.files]]` schema diagnostics live in
`pipeline_report_native_dynamic_package_report_schema.py`. `pipeline_report_native_dynamic_payload.py`
keeps the path containment, stage-report, hash, package-report, and operation-audit consistency
checks and imports both schema modules.
The payload schema imports the common bool/integer/string/string-array/object/object-array
diagnostic primitives from `pipeline_report_schema_primitives.py`, while its local table and
sequence helpers keep the NativeDynamic-specific field labels, object-array row dispatch, and
unknown-field attribution.
`test_report_rejects_native_plugins_payload_content_hash_non_string_without_semantic_fallback`
locks the handoff between those layers: wrong-typed top-level payload fields stop at dotted schema
diagnostics such as `native_plugins_payload.content_hash must be a string` and do not continue into
non-empty/hash semantic checks that assume the field type is already trustworthy.
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
schema helpers are reusable from `pipeline_report_native_dynamic_payload_schema.py`; bundled
package-report schema lives in `pipeline_report_native_dynamic_package_report_schema.py` and reuses
the same file-manifest row contract for package-local payload files. Final Report only calls the
normalized manifest/package fallback after the matching object-array schema has no shape or
field-type diagnostics. The file-manifest and materialized-package schema tests now assert
that object-array and row-field type errors do not also emit `file_manifest is malformed` or
`materialized_packages are malformed`; the file-manifest test also covers missing `path`, `sha256`,
or `bytes` fields so incomplete rows are not accepted as publishable NativeDynamic payload evidence.
The materialized-package test mirrors that rule for the package audit itself: `package_id`,
`destination`, `loadable_artifact_count`, and `loadable_artifacts` are required release evidence,
while `source` and `package_report` remain optional typed fields.
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
`test_report_rejects_native_plugins_package_report_abi_string_field_types` closes the typed shape
of the generated package-report `[abi]` contract table. ABI v3 contract fields must be strings
before final Report checks them against `NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS`, so numeric or
boolean TOML values report direct `abi.<field> must be a string` schema diagnostics instead of
being conflated with missing/empty-string contract failures.
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
`test_report_rejects_native_plugins_payload_package_report_payload_missing_required_field` extends
that typed gate to missing generated `[payload]` header fields: `file_count` and `content_hash` must
both be present before package-local payload evidence can be compared.
`test_report_rejects_native_plugins_package_report_payload_file_unknown_field` extends the same
closed-schema rule to generated `[[payload.files]]` entries: each file row may contain only `path`,
`bytes`, and `sha256`, so file-level sidecar fields are reported as direct package-report
diagnostics before the final bundle audit falls back to outer payload hash drift.
`test_report_rejects_native_plugins_package_report_payload_files_non_object_array` and
`test_report_rejects_native_plugins_package_report_payload_file_field_types` close the typed shape
of those generated payload file rows. `[payload].files` must be an object array, every row must be a
TOML table, `path` and `sha256` must be strings, and `bytes` must be an integer before final Report
uses package-report payload file evidence for content-hash and file-manifest comparison. The same
tests now also assert that shape/type failures do not fall through to `payload files are malformed`;
that fallback is reserved for schema-clean payload file evidence that still cannot normalize.
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
`test_report_stage_rejects_validate_profile_summary_string_fields_non_string`,
`test_report_stage_rejects_validate_selected_plugins_non_string_array`, and
`test_report_stage_rejects_validate_selected_plugin_ids_invalid` close the core typed
profile fields and selected plugin id shape inside that object. `name`, `target_mode`,
`target_platform`, `build_mode`, and `asset_filter` must be strings, while `selected_plugins`
must be a string array whose entries are valid project plugin package ids before final Report
accepts the profile summary as release-plan evidence.
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
load. Non-string strategy entries now fail as Validate schema diagnostics and mark `Validate` in
`fatal_stages`, instead of falling through to a later unsupported-strategy diagnostic with no stage
attribution.
`test_report_stage_rejects_validate_profile_strategies_empty_as_schema` gives an explicitly empty
strategy list the same Validate-stage ownership: empty `profile_summary.strategies` now marks the
Validate wrapper fatal before final Report falls back to generic strategy requirement handling.
`test_report_stage_rejects_validate_profile_strategies_unknown_as_schema` and
`test_report_stage_rejects_validate_profile_strategies_not_trimmed` apply the same ownership to
individual strategy values. Unknown strategy names still use the shared `unsupported export
strategy ...` diagnostic, but now mark `Validate` in `fatal_stages`; empty or padded strategy
strings fail as non-empty trimmed export strategy schema before normalization aliases are applied.
`test_report_stage_rejects_validate_profile_features_non_object`,
`test_report_stage_rejects_validate_profile_feature_list_non_string_array`, and
`test_report_stage_rejects_validate_profile_feature_ids_not_trimmed` close the selected feature
matrix shape. `profile_summary.features` must be an object whose values are string arrays, and
owner plugin ids plus selected feature ids must be non-empty trimmed strings, matching the Rust
`BTreeMap<String, Vec<String>>` shape before final Report accepts it as profile feature-selection
evidence. `test_report_stage_rejects_validate_profile_feature_owner_plugin_ids_invalid` also routes
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
`fatal_diagnostics` must be a string array; wrong types now mark `Validate` in `fatal_stages`
instead of passing through as harmless sidecar metadata.
`test_report_stage_rejects_validate_summary_objects_non_object` closes the two summary containers
themselves. `profile_summary` and `plan_summary` must be objects before final Report derives the
required stage set or inspects build-plan evidence; malformed containers now stop at `Validate`
instead of causing fallback stage requirements and unrelated missing-stage diagnostics.
`test_report_stage_rejects_validate_plan_summary_unknown_field` closes the nested `plan_summary`
object that carries linked crates, NativeDynamic package exports, SourceTemplate generated files,
LibraryEmbed compile plans, and runtime plugin availability. Unknown plan-summary keys make Validate
fatal before any stage consumes plan evidence.
Validate wrapper schema and `plan_summary` dispatch now live in
`tools/zircon_export/pipeline_report_validate_stage_schema.py`; the generic
`pipeline_report_stage_schema.py` only registers the Validate stage fields and delegates the
wrapper-specific schema work to that module.
Shared bool/integer/string/string-array/object/object-array primitive diagnostics used by the
remaining injected NativeDynamic and Pack schema paths live in
`tools/zircon_export/pipeline_report_schema_primitives.py`, so the stage dispatcher only wires
those helpers into downstream schema modules.
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
`native_dynamic_packages` must be string arrays, `enabled_runtime_plugins` entries must also be
valid project plugin package ids, and `native_dynamic_packages` entries must be non-empty trimmed
NativeDynamic package ids before final Report accepts them as selected runtime plugin, linked
crate, or NativeDynamic package evidence.
`test_report_stage_rejects_validate_linked_runtime_crate_names_invalid` also validates
`linked_runtime_crates[]` entries against the project runtime crate naming contract before final
Report trusts linked crate evidence.
The identifier helpers live in `pipeline_report_validate_identifier_schema.py`, keeping stage
report orchestration separate from reusable id/token diagnostics.
The top-level plan-vector dispatch for `enabled_runtime_plugins`, `linked_runtime_crates`, and
`native_dynamic_packages` lives in
`tools/zircon_export/pipeline_report_validate_plan_vector_schema.py`, so the final Report stage
dispatcher only routes `plan_summary` rather than owning every vector's identity contract.
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
build plan.
`test_report_stage_rejects_validate_compile_host_linked_crate_unknown_field` applies that same
closed-schema rule to `library_embed_compile_host.linked_runtime_crates[]`. Each linked crate row
may only carry crate name, path, registration kind, and provider package id, and
`test_report_stage_rejects_validate_compile_host_linked_crate_missing_field` requires all four
fields before final Report accepts it as compile-plan linkage evidence.
`test_report_stage_rejects_validate_compile_host_linked_crate_string_fields_non_string`,
`test_report_stage_rejects_validate_compile_host_linked_crate_names_invalid`,
`test_report_stage_rejects_validate_compile_host_linked_crate_provider_ids_invalid`, and
`test_report_stage_rejects_validate_compile_host_linked_crate_registration_kind_invalid` also
check those linked crate row values in the dedicated CompileHost linkage schema module.
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
required flag, maturity, and reason.
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
That reused schema also requires every CompileHost `link_plan.linked_runtime_crates[]` row to carry
`crate_name`, `path`, `provider_package_id`, and `registration_kind`;
`test_report_stage_rejects_compile_host_linked_crate_missing_field` covers the CompileHost label.
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
`test_report_stage_rejects_cook_assets_unknown_top_level_field` closes the CookAssets handoff on
the same shared loader path. A non-fatal `cook_assets` report may only carry its stage metadata,
source/project manifest provenance, project fallback summary, cooked manifest path, asset/root
counts, staged manifest SHA-256, and asset filter; unknown sidecar fields make the wrapper stage
fatal before Pack can trust `cooked_asset_manifest`.
`test_report_stage_rejects_cook_assets_string_fields_non_string`,
`test_report_stage_rejects_cook_assets_count_fields_non_integer`, and
`test_report_stage_rejects_cook_assets_generated_from_project_non_bool`, and
`test_report_stage_rejects_cook_assets_missing_release_evidence_field` close the typed shape of the
same handoff report. Manifest/default-scene/asset-filter fields must be strings, asset/root counts
must be integers, and `generated_from_project` must be boolean before final Report accepts CookAssets
release evidence. `cooked_asset_manifest`, `cooked_asset_manifest_sha256`, `asset_count`,
`root_count`, and `generated_from_project` are required on non-fatal reports, while
source/project/default-scene and asset-filter provenance may stay `null` when CookAssets did not
consume those optional inputs. `test_report_stage_rejects_cook_assets_manifest_hash_mismatch` also
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
`test_report_stage_rejects_native_dynamic_count_fields_non_integer`,
`test_report_stage_rejects_native_dynamic_bool_fields_non_bool`,
`test_report_stage_rejects_native_dynamic_object_fields_non_object`, and
`test_report_stage_rejects_native_dynamic_object_array_fields_non_object_array` close the typed
shape of the same report. NativeDynamic path/provenance fields must be strings when present,
selected package and artifact extension fields must be string arrays, `package_count` must be an
integer, `payload_cleaned` must be boolean, build/signing/notarization audit summaries must be
objects, and manifest/materialized/package-export evidence must be object arrays before final
Report accepts NativeDynamic release evidence.
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
The same module owns `native_build_plan` / `native_build_execution` nested schema coverage:
`test_report_stage_rejects_native_dynamic_build_plan_unknown_field`,
`test_report_stage_rejects_native_dynamic_build_plan_missing_release_evidence_field`,
`test_report_stage_rejects_native_dynamic_build_plan_field_types`,
`test_report_stage_rejects_native_dynamic_build_plan_package_unknown_field`,
`test_report_stage_rejects_native_dynamic_build_plan_package_field_types`,
`test_report_stage_rejects_native_dynamic_build_execution_unknown_field`,
`test_report_stage_rejects_native_dynamic_build_execution_missing_release_evidence_field`,
`test_report_stage_rejects_native_dynamic_build_execution_field_types`,
`test_report_stage_rejects_native_dynamic_build_execution_package_unknown_field`, and
`test_report_stage_rejects_native_dynamic_build_execution_package_field_types` close the Cargo
build plan and build execution evidence before final Report can trust it. The implementation lives
in `pipeline_report_native_dynamic_stage_schema.py`, keeping the shared stage schema dispatcher
focused on cross-stage wiring.
`test_report_stage_rejects_native_dynamic_operation_audit_missing_stage_evidence_field`
locks the sibling operation-audit header gate: `native_signing` / `native_notarization` stage
objects must carry `diagnostics` and `packages` even when the operation is disabled and the package
table is empty.
`test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_execution_evidence_field`
in `test_pipeline_report_native_dynamic_operation_audit_schema.py` locks the artifact execution
gate: enabled operation audit artifact rows must carry `exit_code`, `before_sha256`, and
`after_sha256`, not just the expanded command and captured streams.
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
`test_report_stage_rejects_pack_delta_manifest_count_mismatch` applies the same rule to delta
reports by binding `delta_asset_count` to `delta_manifest.changed_assets[]` length and
`delta_chunk_count` to `delta_manifest.chunks[]` length.
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
`test_report_stage_rejects_pack_stage_output_outside_current_stage` rejects Pack reports whose
`stage_output` resolves outside the current `<out>/stages/pack` directory derived from the loaded
report path.
`test_report_stage_rejects_pack_deduplicated_assets_manifest_mismatch` derives the expected
`deduplicated_assets` set from repeated `manifest.assets[].chunk_hash` values, matching the writer's
path-sorted first-owner rule before accepting duplicate-content audit evidence.
`test_report_stage_rejects_pack_trim_report_unknown_fields` and
`test_report_stage_rejects_pack_trim_report_field_types` close `trim_report`, trimmed-asset rows,
missing-dependency rows, duplicate/included asset arrays, and diagnostics arrays. These checks live
across the Pack schema modules: stage/trim checks stay in `pipeline_report_pack_stage_schema.py`,
pack manifest/count/dedup checks stay in `pipeline_report_pack_manifest_schema.py`, and delta
manifest/count/asset-set/chunk checks stay in `pipeline_report_pack_delta_schema.py`. The shared
`pipeline_report_stage_schema.py` remains only the cross-stage dispatcher.
Common Pack manifest, chunk, asset, and delta-manifest fixtures now live in
`tools/zircon_export/tests/pack_test_support.py`, while Pack schema report staging and assertion
helpers live in `tools/zircon_export/tests/pack_schema_test_support.py`.
`test_pipeline_report_pack_stage_schema.py` now owns outer Pack manifest and trim-report regressions;
`test_pipeline_report_pack_delta_schema.py` owns delta manifest, delta asset-set, target-manifest, and
publication-pairing regressions.
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
Those PlatformBundle stage-report and loaded `bundle.json` schema rules now live in
`pipeline_report_platform_bundle_schema.py`; `pipeline_report_platform_bundle.py` keeps the final
Report orchestration, manifest loading, path containment, hash, and NativeDynamic payload
consistency checks. Its local primitive checks are imported from
`pipeline_report_schema_primitives.py`, while nested template/payload schema ownership stays in the
PlatformBundle-specific helper modules.
`pipeline_report_platform_bundle_template.py` follows the same split: shared schema primitives own
the basic bool/integer/string/string-array/object/object-array diagnostic text, while the template
helper keeps resolution list wrappers, candidate/skipped-candidate row attribution, copied-template
file matching, path resolution, and sha256 checks.
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
`test_report_rejects_template_bundle_unknown_field` keeps the same gate on the embedded
`template.bundle` object; only `root`, `manifest_path`, `host_path`, `pack_path`, and
`delta_pack_path` are accepted.
`test_report_rejects_template_resolution_unknown_field` closes the template-root selection report
as well: `template_resolution`, `candidates[]`, and `skipped_candidates[]` each reject unknown
fields before final Report accepts them from matching `bundle.json` and stage-report evidence.
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
`stage_output` is also bound to the loaded report's current stage directory, `validate_report`
records the Validate report that drove stage materialization, `target_platform` and
`artifact_extensions[]` record the platform strategy and loadable-artifact suffix set used for staged
artifacts and operation audits, `native_plugin_root` records the source package root used during
materialization, and the `native_dynamic_packages[]` table is the stage's own selected-package
evidence and is later reconciled with Validate handoff and materialized package ids; `package_count`
is likewise required and then compared with the current `materialized_packages[]` length. The
build/sign/notarization audit objects may still report `enabled = false` with
`package_count = 0`; the release gate requires the audit boundary itself to be present, while
package-id closure is enforced only for enabled/non-empty audit package tables.
Inside the required `native_build_plan` object, successful reports must include the plan header
fields `workspace_manifest`, `target_dir`, `cargo_profile`, `release`, `build_features`,
`package_count`, `diagnostics`, `packages`, and `fatal`; a half-written plan object is fatal even
when its package rows are otherwise well typed.
Inside the required `native_build_execution` object, successful reports must include `enabled`,
`fatal`, `diagnostics`, `package_count`, and `packages`; disabled execution is represented by an
explicit empty audit table, not by omitting the audit header.
Inside the required `native_signing` and `native_notarization` objects, successful reports must
also include `diagnostics` and `packages`; disabled operation audits may use an empty package table,
but the stage evidence boundary itself must stay explicit.
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
array of plugin tables with non-empty string `id` fields;
`test_report_stage_rejects_native_dynamic_loader_manifest_package_mismatch` then requires the
resulting `[[plugins]].id` sequence to match the package ids in `materialized_packages[]`, and
`test_report_stage_rejects_native_dynamic_malformed_loader_manifest` marks the NativeDynamic stage
fatal when `native_plugins.toml` is no longer valid TOML. This keeps a hash-updated but semantically
stale load manifest from telling runtime loaders to open a different package set than the payload
summary publishes.
When loader manifest rows carry the full runtime load-entry fields, final Report also compares the
current TOML row against the accepted package export row. The preferred source is the NativeDynamic
stage report's required `package_exports[]`; the Validate
`plan_summary.native_dynamic_package_exports[]` table remains the plan handoff used for cross-stage
drift diagnostics. The comparison covers `path`,
`manifest`, `package_report`, `abi.abi_version`, and each ABI v3 string contract field that appears
in both places. `test_report_stage_rejects_native_dynamic_loader_manifest_path_mismatch` and
`test_report_stage_rejects_native_dynamic_loader_manifest_abi_mismatch` keep a hash-updated
`native_plugins.toml` from redirecting runtime load paths or ABI descriptor contracts while the
stage report and Validate handoff still publish the expected package export table.
NativeDynamic stage final Report also opens each current materialized package's
`native_dynamic_package.toml` before the payload is considered releasable. The stage gate requires
the package report path to be the package-local generated file, then reuses the shared package-report
content diagnostics for `format_version`, `package_id`, `directory`, `path`, `manifest`, `[abi]`, and
`[payload]` so stage-local evidence is rejected before PlatformBundle copies it. The focused
`test_report_stage_rejects_native_dynamic_package_report_id_mismatch` and
`test_report_stage_rejects_native_dynamic_package_report_payload_hash_mismatch` cover reports whose
outer `file_manifest` and `content_hash` were recomputed after the package TOML drifted, preventing a
hash-updated NativeDynamic stage from publishing a forged package identity or stale package-local
payload digest.
`pipeline_report_native_dynamic_stage_payload.py` then recomputes the current stage `plugins/`
file manifest and content hash before final Report trusts the NativeDynamic stage wrapper.
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
`test_report_stage_rejects_native_dynamic_build_execution_package_mismatch` applies the same
package-id closure to `native_build_execution.packages[]`; executed build/copy evidence must describe
the packages that were actually materialized.
`test_report_stage_rejects_native_dynamic_signing_package_mismatch` and
`test_report_stage_rejects_native_dynamic_notarization_package_mismatch` apply the same package-id
closure to operation audits. Signing and notarization reports cannot name a different package set
from the staged NativeDynamic payload.
The operation-audit closure also validates package-level artifact evidence: signing/notarization
`packages[].artifacts[].package_relative_artifact` must match the staged package's current
`materialized_packages[].loadable_artifacts` after stripping that package's `plugins/<package>/`
prefix, and each audit `artifact_count` must match its `artifacts[]` length.
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
successful shape.
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
python -m zircon_export --profile native-dynamic-fixture-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-smoke-0615\out --stage native_dynamic --native-dynamic-build --offline --pretty
```

It returned `fatal=false`, built one cdylib package with Cargo exit code `0`, and staged the
fixture `.dll` plus `.pdb` sidecar into the NativeDynamic payload.

The ABI v2 fixture feature smoke used the same package with the `abi_v2_only` Cargo feature:

```powershell
python -m zircon_export --profile native-dynamic-fixture-v2-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-v2-smoke-0615\out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
```

It returned `fatal=false`, wrote `native_build_plan.build_features = ["abi_v2_only"]`, built the
cdylib with Cargo exit code `0`, and staged the fixture `.dll` plus `.pdb` sidecar. This proves the
feature-matrix hook and ABI v2 fixture build path, not runtime loading of the fallback ABI.

The same checked-in fixture also has release-mode stage smokes:

```powershell
python -m zircon_export --profile native-dynamic-fixture-release-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-release-smoke-0615\out --stage native_dynamic --native-dynamic-build --offline --pretty
python -m zircon_export --profile native-dynamic-fixture-release-v2-smoke --repo-root E:\Git\ZirconEngine --out D:\zircon-native-dynamic-real-fixture-release-v2-smoke-0615\out --stage native_dynamic --native-dynamic-build --native-dynamic-build-feature abi_v2_only --offline --pretty
```

Both returned `fatal=false`, wrote `cargo_profile = "release"`, completed Cargo with exit code `0`,
and staged the fixture `.dll` plus `.pdb` sidecar. Taken together, the local Windows fixture matrix
now covers debug/release and default/`abi_v2_only` build variants. Linux/macOS cross-platform
fixture execution and runtime load/startup evidence remain pending.

The M1 testing stage passed the validator binary check and a real CLI Validate run:

```powershell
cargo check -p zircon_runtime --bin zircon_export_validate --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
python -m zircon_export --profile windows-release --project D:\zircon-export-m1-smoke\project\zircon-project.toml --out D:\zircon-export-m1-smoke\run-valid --stage validate --offline --target-dir D:\cargo-targets\zircon-export-m1-validate-0614
```

The real smoke wrote `<out>/stages/validate/report.json`, returned non-fatal JSON, and confirmed the
report contained selected linked crates such as `zircon_plugin_rendering_runtime` and
`zircon_plugin_net_http_runtime`. Focused runtime lib-test execution is still blocked by unrelated
UI test compile drift in `table_pointer_routes.rs`, so no `cargo test -p zircon_runtime --lib`
pass is claimed yet.

The M2 Pack smoke used:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-m2-smoke --stage pack --asset-manifest D:\zircon-export-m2-smoke\assets\assets.json --determinism-check --offline --target-dir D:\cargo-targets\zircon-export-m1-validate-0614 --pretty
```

It wrote `<out>/stages/pack/assets.zrpack` and `<out>/stages/pack/report.json`. The report returned
`fatal=false`, included `scenes/main.zscene` and `textures/hero.png`, trimmed
`textures/unused.png`, and set `deterministic_double_run=true`.

The CookAssets project fallback smoke used:

```powershell
python -m zircon_export --profile windows-release --project D:\zircon-export-cook-project-smoke\project\zircon-project.toml --out D:\zircon-export-cook-project-smoke\out --stage cook_assets --asset-filter shipping --pretty
```

It returned `fatal=false`, `generated_from_project=true`, and one staged root
`scenes/main.scene.toml`. The staged manifest stores the resolved source path under the project
`assets/` directory and labels the entry `shipping` because the temporary fallback received
`--asset-filter shipping`.

The Pack profile smoke used:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-pack-profile-smoke\out --stage pack --asset-manifest D:\zircon-export-pack-profile-smoke\assets\assets.json --target-dir D:\cargo-targets\zircon-export-pack-profile-0615 --offline --pretty
```

It returned `fatal=false`, wrote `profile=windows-release` in the Pack report, and packed one asset
into one chunk through the real `zircon_export_pack` binary.

The Pack missing-manifest smoke used the default CookAssets handoff path without creating
`<out>/stages/cook_assets/assets.json`:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-pack-missing-manifest-smoke --stage pack --pretty
```

It returned exit code `2`, wrote `<out>/stages/pack/report.json`, and recorded `fatal=true`,
`profile=windows-release`, and a diagnostic telling the caller to run CookAssets first or pass
`--asset-manifest`.

The M3-T1 template smoke used a placeholder pack and the checked-in template package:

```powershell
python -m zircon_export --profile windows-release --out D:\zircon-export-m3-template-smoke --stage platform_bundle --pack-file D:\zircon-export-m3-template-smoke\inputs\assets.zrpack --template-dir E:\Git\ZirconEngine\export-templates\windows-x86_64-library_embed-debug --target-platform windows-x86_64
```

It returned `fatal=false`, copied the template-declared host placeholder and pack into
`bundle/windows-release`, and wrote a report containing the validated template manifest, file hash,
and computed aggregate `content_hash`. A second smoke with `format_version = 999` returned exit code
`2`, recorded `template format_version 999 is not supported; expected 1`, and skipped bundle copy.

The M3-T2 template-root smoke used the checked-in template repository:

```powershell
python -m zircon_export --profile linux-release --out D:\zircon-export-template-root-smoke --stage platform_bundle --pack-file D:\zircon-export-template-root-smoke\inputs\assets.zrpack --template-root E:\Git\ZirconEngine\export-templates --target-platform linux-x86_64
```

It returned `fatal=false`, resolved `linux-x86_64-library_embed-debug`, wrote
`template_resolution`, and materialized `bundle/linux-release/ZirconRuntime`,
`bundle/linux-release/data/assets.zrpack`, and `bundle/linux-release/zircon-export.json`.
