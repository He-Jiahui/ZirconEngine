use crate::rhi::{DeviceGeneration, DeviceId, RenderDeviceProfile};

/// Device generation admitted for one runtime-prepare dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimePrepareDeviceEpoch {
    device_id: DeviceId,
    generation: DeviceGeneration,
}

impl RuntimePrepareDeviceEpoch {
    pub const fn new(device_id: DeviceId, generation: DeviceGeneration) -> Self {
        Self {
            device_id,
            generation,
        }
    }

    pub(crate) fn from_device_profile(device_profile: &RenderDeviceProfile) -> Self {
        Self::new(device_profile.device_id(), device_profile.generation())
    }

    pub const fn device_id(self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(self) -> DeviceGeneration {
        self.generation
    }
}
