//! GPU device and surface management.

mod render_backend;

#[cfg(test)]
pub(crate) use render_backend::read_buffer_f32x4;
#[cfg(test)]
pub(crate) use render_backend::read_texture_rgba16float_3d;
#[cfg(test)]
pub(crate) use render_backend::RenderBackendConfig;
#[allow(unused_imports)]
pub(crate) use render_backend::{
    read_buffer_bytes, read_buffer_f32x4_array_bytes, read_buffer_sh9_f32x4_bytes,
    read_ibl_bake_artifact_wgpu_sections, read_texture_rgba16float_cube_mip_chain,
    read_texture_rgba16float_region, BufferByteReadback, IblBakeArtifactWgpuReadbackResources,
    Rgba16FloatTextureRegionReadback,
};
pub(crate) use render_backend::{
    read_texture_rgba, GraphicsDebuggerCaptureStop, OffscreenTarget, RenderBackend, ViewportSurface,
};
