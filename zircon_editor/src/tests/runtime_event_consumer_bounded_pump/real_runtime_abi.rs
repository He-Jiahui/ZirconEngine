use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use zircon_runtime::{
    builtin::RuntimePluginId,
    core::framework::{platform::RuntimeTargetMode, project::ProjectPluginSelection},
    core::runtime::ServiceObject,
    core::{DriverDescriptor, ModuleDescriptor, RegistryName, StartupMode},
    dynamic_api::{create_linked_runtime_session, zircon_runtime_get_api_v6},
    engine_module::factory,
    plugin::{
        PluginEventManifest, PluginPackageManifest, RuntimeExtensionRegistry,
        RuntimePluginRegistrationReport,
    },
    scene::SystemStage,
};
use zircon_runtime_interface::{
    ZrRuntimeApiV6, ZrRuntimeFrameDemandV1, ZrRuntimeSessionHandle, ZrStatusCode,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

use crate::core::gateway::{EditorRuntimeGatewayHandle, RuntimeCapabilities, SessionGateway};
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerHost, EditorRuntimeEventPumpBudget,
};

use super::{percentile_index, register_state, RecordingState, CAPABILITY};

#[derive(Clone, Debug, Serialize)]
struct RealRuntimeAbiEvent {
    value: u64,
}

#[derive(Debug)]
struct RealRuntimeAbiDriver;

const REAL_RUNTIME_ABI_EVENT_ID: &str = "navigation.events.editor_real_abi_tick";
const REAL_RUNTIME_ABI_PAYLOAD_SCHEMA: &str = "navigation.schemas.editor_real_abi_tick.v1";
const REAL_RUNTIME_ABI_DRIVER_NAME: &str = "navigation.runtime.Driver.EditorRealAbi";

#[test]
#[ignore = "managed end-to-end runtime ABI evidence; run alone with --test-threads=1"]
fn managed_real_runtime_abi_thousand_and_ten_thousand_delivery_budget_report() {
    let reports = [1_000_u64, 10_000]
        .into_iter()
        .map(run_real_runtime_abi_delivery_storm)
        .collect::<Vec<_>>();

    println!(
        "PLUGINS01_REAL_RUNTIME_EVENT_ABI_PUMP_BENCHMARK={}",
        serde_json::Value::Array(reports)
    );
}

fn run_real_runtime_abi_delivery_storm(delivery_count: u64) -> serde_json::Value {
    const MAX_EVENTS_PER_TICK: usize = 64;

    let api = real_runtime_api();
    let session = create_linked_runtime_session(
        b"headless",
        None,
        vec![real_runtime_abi_plugin_registration(
            u32::try_from(delivery_count).expect("benchmark event count fits u32"),
        )],
    )
    .expect("construct linked runtime session for the editor ABI pump");
    let gateway = Arc::new(unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            session,
            RuntimeCapabilities::editor_default(),
        )
        .expect("construct editor gateway over the real runtime API")
    });
    let host = EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(gateway));
    let state = Arc::new(Mutex::new(RecordingState::default()));
    register_state(
        &host,
        "tests.consumer.real_runtime_abi",
        REAL_RUNTIME_ABI_EVENT_ID,
        state.clone(),
    );
    host.begin_play_session(700, &[CAPABILITY.to_string()])
        .expect("start real runtime ABI event consumer session");

    let tick = api.tick_frame.expect("real runtime API exposes tick_frame");
    let mut demand = ZrRuntimeFrameDemandV1::idle();
    let status = unsafe { tick(session, &mut demand) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok);

    let mut tick_durations = Vec::new();
    let mut runtime_drain_durations = Vec::new();
    let mut decode_durations = Vec::new();
    let mut applied = 0_usize;
    let mut max_applied_per_tick = 0_usize;
    let mut max_drained_per_tick = 0_usize;
    let mut max_page_bytes = 0_usize;
    let mut pending_peak = 0_usize;
    let mut last_observed_runtime_remaining_peak = 0_usize;

    for _ in 0..delivery_count as usize {
        if applied == delivery_count as usize {
            break;
        }
        let started = Instant::now();
        let report = host
            .pump_with_budget(EditorRuntimeEventPumpBudget::new(
                MAX_EVENTS_PER_TICK,
                MAX_EVENTS_PER_TICK,
                Duration::from_secs(1),
                Duration::from_millis(1),
            ))
            .expect("real runtime ABI page should pump through the normal editor path");
        tick_durations.push(started.elapsed());
        runtime_drain_durations.push(report.runtime_drain_elapsed());
        decode_durations.push(report.decode_elapsed());
        applied = applied.saturating_add(report.applied());
        max_applied_per_tick = max_applied_per_tick.max(report.applied());
        max_drained_per_tick = max_drained_per_tick.max(report.drained());
        max_page_bytes = max_page_bytes.max(report.drained_encoded_bytes());
        pending_peak = pending_peak.max(report.queue_depth());
        let last_observed_runtime_remaining = report
            .last_observed_runtime_remaining_deliveries()
            .expect("each real ABI pump tick drains a complete runtime page");
        last_observed_runtime_remaining_peak =
            last_observed_runtime_remaining_peak.max(last_observed_runtime_remaining);

        assert!(report.applied() <= MAX_EVENTS_PER_TICK);
        assert!(
            report.applied() > 0,
            "real runtime ABI event pump made no progress with {delivery_count} deliveries pending"
        );
        assert!(report.drained() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1);
        assert!(
            report.drained_encoded_bytes() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1
        );
        assert_eq!(report.dropped(), 0);
        assert_eq!(
            last_observed_runtime_remaining,
            delivery_count as usize - applied
        );
    }

    assert_eq!(applied, delivery_count as usize);
    assert_eq!(
        state
            .lock()
            .expect("read real ABI consumer state")
            .sequences,
        (1..=delivery_count).collect::<Vec<_>>()
    );
    host.end_play_session(700)
        .expect("stop real runtime ABI event consumer session");
    destroy_real_runtime_session(api, session);

    tick_durations.sort_unstable();
    runtime_drain_durations.sort_unstable();
    decode_durations.sort_unstable();
    let p95_index = percentile_index(tick_durations.len());
    serde_json::json!({
        "deliveries": delivery_count,
        "ticks": tick_durations.len(),
        "max_events_per_tick": MAX_EVENTS_PER_TICK,
        "max_applied_per_tick": max_applied_per_tick,
        "max_drained_per_tick": max_drained_per_tick,
        "max_page_bytes": max_page_bytes,
        "pending_peak": pending_peak,
        "last_observed_runtime_remaining_peak": last_observed_runtime_remaining_peak,
        "tick_p95_ns": u64::try_from(tick_durations[p95_index].as_nanos()).unwrap_or(u64::MAX),
        "runtime_drain_p95_ns": u64::try_from(runtime_drain_durations[p95_index].as_nanos())
            .unwrap_or(u64::MAX),
        "decode_p95_ns": u64::try_from(decode_durations[p95_index].as_nanos())
            .unwrap_or(u64::MAX),
    })
}

