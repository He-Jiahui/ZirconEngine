---
related_code:
  - zircon_plugins/editor_build_export_desktop/plugin.toml
  - zircon_plugins/editor_build_export_desktop/editor/Cargo.toml
  - zircon_plugins/editor_build_export_desktop/editor/panel.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/source_template_report.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/library_embed_report.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/native_dynamic_report.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/export_profile_drawer.zui
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/export_wizard.rs
  - zircon_plugins/editor_build_export_desktop/templates/desktop_export_profile.toml
  - zircon_plugins/Cargo.toml
  - tools/zircon_export/cli.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/tests/test_pipeline_resume_flow.py
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/library_embed_compile_plan.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/tests/editor_authoring_extension_descriptors.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/options.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/stage.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/command.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/progress.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_output_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_report_body_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/streaming_output_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/cancellation_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_handoff_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_launch_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_report_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_host_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_host_projection_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session_control_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/support.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/view_model.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/tests.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/tests.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/tests.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs
implementation_files:
  - zircon_plugins/editor_build_export_desktop/plugin.toml
  - zircon_plugins/editor_build_export_desktop/editor/Cargo.toml
  - zircon_plugins/editor_build_export_desktop/editor/panel.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/source_template_report.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/library_embed_report.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/native_dynamic_report.v2.ui.toml
  - zircon_plugins/editor_build_export_desktop/editor/export_profile_drawer.zui
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/export_wizard.rs
  - zircon_plugins/editor_build_export_desktop/templates/desktop_export_profile.toml
  - zircon_plugins/Cargo.toml
  - tools/zircon_export/cli.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/tests/test_pipeline_resume_flow.py
  - zircon_runtime/src/plugin/export_build_plan/export_validate_report.rs
  - zircon_runtime/src/plugin/export_build_plan/library_embed_compile_plan.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/tests/editor_authoring_extension_descriptors.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/options.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/stage.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/command.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/progress.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_output_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_report_body_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/streaming_output_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/cancellation_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_handoff_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_launch_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_report_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_host_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_host_projection_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session_control_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/support.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/view_model.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/tests.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/tests.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/tests.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs
