use std::sync::atomic::{AtomicU64, Ordering};
use zr_rhi::{
    DeviceGeneration, DeviceId, DiagnosticReadbackBudget, GpuMemoryBudget, RenderAdapterClass,
    RenderAdapterFacts, RenderBackendKind, RenderDeviceFeature, RenderDeviceFeatureNegotiation,
    RenderDeviceFeatureSet, RenderDeviceLimits, RenderDeviceNegotiationError, RenderDeviceProfile,
    RenderDeviceQueueTopology, RenderDeviceRequestPolicy, SubmissionLimits,
};

use crate::GPU_TIMESTAMP_REQUIRED_FEATURES;

static NEXT_WGPU_DEVICE_ID: AtomicU64 = AtomicU64::new(1);

pub const WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES: wgpu::Features =
    wgpu::Features::TEXTURE_BINDING_ARRAY
        .union(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY)
        .union(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING);

/// Immutable request receipt shared by WGPU device creation and the neutral device profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WgpuDeviceRequest {
    requested_features: wgpu::Features,
    feature_negotiation: RenderDeviceFeatureNegotiation,
}

impl WgpuDeviceRequest {
    pub const fn requested_features(&self) -> wgpu::Features {
        self.requested_features
    }

    pub const fn feature_negotiation(&self) -> &RenderDeviceFeatureNegotiation {
        &self.feature_negotiation
    }
}

/// Converts an RHI policy receipt into the WGPU feature bitset for one device request.
pub fn wgpu_features_for_device_request(
    adapter_features: wgpu::Features,
    policy: &RenderDeviceRequestPolicy,
) -> Result<wgpu::Features, RenderDeviceNegotiationError> {
    Ok(wgpu_device_request(adapter_features, policy)?.requested_features())
}

/// Creates an explicit WGPU request and preserves the neutral negotiation receipt.
pub fn wgpu_device_request(
    adapter_features: wgpu::Features,
    policy: &RenderDeviceRequestPolicy,
) -> Result<WgpuDeviceRequest, RenderDeviceNegotiationError> {
    let negotiation = policy.negotiate(&wgpu_device_features(adapter_features))?;
    let mut requested_features = wgpu::Features::empty();

    for feature in negotiation.requested_features().iter() {
        requested_features |= wgpu_features_for_neutral_feature(feature);
    }

    Ok(WgpuDeviceRequest {
        requested_features,
        feature_negotiation: negotiation,
    })
}

/// Converts concrete WGPU adapter data to the neutral, serializable selection record.
pub fn wgpu_adapter_facts(
    adapter_info: &wgpu::AdapterInfo,
    adapter_features: wgpu::Features,
) -> RenderAdapterFacts {
    RenderAdapterFacts::new(
        wgpu_backend_kind(adapter_info.backend),
        &adapter_info.name,
        adapter_info.vendor,
        adapter_info.device,
        format!("{} {}", adapter_info.driver, adapter_info.driver_info)
            .trim()
            .to_owned(),
        wgpu_adapter_class(adapter_info.backend, adapter_info.device_type),
        None,
        wgpu_device_features(adapter_features),
    )
}

/// Allocates a process-local WGPU device identity after successful device creation.
pub fn next_wgpu_device_id() -> DeviceId {
    DeviceId::new(NEXT_WGPU_DEVICE_ID.fetch_add(1, Ordering::Relaxed))
}

/// Builds the shared cold-start profile for one newly requested WGPU device generation.
pub fn initial_wgpu_render_device_profile(
    adapter: RenderAdapterFacts,
    device: &wgpu::Device,
    request: &WgpuDeviceRequest,
) -> RenderDeviceProfile {
    RenderDeviceProfile::new(
        next_wgpu_device_id(),
        DeviceGeneration::initial(),
        adapter,
        request.feature_negotiation().clone(),
        wgpu_device_limits(&device.limits()),
        RenderDeviceQueueTopology::single_serialized_queue(),
        GpuMemoryBudget::reference_1080p_mid(),
        SubmissionLimits::default(),
        DiagnosticReadbackBudget::default(),
    )
}

