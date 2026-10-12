use std::sync::Arc;

use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::{CoreRuntime, InitLevel, ModuleDescriptor};
use zircon_runtime::scene::{
    create_level, prepare_level, DefaultLevelManager, LevelMetadata, World,
    DEFAULT_LEVEL_MANAGER_NAME,
};

use crate::core::editing::authoring_world::EditorAuthoringWorld;
use crate::core::gateway::{DetachedEditorRuntimeGateway, GatewayError};

use super::super::{EditorRuntimeGatewayHandle, GatewayGeneration};

#[test]
fn prepared_authoring_replace_preserves_generation_exhausted_and_rolls_back_registry() {
    let runtime = CoreRuntime::new();
    let descriptor = zircon_runtime::scene::module_descriptor();
    for dependency in &descriptor.module_dependencies {
        runtime
            .register_module(
                ModuleDescriptor::new(
                    &dependency.module_name,
                    "Editor publication test dependency",
                )
                .with_init_level(InitLevel::Kernel),
            )
            .unwrap();
    }
    runtime.register_module(descriptor).unwrap();
    runtime.activate_registered_modules().unwrap();
    let core = runtime.handle();
    let old_level = create_level(&core, World::empty(), LevelMetadata::default()).unwrap();
    let old_handle = old_level.handle();
    let prepared_handle = WorldHandle::new(old_handle.get().checked_add(1).unwrap());
    let gateway = EditorRuntimeGatewayHandle::detached();
    // Reach MAX through the actual initial authoring replacement, leaving a
    // coherent author identity and the original gateway fully observable.
    gateway.inner.current.store(Arc::new(GatewayGeneration::new(
        u64::MAX - 1,
        Arc::new(DetachedEditorRuntimeGateway),
        None,
    )));
    let mut author = EditorAuthoringWorld::loaded(&gateway, old_level).unwrap();
    assert_eq!(gateway.generation(), u64::MAX);
    let original_identity = gateway.identity();
    let original_lease = gateway.current_lease();
    assert!(author.is_loaded());
    let original_nodes = author.with_world(|world| world.nodes().len()).unwrap();
    assert!(original_nodes.is_some());

    let prepared = prepare_level(&core, World::empty(), LevelMetadata::default()).unwrap();
    {
        let manager = core
            .resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
            .unwrap();
        assert!(manager.level(old_handle).is_some());
        assert!(manager.level(prepared_handle).is_none());
    }
    let failure = author
        .replace(prepared)
        .expect_err("MAX must reject actual gateway replacement");
    assert_eq!(failure, GatewayError::GenerationExhausted);
    assert_eq!(gateway.generation(), u64::MAX);
    assert_eq!(gateway.identity(), original_identity);
    let current_lease = gateway.current_lease();
    assert!(Arc::ptr_eq(
        original_lease.gateway(),
        current_lease.gateway()
    ));
    assert!(author.is_loaded());
    assert_eq!(
        author.with_world(|world| world.nodes().len()).unwrap(),
        original_nodes
    );
    let manager = core
        .resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
        .unwrap();
    assert!(manager.level(old_handle).is_some());
    assert!(manager.level(prepared_handle).is_none());
}
