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
    let state_source = include_str!("../session/state.rs");
    let profile_source = include_str!("../session/profile.rs");

    assert!(profile_source.contains("RUNTIME_SESSION_PROFILE_DEV => Some(Self::Dev)"));
    assert!(profile_source.contains("DiagnosticStoreLogSchedule::repeating"));
    assert!(profile_source.contains("DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT"));
    assert!(
        state_source.contains("collect_runtime_diagnostic_current_store(&self.runtime.handle())")
    );
    assert!(!state_source.contains("collect_runtime_diagnostics(&self.runtime.handle()).store"));
    assert!(state_source.contains("write_diagnostic_store_current_snapshot"));
    assert!(!state_source.contains("write_diagnostic_store_snapshot"));
}

#[test]
fn minimal_and_headless_profiles_skip_render_bridge_bootstrap() {
    let state_source = include_str!("../session/state.rs");
    let construction_source = include_str!("../session/construction.rs");
    let profile_source = include_str!("../session/profile.rs");

    assert!(profile_source.contains("fn uses_render_bridge(self) -> bool"));
    assert!(profile_source.contains("matches!(self, Self::Runtime | Self::Editor | Self::Dev)"));
    assert!(construction_source.contains("runtime_dynamic_session_render_bridge_skipped"));
    assert!(state_source.contains("let Some(render_bridge) = &mut self.render_bridge else"));
}

#[test]
fn tick_frame_drives_loaded_level_before_clearing_frame_input() {
    let source = include_str!("../session/state.rs");
    let tick_start = source
        .find("fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()>")
        .expect("runtime dynamic session tick_frame implementation");
    let level_tick = source[tick_start..]
        .find(".tick(&self.runtime.handle(), advance)")
        .expect("runtime frame should tick the loaded LevelSystem");
    let input_begin_frame = source[tick_start..]
        .find(".begin_frame();")
        .expect("runtime frame should clear per-frame input after gameplay tick");

    assert!(
        source[tick_start..tick_start + input_begin_frame].contains("self.resolve_input_manager()"),
        "runtime frame should resolve the versioned input handle at the use point"
    );

    assert!(
        !source[tick_start..].contains(".tick(&self.runtime.handle(), advance.raw_real_delta()"),
        "runtime frame should pass FrameTimeSnapshot through instead of reducing it to raw delta"
    );
    assert!(
        level_tick < input_begin_frame,
        "runtime gameplay tick should observe the current frame's input before frame input is cleared"
    );
}

#[test]
fn session_ui_extract_remains_documented_dynamic_session_side_path() {
    let session_source = include_str!("../session/state.rs");
    let extract_source = include_str!("../session/extract.rs");
    let ui_extract_cache_source = include_str!("../session/ui_extract_cache.rs");
    let capture_start = session_source
        .find("fn capture_frame(\n        &mut self,")
        .expect("RuntimeDynamicSession::capture_frame implementation");
    let present_start = session_source
        .find("fn present_viewport(\n        &mut self,")
        .expect("RuntimeDynamicSession::present_viewport implementation");
    let ui_submission_start = extract_source
        .find("fn current_ui_submission(\n        &mut self,")
        .expect("current_ui_submission implementation");
    let resize_start = extract_source[ui_submission_start..]
        .find("fn resize_viewport")
        .map(|offset| ui_submission_start + offset)
        .expect("method after current_ui_submission");
    let ui_submission_body = &extract_source[ui_submission_start..resize_start];

    assert!(
        session_source[capture_start..present_start]
            .contains("let ui = self.current_ui_submission()?;"),
        "capture_frame should keep the documented UI submission side path explicit"
    );
    assert!(
        session_source[present_start..].contains("let ui = self.current_ui_submission()?;"),
        "present_viewport should keep the documented UI submission side path explicit"
    );
    assert!(
        ui_submission_body.contains("ui_extract_cache")
            && ui_submission_body.contains(".current_extract(world, viewport_size)")
            && ui_submission_body.contains(".map(UiRenderSubmission::single)"),
        "fallback UI capture should delegate to the component-generation cache"
    );
    let menu_extract = ui_extract_cache_source
        .find("runtime_session_menu_extract(")
        .expect("menu fallback extraction");
    let hud_extract = ui_extract_cache_source
        .find("None => runtime_session_hud_extract(")
        .expect("HUD fallback extraction");
    assert!(
        menu_extract < hud_extract
            && ui_extract_cache_source[menu_extract..hud_extract]
                .contains("&mut self.text_measure_cache")
            && ui_extract_cache_source[hud_extract..].contains("&mut self.text_measure_cache"),
        "the cache miss path must preserve menu-over-HUD fallback priority with one producer-owned text cache"
    );
    assert!(
        ui_extract_cache_source.contains("self.text_measure_cache.begin_frame()")
            && ui_extract_cache_source.contains("self.text_measure_cache.finish_frame()"),
        "fallback UI extraction must establish the standard text-cache frame boundary"
    );
    assert!(
        !ui_submission_body.contains("SystemStage::RenderExtract"),
        "current UI submission side path is not owned by the scheduled RenderExtract stage yet"
    );
    assert!(
        ui_submission_body.contains(".runtime_ui")
            && ui_submission_body.contains(".render_submission(viewport_size)"),
        "project-owned UI surfaces must take precedence over legacy world preview extractors"
    );
}

