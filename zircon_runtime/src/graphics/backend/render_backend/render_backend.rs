use super::config::RenderBackendConfig;
use crate::rhi::RenderBackendCaps;
use crate::rhi_wgpu::wgpu_backend_caps;

pub(crate) struct RenderBackend {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) backend_name: String,
    pub(crate) config: RenderBackendConfig,
}

impl RenderBackend {
    pub(crate) const RETAINED_STATE_OWNER_COUNT: usize = 3;

    pub(crate) fn retained_state_owner_count(&self) -> usize {
        let _retained_state_owners = (&self.instance, &self.adapter, &self.config);
        Self::RETAINED_STATE_OWNER_COUNT
    }

    pub(crate) fn caps(&self) -> RenderBackendCaps {
        debug_assert_eq!(
            self.retained_state_owner_count(),
            Self::RETAINED_STATE_OWNER_COUNT,
            "RenderBackend must retain instance, adapter, and config owners while reporting caps",
        );
        wgpu_backend_caps(
            self.backend_name.clone(),
            self.adapter.features(),
            self.device.limits(),
            false,
            self.adapter
                .get_downlevel_capabilities()
                .flags
                .contains(wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE),
        )
    }
}
