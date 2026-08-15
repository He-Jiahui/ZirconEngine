use std::time::Duration;

use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ZrRuntimeApiV6, ZrRuntimeEventV1,
    ZrRuntimeTickFrameFnV2, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
};

use crate::core::gateway::{EditorRuntimeFrameDemand, EditorRuntimeGateway, GatewayError};

use super::fixture::{
    api_table, fake_tick_after, fake_tick_after_maximum_delay, fake_tick_idle_with_delay,
    fake_tick_invalid_demand_abi, fake_tick_leaves_demand_untouched, fake_tick_unknown_demand_kind,
    gateway, EVENT_CALLS, TICK_CALLS,
};

#[test]
fn session_gateway_forwards_abi_tick_event_and_frame_calls() {
    TICK_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
    EVENT_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
    let gateway = gateway(api_table());

    assert_eq!(
        gateway.tick_frame().expect("tick runtime session"),
        EditorRuntimeFrameDemand::Continuous
    );
    gateway
        .handle_event(ZrRuntimeEventV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            0,
            ZrRuntimeViewportHandle::new(3),
        ))
        .expect("forward runtime event");
    let frame = gateway
        .capture_frame(
            ZrRuntimeViewportHandle::new(3),
            ZrRuntimeViewportSizeV1::new(640, 360),
        )
        .expect("capture runtime frame");

    assert_eq!(TICK_CALLS.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(EVENT_CALLS.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!((frame.width(), frame.height()), (640, 360));
    assert!(frame.rgba().is_empty());
}

#[test]
fn session_gateway_reports_missing_optional_pointer_as_typed_capability_error() {
    let gateway = gateway(ZrRuntimeApiV6::empty());

    assert_eq!(
        gateway.tick_frame(),
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.frame.tick",
        })
    );
}

#[test]
fn session_gateway_initializes_runtime_frame_demand_before_tick() {
    let mut api = api_table();
    api.tick_frame = Some(fake_tick_leaves_demand_untouched);

    assert_eq!(
        gateway(api)
            .tick_frame()
            .expect("accept initialized idle demand"),
        EditorRuntimeFrameDemand::OnDemand
    );
}

#[test]
fn session_gateway_preserves_runtime_sleep_until_delay() {
    let mut api = api_table();
    api.tick_frame = Some(fake_tick_after);

    assert_eq!(
        gateway(api)
            .tick_frame()
            .expect("map delayed runtime demand"),
        EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(25))
    );
}

#[test]
fn session_gateway_bounds_runtime_frame_wake_delay() {
    let mut api = api_table();
    api.tick_frame = Some(fake_tick_after_maximum_delay);

    assert_eq!(
        gateway(api)
            .tick_frame()
            .expect("bound an extreme runtime delay to a host-safe wake"),
        EditorRuntimeFrameDemand::SleepUntil(Duration::from_secs(60))
    );
}

#[test]
fn session_gateway_rejects_malformed_runtime_frame_demand() {
    for tick_frame in [
        fake_tick_invalid_demand_abi as ZrRuntimeTickFrameFnV2,
        fake_tick_unknown_demand_kind,
        fake_tick_idle_with_delay,
    ] {
        let mut api = api_table();
        api.tick_frame = Some(tick_frame);

        let error = gateway(api)
            .tick_frame()
            .expect_err("malformed runtime frame demand must cross no editor boundary");

        assert!(matches!(error, GatewayError::Protocol { .. }));
        assert!(error.to_string().contains("frame demand"));
    }
}

#[test]
fn session_gateway_reports_an_unavailable_optional_profile_control_as_none() {
    let gateway = gateway(api_table());

    let response = gateway
        .profile_control(&ProfileControlRequest {
            command: ProfileControlCommand::Snapshot,
            config: None,
        })
        .expect("query optional runtime profile control");

    assert_eq!(response, None);
}
