use super::super::*;

#[test]
fn browser_gamepad_backend_is_separate_from_gilrs() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.gamepad_gilrs = true;
    features.gamepad_browser = false;

    let without_browser_backend = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::WebGpu,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        without_browser_backend.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        without_browser_backend.gamepad_events,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        without_browser_backend.gamepad_rumble,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );

    features.gamepad_browser = true;
    let with_browser_backend = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::WebGpu,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        with_browser_backend.gamepad_input,
        CapabilityStatus::Supported(GamepadBackend::BrowserGamepadApi)
    );
    assert_eq!(
        with_browser_backend.gamepad_events,
        CapabilityStatus::Supported(GamepadEventBackend::BrowserGamepadApiPolling)
    );
    assert_eq!(
        with_browser_backend.gamepad_rumble,
        CapabilityStatus::Unavailable {
            reason: "browser gamepad rumble host backend is not implemented yet"
        }
    );
}