#[test]
fn project_sessions_open_assets_before_loading_default_level() {
    let source = include_str!("../session/construction.rs");
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

#[test]
fn project_session_startup_reuses_one_prepared_project_manager_snapshot() {
    let construction_source = include_str!("../session/construction.rs");
    let project_source = include_str!("../session/project.rs");
    let scene_source = include_str!("../../scene/module/mod.rs");
    let asset_contract_source =
        include_str!("../../asset/pipeline/manager/asset_manager/asset_manager.rs");
    let asset_manager_source =
        include_str!("../../asset/pipeline/manager/service_contracts/asset_manager_contract.rs");
    let open_project_assets_source = project_source
        .split("pub(super) fn open_project_assets")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn load_default_level").next())
        .expect("prepared project asset-open owner should remain explicit");

    let prepare = construction_source
        .find("RuntimeProjectConfig::prepare")
        .expect("project startup should prepare one ProjectManager before module selection");
    let linked_plugins = construction_source
        .find("LinkedRuntimePluginPlan::prepare")
        .expect("linked plugin selection should remain explicit");
    assert!(
        prepare < linked_plugins,
        "the authoritative project manifest snapshot must exist before plugin selection"
    );
    assert_eq!(
        construction_source
            .matches("RuntimeProjectConfig::prepare")
            .count(),
        1,
        "session construction must prepare the project exactly once"
    );
    assert!(project_source.contains("struct RuntimePreparedProject"));
    assert!(project_source.contains(".open_prepared_project(project)"));
    assert!(open_project_assets_source.contains("asset_manager_handle(core)"));
    assert!(!open_project_assets_source.contains("project_asset_manager_handle(core)"));
    assert!(
        !construction_source.contains("RuntimeProjectConfig::load_plugin_manifest")
            && !construction_source.contains(".load_manifest()"),
        "startup consumers must reuse the prepared manifest instead of reopening it"
    );
    assert!(
        asset_contract_source.contains("fn open_prepared_project(")
            && asset_contract_source.contains("fn current_project_snapshot("),
        "prepared activation and deadlock-safe current-project snapshots belong to the abstract AssetManager service"
    );
    assert!(
        asset_manager_source.contains("open_prepared_project(project)"),
        "the normal path-based AssetManager entry must delegate to the same prepared owner"
    );
    assert!(
        scene_source.contains("current_project_snapshot()")
            && !scene_source.contains("ProjectManager::open")
            && !scene_source.contains("scan_and_import"),
        "default-scene loading must snapshot the activated project without reopening or rescanning it"
    );
}
