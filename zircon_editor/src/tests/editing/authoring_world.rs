use std::sync::{Arc, Mutex};

use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, Scene, World};
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::editing::authoring_world::{
    AuthoringWorldAccessError, AuthoringWorldSeed, EditorAuthoringWorld,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::EditCommandError;
use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError,
};

struct RepeatingWorldCallbackGateway {
    world: Mutex<World>,
}

impl RepeatingWorldCallbackGateway {
    fn new() -> Self {
        Self {
            world: Mutex::new(Scene::default()),
        }
    }
}

impl EditorRuntimeGateway for RepeatingWorldCallbackGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn session_identity(&self) -> zircon_runtime_interface::GatewaySessionIdentity {
        zircon_runtime_interface::GatewaySessionIdentity::detached()
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        let world = self
            .world
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        read(&world);
        read(&world);
        Ok(())
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        let mut world = self
            .world
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        write(&mut world);
        write(&mut world);
        Ok(())
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

fn assert_repeated_callback_protocol(error: EditCommandError) {
    let EditCommandError::ExternalEffect { source } = error else {
        panic!("repeated gateway callback must be reported as an external effect");
    };
    assert_eq!(
        source.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "borrowed world callback was invoked more than once".to_owned(),
        })
    );
}

#[test]
fn authoring_facade_replaces_and_clears_the_stable_gateway() {
    let handle = EditorRuntimeGatewayHandle::detached();
    let initial =
        DefaultLevelManager::default().create_level(Scene::default(), LevelMetadata::default());
    let replacement =
        DefaultLevelManager::default().create_level(Scene::default(), LevelMetadata::default());
    let mut facade = EditorAuthoringWorld::loaded(&handle, AuthoringWorldSeed::from(initial))
        .expect("initial authoring world");

    assert!(facade.is_loaded());
    assert_eq!(facade.with_world(|scene| scene.nodes().len()), Ok(Some(0)));

    facade
        .replace(AuthoringWorldSeed::from(replacement))
        .expect("replacement authoring world");
    assert_eq!(facade.with_world(|scene| scene.nodes().len()), Ok(Some(0)));

    facade.clear().expect("clear authoring world");
    assert!(!facade.is_loaded());
    assert_eq!(facade.snapshot(), Ok(None));
    assert_eq!(
        handle.with_world(&mut |_| {}),
        Err(GatewayError::RequiresSerializedAccess)
    );
}

#[test]
fn repeated_borrowed_world_callbacks_fail_closed() {
    let handle = EditorRuntimeGatewayHandle::detached();
    let level =
        DefaultLevelManager::default().create_level(Scene::default(), LevelMetadata::default());
    let mut facade = EditorAuthoringWorld::loaded(&handle, AuthoringWorldSeed::from(level))
        .expect("initial authoring world");
    facade
        .replace_gateway_for_test(Arc::new(RepeatingWorldCallbackGateway::new()))
        .expect("install repeating callback gateway");

    assert_eq!(
        facade.with_world(|scene| scene.nodes().len()),
        Err(AuthoringWorldAccessError::ProtocolViolation {
            source: GatewayError::Protocol {
                message: "borrowed world callback was invoked more than once".to_owned(),
            },
        })
    );
    let outcome = facade
        .with_world_mut(|scene| scene.nodes().len())
        .expect("the first mutable callback should produce an outcome")
        .expect("the authoring world should remain loaded");
    let (node_count, post_callback_error) = outcome.into_parts();
    assert_eq!(node_count, 0);
    assert_eq!(
        post_callback_error,
        Some(AuthoringWorldAccessError::ProtocolViolation {
            source: GatewayError::Protocol {
                message: "borrowed world callback was invoked more than once".to_owned(),
            },
        })
    );

    let context = CoreEditContext::new(handle);
    assert_repeated_callback_protocol(
        context
            .with_scene(|scene| scene.nodes().len())
            .expect_err("duplicate read callback must fail"),
    );
    let outcome = context
        .with_scene_mut(|scene| scene.nodes().len())
        .expect("first mutable callback must expose its execution outcome");
    let (node_count, protocol_error) = outcome.into_parts();
    assert_eq!(node_count, 0);
    assert_repeated_callback_protocol(
        protocol_error.expect("duplicate mutable callback must be reported after execution"),
    );
}

#[test]
fn authoring_gateway_generation_mismatch_and_gateway_failure_remain_distinct_from_unloaded() {
    let handle = EditorRuntimeGatewayHandle::detached();
    let level =
        DefaultLevelManager::default().create_level(Scene::default(), LevelMetadata::default());
    let facade = EditorAuthoringWorld::loaded(&handle, AuthoringWorldSeed::from(level))
        .expect("initial authoring world");
    handle
        .replace(Arc::new(DetachedEditorRuntimeGateway))
        .expect("install detached gateway");

    let stale_error = facade
        .with_world(|scene| scene.nodes().len())
        .expect_err("a replacement must not become an unloaded world");
    assert!(facade.is_loaded());
    assert_eq!(
        stale_error,
        AuthoringWorldAccessError::StaleGeneration {
            expected_generation: 1,
            current_generation: 2,
        }
    );
    assert!(facade.should_report_access_failure(&stale_error));
    assert!(!facade.should_report_access_failure(&stale_error));

    facade
        .replace_gateway_for_test(Arc::new(DetachedEditorRuntimeGateway))
        .expect("install a detached current generation");
    let error = facade
        .with_world(|scene| scene.nodes().len())
        .expect_err("a detached gateway must not become an unloaded world");
    assert_eq!(
        error,
        AuthoringWorldAccessError::SerializedAccessRequired {
            source: GatewayError::RequiresSerializedAccess,
        }
    );
}
