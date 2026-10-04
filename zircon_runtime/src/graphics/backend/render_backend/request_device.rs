use crate::graphics::resource_limits::{
    HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
    POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
    REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
};
use crate::graphics::types::GraphicsError;
use zr_rhi::{RenderDeviceRequestFailure, RenderDeviceRequestPolicy};
use zr_rhi_wgpu::{
    wgpu_adapter_facts, wgpu_device_limits, wgpu_device_request, WgpuDeviceRequest,
    WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES,
};

const REQUIRED_RENDER_BIND_GROUP_LIMIT: u32 = 5;
const BINDLESS_MATERIAL_MIN_SLOT_COUNT: u32 = 2;
// Sampler-array limits are the narrower WGPU binding-array limit. Cap the initial material slab
// so the negotiated table remains portable when the renderer switches away from per-material
// texture groups.
const BINDLESS_MATERIAL_MAX_SLOT_COUNT: u32 = 1_000;
pub(super) struct RequestedDevice {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) profile_request: WgpuDeviceRequest,
}

pub(super) fn request_device(adapter: &wgpu::Adapter) -> Result<RequestedDevice, GraphicsError> {
    request_device_with_policy(adapter, &RenderDeviceRequestPolicy::mvp_baseline())
}

pub(super) fn request_device_with_policy(
    adapter: &wgpu::Adapter,
    policy: &RenderDeviceRequestPolicy,
) -> Result<RequestedDevice, GraphicsError> {
    let adapter_features = adapter.features();
    let profile_request = wgpu_device_request(adapter_features, policy)?;
    let requested_features = profile_request.requested_features();
    let requested_limits = required_render_limits(&adapter.limits(), requested_features);
    let adapter_facts = wgpu_adapter_facts(&adapter.get_info(), adapter_features);
    let failure_feature_negotiation = profile_request.feature_negotiation().clone();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-device"),
        required_features: requested_features,
        required_limits: requested_limits.clone(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(|error| {
        GraphicsError::DeviceRequest(RenderDeviceRequestFailure::new(
            adapter_facts,
            failure_feature_negotiation,
            wgpu_device_limits(&requested_limits),
            error.to_string(),
        ))
    })?;

    Ok(RequestedDevice {
        device,
        queue,
        profile_request,
    })
}

fn required_render_features(
    adapter_features: wgpu::Features,
    policy: &RenderDeviceRequestPolicy,
) -> Result<wgpu::Features, GraphicsError> {
    Ok(wgpu_device_request(adapter_features, policy)?.requested_features())
}

fn required_render_limits(
    adapter_limits: &wgpu::Limits,
    requested_features: wgpu::Features,
) -> wgpu::Limits {
    let mut limits = wgpu::Limits {
        max_bind_groups: REQUIRED_RENDER_BIND_GROUP_LIMIT,
        max_sampled_textures_per_shader_stage:
            POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE,
        max_texture_array_layers: REFLECTION_PROBE_REQUIRED_TEXTURE_ARRAY_LAYERS,
        ..wgpu::Limits::default()
    };
    let required_storage_buffers_per_shader_stage =
        HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
            .max(OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE);
    if adapter_limits.max_storage_buffers_per_shader_stage
        >= required_storage_buffers_per_shader_stage
    {
        limits.max_storage_buffers_per_shader_stage = required_storage_buffers_per_shader_stage;
    } else if adapter_limits.max_storage_buffers_per_shader_stage
        >= MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
    {
        limits.max_storage_buffers_per_shader_stage =
            MESH_FORWARD_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
    }
    if requested_features.contains(WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES) {
        let binding_array_slot_count = adapter_limits
            .max_binding_array_elements_per_shader_stage
            .min(adapter_limits.max_binding_array_sampler_elements_per_shader_stage)
            .min(BINDLESS_MATERIAL_MAX_SLOT_COUNT);
        if binding_array_slot_count >= BINDLESS_MATERIAL_MIN_SLOT_COUNT {
            limits.max_binding_array_elements_per_shader_stage = binding_array_slot_count;
            limits.max_binding_array_sampler_elements_per_shader_stage = binding_array_slot_count;
        }
    }
    limits
}

#[cfg(test)]
#[path = "tests/request_device.rs"]
mod tests;
