mod capability_history_visibility;
mod gpu_sprite_ui_advanced;
mod graph_execution;
mod graph_resources;
mod hzb_light_camera_capture;
mod motion_vector;
mod post_process_material_mesh;
mod support;

use crate::core::CoreRuntime;

use support::{fake_render_module, DIAGNOSTICS_TEST_MODULE};

#[test]
fn runtime_diagnostics_reports_missing_runtime_contracts_without_panicking() {
    let runtime = CoreRuntime::new();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(!snapshot.render.available);
    assert!(snapshot.render.stats.is_none());
    assert!(snapshot.render.error.is_some());
    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());
    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());
    assert!(snapshot.store.is_empty());
}

#[test]
fn runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins() {
    let runtime = CoreRuntime::new();
    runtime.register_module(fake_render_module()).unwrap();
    runtime.activate_module(DIAGNOSTICS_TEST_MODULE).unwrap();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(snapshot.render.available);
    let render_stats = snapshot.render.stats.as_ref().expect("render stats");
    assert_eq!(render_stats.active_viewports, 2);
    assert_eq!(render_stats.submitted_frames, 7);
    assert_eq!(
        render_stats.capabilities.backend_name,
        "diagnostics-test-renderer"
    );
    assert!(!snapshot.render.virtual_geometry_debug_available);
    assert!(snapshot.render.error.is_none());

    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());

    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());

    assert!(snapshot
        .store
        .series
        .iter()
        .any(|series| series.path.as_str() == "render.submitted_frames"
            && series.current == Some(7.0)));
    capability_history_visibility::assert_capability_history_visibility(&snapshot);
    hzb_light_camera_capture::assert_hzb_light_camera_capture(&snapshot);
    graph_resources::assert_graph_resources(&snapshot);
    graph_execution::assert_graph_execution(&snapshot);
    post_process_material_mesh::assert_post_process_material_mesh(&snapshot);
    gpu_sprite_ui_advanced::assert_gpu_sprite_ui_advanced(&snapshot);

    let devtools = crate::core::diagnostics::collect_runtime_devtools_snapshot(&runtime.handle());
    assert!(devtools
        .modules
        .iter()
        .any(|module| module.name == DIAGNOSTICS_TEST_MODULE));
    assert!(devtools
        .services
        .iter()
        .any(|service| service.name == crate::core::manager::RENDER_FRAMEWORK_NAME));
    assert!(devtools
        .plugin_catalog
        .iter()
        .any(|plugin| plugin.package_id == "physics"));
    assert!(devtools
        .diagnostics_summary
        .tagged_subsystems
        .contains(&"render".to_string()));
}