fn real_runtime_api() -> ZrRuntimeApiV6 {
    let api = unsafe { zircon_runtime_get_api_v6(std::ptr::null()) };
    assert!(!api.is_null(), "real runtime API table should be available");
    unsafe { std::ptr::read(api) }
}

fn destroy_real_runtime_session(api: ZrRuntimeApiV6, session: ZrRuntimeSessionHandle) {
    let destroy = api
        .destroy_session
        .expect("real runtime API exposes destroy_session");
    let status = unsafe { destroy(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok);
}

fn real_runtime_abi_plugin_registration(events_per_tick: u32) -> RuntimePluginRegistrationReport {
    let module_name = "navigation.runtime";
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions
        .register_module(
            ModuleDescriptor::new(module_name, "Real editor ABI transport test module")
                .with_driver(DriverDescriptor::new(
                    RegistryName::new(REAL_RUNTIME_ABI_DRIVER_NAME).expect("valid driver name"),
                    StartupMode::Lazy,
                    Vec::new(),
                    factory(|_| Ok(Arc::new(RealRuntimeAbiDriver) as ServiceObject)),
                )),
        )
        .expect("register real editor ABI transport test module");
    let owner = extensions
        .intern_plugin_module(module_name)
        .expect("intern real editor ABI test module");
    extensions
        .register_mirrored_event::<RealRuntimeAbiEvent>(
            owner,
            PluginEventManifest {
                id: REAL_RUNTIME_ABI_EVENT_ID.to_string(),
                display_name: "Real Editor ABI Tick".to_string(),
                payload_schema: REAL_RUNTIME_ABI_PAYLOAD_SCHEMA.to_string(),
            },
            |_world, _reader_count| Ok(()),
        )
        .expect("register real editor ABI mirrored event");
    let mut value = 0_u64;
    extensions
        .register_runtime_scene_system(
            owner,
            "navigation.editor_real_abi_tick",
            SystemStage::Update,
            move |context| {
                context
                    .core
                    .resolve_driver::<RealRuntimeAbiDriver>(REAL_RUNTIME_ABI_DRIVER_NAME)?;
                context.level.with_world_mut(|world| {
                    for _ in 0..events_per_tick {
                        value = value.saturating_add(1);
                        world.send_event(RealRuntimeAbiEvent { value });
                    }
                });
                Ok(())
            },
        )
        .register()
        .expect("register real editor ABI event producer system");

    RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new("navigation", "Navigation"),
        project_selection: ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            true,
            false,
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        extensions,
        diagnostics: Vec::new(),
    }
}