plan_sources:
  - user: 2026-05-02 continue independent plugin gap implementation
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/10-editor-integration.md
tests:
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - tools/zircon_export/tests/test_pipeline_resume_flow.py
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/support.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/view_model.rs
  - export_wizard_descriptor_covers_build_layout_stages_and_reports
  - report view summary_entry_keys covered by export_wizard_descriptor_covers_build_layout_stages_and_reports
  - report view template_control_ids covered by export_wizard_descriptor_covers_build_layout_stages_and_reports
  - report view template_document covered by export_wizard_descriptor_covers_build_layout_stages_and_reports
  - NativeDynamic shared stage/required_stage covered by export_wizard_descriptor_covers_build_layout_stages_and_reports
  - export_wizard_progress_parses_cli_stream_into_stage_snapshots
  - export_wizard_progress_marks_fatal_stage_reports
  - export_pipeline_stage_parser_accepts_cli_and_report_stage_names
  - NativeDynamic stage parser covered by export_pipeline_stage_parser_accepts_cli_and_report_stage_names
  - test_cli_stage_choices_match_shared_pipeline_order
  - export_wizard_pipeline_plan_selects_stages_from_packaging_strategies
  - export_wizard_pipeline_plan_builds_stage_commands_in_cli_order
  - export_wizard_pipeline_plan_threads_stage_artifact_inputs
  - NativeDynamic command/loader-manifest plan covered by export_wizard_pipeline_plan_builds_stage_commands_in_cli_order and export_wizard_pipeline_plan_threads_stage_artifact_inputs
  - export_wizard_pipeline_plan_reports_missing_execution_inputs
  - export_wizard_pipeline_banners_drive_progress_parser
  - export_wizard_stage_execution_feeds_stdout_into_progress
  - export_wizard_pipeline_execution_stops_on_missing_inputs_before_process_run
  - export_wizard_pipeline_execution_stops_on_plan_diagnostics_before_process_run
  - export_wizard_pipeline_execution_stops_on_process_failure
  - export_wizard_job_state_finishes_from_successful_pipeline_execution
  - export_wizard_job_state_exposes_plan_diagnostic_failure_without_starting
  - export_wizard_job_state_tracks_cancel_request_and_cancelled_terminal_state
  - export_wizard_job_runner_emits_successful_snapshot_events
  - export_wizard_job_runner_stops_after_fatal_stage_event
  - export_wizard_job_runner_cancels_after_stage_boundary
  - export_wizard_job_controller_streams_events_and_finishes_worker
  - export_wizard_job_controller_handle_requests_stage_boundary_cancel
  - export_wizard_view_model_projects_plan_stage_rows_and_controls
  - export_wizard_view_model_reports_missing_inputs_before_start
  - export_wizard_view_model_drains_job_events_into_terminal_rows
  - export_wizard_panel_bindings_project_template_button_events
  - export_wizard_panel_session_rejects_unready_start_until_plan_regenerates
  - export_wizard_panel_session_dispatches_generate_plan_request
  - export_wizard_panel_session_rejects_generate_plan_call_without_options
  - export_wizard_panel_session_starts_polls_and_cancels_job
  - export_wizard_panel_session_poll_finishes_terminal_job
  - export_wizard_panel_session_start_updates_controls_before_worker_poll
  - export_wizard_panel_session_cancel_disables_cancel_before_terminal_poll
  - export_wizard_panel_template_state_projects_stage_stdout_and_stderr
  - export_wizard_panel_template_state_projects_pipeline_report_body_entry
  - report.export_plan.* ReportBody entries covered by export_wizard_panel_template_state_projects_pipeline_report_body_entry
  - report.native_plugins_payload.* ReportBody entries covered by export_wizard_panel_template_state_projects_pipeline_report_body_entry
  - export_wizard_job_runner_streams_stage_output_before_stage_finished
  - export_wizard_job_runner_cancels_during_active_stage_without_failing
  - export_wizard_compile_host_path_feeds_platform_bundle_host_input
  - NativeDynamic validate-report handoff covered by export_wizard_compile_host_path_feeds_platform_bundle_host_input
  - export_wizard_compile_host_path_respects_target_dir_override_and_build_mode
  - export_wizard_pipeline_commands_use_repo_root_as_working_dir
  - export_wizard_pipeline_commands_leave_working_dir_unset_without_repo_root
  - export_wizard_report_command_consumes_source_template_report
  - NativeDynamic report handoff covered by export_wizard_report_command_consumes_source_template_report
  - export_wizard_report_command_skips_unplanned_strategy_reports
  - export_wizard_panel_template_state_projects_template_slots
  - export_wizard_panel_template_state_reports_missing_inputs
  - export_wizard_panel_retained_projection_applies_controls_and_slot_entries
  - export_wizard_panel_retained_projection_disables_start_for_missing_inputs
  - export_wizard_panel_retained_projection_preserves_report_body_native_payload_entry
  - build_export_wizard_panel_nodes_project_retained_export_wizard_panel
  - build_export_actions_parse_execute_profile
  - build_export_wizard_surface_action_maps_panel_buttons_to_session_actions
  - desktop_export_wizard_sessions_project_view_model_after_generate_plan
  - desktop_export_wizard_sessions_start_refreshes_existing_plan_options
  - desktop_export_wizard_sessions_use_profile_strategies_for_stage_plan
  - export_wizard_default_host_executable_points_to_compile_host_output
  - export_wizard_engine_repo_root_contains_python_module_entrypoint
  - build_export_wizard_panel_nodes_respect_target_strategy_list
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 app-owned wizard session validation recovery: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 app-owned wizard session validation recovery: passed with existing warnings)
  - cargo test -p zircon_editor --lib build_export_wizard_session --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 app-owned wizard session focused tests: timed out after 904 seconds without target output; matching cargo/rustc leftovers stopped)
  - cargo test -p zircon_editor --lib export_wizard_panel_session_start_updates_controls_before_worker_poll --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture (2026-06-15 Start/Cancel control focused test: blocked before target test by unrelated RenderQualityProfile::with_history_resolve and notification_center.rs partial-move compile drift; matching cargo/rustc/rustdoc process audit clean)
  - cargo test -p zircon_editor --lib export_wizard_panel_template_state_projects_stage_stdout_and_stderr --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture (2026-06-15 stage stdout/stderr projection focused test: blocked before target test by unrelated RenderQualityProfile::with_history_resolve compile drift; matching cargo/rustc/rustdoc process audit clean)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 staged CompileHost host handoff: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 staged CompileHost host handoff: passed with existing warnings)
  - cargo test -p zircon_editor export_wizard_compile_host_path --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 -- --nocapture (2026-06-15 staged CompileHost host handoff focused tests: timed out twice during lib-test compilation without target output; matching cargo/rustc leftovers stopped)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 CLI working directory handoff: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 CLI working directory handoff: passed with existing warnings)
  - cargo test -p zircon_editor --lib export_wizard_pipeline_commands_use_repo_root_as_working_dir --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture (2026-06-15 CLI working directory focused test: blocked before target test by unrelated RenderQualityProfile::with_history_resolve compile drift)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 Report SourceTemplate handoff: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 Report SourceTemplate handoff: passed with existing warnings)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 Start latest-options plan refresh: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 Start latest-options plan refresh: passed with existing warnings)
  - cargo test -p zircon_editor --lib desktop_export_wizard_sessions_start_refreshes_existing_plan_options --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture (2026-06-15 Start latest-options focused test: timed out after 604 seconds without target output; matching target-dir process audit clean)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 ReportBody pipeline report projection: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 ReportBody pipeline report projection: passed with existing warnings)
  - cargo test -p zircon_editor --lib export_wizard_panel_template_state_projects_pipeline_report_body_entry --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture (2026-06-15 ReportBody pipeline report focused test: timed out after 604 seconds without target output; matching target-dir cargo/rustc leftovers stopped)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-streaming-0615 --message-format short --color never (2026-06-15 streaming StageOutput events: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-streaming-0615 --message-format short --color never (2026-06-15 streaming StageOutput events: passed with existing warnings)
  - cargo test -p zircon_editor --lib export_wizard_job_runner_streams_stage_output_before_stage_finished --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-streaming-0615 --message-format short --color never -- --exact --nocapture (2026-06-15 streaming StageOutput focused test: timed out after 904 seconds without target output; matching target-dir cargo/rustc leftovers stopped)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/mod.rs zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/cancellation_tests.rs (2026-06-15 in-stage cancellation: passed)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-cancel-0615 --message-format short --color never (2026-06-15 in-stage cancellation: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-cancel-0615 --message-format short --color never (2026-06-15 in-stage cancellation: passed with existing warnings)
  - cargo test -p zircon_editor --lib export_wizard_job_runner_cancels_during_active_stage_without_failing --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-cancel-0615 --message-format short --color never -- --exact --nocapture (2026-06-15 in-stage cancellation focused test: timed out after 908 seconds without target output; matching target-dir cargo/rustc/rustdoc process audit clean)
  - plugin_export_wizard_action_id_routes_to_build_export_dispatch
  - export_wizard_panel_route_prefers_action_over_binding
  - desktop_export_private_template_assets_match_registered_documents
  - authoring_registry_accepts_view_templates_but_keeps_drawers_component_only
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor -p zircon_plugin_sdk_examples_editor --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1 --locked --offline (passed)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor -p zircon_plugin_sdk_examples_editor --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics --message-format short --color never (timed out after 10 minutes without Rust diagnostics while compiling the shared editor host)
doc_type: module-detail
---

# Desktop Build Export Plugin

`editor_build_export_desktop` is an editor-only plugin package for the desktop
export authoring surface. It does not take ownership of export plan generation:
`EditorManager::{generate_export_plan, generate_native_aware_export_plan,
execute_export_build, execute_native_aware_export_build}` remain the host-owned
authority. The plugin contributes the editor surface that calls into that host
path. The host-owned native-aware export path now exits before native package
preparation when the resolved plan has fatal diagnostics, so blocked exports can
write diagnostics without staging NativeDynamic packages or invoking Cargo. When
the host does execute a native-aware build, it reuses the same native package
discovery report for plan completion and package preparation instead of scanning
the plugin directory twice.

## Contributions

The package manifest declares SDK/API version `0.1.0`, category `platform`,
`editor_host` target support, Windows/Linux/macOS platform support, capabilities
for the desktop export panel, diagnostics, and NativeDynamic report view, plus
SourceTemplate and LibraryEmbed as default packaging strategies.

The editor crate registers one `editor.build_export_desktop` view and
`Desktop Export Tools` drawer, a main panel UI template, SourceTemplate,
LibraryEmbed, and NativeDynamic report templates, menu-backed operations for
plan generation and each desktop packaging mode, an asset creation template and
asset editor for desktop export profiles, and a component drawer whose bindings
point at the export operations. The registered panel/report documents are now
backed by plugin-private `.v2.ui.toml` view templates, while the export profile
drawer is a plugin-private `.zui` component template and the default profile
document is a TOML template under the plugin `templates/` directory.

`export_wizard.rs` is the M6 data contract for the plugin panel itself. It references
`docs/ui-and-layout/ai-workbench-style/ai-build-export-layout.png`, declares the
Profiles/Pipeline/Report regions, lists the full `Validate -> SourceTemplate ->
NativeDynamic -> CompileHost -> CookAssets -> Pack -> PlatformBundle -> Report`
stage flow, standardizes every stage report path as `report.json`, exposes
Pending/Running/Passed/Fatal progress states, and maps SourceTemplate,
LibraryEmbed, and NativeDynamic report views to the registered `.v2.ui.toml`
templates. Each report view also declares the stable ReportBody summary entries
it expects: SourceTemplate and LibraryEmbed consume `report.pipeline_report` plus
`report.export_plan.*`, while NativeDynamic additionally consumes
`report.native_plugins_payload.*` bundle/count/hash/package-id rows. The report
view descriptors also carry the registered template document URI and list the
template control ids that must exist in that `.v2.ui.toml` document, including
root, summary, primary field, list/space, and diagnostics anchors for all three
report types. The plugin crate re-exports the host-owned wizard contract so external
consumers can continue to use the plugin package as the editor extension entry
point without creating a reverse dependency from `zircon_editor` into the plugin
crate.

`zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/`
is the M6 host-owned stream, command-plan, process-adapter, and job-state model.
`progress.rs` consumes stdout lines from the shared `python -m zircon_export` CLI
without owning process execution. A `zircon_export stage=<Stage>
profile=<Profile>` banner marks the stage as Running, recognized `key=value`
lines record report/artifact paths, and top-level JSON `"fatal": false/true`
report fields move the stage to Passed or Fatal. The parser accepts both CLI
spellings such as `source_template`/`native_dynamic` and report spellings such
as `SourceTemplate`/`NativeDynamic` for the shared eight-stage pipeline.

The wizard subtree keeps the editor-facing command contract folder-backed:
`options.rs` describes user/host inputs, `stage.rs` owns CLI and report stage
names, `command.rs` owns the per-stage command DTO, `plan.rs` projects those
inputs into `python -m zircon_export` stage invocations, and `execution.rs`
connects that plan to the progress parser through a replaceable
`ExportWizardCommandRunner`. The default `ProcessCommandRunner` is synchronous
and returns captured stdout/stderr; host code can substitute its own runner for
background execution and live UI event dispatch. Each stage command also carries
an optional `working_dir`: when `repo_root` is present, `plan.rs` uses the same
engine root for both `--repo-root` and the process working directory, and
`ProcessCommandRunner` applies it before spawning Python. The retained app
session fills this root from the editor crate's parent directory so
`python -m zircon_export` can resolve the repo-local package even when the
editor process was launched from a project directory. Stage and pipeline execution
reports carry argv, exit code, stdout/stderr, diagnostics, explicit `fatal`, and
the current `ExportWizardProgressState`. `job.rs` adds
`ExportWizardJobState/Status/Snapshot` so the retained host can expose
Pending/Running/Cancelling/Cancelled/Finished/Failed snapshots without binding
that state machine to the plugin crate. `run.rs` adds the synchronous host job
driver. It accepts a replaceable command runner, an `ExportWizardCancelSignal`,
and an event callback, then emits Created/Started/StageStarted/StageFinished and
terminal Finished/Failed/Cancelled snapshot events. `controller.rs` wraps that
runner in a host-owned background worker and exposes an
`ExportWizardJobController` with an event receiver, shared cancel handle, and
`finish()` join/final-snapshot handoff. This gives retained UI code a stable
event and cancellation boundary to bind to later without making the plugin crate
own threads, buttons, or UI routing. `view_model.rs` is the data-only retained UI
projection layer. `ExportWizardPanelViewModel` starts from the pipeline plan,
tracks latest job events, drains the controller receiver, and projects
start/cancel/close controls, stage rows, planned report/artifact paths, missing
inputs, diagnostics, and terminal snapshot state for the future `.zui` binding.
The plugin crate's template constants are the single source for the registered
resource URIs, so registration tests can map every UI/template descriptor back to
an on-disk asset before retained UI rendering is wired.
The same plan path is now strategy-aware. `ExportWizardPipelineOptions` can carry
the validated `ExportProfile.strategies`, and `export_pipeline_stages_for_strategies(...)`
builds the exact SourceTemplate, LibraryEmbed, and NativeDynamic stage closure
used by the editor command plan, progress snapshots, job snapshots, and
execution results. SourceTemplate-only profiles plan only Validate,
SourceTemplate, and Report; LibraryEmbed profiles plan Validate, CompileHost,
CookAssets, Pack, PlatformBundle, and Report; NativeDynamic profiles include the
NativeDynamic stage before the shared build/package stages. The Report command
consumes only the reports for stages in that closure, so the editor no longer
asks the final Report step for SourceTemplate or NativeDynamic reports when the
profile did not choose those strategies.
The Python CLI exposes the same stage universe in the same order through
`STAGES` and `RESUMABLE_STAGES`: `validate`, `source_template`,
`native_dynamic`, `compile_host`, `cook_assets`, `pack`, `platform_bundle`, and
`report`. `pipeline_stages.py` still derives the strategy-specific execution
closure from the Validate report, but the public `--stage` choices, resume
choices, editor wizard rows, and final Report `required_stages` now share one
visible ordering.
`session.rs` adds the retained UI action/session boundary above the view model:
it owns the stable Desktop Export button control IDs, binding IDs, custom
`DesktopExportWizard` call payload, and `GeneratePlan`/`Start`/`Cancel` action
mapping. It also provides `ExportWizardPanelSession`, which can regenerate a
plan from `ExportWizardPipelineOptions`, start the background
`ExportWizardJobController`, poll job events into `ExportWizardPanelViewModel`,
request cancellation, and join the worker into a final snapshot. The private
`panel.v2.ui.toml` template now attaches Click events to Generate Plan, Start,
and Cancel, so retained projection can resolve these buttons through
`register_export_wizard_panel_bindings(...)`.
`register_export_wizard_panel_template(...)` is the standard host installation
entry for the private panel asset: it registers the fixed
`editor_build_export_desktop.panel` document id and the three DesktopExportWizard
button bindings in one call. `project_export_wizard_panel(...)` then projects
that same document id, so tests and future retained UI hosts no longer need to
duplicate the template id or the binding installation sequence. The plugin crate
reuses the host constant for `EXPORT_TEMPLATE_ID` and re-exports both helper
functions. `ExportWizardPanelRequest` and `ExportWizardPanelUpdate` are the
host-owned action dispatch DTOs above that installation path: GeneratePlan must
carry a fresh job id and `ExportWizardPipelineOptions`, while Start, Cancel, and
Poll can be driven directly by `ExportWizardPanelSession::handle_request(...)`.
The update returns the action, drained event count, active job id, latest event
kind, and current snapshot, so retained hosts can refresh the panel without
owning the controller lifecycle. `handle_action_call(...)` accepts parsed
`DesktopExportWizard` binding calls for simple actions and deliberately rejects
GeneratePlan without explicit options, keeping project/profile form extraction
outside the button binding payload.
`poll_events_and_finish_terminal(...)` completes the retained host polling
contract: when a Poll or Cancel request drains a Finished/Failed/Cancelled
snapshot, the session joins the worker through the existing `finish_job()` path,
clears the active controller, and returns an update with no active job id. This
keeps frame-by-frame UI code from needing a separate terminal cleanup command.
`panel_projection.rs` is the final data-only projection step before native
retained rendering. `export_wizard_panel_template_state(...)` converts the view
model into five stable template slots: missing inputs, stage rows, terminal
output, artifact paths, and report body. Each slot entry carries a stable key,
display label, detail text, optional stage, and severity so the `.v2.ui.toml`
panel can bind to predictable data without knowing how jobs, controllers, or
pipeline plans are mutated. Terminal output entries now include per-stage
stdout/stderr captured in `ExportWizardStageExecution`, using stable
`stage-output.<stage>.<stream>.<index>` keys so the report area can show real
CLI output in addition to diagnostics. The report body also carries a stable
`report.pipeline_report` entry: before execution it points at the planned
`<out>/report.json`, and after the Report stage emits `pipeline_report=...` it
prefers that runtime path while mapping severity from the Report stage state.
When the Report stage stdout also includes the final pipeline report JSON,
`panel_projection.rs` reads its top-level `export_plan` summary and adds stable
`report.export_plan.strategies`, `report.export_plan.required_stages`,
`report.export_plan.completed_stages`, and
`report.export_plan.unsupported_strategies` entries. Empty unsupported strategy
sets are rendered as `none` with success severity; non-empty sets are danger
severity so users can see unsupported export paths from the retained report
body without opening the raw JSON. The JSON extraction tracks object braces only
outside strings, so braces inside diagnostics or other report text do not
truncate the parsed report body.
When the final Report JSON contains a top-level `native_plugins_payload`, the
same projection adds `report.native_plugins_payload.bundle_path`,
`report.native_plugins_payload.package_count`,
`report.native_plugins_payload.file_count`,
`report.native_plugins_payload.content_hash`, and
`report.native_plugins_payload.package_ids` rows. These rows expose the
PlatformBundle-projected NativeDynamic payload summary without making the
editor parse nested stage evidence; absent payloads or missing fields simply do
not add rows.
The plugin report view descriptors publish those same stable summary keys
through `summary_entry_keys`, so template metadata, retained host projection, and
plugin contract tests can agree on the ReportBody rows without re-parsing the
template documents.
Those descriptors also publish `template_document`, letting host code and tests
resolve the exact registered report template without keeping a separate
template-id lookup table.
The same template state also carries
`ExportWizardPanelControlBindingState` entries for
`DesktopExportGeneratePlanButton`, `DesktopExportStartButton`, and
`DesktopExportCancelButton`. Their enabled flags are projected from the view
model control state, keeping retained rendering code on stable button ids
instead of duplicating session lifecycle decisions.
`panel_host_projection.rs` is the first retained-host materialization for that
template state. `export_wizard_panel_retained_projection(...)` installs the
standard panel projection, builds and lays out the shared `panel.v2.ui.toml`
surface, converts it into `RetainedUiHostProjection`, applies Generate
Plan/Start/Cancel disabled state, and appends synthetic label nodes below the
five slot anchors. `apply_export_wizard_panel_template_state(...)` is split out
so future frame polling can refresh an existing projection without reowning the
session or command-runner lifecycle.
`retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs` is the
first production BuildExport pane adapter for that projection. When a workbench
pane carries a `BuildExportV1` presentation, `build_export.rs` now prefers the
plugin panel projection, registers the private panel template in a cached
`EditorUiHostRuntime`, builds a read-only `ExportWizardPanelViewModel` from the
native BuildExport target rows, converts `RetainedUiHostProjection` into the
existing `TemplatePaneNodeData` host contract, preserves authored
`DesktopExportWizard/...` binding ids, and maps Generate Plan/Start/Cancel
buttons onto standardized `workbench.build_export.plan/execute/cancel.<profile>`
action ids. `template_activation_semantics.rs` now routes
`export_wizard_panel` primary activation through those action ids, and
`pane_surface_actions.rs` accepts parsed BuildExport action ids in addition to
the legacy `BuildExportAction` control id, so retained button clicks enter the
BuildExport dispatch path. `retained_host/app/build_export_wizard_session.rs`
now distinguishes the wizard panel controls from legacy BuildExport row
buttons: Generate Plan and Start build profile-specific
`ExportWizardPipelineOptions` including the selected `ExportProfile.strategies`,
dispatch into `ExportWizardPanelSession`, and
store the resulting `ExportWizardPanelViewModel` by profile. Start deliberately
refreshes the existing profile plan with the latest options before launching the
worker, so changing the output root or derived artifact paths after Generate
Plan cannot execute a stale command plan. Cancel and frame Poll requests go back
through the same session so controller cleanup and event draining stay
host-owned. `host_lifecycle.rs` initializes this app state,
polls it from the retained host tick, and writes the first target's session view
model into `BuildExportPaneViewData.wizard_view_model`. The BuildExport panel
adapter now prefers that live view model and only falls back to the synthetic
dry-run view model when no session exists. That synthetic fallback parses the
target row strategy text as SourceTemplate, LibraryEmbed, and NativeDynamic
labels before building a plan, so first-frame browser or library-only export
panes use the same stage list as a live retained session. Legacy target-row
actions continue to use the older queued export path. If the wizard projection
cannot be built, the existing legacy target-row projection remains the fallback.
The follow-up
Start/Cancel control-state slice keeps this path responsive before worker events
return: `ExportWizardPanelViewModel` tracks an explicit `active_job` flag,
`ExportWizardPanelSession` marks a job started immediately after spawning the
controller, marks cancel requested immediately after forwarding the cancel
signal, and marks the job finished after worker join. That means retained panel
buttons project the same-frame states users expect: Start disables as soon as
execution starts, Cancel enables while the worker is active, Cancel disables as
soon as cancellation is requested, and Close stays disabled until the active job
is finished or cancelled. The latest scoped checks pass, but this path still
does not claim successful end-to-end CLI execution from the native retained
window.

The staged host handoff now mirrors the CLI's real CompileHost output layout.
`LibraryEmbedCompileHostPlan` exposes the target-mode binary and build-mode
cargo profile rules used by the runtime build plan, and wizard `plan.rs`
projects those rules into `<out>/stages/compile_host/target/<cargo-profile>/<binary>[.exe]`
or an explicit target-dir override. `retained_host/app/build_export_wizard_session.rs`
uses that helper when creating profile-specific `ExportWizardPipelineOptions`,
so PlatformBundle no longer defaults to the old placeholder
`host/zircon_game.exe` path. The retained app still does not prove the host file
exists; CompileHost owns producing it, and PlatformBundle owns copying it.

The Report stage handoff now matches the Python CLI aggregator. The editor
command plan lists Validate, SourceTemplate, NativeDynamic, CompileHost,
CookAssets, Pack, and PlatformBundle report files as consumed artifacts for the
final Report command. `pipeline_report.py` still treats optional strategy stages
according to the validated profile, but the editor panel can now display the
planned SourceTemplate and NativeDynamic report paths instead of omitting those
stages from the final aggregation inputs.

## Boundary

The plugin intentionally contributes descriptors, menu operations, and the
wizard/report-view data contract. The stdout-to-progress parser, pipeline
command plan, synchronous process-runner contract, and job snapshot state are
owned by `zircon_editor` host code and re-exported by the plugin crate. The
synchronous job runner owns host-side snapshot event sequencing and
phase-boundary cancellation checks, while the background controller owns only the
worker thread, receiver, cancel handle, and join handoff. The view model owns
only projection from plan/events into retained-UI-friendly data. The panel
projection owns only slot-shaped template state and severity mapping. The panel
session owns retained button action routing and controller lifecycle, but does
not own native retained rendering or per-frame painting. The retained app owns
profile-keyed session storage and frame polling, and the pane DTO carries only
the current view-model snapshot. The actual build plan, materialization, native
package preparation, raw incremental stream rendering, and diagnostics file
policy stay in
`zircon_editor`/`zircon_runtime` host code so runtime export remains
deterministic and independent of editor plugin state.

`EditorExtensionRegistry` now distinguishes UI template documents from component
drawer documents. UI templates may point at `.zui` components or `.v2.ui.toml`
view templates, which matches the retained host asset loader split. Component
drawers remain `.zui`-only because the drawer surface mounts one component asset
inside an inspector host. The export plugin uses that split directly: panel and
report surfaces are view templates, and `export_profile_drawer.zui` is the only
component drawer document.

## Validation

2026-06-22 export wizard test owner-tree split:
the oversized root test owner was deleted and replaced with
`wizard/tests/{mod,support,pipeline_plan,pipeline_execution,job,panel_session,view_model}.rs`.
The same slice fixed cancellation classification so in-stage cancellation is
reported separately from phase-boundary cancellation, and registered the v2 panel
template with its `editor_base.v2.ui.toml` import source. `cargo fmt -p
zircon_editor --check`, `audit_editor_structure.py --json`, old-file existence
checks, line-count sampling, and scoped `git diff --check` passed. Focused
`cargo test -p zircon_editor --lib export_wizard --locked --target-dir
E:\cargo-targets\zircon-editor-export-wizard-0622 --message-format short --color
never -- --test-threads=1` previously stopped before editor tests on active
runtime dirty state: `GpuMeshResource::indirect_order_signature` is private
during `zircon_runtime` compilation. The latest rerun timed out after 304 seconds
without diagnostics; matching cargo/rustc leftovers were stopped, so no focused
export_wizard pass is claimed.

2026-06-18 CLI/editor shared stage order validation:
`tools/zircon_export/cli.py` now exposes `STAGES` in the shared pipeline order:
`validate`, `source_template`, `native_dynamic`, `compile_host`, `cook_assets`,
`pack`, `platform_bundle`, and `report`. This matches `RESUMABLE_STAGES`, the
editor wizard `export_pipeline_stages()` order, and the strategy-specific
`pipeline_execution_stage_keys(...)` closure used by final Report. The new
`test_cli_stage_choices_match_shared_pipeline_order` first failed against the
old `compile_host`-before-SourceTemplate order, then passed after the CLI
constant was updated. Validation passed: `python -m py_compile tools\zircon_export\cli.py tools\zircon_export\tests\test_pipeline_resume_flow.py`,
`python -m unittest tools.zircon_export.tests.test_pipeline_resume_flow` (31
tests), `python -m unittest tools.zircon_export.tests.test_pipeline_report_stage`
(28 tests), and full `python -m unittest discover tools.zircon_export.tests`
(652 tests).

2026-06-18 strategy-aware stage plan validation:
`ExportWizardPipelineOptions` now carries optional packaging strategies, and
`export_pipeline_stages_for_strategies(...)` mirrors the CLI strategy closure
for SourceTemplate, LibraryEmbed, NativeDynamic, and combined profiles. `plan.rs`
uses that closure for command generation and final Report consumed artifacts,
while `job.rs`, `run.rs`, and `execution.rs` initialize progress from the same
planned stage list. The retained app passes `ExportProfile.strategies` into
session options, and the synthetic BuildExport pane parses target strategy text
before projecting first-frame stage rows. Focused coverage was added in
`export_wizard_pipeline_plan_selects_stages_from_packaging_strategies`,
`export_wizard_report_command_skips_unplanned_strategy_reports`,
`desktop_export_wizard_sessions_use_profile_strategies_for_stage_plan`, and
`build_export_wizard_panel_nodes_respect_target_strategy_list`. Static validation
passed: `rustfmt --edition 2021 --check`, conflict-marker scan, and
`git diff --check` with only LF/CRLF warnings. Scoped
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-strategy-stage-0618 --message-format short --color never`
timed out after about 184 seconds without Rust diagnostics, and no matching
cargo/rustc/rustdoc processes remained afterward, so no Cargo pass is claimed for
this slice.

The plugin crate test checks that registration produces the panel view,
operations, report templates, menu entries, asset profile template, and component
drawer. `export_wizard_descriptor_covers_build_layout_stages_and_reports`
checks the M6 layout reference, stage flow, report paths, progress states,
report-view summary entry keys, report-template document registration,
report-template control ids against the real `.v2.ui.toml` assets, and
NativeDynamic report mapping. The host wizard
stream-model tests cover CLI banner
parsing, artifact/report path capture, Passed/Fatal report-state transitions, and
`source_template`/`SourceTemplate` stage-name normalization. The host wizard
pipeline tests cover stage command ordering, artifact handoff paths,
missing-input diagnostics, banner compatibility with `ExportWizardProgressState`,
stdout-to-progress execution handoff, missing-input process suppression, global
plan diagnostic short-circuiting, and process-failure fatal reporting. The job
state tests cover successful pipeline completion, plan-diagnostic failure before
process start, and cancel-request/cancelled terminal snapshots. The job-runner
tests cover successful event sequencing, fatal-stage stop behavior, and
phase-boundary cancellation. The background-controller tests cover event receiver
delivery, worker join/final snapshot handoff, and handle-driven phase-boundary
cancellation. The view-model tests cover plan-to-control projection, missing
input surfacing before start, and draining job events into terminal stage rows.
The panel-session tests cover retained template button binding projection,
unready plan rejection/regeneration, explicit GeneratePlan request dispatch,
GeneratePlan call rejection when options are missing, background controller
start, polling, cancel request, final worker join, and Poll-driven terminal
controller cleanup. The split-out `session_control_tests.rs` tests cover
same-frame Start and Cancel control projection without adding more cases to the
pre-split oversized root test owner: Start immediately disables Start and enables
Cancel before the worker event is polled, while Cancel immediately moves the
snapshot into Cancelling and disables Cancel before the terminal event returns.
The panel-template-state tests cover the five retained template slots, stable
slot control ids, stage row projection, report body status entry, artifact path
handoff, missing-input warnings, per-stage stdout/stderr TerminalOutput entries,
final pipeline report ReportBody entry, and GeneratePlan/Start/Cancel control
enablement. The retained projection tests cover building the shared panel
surface, applying button disabled state, appending stage rows under their slot
anchor, projecting missing-input rows under their slot anchor, and preserving
ReportBody native payload entry metadata on retained nodes. The
BuildExport pane projection test covers the production adapter shape: wizard
panel root projection, removal of legacy `BuildExportRow.*` nodes on the new
path, Start/Cancel enablement, button binding/action metadata, and synthetic
stage rows under `DesktopExportStageRows`.
The retained app session tests cover profile-action routing, GeneratePlan view
model storage, CompileHost default host path derivation, engine repo-root
discovery for `python -m zircon_export`, and Start-time regeneration of an
existing profile plan with the latest options before the worker starts.
`desktop_export_private_template_assets_match_registered_documents` checks that
the registered panel/report/profile drawer/default profile document URIs point to
plugin-private files and that the view/component templates parse through the
existing UI v2 asset loaders. `authoring_registry_accepts_view_templates_but_keeps_drawers_component_only`
locks the registry-level rule that UI templates can use `.v2.ui.toml` while
component drawers stay `.zui`-only.
The editor catalog consistency test covers the
workspace member, `plugin.toml`, and builtin catalog entry alignment. Current
2026-06-14 M6 follow-up validation passed Python `tomllib` parsing for the new
template assets, rustfmt, final `rustfmt --edition 2021 --check`,
`git diff --check` with LF/CRLF warnings only, conflict-marker and
trailing-whitespace scans, and the scoped
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614`
with existing warnings, then the scoped
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614`
with existing warnings after the template-asset slice. The focused plugin test
`cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --lib desktop_export_private_template_assets_match_registered_documents --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614 --message-format short --color never -- --test-threads=1 --nocapture`
timed out after 604 seconds while compiling and emitted no Rust diagnostics;
target-dir cargo/rustc leftovers were stopped. The view-model-focused
`cargo test -p zircon_editor --lib export_wizard_view_model --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614 --no-run`
timed out after 604 seconds with no Rust diagnostics; target-dir-specific
cargo/rustc leftovers were stopped. An earlier cold plugin check timed out after
244 seconds with no Rust diagnostics. A controller-focused
`cargo test -p zircon_editor --lib export_wizard_job_controller --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614 --no-run`
attempt exited with code `1073807364` after 312 seconds during dependency
compilation and no Rust diagnostics. Focused `cargo test ... export_wizard ...`
previously timed out after 604 seconds, focused `cargo test ... export_wizard_ ...`
timed out after 244 seconds before reporting target test results, and the narrower
`cargo test -p zircon_editor --lib export_wizard_job_state --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614 --no-run`
timed out after 304 seconds with no Rust diagnostics, so no focused test pass is
claimed yet.

2026-06-14 panel-session follow-up validation: targeted `rustfmt --check` over
the touched editor/plugin files passed, and Python `tomllib` confirmed
`panel.v2.ui.toml` contains Click events for
`DesktopExportWizard/GeneratePlan`, `DesktopExportWizard/Start`, and
`DesktopExportWizard/Cancel`. The later `zircon_editor` check was blocked by
`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs`
moving `option.id`/`option.label` before reading option flags; that shared
retained projection now computes `matched/selected/disabled/special/focused/hovered/pressed/loading`
before moving `id/label`, so the option metadata projection remains unchanged
while the E0382 partial move is removed. After that fix,
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614`
passed with existing warnings, and the scoped
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614`
also passed with existing warnings. Focused
`cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --lib desktop_export_private_template_assets_match_registered_documents ...`
and `cargo test -p zircon_editor --lib export_wizard_panel ...` both timed out
after 364 seconds during test binary compilation with no Rust diagnostics; the
target-dir cargo/rustc leftovers were stopped, and no retained panel focused
test pass is claimed yet.

2026-06-14 host panel installation validation: the projection test now uses
`register_export_wizard_panel_template(...)` and `project_export_wizard_panel(...)`
instead of hand-registering the private template and button bindings, and asserts
the projected document id is the shared `EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID`.
Focused test compilation for
`export_wizard_panel_bindings_project_template_button_events` timed out after
184 seconds with no Rust diagnostics and no target-dir leftovers. Formatting and
`git diff --check` passed for the touched host/plugin files. A longer
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-0614`
rerun passed with existing warnings. A later request/update dispatch validation
window also passed `cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`,
both with existing warnings. Focused
`cargo test -p zircon_editor --lib export_wizard_panel_session --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --test-threads=1 --nocapture`
timed out after 904 seconds during lib-test compilation without Rust diagnostics;
matching target-dir cargo/rustc leftovers were stopped, and no panel-session
focused test pass is claimed for the request/update dispatch slice.

2026-06-14 panel template slot projection validation: `panel_projection.rs`
adds the host-owned slot state consumed by the retained export panel, and the
new tests are `export_wizard_panel_template_state_projects_template_slots` and
`export_wizard_panel_template_state_reports_missing_inputs`. `rustfmt --edition
2021 --check` passed for the touched editor/plugin files. Both scoped checks
passed with existing warnings under
`D:\cargo-targets\zircon-export-m6-editor-dispatch-0614`:
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused
`cargo test -p zircon_editor --lib export_wizard_panel_template_state --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --test-threads=1 --nocapture`
stopped before running the target tests on unrelated editor compile drift:
`zircon_editor/src/tests/editing/state.rs` calls
`RenderQualityProfile::with_history_resolve`, which is not present in the
current runtime render profile API. No focused panel-template-state test pass is
claimed.

2026-06-14 panel Poll terminal cleanup validation:
`ExportWizardPanelSession::poll_events_and_finish_terminal(...)` now joins and
clears the worker after Poll or Cancel drains a terminal snapshot. The new test
is `export_wizard_panel_session_poll_finishes_terminal_job`, which asserts that
a terminal Poll returns `active_job_id = None` and allows the next GeneratePlan
request to replace the plan. `rustfmt --edition 2021 --check` passed for
`session.rs` and `tests.rs`. Both scoped checks passed with existing warnings:
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused
`cargo test -p zircon_editor --lib export_wizard_panel_session_poll_finishes_terminal_job --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --test-threads=1 --nocapture`
timed out after 304 seconds during lib-test compilation without Rust
diagnostics or target test output; matching target-dir cargo/rustc leftovers
were stopped, so no focused panel-session Poll test pass is claimed.

2026-06-15 panel control binding projection validation:
`ExportWizardPanelTemplateState` now includes `control_bindings`, and
`ExportWizardPanelControlBindingState` exposes the enabled state for
GeneratePlan, Start, and Cancel button ids. The existing panel-template tests
were extended so `export_wizard_panel_template_state_projects_template_slots`
covers ready-plan button enablement and
`export_wizard_panel_template_state_reports_missing_inputs` covers missing-input
disablement. `rustfmt --edition 2021 --check` passed for the touched
editor/plugin files. Both scoped checks passed with existing warnings:
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
No separate focused test pass is claimed for this control-binding slice.

2026-06-15 retained panel host projection validation:
`panel_host_projection.rs` adds `export_wizard_panel_retained_projection(...)`
and `apply_export_wizard_panel_template_state(...)`, while
`panel_host_projection_tests.rs` adds
`export_wizard_panel_retained_projection_applies_controls_and_slot_entries` and
`export_wizard_panel_retained_projection_disables_start_for_missing_inputs`.
`rustfmt --edition 2021 --check` passed for the touched editor/plugin files.
Both scoped checks passed with existing warnings:
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused test compilation
`cargo test -p zircon_editor --lib export_wizard_panel_retained_projection --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --no-run --message-format short --color never`
stopped before target tests on unrelated editor compile drift:
`zircon_editor/src/tests/editing/state.rs` calls
`RenderQualityProfile::with_history_resolve`, which is not present in the
current runtime render profile API. No focused retained projection test pass is
claimed.

2026-06-15 BuildExport pane retained projection validation:
`build_export_wizard_panel.rs` adds the cached runtime/template registration,
read-only export wizard view model, and `RetainedUiHostProjection` to
`TemplatePaneNodeData` adapter. `build_export.rs` now prefers that adapter for
real `BuildExportV1` pane presentations and keeps the old target-row projection
as fallback. The new focused test is
`build_export_wizard_panel_nodes_project_retained_export_wizard_panel`.
`rustfmt --edition 2021` was applied to the touched pane conversion files. Both
scoped checks passed with existing warnings:
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused test compilation
`cargo test -p zircon_editor --lib build_export_wizard_panel_nodes_project_retained_export_wizard_panel --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --no-run --message-format short --color never`
first failed before editor tests because Cargo could not write runtime dep-info
inside the target directory (`os error 3`); the retry with
`CARGO_INCREMENTAL=0` timed out after 1204 seconds during compilation. Matching
target-dir cargo/rustc processes were stopped, and no focused pane projection
test pass is claimed.

2026-06-15 BuildExport retained button dispatch validation:
`build_export_wizard_panel.rs` now projects Generate Plan/Start/Cancel buttons
with `workbench.build_export.plan/execute/cancel.<profile>` action ids while
preserving their `DesktopExportWizard/...` binding ids.
`template_activation_semantics.rs` routes `export_wizard_panel` activation by
action id, `pane_surface_actions.rs` dispatches parsed BuildExport action ids,
and `build_export_actions.rs` accepts the new GeneratePlan action while its
tests are split into `build_export_actions/tests.rs`. New/updated coverage
includes `build_export_actions_parse_execute_profile`,
`plugin_export_wizard_action_id_routes_to_build_export_dispatch`,
`export_wizard_panel_route_prefers_action_over_binding`, and a tightened
`build_export_wizard_panel_nodes_project_retained_export_wizard_panel`
assertion for standardized wizard button action ids. `rustfmt --edition 2021`
was applied to the touched editor files. Scoped editor validation passed with
existing warnings:
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused test compilation
`cargo test -p zircon_editor --lib export_wizard_panel_route_prefers_action_over_binding --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --no-run --message-format short --color never`
stopped before target tests on unrelated editor compile drift:
`zircon_editor/src/tests/editing/state.rs` calls
`RenderQualityProfile::with_history_resolve`, which is not present in the
current runtime render profile API. No focused retained dispatch test pass is
claimed.

2026-06-15 BuildExport app-owned wizard session validation:
`build_export_wizard_session.rs` adds the retained app profile-session map,
wizard button action classifier, profile-specific pipeline options, dispatch
entry, and tick-time Poll bridge. `host_lifecycle.rs` initializes and polls that
state, and `BuildExportPaneViewData.wizard_view_model` lets
`build_export_wizard_panel.rs` prefer the live session view model over the
synthetic fallback. New coverage is
`build_export_wizard_surface_action_maps_panel_buttons_to_session_actions` and
`desktop_export_wizard_sessions_project_view_model_after_generate_plan`.
`rustfmt --edition 2021 --check` passed over the touched editor files.
Conflict-marker scan was clean, and `git diff --check` reported only LF/CRLF
notices for touched files. Scoped Cargo validation
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
did not reach the editor code: it stopped in unrelated current runtime UI
compile drift at `zircon_runtime/src/ui/surface/render/command_palette.rs:311`
with `E0282` type annotations needed. No focused session test pass is claimed.

2026-06-15 BuildExport app-owned wizard session validation recovery:
the runtime UI blocker was the local `CommandPalette` row collection using
untyped `collect()` branches. Adding the explicit `Vec<CommandPaletteRow>`
target kept row order/mutation semantics unchanged and allowed the app-owned
session slice to validate through both scoped checks. The rerun
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
passed with existing warnings, and the desktop export plugin scoped check
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
also passed with existing warnings. Focused
`cargo test -p zircon_editor --lib build_export_wizard_session --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
timed out after 904 seconds without target test output; matching target-dir
cargo/rustc leftovers were stopped, so no focused session test pass is claimed.

2026-06-15 Start/Cancel same-frame control validation:
`view_model.rs` now owns the `active_job` flag and exposes
`mark_job_started`, `mark_cancel_requested`, and `mark_job_finished`; `session.rs`
calls those hooks from Start, Cancel, and worker join paths. The new focused
tests are `export_wizard_panel_session_start_updates_controls_before_worker_poll`
and `export_wizard_panel_session_cancel_disables_cancel_before_terminal_poll` in
`session_control_tests.rs`. `rustfmt --edition 2021` was applied, and scoped
Cargo validation passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
The focused
`cargo test -p zircon_editor --lib export_wizard_panel_session_start_updates_controls_before_worker_poll --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture`
attempt stopped before the target test on unrelated compile drift:
`zircon_editor/src/tests/editing/state.rs` still calls the missing
`RenderQualityProfile::with_history_resolve`, and
`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center.rs`
has a partial-move borrow error. The follow-up process audit found no matching
cargo/rustc/rustdoc leftovers for this target directory. No focused Start/Cancel
test pass is claimed.

2026-06-15 stage stdout/stderr projection validation:
`view_model.rs` now carries per-stage `stdout_lines` and `stderr_lines` from
`ExportWizardStageExecution`, and `panel_projection.rs` projects those lines
into the retained `TerminalOutput` slot before diagnostics. The new focused
test is `export_wizard_panel_template_state_projects_stage_stdout_and_stderr`
in `panel_output_tests.rs`, split out because the pre-split root test owner was
already over 1000 lines. `rustfmt --edition 2021` was applied, and scoped Cargo validation
passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
The focused
`cargo test -p zircon_editor --lib export_wizard_panel_template_state_projects_stage_stdout_and_stderr --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture`
attempt stopped before the target test on unrelated compile drift:
`zircon_editor/src/tests/editing/state.rs` still calls the missing
`RenderQualityProfile::with_history_resolve`. The follow-up process audit found
no matching cargo/rustc/rustdoc leftovers for this target directory. No focused
stage-output test pass is claimed.

2026-06-15 staged CompileHost host handoff validation:
`library_embed_compile_plan.rs` now exposes the same binary/profile lookup used
to build `LibraryEmbedCompileHostPlan`, `wizard/plan.rs` adds
`export_wizard_compile_host_executable_path(...)`, and
`build_export_wizard_session.rs` uses that helper for profile-specific
`ExportWizardPipelineOptions.host_executable`. New focused coverage is split
into `pipeline_handoff_tests.rs`:
`export_wizard_compile_host_path_feeds_platform_bundle_host_input` and
`export_wizard_compile_host_path_respects_target_dir_override_and_build_mode`,
plus `export_wizard_default_host_executable_points_to_compile_host_output` in
the retained app session tests. `rustfmt --edition 2021` was applied, and scoped
Cargo validation passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused
`cargo test -p zircon_editor export_wizard_compile_host_path --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 -- --nocapture`
timed out twice during lib-test compilation without target output; matching
cargo/rustc leftovers were stopped, so no focused handoff test pass is claimed.

2026-06-15 CLI working directory handoff validation:
`ExportWizardPipelineStageCommand` now carries `working_dir`, `wizard/plan.rs`
sets it from `ExportWizardPipelineOptions.repo_root`, and `ProcessCommandRunner`
uses it as the child process current directory before executing
`python -m zircon_export`. `retained_host/app/build_export_wizard_session.rs`
fills `repo_root` from the engine repository root so the repo-local
`zircon_export` Python package is importable even when the editor host runs from
a project directory. New focused coverage lives in `pipeline_launch_tests.rs`
(`export_wizard_pipeline_commands_use_repo_root_as_working_dir` and
`export_wizard_pipeline_commands_leave_working_dir_unset_without_repo_root`) and
the retained app session test
`export_wizard_engine_repo_root_contains_python_module_entrypoint`.
`rustfmt --edition 2021 --check` passed for touched Rust files. Scoped Cargo
validation passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused
`cargo test -p zircon_editor --lib export_wizard_pipeline_commands_use_repo_root_as_working_dir --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture`
stopped before the target test on unrelated compile drift in
`zircon_editor/src/tests/editing/state.rs:553`
(`RenderQualityProfile::with_history_resolve` is absent from the active runtime
render profile API), so no focused CLI working-dir test pass is claimed.

2026-06-15 Report SourceTemplate handoff validation:
`wizard/plan.rs` now includes `<out>/stages/source_template/report.json` in the
Report command's consumed artifacts, matching the Python CLI report aggregator's
optional SourceTemplate stage. New focused coverage is
`export_wizard_report_command_consumes_source_template_report` in
`pipeline_report_tests.rs`, which keeps the final Report command at six stage
report inputs. `rustfmt --edition 2021 --check` passed for touched Rust files.
Scoped Cargo validation passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
The focused editor lib-test was not rerun separately because the same validation
round already confirmed the current lib-test compile blocker:
`RenderQualityProfile::with_history_resolve` is still missing in
`zircon_editor/src/tests/editing/state.rs:553`.

2026-06-15 Start latest-options plan refresh validation:
`build_export_wizard_session.rs` now routes GeneratePlan and Start through a
shared profile-plan regeneration helper. Start receives the retained app's
latest `ExportWizardPipelineOptions`, replaces any inactive existing profile
plan, and then launches through `ProcessCommandRunner`; tests can inject a
runner so this behavior is covered without invoking the real CLI. The new
focused test is
`desktop_export_wizard_sessions_start_refreshes_existing_plan_options`, which
first generates a plan for one output root and then starts with a second output
root, asserting that CookAssets consumes the second source asset manifest and
PlatformBundle consumes the second host path before the worker is joined.
`rustfmt --edition 2021 --check`, `git diff --check`, conflict-marker scan, and
trailing-whitespace scan passed for the touched files. Scoped Cargo validation
passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused
`cargo test -p zircon_editor --lib desktop_export_wizard_sessions_start_refreshes_existing_plan_options --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture`
timed out after 604 seconds without target output; the follow-up process audit
found no matching cargo/rustc/rustdoc leftovers for this target directory. No
focused retained app session test pass is claimed for this slice.

2026-06-15 ReportBody pipeline report projection validation:
`panel_projection.rs` now adds a stable `report.pipeline_report` entry to the
ReportBody slot. The entry prefers the Report stage runtime `pipeline_report=`
artifact path when stdout provides one, otherwise it falls back to the planned
pipeline report path from the command plan, and its severity follows the Report
stage progress state. The new focused coverage is
`export_wizard_panel_template_state_projects_pipeline_report_body_entry` in
`panel_report_body_tests.rs`, which checks both planned and runtime-overridden
paths. `rustfmt --edition 2021 --check`, `git diff --check`,
conflict-marker scan, and trailing-whitespace scan passed for the touched
files. Scoped Cargo validation passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never`.
Focused
`cargo test -p zircon_editor --lib export_wizard_panel_template_state_projects_pipeline_report_body_entry --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture`
timed out after 604 seconds without target output; matching target-dir
cargo/rustc leftovers were stopped, so no focused ReportBody test pass is
claimed.

2026-06-18 ReportBody export-plan summary projection validation:
`panel_projection.rs` now reads the final Report stage JSON from stdout and
projects `export_plan.strategies`, `required_stages`, `completed_stages`, and
`unsupported_strategies` into the ReportBody slot as stable
`report.export_plan.*` entries. The existing focused coverage
`export_wizard_panel_template_state_projects_pipeline_report_body_entry` now
checks the strategy/stage/unsupported-strategy rows and includes diagnostic text
with `{}` so object-depth parsing ignores braces inside JSON strings.
`rustfmt --edition 2021 --check`, conflict-marker scan, trailing-whitespace
scan, and `git diff --check` passed for the touched files with only LF/CRLF
warnings. Scoped
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-report-plan-ui-0618 --message-format short --color never`
timed out after about 308 seconds without Rust diagnostics; the follow-up
process audit found no matching cargo/rustc/rustdoc leftovers for that target
directory, so no scoped Cargo pass is claimed for this slice.

2026-06-18 ReportBody NativeDynamic payload summary projection validation:
`panel_projection.rs` now reads the final Report stage JSON top-level
`native_plugins_payload` object from stdout and projects its bundle path,
package count, file count, content hash, and materialized package ids into the
ReportBody slot as stable `report.native_plugins_payload.*` entries. The
existing focused coverage
`export_wizard_panel_template_state_projects_pipeline_report_body_entry` now
checks those rows alongside the pipeline report path and `export_plan` rows.
`rustfmt --edition 2021 --check`, conflict-marker scan, and trailing-whitespace
scan passed for the touched Rust files. `git diff --check` passed with only
LF/CRLF warnings. Scoped
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-native-payload-ui-0618 --message-format short --color never`
timed out after about 364 seconds without Rust diagnostics; matching
target-dir cargo/rustc/rustdoc leftovers were cleaned, so no scoped Cargo pass
is claimed for this slice.

2026-06-18 NativeDynamic shared stage contract validation:
`ExportPipelineStage` now includes `NativeDynamic`, and the editor wizard uses
the CLI-aligned `Validate -> SourceTemplate -> NativeDynamic -> CompileHost ->
CookAssets -> Pack -> PlatformBundle -> Report` order. `stage.rs`,
`progress.rs`, and `plan.rs` now recognize `native_dynamic`/`NativeDynamic`;
NativeDynamic commands carry `--validate-report`, plan `plugins_dir` and
`loader_manifest` artifacts, and expect the loader/native plugin stdout keys.
The final Report command consumes the NativeDynamic stage report, and the
desktop plugin descriptor exposes a NativeDynamic stage row while binding the
NativeDynamic report view to `required_stage = NativeDynamic` instead of Pack.
The focused coverage is extended in
`export_pipeline_stage_parser_accepts_cli_and_report_stage_names`,
`export_wizard_pipeline_plan_builds_stage_commands_in_cli_order`,
`export_wizard_pipeline_plan_threads_stage_artifact_inputs`,
`export_wizard_compile_host_path_feeds_platform_bundle_host_input`,
`export_wizard_report_command_consumes_source_template_report`, and
`export_wizard_descriptor_covers_build_layout_stages_and_reports`.
`rustfmt --edition 2021 --check`, conflict-marker scan, trailing-whitespace
scan, and `git diff --check` passed for the touched code/docs with only LF/CRLF
warnings. Plugin scoped
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-native-dynamic-stage-0618 --message-format short --color never`
stopped before compilation because `zircon_plugins/Cargo.lock` needs an update
and `--locked` forbids writing it. Focused editor test
`cargo test --manifest-path Cargo.toml -p zircon_editor export_pipeline_stage_parser_accepts_cli_and_report_stage_names --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-native-dynamic-stage-0618 --message-format short --color never -- --exact --nocapture`
and runtime check
`cargo check --manifest-path Cargo.toml -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-native-dynamic-stage-0618-runtime --message-format short --color never`
both timed out after about 124 seconds without target output, so no Cargo pass is
claimed for this slice.

2026-06-18 report-view summary entry contract validation:
`export_wizard.rs` now gives each `ExportWizardReportViewDescriptor` a
`summary_entry_keys` slice. SourceTemplate and LibraryEmbed declare the shared
pipeline report and `report.export_plan.*` rows; NativeDynamic declares those
plus the `report.native_plugins_payload.*` rows. The plugin crate re-exports the
constants, and `export_wizard_descriptor_covers_build_layout_stages_and_reports`
now checks all three report views. `rustfmt --edition 2021 --check`,
conflict-marker scan, trailing-whitespace scan, and `git diff --check` passed
for the touched files with only LF/CRLF warnings. Plugin scoped
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-report-view-keys-0618 --message-format short --color never`
stopped before compilation because `zircon_plugins/Cargo.lock` needs an update
and `--locked` forbids writing it, so no scoped Cargo pass is claimed for this
slice.

2026-06-18 report-template control id contract validation:
`export_wizard.rs` now gives each report view descriptor a
`template_control_ids` slice. SourceTemplate, LibraryEmbed, and NativeDynamic
declare their root, summary, primary field/list, and diagnostics controls, and
the descriptor test loads each registered report `.v2.ui.toml` file to verify
those ids are present in the real template nodes. `rustfmt --edition 2021
--check`, conflict-marker scan, trailing-whitespace scan, and `git diff --check`
passed for the touched files with only LF/CRLF warnings. Plugin scoped
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-report-template-controls-0618 --message-format short --color never`
stopped before compilation because `zircon_plugins/Cargo.lock` needs an update
and `--locked` forbids writing it, so no scoped Cargo pass is claimed for this
slice.

2026-06-18 report-template document descriptor validation:
`export_wizard.rs` now gives each report view descriptor a `template_document`
URI. SourceTemplate, LibraryEmbed, and NativeDynamic point directly at their
registered `.v2.ui.toml` assets, and
`export_wizard_descriptor_covers_build_layout_stages_and_reports` checks each
template id/document pair against `EXPORT_REPORT_TEMPLATE_DOCUMENTS` before
loading the document through `template_document` for control-id validation.
`rustfmt --edition 2021 --check`, conflict-marker scan, trailing-whitespace
scan, and `git diff --check` passed for the touched files with only LF/CRLF
warnings. Plugin scoped
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-report-template-document-0618 --message-format short --color never`
stopped before compilation because `zircon_plugins/Cargo.lock` needs an update
and `--locked` forbids writing it, so no scoped Cargo pass is claimed for this
slice.

2026-06-18 retained ReportBody native payload projection validation:
`panel_host_projection_tests.rs` now covers the final retained-host hop for
NativeDynamic payload rows. The focused test injects
`report.native_plugins_payload.bundle_path` into the ReportBody slot on a real
panel retained projection and asserts that the resulting label node preserves
the display text, value text, success validation level/message, stable
`entry_key`, ReportBody `slot_kind`, detail, severity, and `stage=report`
properties. `rustfmt --edition 2021 --check`, conflict-marker scan,
trailing-whitespace scan, and `git diff --check` passed for the touched files
with only LF/CRLF warnings. Focused
`cargo test -p zircon_editor --lib export_wizard_panel_retained_projection_preserves_report_body_native_payload_entry --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-retained-native-payload-0618 --message-format short --color never -- --exact --nocapture`
timed out after about 364 seconds without target test output; matching
target-dir cargo/rustc/rustdoc leftovers were cleaned, so no focused pass is
claimed for this slice.

2026-06-15 streaming StageOutput event validation:
`execution.rs` now lets command runners expose `run_with_output(...)`, and the
default `ProcessCommandRunner` spawns the CLI process with piped stdout/stderr
so output lines are observed before the child exits. `run.rs` turns each line
into `ExportWizardJobEventKind::StageOutput` with a fresh job snapshot; `job.rs`
stores running output in `live_stage_outputs`; `view_model.rs` reads that
buffer so stage rows can expose terminal output and progress before
`StageFinished`. The focused coverage is
`export_wizard_job_runner_streams_stage_output_before_stage_finished` in
`streaming_output_tests.rs`, which asserts StageOutput precedes StageFinished
and that the retained view model sees live stdout. `rustfmt --edition 2021` was
applied. Scoped Cargo validation passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-streaming-0615 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-streaming-0615 --message-format short --color never`.
Focused
`cargo test -p zircon_editor --lib export_wizard_job_runner_streams_stage_output_before_stage_finished --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-streaming-0615 --message-format short --color never -- --exact --nocapture`
timed out after 904 seconds without target output; matching target-dir
cargo/rustc leftovers were stopped, so no focused streaming-output test pass is
claimed.

2026-06-15 in-stage cancellation validation:
`ExportWizardCommandRunner` now exposes `run_with_output_and_cancel(...)`, and
`ProcessCommandRunner` polls that callback while the child process is active.
When cancellation is requested during a stage, the default runner terminates the
child, drains stdout/stderr reader threads, and returns the captured output. The
stage execution carries an explicit `cancelled` flag, so `run_export_wizard_job`
records the stage, emits `StageFinished`, then publishes a `Cancelled` terminal
snapshot instead of treating the killed process exit code as a fatal build
failure. `ExportWizardPanelViewModel` no longer promotes cancelled running
stages to Passed. The focused coverage is
`export_wizard_job_runner_cancels_during_active_stage_without_failing` in
`cancellation_tests.rs`, using an injected runner so no real CLI process is
started by the test. `rustfmt --edition 2021 --check` passed for the touched
wizard files. Scoped Cargo validation passed with existing warnings for both
`cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-cancel-0615 --message-format short --color never`
and
`cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-cancel-0615 --message-format short --color never`.
Focused
`cargo test -p zircon_editor --lib export_wizard_job_runner_cancels_during_active_stage_without_failing --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-cancel-0615 --message-format short --color never -- --exact --nocapture`
timed out after 908 seconds without target output; matching target-dir
cargo/rustc/rustdoc process audit was clean, so no focused cancellation test pass
is claimed.

2026-06-20 host native-aware fatal export validation:
`export_build/manager.rs` now checks `plan.has_fatal_diagnostics()` immediately
after native-aware plan generation and before the `prepare-native-packages`
stage. Fatal plans reuse the runtime no-write materialization report, write
export diagnostics, and return without generated files, copied packages, native
Cargo invocations, or SourceTemplate Cargo invocation. Validation covered
`rustfmt --edition 2021 --check` for the export manager, source scans proving
the fatal check appears before native package preparation, conflict-marker scan,
trailing-whitespace scan, and path-scoped `git diff --check` with only LF/CRLF
warnings. Cargo and focused behavior tests are deferred under the current
implementation-first direction.

2026-06-20 host native-aware discovery reuse validation:
`manifest_completion/native.rs` now exposes a host-internal completion helper
that accepts an existing `NativePluginLoadReport`, and `export_build/manager.rs`
uses it after the execution path's initial native package discovery. Public plan
generation still discovers packages through `complete_native_aware_project_plugin_manifest(...)`,
while the executing build path avoids a second plugin-directory scan and keeps
plan completion and package preparation on the same report snapshot. Validation
covered `rustfmt --edition 2021 --check` for the manager/completion files,
source scans for the helper and discovery call sites, conflict-marker scan,
trailing-whitespace scan, and path-scoped `git diff --check` with only LF/CRLF
warnings. Cargo and focused behavior tests remain deferred under the current
implementation-first direction.
