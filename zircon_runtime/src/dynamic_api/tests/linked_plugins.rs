use std::sync::Arc;

use serde::Serialize;

use super::support::*;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;
use crate::core::runtime::ServiceObject;
use crate::core::{DriverDescriptor, ModuleDescriptor, RegistryName, StartupMode};
use crate::dynamic_api::session::{
    RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES, RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES,
};
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
const LINKED_EVENT_BACKLOG: u32 = 10_000;

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
fn linked_plugin_event_drain_returns_empty_owned_buffer_for_idle_subscription() {
    let api = runtime_api();
    let session = create_linked_runtime_session(
        b"headless",
        None,
        vec![linked_plugin_registration(RuntimeTargetMode::ClientRuntime)],
    )
    .expect("linked runtime plugin session");
    let subscribe = api.subscribe_plugin_event.expect("subscribe plugin event");
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

    let mut output = ZrOwnedByteBuffer::empty();
    let status = unsafe { drain(session, subscription, &mut output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert!(output.is_empty());
    assert!(output.free.is_none());

    destroy_test_session(api, session);
}

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

    let mut demand = ZrRuntimeFrameDemandV1::idle();
    let status = unsafe { tick(session, &mut demand) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");

    let first_generation = drain_plugin_event_batch(drain, session, subscription)
        .expect("linked runtime tick must publish one event");
    assert_eq!(first_generation.deliveries.len(), 1);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(first_generation.deliveries[0].payload.get())
            .unwrap()["frame"],
        1
    );

    let status = unsafe { tick(session, &mut demand) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    let batch = drain_plugin_event_batch(drain, session, subscription)
        .expect("linked runtime tick must publish one event");

    assert_eq!(batch.deliveries.len(), 1);
    assert_eq!(batch.deliveries[0].event_id, EVENT_ID);
    assert_eq!(batch.deliveries[0].payload_schema, PAYLOAD_SCHEMA);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(batch.deliveries[0].payload.get()).unwrap()
            ["frame"],
        2
    );
    destroy_test_session(api, session);
}

#[test]
fn linked_plugin_event_drain_pages_a_large_backlog_through_runtime_abi() {
    let api = runtime_api();
    let session = create_linked_runtime_session(
        b"headless",
        None,
        vec![linked_plugin_registration_with_events_per_tick(
            RuntimeTargetMode::ClientRuntime,
            LINKED_EVENT_BACKLOG,
        )],
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

    let mut demand = ZrRuntimeFrameDemandV1::idle();
    let status = unsafe { tick(session, &mut demand) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");

    let mut delivered = 0u32;
    let mut page_count = 0usize;
    loop {
        let Some(batch) = drain_plugin_event_batch(drain, session, subscription) else {
            break;
        };
        page_count += 1;
        let page_deliveries = batch.deliveries.len();
        for delivery in batch.deliveries {
            delivered += 1;
            assert_eq!(delivery.event_id, EVENT_ID);
            assert_eq!(delivery.payload_schema, PAYLOAD_SCHEMA);
            assert_eq!(delivery.sequence, u64::from(delivered));
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(delivery.payload.get()).unwrap()["frame"],
                delivered
            );
        }
        assert_eq!(batch.remaining_deliveries, LINKED_EVENT_BACKLOG - delivered);
        assert!(page_deliveries <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES);
    }

    assert_eq!(delivered, LINKED_EVENT_BACKLOG);
    assert_eq!(
        page_count,
        (LINKED_EVENT_BACKLOG as usize).div_ceil(RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES)
    );
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
) -> Option<ZrRuntimePluginEventDeliveryBatchV1> {
    let mut output = ZrOwnedByteBuffer::empty();
    let status = unsafe { drain(session, subscription, &mut output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    if output.is_empty() {
        assert!(output.free.is_none());
        return None;
    }
    assert!(output.len <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES);
    let bytes = unsafe { core::slice::from_raw_parts(output.data, output.len) };
    let batch = serde_json::from_slice::<ZrRuntimePluginEventDeliveryBatchV1>(bytes).unwrap();
    assert!(batch.deliveries.len() <= RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES);
    free_output(output);
    Some(batch)
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

#[test]
fn linked_animation_plugin_registers_its_canonical_runtime_module_once() {
    let api = runtime_api();
    let session = create_linked_runtime_session(
        b"headless",
        None,
        vec![linked_animation_plugin_registration(
            RuntimeTargetMode::ClientRuntime,
        )],
    )
    .expect("linked animation runtime plugin session");

    destroy_test_session(api, session);
}

fn linked_plugin_registration(target: RuntimeTargetMode) -> RuntimePluginRegistrationReport {
    linked_plugin_registration_with_events_per_tick(target, 1)
}

fn linked_plugin_registration_with_events_per_tick(
    target: RuntimeTargetMode,
    events_per_tick: u32,
) -> RuntimePluginRegistrationReport {
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
    extensions
        .register_runtime_scene_system(
            owner,
            "navigation.linked_session_tick",
            SystemStage::Update,
            move || {
                let mut frame = 0;
                move |context| {
                    context
                        .core
                        .resolve_driver::<LinkedSessionDriver>(LINKED_DRIVER_NAME)?;
                    context.level.with_world_mut(|world| {
                        for _ in 0..events_per_tick {
                            frame += 1;
                            world.send_event(LinkedSessionTick { frame });
                        }
                    });
                    Ok(())
                }
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

fn linked_animation_plugin_registration(
    target: RuntimeTargetMode,
) -> RuntimePluginRegistrationReport {
    let module_name = "animation.runtime";
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions
        .register_module(ModuleDescriptor::new(
            module_name,
            "Linked animation session test module",
        ))
        .unwrap();

    RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new("animation", "Animation"),
        project_selection: ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Animation,
            true,
            false,
        )
        .with_target_modes([target]),
        extensions,
        diagnostics: Vec::new(),
    }
}
