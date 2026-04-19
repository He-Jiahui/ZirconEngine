use crate::core::manager::{
    ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_MANAGER_NAME,
    LEVEL_MANAGER_NAME, PHYSICS_MANAGER_NAME, RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME,
    RESOURCE_MANAGER_NAME,
};

#[test]
fn manager_service_names_cover_runtime_owned_modules() {
    assert_eq!(RESOURCE_MANAGER_NAME, "AssetModule.Manager.ResourceManager");
    assert_eq!(INPUT_MANAGER_NAME, "InputModule.Manager.InputManager");
    assert_eq!(CONFIG_MANAGER_NAME, "FoundationModule.Manager.ConfigManager");
    assert_eq!(EVENT_MANAGER_NAME, "FoundationModule.Manager.EventManager");
    assert_eq!(RENDERING_MANAGER_NAME, "GraphicsModule.Manager.RenderingManager");
    assert_eq!(RENDER_FRAMEWORK_NAME, "GraphicsModule.Manager.RenderFramework");
    assert_eq!(LEVEL_MANAGER_NAME, "SceneModule.Manager.LevelManager");
    assert_eq!(PHYSICS_MANAGER_NAME, "PhysicsModule.Manager.PhysicsManager");
    assert_eq!(ANIMATION_MANAGER_NAME, "AnimationModule.Manager.AnimationManager");
}

#[test]
fn manager_module_uses_core_framework_contracts() {
    let mod_source = include_str!("mod.rs");
    let resolver_source = include_str!("resolver.rs");

    assert!(mod_source.contains("crate::core::framework"));
    assert!(resolver_source.contains("crate::core::framework"));
    assert!(!mod_source.contains("zircon_framework"));
    assert!(!resolver_source.contains("zircon_framework"));
}
