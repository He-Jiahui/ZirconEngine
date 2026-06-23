#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderingBackendInfo {
    pub backend_name: String,
    pub supports_runtime_preview: bool,
    pub supports_shared_texture_viewports: bool,
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
