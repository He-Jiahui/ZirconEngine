use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_runtime_interface::{
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
};

use crate::core::gateway::{
    EditorRuntimeGateway, PluginActivationState, PluginSummaryEntry, RuntimeCapabilities,
    SessionGateway, SessionProfileKind,
};

use super::fixture::{
    api_table, capabilities, fake_capture_owned_frame, gateway, OwnerDropProbe, FREED_OUTPUTS,
    OUTPUT_TEST_LOCK,
};

#[test]
fn session_gateway_keeps_the_runtime_provider_alive() {
    let drops = Arc::new(AtomicUsize::new(0));
    let owner: Arc<dyn Send + Sync> = Arc::new(OwnerDropProbe(drops.clone()));
    let gateway = unsafe {
        SessionGateway::new(
            owner,
            api_table(),
            ZrRuntimeSessionHandle::new(17),
            capabilities(),
        )
        .expect("valid session gateway")
    };

    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(gateway);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn session_gateway_retains_foreign_frame_storage_until_explicit_release() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let drops = Arc::new(AtomicUsize::new(0));
    let owner: Arc<dyn Send + Sync> = Arc::new(OwnerDropProbe(drops.clone()));
    let mut api = api_table();
    api.capture_frame = Some(fake_capture_owned_frame);
    let gateway = unsafe {
        SessionGateway::new(owner, api, ZrRuntimeSessionHandle::new(17), capabilities())
            .expect("valid session gateway")
    };

    let frame = gateway
        .capture_frame(
            ZrRuntimeViewportHandle::new(3),
            ZrRuntimeViewportSizeV1::new(1, 1),
        )
        .expect("retain runtime frame storage in the returned frame");

    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 0);
    drop(gateway);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(frame.abi_version(), 1);
    assert_eq!(frame.generation(), 31);
    assert_eq!(frame.rgba(), &[1, 2, 3, 4]);

    frame
        .release()
        .expect("release the provider-owned frame storage");
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn session_gateway_frame_drop_releases_runtime_storage_exactly_once() {
    let _output_test_guard = OUTPUT_TEST_LOCK.lock().expect("lock output test fixture");
    FREED_OUTPUTS.store(0, Ordering::SeqCst);
    let mut api = api_table();
    api.capture_frame = Some(fake_capture_owned_frame);

    let frame = gateway(api)
        .capture_frame(
            ZrRuntimeViewportHandle::new(3),
            ZrRuntimeViewportSizeV1::new(1, 1),
        )
        .expect("retain runtime frame storage before returning the frame");

    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 0);
    drop(frame);
    assert_eq!(FREED_OUTPUTS.load(Ordering::SeqCst), 1);
}

#[test]
fn runtime_capabilities_preserve_conflicts_in_deterministic_order() {
    let active =
        PluginSummaryEntry::new("zircon.navigation", "1.2.0", PluginActivationState::Active);
    let rejected = PluginSummaryEntry::new(
        "zircon.navigation",
        "1.2.0",
        PluginActivationState::Rejected,
    );
    let left = RuntimeCapabilities::new(
        SessionProfileKind::Editor,
        Vec::<String>::new(),
        [rejected.clone(), active.clone()],
    );
    let right = RuntimeCapabilities::new(
        SessionProfileKind::Editor,
        Vec::<String>::new(),
        [active, rejected],
    );

    assert_eq!(left, right);
    assert_eq!(left.plugin_summary().len(), 2);
    assert_eq!(
        left.plugin_summary()[0].activation(),
        PluginActivationState::Active
    );
    assert_eq!(
        left.plugin_summary()[1].activation(),
        PluginActivationState::Rejected
    );
}
