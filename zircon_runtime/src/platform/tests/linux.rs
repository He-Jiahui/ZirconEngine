use super::super::*;

#[test]
fn linux_protocols_are_declared_independently() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.platform_wayland = false;

    let report = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        report.linux_x11,
        CapabilityStatus::Supported(LinuxWindowProtocol::X11)
    );
    assert_eq!(
        report.linux_wayland,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-wayland"
        }
    );
}