/// Converts WGPU's negotiated limits to the backend-neutral profile snapshot.
pub fn wgpu_device_limits(limits: &wgpu::Limits) -> RenderDeviceLimits {
    RenderDeviceLimits {
        max_bind_groups: limits.max_bind_groups,
        max_texture_dimension_2d: limits.max_texture_dimension_2d,
        max_texture_array_layers: limits.max_texture_array_layers,
        max_sampled_textures_per_shader_stage: limits.max_sampled_textures_per_shader_stage,
        max_binding_array_elements_per_shader_stage: limits
            .max_binding_array_elements_per_shader_stage,
        max_binding_array_sampler_elements_per_shader_stage: limits
            .max_binding_array_sampler_elements_per_shader_stage,
        min_uniform_buffer_offset_alignment: limits.min_uniform_buffer_offset_alignment,
        min_storage_buffer_offset_alignment: limits.min_storage_buffer_offset_alignment,
        max_storage_buffers_per_shader_stage: limits.max_storage_buffers_per_shader_stage,
        max_storage_buffer_binding_size: u64::from(limits.max_storage_buffer_binding_size),
    }
}

/// Projects WGPU's concrete enabled or supported features into the neutral feature vocabulary.
pub(crate) fn wgpu_device_features(features: wgpu::Features) -> RenderDeviceFeatureSet {
    let mut supported_features = RenderDeviceFeatureSet::default();

    for (feature, required_wgpu_features) in [
        (
            RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget,
            wgpu::Features::RG11B10UFLOAT_RENDERABLE,
        ),
        (
            RenderDeviceFeature::IndirectFirstInstance,
            wgpu::Features::INDIRECT_FIRST_INSTANCE,
        ),
        (
            RenderDeviceFeature::MultiDrawIndirectCount,
            wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
        ),
        (
            RenderDeviceFeature::BindlessMaterialArrays,
            WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES,
        ),
        (
            RenderDeviceFeature::GpuTimestamp,
            GPU_TIMESTAMP_REQUIRED_FEATURES,
        ),
        (
            RenderDeviceFeature::PipelineStatistics,
            wgpu::Features::PIPELINE_STATISTICS_QUERY,
        ),
        (RenderDeviceFeature::Subgroups, wgpu::Features::SUBGROUP),
    ] {
        if features.contains(required_wgpu_features) {
            supported_features.insert(feature);
        }
    }

    supported_features
}

const fn wgpu_backend_kind(backend: wgpu::Backend) -> RenderBackendKind {
    match backend {
        wgpu::Backend::Dx12 => RenderBackendKind::Dx12,
        wgpu::Backend::Vulkan => RenderBackendKind::Vulkan,
        wgpu::Backend::Metal => RenderBackendKind::Metal,
        wgpu::Backend::Gl => RenderBackendKind::Gl,
        wgpu::Backend::BrowserWebGpu => RenderBackendKind::BrowserWebGpu,
        wgpu::Backend::Noop => RenderBackendKind::Other,
    }
}

const fn wgpu_adapter_class(
    backend: wgpu::Backend,
    device_type: wgpu::DeviceType,
) -> RenderAdapterClass {
    if matches!(backend, wgpu::Backend::Noop) {
        return RenderAdapterClass::Cpu;
    }

    match device_type {
        wgpu::DeviceType::DiscreteGpu => RenderAdapterClass::Discrete,
        wgpu::DeviceType::IntegratedGpu => RenderAdapterClass::Integrated,
        wgpu::DeviceType::VirtualGpu => RenderAdapterClass::Virtual,
        wgpu::DeviceType::Cpu => RenderAdapterClass::Cpu,
        wgpu::DeviceType::Other => RenderAdapterClass::Other,
    }
}

const fn wgpu_features_for_neutral_feature(feature: RenderDeviceFeature) -> wgpu::Features {
    match feature {
        RenderDeviceFeature::HdrR11G11B10UfloatRenderTarget => {
            wgpu::Features::RG11B10UFLOAT_RENDERABLE
        }
        RenderDeviceFeature::IndirectFirstInstance => wgpu::Features::INDIRECT_FIRST_INSTANCE,
        RenderDeviceFeature::MultiDrawIndirectCount => wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
        RenderDeviceFeature::BindlessMaterialArrays => WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES,
        RenderDeviceFeature::GpuTimestamp => GPU_TIMESTAMP_REQUIRED_FEATURES,
        RenderDeviceFeature::PipelineStatistics => wgpu::Features::PIPELINE_STATISTICS_QUERY,
        RenderDeviceFeature::Subgroups => wgpu::Features::SUBGROUP,
    }
}
