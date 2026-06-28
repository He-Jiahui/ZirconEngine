use super::support::*;

#[test]
fn create_session_accepts_named_headless_profile_without_render_bridge() {
    let api = runtime_api();
    let session = create_test_session_with_profile(api, b"headless");

    assert!(session.is_valid());
    destroy_test_session(api, session);
}

#[test]
fn dev_profile_ticks_runtime_diagnostic_store_log_schedule() {
    let source = include_str!("../session.rs");

    assert!(source.contains("RUNTIME_SESSION_PROFILE_DEV => Some(Self::Dev)"));
    assert!(source.contains("DiagnosticStoreLogSchedule::repeating"));
    assert!(source.contains("DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT"));
    assert!(source.contains("collect_runtime_diagnostics(&self.runtime.handle()).store"));
    assert!(source.contains("write_diagnostic_store_snapshot"));
}

#[test]
fn minimal_and_headless_profiles_skip_render_bridge_bootstrap() {
    let source = include_str!("../session.rs");

    assert!(source.contains("fn uses_render_bridge(self) -> bool"));
    assert!(source.contains("matches!(self, Self::Runtime | Self::Editor | Self::Dev)"));
    assert!(source.contains("runtime_dynamic_session_render_bridge_skipped"));
    assert!(source.contains("let Some(render_bridge) = &mut self.render_bridge else"));
}

#[test]
fn tick_frame_drives_loaded_level_before_clearing_frame_input() {
    let source = include_str!("../session.rs");
    let tick_start = source
        .find("fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()>")
        .expect("runtime dynamic session tick_frame implementation");
    let level_tick = source[tick_start..]
        .find(".tick(&self.runtime.handle(), advance)")
        .expect("runtime frame should tick the loaded LevelSystem");
    let input_begin_frame = source[tick_start..]
        .find("self.input_manager.begin_frame();")
        .expect("runtime frame should clear per-frame input after gameplay tick");

    assert!(
        !source[tick_start..].contains(".tick(&self.runtime.handle(), advance.real_delta()"),
        "runtime frame should pass RuntimeTimeAdvance through instead of reducing it to raw delta"
    );
    assert!(
        level_tick < input_begin_frame,
        "runtime gameplay tick should observe the current frame's input before frame input is cleared"
    );
}

#[test]
fn session_ui_extract_remains_documented_dynamic_session_side_path() {
    let source = include_str!("../session.rs");
    let capture_start = source
        .find("fn capture_frame(")
        .expect("capture_frame implementation");
    let present_start = source
        .find("fn present_viewport(&mut self")
        .expect("present_viewport implementation");
    let ui_extract_start = source
        .find("fn current_ui_extract(&self)")
        .expect("current_ui_extract implementation");
    let resize_start = source[ui_extract_start..]
        .find("fn resize_viewport")
        .map(|offset| ui_extract_start + offset)
        .expect("method after current_ui_extract");
    let ui_extract_body = &source[ui_extract_start..resize_start];

    assert!(
        source[capture_start..present_start].contains("let ui = self.current_ui_extract();"),
        "capture_frame should keep the documented UI extract side path explicit"
    );
    assert!(
        source[present_start..ui_extract_start].contains("let ui = self.current_ui_extract();"),
        "present_viewport should keep the documented UI extract side path explicit"
    );
    assert!(ui_extract_body.contains("runtime_session_menu_extract(world, viewport_size)"));
    assert!(
        ui_extract_body.contains(".or_else(|| runtime_session_hud_extract(world, viewport_size))")
    );
    assert!(
        !ui_extract_body.contains("SystemStage::RenderExtract"),
        "current UI extract side path is not owned by the scheduled RenderExtract stage yet"
    );
}

#[test]
fn project_sessions_open_assets_before_loading_default_level() {
    let source = include_str!("../session.rs");
    let level_start = source
        .find("runtime_session_level")
        .expect("runtime dynamic session project level bootstrap");
    let open_assets = source[level_start..]
        .find(".open_project_assets(&core)")
        .expect("project sessions should open and sync project assets");
    let load_scripts = source[level_start..]
        .find(".load_startup_scripts(&core)")
        .expect("project sessions should load startup scripts");
    let load_level = source[level_start..]
        .find(".load_default_level(&core)")
        .expect("project sessions should load the default level");

    assert!(
        open_assets < load_scripts && open_assets < load_level,
        "project assets must be synchronized before scripts or scene rendering use project resources"
    );
}
