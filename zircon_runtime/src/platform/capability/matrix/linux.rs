use super::super::backends::LinuxWindowProtocol;
use super::super::status::CapabilityStatus;
use super::PlatformCapabilityMatrix;
use crate::platform::PlatformTarget;

impl PlatformCapabilityMatrix {
    pub(super) fn linux_protocol(
        self,
        target: PlatformTarget,
        feature_enabled: bool,
        feature: &'static str,
        protocol: LinuxWindowProtocol,
    ) -> CapabilityStatus<LinuxWindowProtocol> {
        if target != PlatformTarget::Linux {
            return CapabilityStatus::Unavailable {
                reason: "protocol is linux-specific",
            };
        }

        if feature_enabled {
            CapabilityStatus::Supported(protocol)
        } else {
            CapabilityStatus::FeatureDisabled { feature }
        }
    }
}
