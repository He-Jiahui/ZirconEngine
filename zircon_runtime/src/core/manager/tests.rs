use crate::core::manager::{
    AI_MANAGER_NAME, ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME,
    INPUT_ACTION_MANAGER_NAME, INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME, NET_MANAGER_NAME,
    PHYSICS_MANAGER_NAME, RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME, RESOURCE_MANAGER_NAME,
    SOUND_MANAGER_NAME,
};

#[test]
fn manager_service_names_cover_runtime_owned_modules() {
    assert_eq!(RESOURCE_MANAGER_NAME, "AssetModule.Manager.ResourceManager");
    assert_eq!(INPUT_MANAGER_NAME, "InputModule.Manager.InputManager");
    assert_eq!(
        INPUT_ACTION_MANAGER_NAME,
        "InputModule.Manager.InputActionManager"
    );
    assert_eq!(
        CONFIG_MANAGER_NAME,
        "FoundationModule.Manager.ConfigManager"
    );
    assert_eq!(EVENT_MANAGER_NAME, "FoundationModule.Manager.EventManager");
    assert_eq!(
        RENDERING_MANAGER_NAME,
        "GraphicsModule.Manager.RenderingManager"
    );
    assert_eq!(
        RENDER_FRAMEWORK_NAME,
        "GraphicsModule.Manager.RenderFramework"
    );
    assert_eq!(LEVEL_MANAGER_NAME, "SceneModule.Manager.LevelManager");
    assert_eq!(AI_MANAGER_NAME, "AiModule.Manager.AiManager");
    assert_eq!(NET_MANAGER_NAME, "NetModule.Manager.NetManager");
    assert_eq!(PHYSICS_MANAGER_NAME, "PhysicsModule.Manager.PhysicsManager");
    assert_eq!(
        ANIMATION_MANAGER_NAME,
        "AnimationModule.Manager.AnimationManager"
    );
    assert_eq!(SOUND_MANAGER_NAME, "SoundModule.Manager.SoundManager");
}

#[test]
fn manager_module_uses_core_framework_contracts() {
    let mod_source = include_str!("mod.rs");
    let resolver_source = include_str!("resolver.rs");

    assert!(mod_source.contains("crate::core::framework"));
    assert!(resolver_source.contains("crate::core::framework"));
    assert!(
        resolver_source.contains("let holder = core.resolve_manager::<$holder>($service_name)?;")
    );
    assert!(resolver_source.contains("Ok(holder.shared())"));
    assert!(!mod_source.contains("zircon_framework"));
    assert!(!resolver_source.contains("zircon_framework"));
    assert!(!resolver_source.contains(".map(|holder| holder.shared())"));
}
