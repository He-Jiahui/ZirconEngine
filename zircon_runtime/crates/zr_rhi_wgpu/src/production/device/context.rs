use std::sync::Arc;

use zr_rhi::{RenderDeviceProfile, RenderDeviceQueueTopology, RhiError};

use crate::device_profile::wgpu_device_features;
use crate::{wgpu_adapter_facts, wgpu_device_limits, WgpuUiSharedImageRegistry};

/// One-shot native WGPU ownership handoff for a neutral render-device generation.
///
/// This container deliberately exposes no native-object accessors. The receiver is
/// [`super::WgpuRenderDevice::new`], which becomes the only owner that installs WGPU device-fault
/// callbacks and submits neutral RHI work for the generation.
pub struct WgpuRenderDeviceContext {
    // Preserve the same dependent-before-owner drop order if validation rejects the handoff.
    pub(super) ui_image_registry: Arc<WgpuUiSharedImageRegistry>,
    pub(super) queue: wgpu::Queue,
    pub(super) device: wgpu::Device,
    pub(super) adapter: wgpu::Adapter,
    pub(super) instance: wgpu::Instance,
}

impl WgpuRenderDeviceContext {
    /// Captures all negotiated native state required by one neutral device generation.
    pub fn new(
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        Self {
            instance,
            adapter,
            device,
            queue,
            ui_image_registry: Arc::new(WgpuUiSharedImageRegistry::default()),
        }
    }
}

pub(super) fn validate_context_adapter(
    adapter: &wgpu::Adapter,
    profile: &RenderDeviceProfile,
) -> Result<(), RhiError> {
    let context_adapter = wgpu_adapter_facts(&adapter.get_info(), adapter.features());
    if profile.adapter() == &context_adapter {
        return Ok(());
    }

    Err(RhiError::NativeContextAdapterMismatch {
        profile_adapter: profile.adapter().clone(),
        context_adapter,
    })
}

pub(super) fn validate_context_device_limits(
    device: &wgpu::Device,
    profile: &RenderDeviceProfile,
) -> Result<(), RhiError> {
    let context_limits = wgpu_device_limits(&device.limits());
    if profile.device_limits() == &context_limits {
        return Ok(());
    }

    Err(RhiError::NativeContextDeviceLimitsMismatch {
        profile_limits: profile.device_limits().clone(),
        context_limits,
    })
}

pub(super) fn validate_context_requested_features(
    device: &wgpu::Device,
    profile: &RenderDeviceProfile,
) -> Result<(), RhiError> {
    let context_features = wgpu_device_features(device.features());
    if profile.requested_features() == &context_features {
        return Ok(());
    }

    Err(RhiError::NativeContextRequestedFeaturesMismatch {
        profile_features: profile.requested_features().clone(),
        context_features,
    })
}

/// WGPU exposes graphics, compute, and copy work through one serial physical queue.
pub(super) fn validate_context_queue_topology(
    profile: &RenderDeviceProfile,
) -> Result<(), RhiError> {
    let context_topology = RenderDeviceQueueTopology::single_serialized_queue();
    if profile.queue_topology() == &context_topology {
        return Ok(());
    }

    Err(RhiError::NativeContextQueueTopologyMismatch {
        profile_topology: profile.queue_topology().clone(),
        context_topology,
    })
}
