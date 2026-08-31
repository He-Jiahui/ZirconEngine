#[cfg(feature = "ai-contracts")]
use crate::core::manager::AI_MANAGER_NAME;
#[cfg(feature = "net-contracts")]
use crate::core::manager::NET_MANAGER_NAME;
#[cfg(feature = "physics-contracts")]
use crate::core::manager::PHYSICS_MANAGER_NAME;
#[cfg(feature = "sound-contracts")]
use crate::core::manager::SOUND_MANAGER_NAME;
use std::sync::Arc;

use crate::core::manager::{
    manager_service_handle, resolve_manager_service, ManagerResolver, RegisteredManagerService,
    ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_ACTION_MANAGER_NAME,
    INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME, PLATFORM_MANAGER_NAME, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RESOURCE_MANAGER_NAME,
};
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind,
    StartupMode,
};

const RUNTIME_BOUND_MANAGER_MODULE: &str = "RuntimeBoundManagerModule";
const RUNTIME_BOUND_MANAGER_SERVICE: &str = "RuntimeBoundManagerModule.Manager.RuntimeBoundManager";

#[derive(Debug, PartialEq, Eq)]
struct RuntimeBoundManager {
    runtime_label: &'static str,
}

fn runtime_with_bound_manager(runtime_label: &'static str) -> CoreRuntime {
    let runtime = CoreRuntime::new();
    let manager = Arc::new(RuntimeBoundManager { runtime_label });
    runtime
        .register_module(
            ModuleDescriptor::new(RUNTIME_BOUND_MANAGER_MODULE, "runtime provenance test")
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts(
                        RUNTIME_BOUND_MANAGER_MODULE,
                        ServiceKind::Manager,
                        "RuntimeBoundManager",
                    ),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(
                            Arc::new(RegisteredManagerService::new(Arc::clone(&manager)))
                                as ServiceObject,
                        )
                    }),
                )),
        )
        .expect("runtime-bound manager module should register");
    runtime
        .activate_module(RUNTIME_BOUND_MANAGER_MODULE)
        .expect("runtime-bound manager module should activate");
    runtime
}

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
        PLATFORM_MANAGER_NAME,
        "PlatformModule.Manager.PlatformManager"
    );
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

#[test]
fn manager_service_handle_does_not_keep_core_runtime_alive() {
    let runtime = runtime_with_bound_manager("runtime");
    let weak = runtime.weak();
    let handle = manager_service_handle::<RuntimeBoundManager>(
        &runtime.handle(),
        RUNTIME_BOUND_MANAGER_SERVICE,
    )
    .expect("runtime-bound manager handle should resolve");

    drop(runtime);

    assert!(weak.upgrade().is_none());
    assert_eq!(
        handle.service_name().as_str(),
        RUNTIME_BOUND_MANAGER_SERVICE
    );
}

#[test]
fn manager_service_handle_rejects_cross_runtime_identity_substitution() {
    let first_runtime = runtime_with_bound_manager("first");
    let second_runtime = runtime_with_bound_manager("second");
    let first_core = first_runtime.handle();
    let second_core = second_runtime.handle();
    let mut first_handle =
        manager_service_handle::<RuntimeBoundManager>(&first_core, RUNTIME_BOUND_MANAGER_SERVICE)
            .expect("first runtime handle should resolve");
    let second_handle =
        manager_service_handle::<RuntimeBoundManager>(&second_core, RUNTIME_BOUND_MANAGER_SERVICE)
            .expect("second runtime handle should resolve");

    assert_eq!(
        resolve_manager_service(&first_core, first_handle.clone())
            .expect("the originating runtime should resolve its manager")
            .runtime_label,
        "first"
    );

    first_handle.index = second_handle.index;
    first_handle.generation = second_handle.generation;
    first_handle.service = second_handle.service.clone();

    assert_ne!(first_handle, second_handle);

    assert!(matches!(
        resolve_manager_service(&second_core, first_handle),
        Err(CoreError::ServiceUnavailable(name))
            if name == RUNTIME_BOUND_MANAGER_SERVICE
    ));
}
