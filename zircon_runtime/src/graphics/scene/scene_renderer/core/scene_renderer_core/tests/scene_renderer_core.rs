use super::SceneRendererDeferredLightingProfile;

#[test]
fn renderer_core_imports_the_default_deferred_lighting_profile() {
    assert_eq!(
        SceneRendererDeferredLightingProfile::default(),
        SceneRendererDeferredLightingProfile::FullScene
    );
}

#[test]
fn renderer_core_guards_all_render_entrypoints_with_its_construction_epoch() {
    let core = include_str!("../scene_renderer_core.rs");
    assert!(core.contains("device_id: DeviceId"));
    assert!(core.contains("device_generation: DeviceGeneration"));
    assert!(core.contains("GraphicsError::SceneRendererDeviceEpochMismatch"));

    let direct = include_str!("../../scene_renderer_core_render_scene/render_scene.rs");
    let direct_guard = direct
        .find("self.ensure_device_epoch(backend)?;")
        .expect("direct render must admit the core device epoch");
    let direct_device = direct
        .find("let device = &backend.device;")
        .expect("direct render device borrow");
    assert!(direct_guard < direct_device);

    let compiled = include_str!("../../scene_renderer_core_render_compiled_scene/render/render.rs");
    let compiled_guard = compiled
        .find("self.ensure_device_epoch(backend)?;")
        .expect("compiled render must admit the core device epoch");
    let compiled_device = compiled
        .find("let device = &backend.device;")
        .expect("compiled render device borrow");
    assert!(compiled_guard < compiled_device);
}
