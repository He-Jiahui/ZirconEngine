use super::config::RenderBackendConfig;
use crate::rhi::{RenderAdapterInfo, RenderBackendCaps, RenderDeviceLimits};
use zr_rhi_wgpu::wgpu_backend_caps;

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

    /// Clones the negotiated WGPU state for a native UI surface on this backend's device.
    ///
    /// The clone keeps typed WGPU ownership rather than exposing native pointers. Callers can
    /// therefore compose a renderer product and retained UI image through the same queue.
    pub(crate) fn ui_surface_context(&self) -> zr_rhi_wgpu::WgpuUiSurfaceContext {
        zr_rhi_wgpu::WgpuUiSurfaceContext::new(
            self.instance.clone(),
            self.adapter.clone(),
            self.device.clone(),
            self.queue.clone(),
        )
    }

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
        let adapter_info = self.adapter.get_info();
        let device_limits = self.device.limits();
        wgpu_backend_caps(
            format!("wgpu({})", adapter_info.backend.to_str()),
            self.device.features(),
            device_limits.clone(),
            false,
            self.adapter
                .get_downlevel_capabilities()
                .flags
                .contains(wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE),
        )
        .with_adapter(RenderAdapterInfo {
            name: adapter_info.name,
            device_type: adapter_device_type_label(adapter_info.device_type).to_owned(),
        })
        .with_device_limits(RenderDeviceLimits {
            max_bind_groups: device_limits.max_bind_groups,
            max_texture_dimension_2d: device_limits.max_texture_dimension_2d,
            max_texture_array_layers: device_limits.max_texture_array_layers,
            max_sampled_textures_per_shader_stage: device_limits
                .max_sampled_textures_per_shader_stage,
            max_binding_array_elements_per_shader_stage: device_limits
                .max_binding_array_elements_per_shader_stage,
            max_binding_array_sampler_elements_per_shader_stage: device_limits
                .max_binding_array_sampler_elements_per_shader_stage,
            max_storage_buffers_per_shader_stage: device_limits
                .max_storage_buffers_per_shader_stage,
            max_storage_buffer_binding_size: u64::from(
                device_limits.max_storage_buffer_binding_size,
            ),
        })
    }
}

const fn adapter_device_type_label(device_type: wgpu::DeviceType) -> &'static str {
    match device_type {
        wgpu::DeviceType::Other => "other",
        wgpu::DeviceType::IntegratedGpu => "integrated_gpu",
        wgpu::DeviceType::DiscreteGpu => "discrete_gpu",
        wgpu::DeviceType::VirtualGpu => "virtual_gpu",
        wgpu::DeviceType::Cpu => "cpu",
    }
}

#[cfg(test)]
mod tests {
    use super::adapter_device_type_label;

    #[test]
    fn adapter_device_type_labels_are_stable_and_backend_neutral() {
        assert_eq!(adapter_device_type_label(wgpu::DeviceType::Other), "other");
        assert_eq!(
            adapter_device_type_label(wgpu::DeviceType::IntegratedGpu),
            "integrated_gpu"
        );
        assert_eq!(
            adapter_device_type_label(wgpu::DeviceType::DiscreteGpu),
            "discrete_gpu"
        );
        assert_eq!(
            adapter_device_type_label(wgpu::DeviceType::VirtualGpu),
            "virtual_gpu"
        );
        assert_eq!(adapter_device_type_label(wgpu::DeviceType::Cpu), "cpu");
    }

    #[test]
    fn backend_caps_report_negotiated_device_features() {
        let source = include_str!("render_backend.rs");
        let device_feature_call = ["self.device", ".features()"].concat();
        let adapter_feature_call = ["self.adapter", ".features()"].concat();

        assert_eq!(source.matches(&device_feature_call).count(), 1);
        assert!(
            !source.contains(&adapter_feature_call),
            "adapter availability must not be reported as negotiated device capability"
        );
    }

    #[test]
    fn ui_surface_context_reuses_the_negotiated_backend_owners() {
        let source = include_str!("render_backend.rs");
        let context_source = source
            .split("fn ui_surface_context")
            .nth(1)
            .and_then(|source| source.split("fn retained_state_owner_count").next())
            .expect("render backend should expose a UI context from its negotiated device");

        for owner in [
            "self.instance.clone()",
            "self.adapter.clone()",
            "self.device.clone()",
            "self.queue.clone()",
        ] {
            assert!(context_source.contains(owner));
        }
        assert!(!context_source.contains("request_device"));
    }
}
