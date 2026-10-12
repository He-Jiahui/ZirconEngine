#[test]
fn retained_tick_owns_the_single_editor_job_event_pump_call() {
    let source = include_str!("../tick.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("tick source should contain its production section");
    let pump_call = [".jobs()", ".pump_events()"].concat();
    assert_eq!(production.matches(pump_call.as_str()).count(), 1);
    let pump = production
        .find("self.pump_editor_job_events();")
        .expect("retained tick should pump editor job events");
    let task_diagnostics = production
        .find("self.editor_manager.pump_runtime_task_diagnostics(0)")
        .expect("retained tick should project runtime task diagnostics");
    let heartbeat = production
        .find(".refresh_project_session_heartbeat_if_due(Instant::now())")
        .expect("retained tick should refresh the active project session heartbeat");
    let heartbeat_wake = production
        .find("self.ui.set_lifecycle_frame_update(")
        .expect("retained tick should schedule the active session heartbeat wake");
    let prompted_close_save = production
        .find("self.poll_prompted_close_save();")
        .expect("retained tick should collect prompted close saves after job events");
    let save_all = production
        .find("self.poll_document_save_all();")
        .expect("retained tick should collect Save All completions after prompted closes");
    let export_poll = production
        .find("self.poll_desktop_export_jobs();")
        .expect("retained tick should poll export jobs");
    let wizard_poll = production
        .find("self.poll_desktop_export_wizard_sessions();")
        .expect("retained tick should poll export wizard sessions");
    let progress_sync = production
        .find("self.sync_editor_job_progress();")
        .expect("retained tick should project the unified job progress source");
    let lifecycle_pump = production
        .find("self.runtime.pump_plugin_lifecycle_messages()")
        .expect("retained tick should pump plugin lifecycle message subscriptions");
    assert!(pump < prompted_close_save);
    assert!(pump < task_diagnostics);
    assert!(task_diagnostics < heartbeat);
    assert!(pump < heartbeat);
    assert!(heartbeat < prompted_close_save);
    assert!(heartbeat < heartbeat_wake);
    assert!(heartbeat_wake < prompted_close_save);
    assert!(prompted_close_save < save_all);
    assert!(save_all < export_poll);
    assert!(export_poll < wizard_poll);
    assert!(wizard_poll < progress_sync);
    assert!(progress_sync < lifecycle_pump);
}

#[test]
fn retained_tick_collects_recovery_worker_results_after_job_events_before_heartbeat_io() {
    let source = include_str!("../tick.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("tick source should contain its production section");
    let job_events = production
        .find("self.pump_editor_job_events();")
        .expect("retained tick should pump job events first");
    let task_diagnostics = production
        .find("self.editor_manager.pump_runtime_task_diagnostics(0)")
        .expect("retained tick should project runtime task diagnostics after job events");
    let recovery = production
        .find("self.editor_manager.pump_project_recovery_decisions()")
        .expect("retained tick should collect recovery decisions and worker results");
    let heartbeat = production
        .find(".refresh_project_session_heartbeat_if_due(Instant::now())")
        .expect("retained tick should refresh the active project session heartbeat");

    assert!(job_events < recovery);
    assert!(job_events < task_diagnostics);
    assert!(task_diagnostics < recovery);
    assert!(recovery < heartbeat);
}

#[test]
fn retained_tick_drives_project_autosave_after_recovery_and_session_heartbeat() {
    let source = include_str!("../tick.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("tick source should contain its production section");
    let recovery = production
        .find("self.editor_manager.pump_project_recovery_decisions()")
        .expect("retained tick should process project recovery first");
    let heartbeat = production
        .find(".refresh_project_session_heartbeat_if_due(Instant::now())")
        .expect("retained tick should refresh the active project session heartbeat");
    let autosave = production
        .find("self.poll_editor_autosave();")
        .expect("retained tick should drive the context-owned autosave service");
    let model_import = production
        .find("self.poll_model_import();")
        .expect("retained tick should continue normal tool polling after autosave");

    assert!(recovery < heartbeat);
    assert!(heartbeat < autosave);
    assert!(autosave < model_import);
}

#[test]
fn retained_tick_projects_the_unified_notification_snapshot_after_backend_polling() {
    let source = include_str!("../tick.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("tick source should contain its production section");
    let backend_poll = production
        .find("self.runtime.pump_runtime_event_consumers()")
        .expect("retained tick should poll the play backend");
    let lifecycle_pump = production
        .find("self.runtime.pump_plugin_lifecycle_messages()")
        .expect("retained tick should pump plugin lifecycle message subscriptions before backend polling");
    let template_sync = production
        .find("self.sync_plugin_template_documents_if_changed()")
        .expect("retained tick should synchronize plugin templates after backend polling");
    let toast_sync = production
        .find("self.sync_activity_notifications();")
        .expect("retained tick should project the unified notification authority");
    let settings_sync = production
        .find("self.sync_settings_projections();")
        .expect("retained tick should synchronize authority-owned settings projections");
    let recompute = production
        .find("self.recompute_if_dirty();")
        .expect("retained tick should recompute invalidated presentation");
    assert!(lifecycle_pump < backend_poll);
    assert!(backend_poll < template_sync);
    assert!(template_sync < toast_sync);
    assert!(toast_sync < settings_sync);
    assert!(settings_sync < recompute);
}

#[test]
fn retained_tick_selects_the_terminal_runtime_domain_before_hierarchy_sync() {
    let source = include_str!("../tick.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("tick source should contain its production section");
    let runtime_consumers = production
        .find("self.runtime.pump_runtime_event_consumers()")
        .expect("retained tick should settle the play backend state");
    let selection = production
        .find("self.runtime.sync_active_selection_world_domain()")
        .expect("retained tick should select the matching world selection domain");
    let viewport_pick = production
        .find("self.poll_play_viewport_pick_for_native_host()")
        .expect("retained tick should consume renderer-owned Play viewport picks");
    let hierarchy = production
        .find("self.sync_active_hierarchy_world();")
        .expect("retained tick should synchronize the selected hierarchy domain");
    assert!(runtime_consumers < selection);
    assert!(selection < hierarchy);
    assert!(hierarchy < viewport_pick);
}

#[test]
fn retained_tick_consumes_resize_render_work_after_the_active_recompute() {
    let source = include_str!("../tick.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("tick source should contain its production section");
    let recompute = production
        .find("self.recompute_if_dirty();")
        .expect("window metrics and viewport projection recompute");
    let render = production
        .find("self.submit_render_frame_if_dirty();")
        .expect("render reason consumer");

    assert!(recompute < render);
}
