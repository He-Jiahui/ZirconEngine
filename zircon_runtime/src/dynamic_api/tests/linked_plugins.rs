use std::sync::Arc;

use serde::Serialize;

use super::support::*;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;
use crate::core::runtime::ServiceObject;
use crate::core::{DriverDescriptor, ModuleDescriptor, RegistryName, StartupMode};
use crate::engine_module::factory;
use crate::plugin::{
    PluginEventManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimePluginRegistrationReport,
};
use crate::scene::SystemStage;
use crate::{
    builtin::RuntimePluginId,
    dynamic_api::{create_linked_runtime_session, RuntimeDynamicSessionError},
};
use zircon_runtime_interface::{
    ZrOwnedByteBuffer, ZrRuntimePluginEventDeliveryBatchV1, ZrRuntimePluginEventSubscribeRequestV1,
    ZrRuntimePluginEventSubscriptionHandle,
};

const EVENT_ID: &str = "navigation.events.linked_session_tick";
const PAYLOAD_SCHEMA: &str = "navigation.schemas.linked_session_tick.v1";
const LINKED_DRIVER_NAME: &str = "navigation.runtime.Driver.LinkedSession";

#[test]
fn linked_runtime_session_rejects_unknown_profile_with_typed_error() {
    assert!(matches!(
        create_linked_runtime_session(b"unknown-profile", None, Vec::new()),
        Err(RuntimeDynamicSessionError::UnknownProfile { .. })
    ));
}

#[derive(Clone, Debug, Serialize)]
struct LinkedSessionTick {
    frame: u32,
}

#[derive(Debug)]
struct LinkedSessionDriver;

#[test]
fn linked_plugin_registration_ticks_and_drains_through_runtime_api() {
    let api = runtime_api();
    let session = create_linked_runtime_session(
        b"headless",
        None,
        vec![linked_plugin_registration(RuntimeTargetMode::ClientRuntime)],
    )
    .expect("linked runtime plugin session");
    let subscribe = api.subscribe_plugin_event.expect("subscribe plugin event");
    let tick = api.tick_frame.expect("tick frame");
    let drain = api.drain_plugin_events.expect("drain plugin events");

    let request = serde_json::to_vec(&ZrRuntimePluginEventSubscribeRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        EVENT_ID,
        PAYLOAD_SCHEMA,
    ))
    .unwrap();
    let mut subscription = ZrRuntimePluginEventSubscriptionHandle::invalid();
    let status = unsafe {
        subscribe(
            session,
            ZrByteSlice {
                data: request.as_ptr(),
                len: request.len(),
            },
            &mut subscription,
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");

    let status = unsafe { tick(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");

    let first_generation = drain_plugin_event_batch(drain, session, subscription);
    assert!(first_generation.deliveries.is_empty());

    let status = unsafe { tick(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    let batch = drain_plugin_event_batch(drain, session, subscription);

    assert_eq!(batch.deliveries.len(), 1);
    assert_eq!(batch.deliveries[0].event_id, EVENT_ID);
    assert_eq!(batch.deliveries[0].payload_schema, PAYLOAD_SCHEMA);
    assert_eq!(batch.deliveries[0].payload["frame"], 1);
    destroy_test_session(api, session);
}

fn drain_plugin_event_batch(
    drain: unsafe extern "C" fn(
        ZrRuntimeSessionHandle,
        ZrRuntimePluginEventSubscriptionHandle,
        *mut ZrOwnedByteBuffer,
    ) -> ZrStatus,
    session: ZrRuntimeSessionHandle,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrRuntimePluginEventDeliveryBatchV1 {
    let mut output = ZrOwnedByteBuffer::empty();
    let status = unsafe { drain(session, subscription, &mut output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    let bytes = unsafe { core::slice::from_raw_parts(output.data, output.len) };
    let batch = serde_json::from_slice::<ZrRuntimePluginEventDeliveryBatchV1>(bytes).unwrap();
    free_output(output);
    batch
}

#[test]
fn linked_plugin_editor_profile_composes_editor_host_selection() {
    let api = runtime_api();
    let session = create_linked_runtime_session(
        b"editor",
        None,
        vec![linked_plugin_registration(RuntimeTargetMode::EditorHost)],
    )
    .expect("linked editor runtime plugin session");
    let subscribe = api.subscribe_plugin_event.expect("subscribe plugin event");
    let request = serde_json::to_vec(&ZrRuntimePluginEventSubscribeRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        EVENT_ID,
        PAYLOAD_SCHEMA,
    ))
    .unwrap();
    let mut subscription = ZrRuntimePluginEventSubscriptionHandle::invalid();

    let status = unsafe {
        subscribe(
            session,
            ZrByteSlice {
                data: request.as_ptr(),
                len: request.len(),
            },
            &mut subscription,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    destroy_test_session(api, session);
}

fn linked_plugin_registration(target: RuntimeTargetMode) -> RuntimePluginRegistrationReport {
    let module_name = "navigation.runtime";
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions
        .register_module(
            ModuleDescriptor::new(module_name, "Linked session test module").with_driver(
                DriverDescriptor::new(
                    RegistryName::new(LINKED_DRIVER_NAME).unwrap(),
                    StartupMode::Lazy,
                    Vec::new(),
                    factory(|_| Ok(Arc::new(LinkedSessionDriver) as ServiceObject)),
                ),
            ),
        )
        .unwrap();
    let owner = extensions.intern_plugin_module(module_name).unwrap();
    extensions
        .register_mirrored_event::<LinkedSessionTick>(
            owner,
            PluginEventManifest {
                id: EVENT_ID.to_string(),
                display_name: "Linked Session Tick".to_string(),
                payload_schema: PAYLOAD_SCHEMA.to_string(),
            },
            |_world, _reader_count| Ok(()),
        )
        .unwrap();
    let mut frame = 0;
    extensions
        .register_runtime_scene_system(
            owner,
            "navigation.linked_session_tick",
            SystemStage::Update,
            move |context| {
                context
                    .core
                    .resolve_driver::<LinkedSessionDriver>(LINKED_DRIVER_NAME)?;
                frame += 1;
                context
                    .level
                    .with_world_mut(|world| world.send_event(LinkedSessionTick { frame }));
                Ok(())
            },
        )
        .register()
        .unwrap();

    RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new("navigation", "Navigation"),
        project_selection: ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            true,
            false,
        )
        .with_target_modes([target]),
        extensions,
        diagnostics: Vec::new(),
    }
}
