use crate::{plugin::CapabilityStatus, plugin::CapabilityStatusManifest};

pub(super) fn capability_status(
    capability: impl Into<String>,
    status: CapabilityStatus,
) -> CapabilityStatusManifest {
    CapabilityStatusManifest::new(capability, status)
}
