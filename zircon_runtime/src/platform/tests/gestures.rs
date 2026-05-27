use super::super::*;

#[test]
fn gesture_event_capabilities_declare_feature_gate_and_missing_host_paths() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();

    let default_desktop = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Macos,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        default_desktop.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );

    features.input_gestures = true;

    let macos = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Macos,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        macos.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "winit gesture event host backend is not implemented yet"
        }
    );

    let ios = PlatformCapabilityMatrix::new(features)
        .report(PlatformTarget::Ios, crate::RuntimeTargetMode::ClientRuntime);
    assert_eq!(
        ios.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "winit gesture event host backend is not implemented yet"
        }
    );

    let windows = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        windows.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "winit gesture events are only declared for macOS and iOS targets"
        }
    );

    let browser = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Wasm,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        browser.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "browser gesture event host backend is not implemented yet"
        }
    );
}
