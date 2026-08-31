use zr_rhi::{
    RenderAdapterClass, RenderAdapterInfo, RenderBackendCaps, RenderDeviceProfile, RenderOperation,
    RenderOperationSupport,
};

use crate::wgpu_backend_caps;

pub(super) fn production_caps(
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    profile: &RenderDeviceProfile,
) -> RenderBackendCaps {
    production_caps_from_wgpu(
        device.features(),
        device.limits(),
        adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::INDIRECT_EXECUTION),
        profile,
    )
}

/// Produces the neutral capability view from native WGPU facts.
///
/// This is deliberately narrower than `wgpu_backend_caps`: the latter is a
/// native diagnostics mapper, while this owner may advertise only ABI paths
/// implemented by its public neutral contract.
pub(in crate::production) fn production_caps_from_wgpu(
    features: wgpu::Features,
    limits: wgpu::Limits,
    supports_indirect_execution: bool,
    profile: &RenderDeviceProfile,
) -> RenderBackendCaps {
    // Keep each optional field fail-closed until its matching neutral command
    // or descriptor surface exists atomically with native execution.
    wgpu_backend_caps(
        "wgpu-rhi-production",
        features,
        limits,
        cfg!(target_os = "windows"),
        false,
        supports_indirect_execution,
    )
    .with_adapter(RenderAdapterInfo {
        name: profile.adapter().name.clone(),
        device_type: adapter_class_label(profile.adapter().adapter_class).to_string(),
    })
    .with_device_limits(profile.device_limits().clone())
    .with_operation_support(RenderOperation::DirectDraw, RenderOperationSupport::Native)
    .with_operation_support(RenderOperation::IndexedDraw, RenderOperationSupport::Native)
    .with_operation_support(
        RenderOperation::IndirectDraw,
        indirect_operation_support(supports_indirect_execution),
    )
    .with_operation_support(
        RenderOperation::MultiDrawIndirect,
        indirect_operation_support(supports_indirect_execution),
    )
    .with_operation_support(
        RenderOperation::MultiDrawIndirectCount,
        indirect_count_operation_support(features, supports_indirect_execution),
    )
    .with_operation_support(
        RenderOperation::ComputeDispatch,
        RenderOperationSupport::Native,
    )
    .with_operation_support(
        RenderOperation::ComputeDispatchIndirect,
        RenderOperationSupport::Native,
    )
    .with_operation_support(
        RenderOperation::BufferToBufferCopy,
        RenderOperationSupport::Native,
    )
    .with_operation_support(
        RenderOperation::BufferToTextureCopy,
        RenderOperationSupport::Native,
    )
    .with_operation_support(
        RenderOperation::TextureToBufferCopy,
        RenderOperationSupport::Native,
    )
    .with_operation_support(
        RenderOperation::TextureToTextureCopy,
        RenderOperationSupport::Native,
    )
    .with_operation_support(RenderOperation::DebugMarker, RenderOperationSupport::Native)
    .with_operation_support(RenderOperation::DebugGroup, RenderOperationSupport::Native)
    .with_indirect_draw(supports_indirect_execution)
    .with_multi_draw_indirect(supports_indirect_execution)
    .with_multi_draw_indirect_count(
        supports_indirect_execution && features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT),
    )
    .with_indirect_first_instance(false)
    .with_buffer_readback(false)
    .with_gpu_timestamp(features.contains(
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
    ))
    .with_subgroup(false)
    .with_pipeline_statistics_query(features.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY))
    .with_buffer_binding_array(false)
    .with_texture_binding_array(false)
    .with_non_uniform_resource_indexing(false)
    .with_partially_bound_binding_array(false)
    .with_graphics_debugger_capture(false)
}

const fn indirect_operation_support(supported: bool) -> RenderOperationSupport {
    if supported {
        RenderOperationSupport::Native
    } else {
        RenderOperationSupport::Unsupported
    }
}

fn indirect_count_operation_support(
    features: wgpu::Features,
    supports_indirect_execution: bool,
) -> RenderOperationSupport {
    indirect_operation_support(
        supports_indirect_execution && features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT),
    )
}

const fn adapter_class_label(adapter_class: RenderAdapterClass) -> &'static str {
    match adapter_class {
        RenderAdapterClass::Discrete => "discrete_gpu",
        RenderAdapterClass::Integrated => "integrated_gpu",
        RenderAdapterClass::Virtual => "virtual_gpu",
        RenderAdapterClass::Cpu => "cpu",
        RenderAdapterClass::Other => "other",
    }
}
