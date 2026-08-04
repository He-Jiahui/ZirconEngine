#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderingBackendInfo {
    pub backend_name: String,
    pub supports_runtime_preview: bool,
    pub supports_shared_texture_viewports: bool,
}

/// Backend-neutral render-device identity and the limits available to the renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderDeviceDiagnostics {
    pub adapter_name: String,
    pub adapter_device_type: String,
    pub limits: RenderDeviceLimitDiagnostics,
}

/// Curated render-device limits required to explain runtime renderer behavior.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderDeviceLimitDiagnostics {
    pub max_bind_groups: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_array_layers: u32,
    pub max_sampled_textures_per_shader_stage: u32,
    pub max_binding_array_elements_per_shader_stage: u32,
    pub max_binding_array_sampler_elements_per_shader_stage: u32,
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_storage_buffer_binding_size: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphicsDebuggerStatus {
    /// True when the backend exposes a graphics-debugger capture hook through wgpu.
    /// This does not prove that RenderDoc or another debugger is attached.
    pub available: bool,
    /// Concrete backend selected by wgpu, for example `wgpu(dx12)` or `wgpu(vulkan)`.
    pub backend_name: String,
    pub capture_pending: bool,
    pub active_capture: bool,
    pub last_capture_frame: Option<u64>,
    pub last_error: Option<String>,
}

impl GraphicsDebuggerStatus {
    pub fn unavailable(backend_name: impl Into<String>) -> Self {
        Self {
            backend_name: backend_name.into(),
            ..Self::default()
        }
    }
}
