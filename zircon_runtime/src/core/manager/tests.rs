#[cfg(feature = "ai-contracts")]
use crate::core::manager::AI_MANAGER_NAME;
#[cfg(feature = "net-contracts")]
use crate::core::manager::NET_MANAGER_NAME;
#[cfg(feature = "physics-contracts")]
use crate::core::manager::PHYSICS_MANAGER_NAME;
#[cfg(feature = "sound-contracts")]
use crate::core::manager::SOUND_MANAGER_NAME;
use crate::core::manager::{
    ManagerResolver, ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME,
    INPUT_ACTION_MANAGER_NAME, INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RESOURCE_MANAGER_NAME,
};
use crate::core::{CoreError, CoreRuntime};

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
    #[cfg(feature = "ai-contracts")]
    assert_eq!(AI_MANAGER_NAME, "ai.runtime.Manager.AiManager");
    #[cfg(feature = "net-contracts")]
    assert_eq!(NET_MANAGER_NAME, "net.runtime.Manager.NetManager");
    #[cfg(feature = "physics-contracts")]
    assert_eq!(
        PHYSICS_MANAGER_NAME,
        "physics.runtime.Manager.PhysicsManager"
    );
    assert_eq!(
        ANIMATION_MANAGER_NAME,
        "animation.runtime.Manager.AnimationManager"
    );
    #[cfg(feature = "sound-contracts")]
    assert_eq!(SOUND_MANAGER_NAME, "sound.runtime.Manager.SoundManager");
}

#[test]
fn manager_module_uses_core_framework_contracts() {
    let mod_source = include_str!("mod.rs");
    let resolver_source = include_str!("resolver.rs");

    assert!(mod_source.contains("crate::core::framework"));
    assert!(resolver_source.contains("crate::core::framework"));
    assert!(mod_source.contains("RegisteredManagerService"));
    assert!(mod_source.contains("ManagerServiceHandle"));
    assert!(resolver_source.contains("manager_service_handle(core, $service_name)"));
    assert!(!mod_source.contains("zircon_framework"));
    assert!(!resolver_source.contains("zircon_framework"));
    assert!(!resolver_source.contains("Arc<dyn $trait_name>"));
}

#[test]
fn manager_resolver_does_not_keep_core_runtime_alive() {
    let runtime = CoreRuntime::new();
    let weak = runtime.weak();
    let resolver = ManagerResolver::new(runtime.handle());

    drop(runtime);

    assert!(weak.upgrade().is_none());
    assert!(matches!(
        resolver.config_handle(),
        Err(CoreError::ServiceUnavailable(name)) if name == "CoreRuntime"
    ));
}
