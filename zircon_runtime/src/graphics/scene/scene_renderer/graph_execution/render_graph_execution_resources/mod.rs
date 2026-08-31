mod access_bindings;
mod binding;
mod external_access_bindings;
mod lifecycle;
mod lookup;
mod persistent_texture_access_bindings;
mod reporting;
mod texture_views;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::render_graph::RenderGraphTextureSubresourceRange;
use crate::rhi::{BufferDesc, TextureDesc};

use super::{RenderPassDeviceEpoch, TransientBufferAllocation, TransientTextureAllocation};

#[derive(Clone, Debug)]
pub(in crate::graphics::scene::scene_renderer) struct RenderGraphImportedFinalTarget<'a> {
    pub texture: &'a wgpu::Texture,
    pub view: &'a wgpu::TextureView,
    pub desc: TextureDesc,
}

/// Per-frame logical-to-physical resource bindings for a compiled graph.
///
/// Resource registration, lookup, lifecycle, reporting, and texture-view
/// validation have independent owners below this declaration module. Keeping
/// the state here makes those owners cooperate without a second registry.
#[derive(Default, Debug)]
pub struct RenderGraphExecutionResources {
    device_epoch: Option<RenderPassDeviceEpoch>,
    imported_texture_views: BTreeMap<String, wgpu::TextureView>,
    pub(super) sampled_texture_identities:
        BTreeMap<String, crate::graphics::resource_identity::SampledTextureIdentity>,
    imported_textures: BTreeMap<String, wgpu::Texture>,
    imported_texture_descs: BTreeMap<String, TextureDesc>,
    owned_textures: BTreeMap<String, TransientTextureAllocation>,
    owned_texture_backings: BTreeMap<String, String>,
    texture_view_aliases: BTreeMap<String, (String, RenderGraphTextureSubresourceRange)>,
    buffers: BTreeMap<String, wgpu::Buffer>,
    owned_buffers: BTreeMap<String, TransientBufferAllocation>,
    // Imported buffer descriptors are physical facts, so they follow the
    // backing identity rather than any logical graph alias.
    imported_buffer_descs: BTreeMap<String, BufferDesc>,
    buffer_backings: BTreeMap<String, String>,
    access_bindings: access_bindings::RenderGraphExecutionAccessBindingTable,
    persistent_texture_access_bindings:
        persistent_texture_access_bindings::RenderGraphExecutionPersistentTextureAccessBindings,
    external_access_bindings: external_access_bindings::RenderGraphExecutionExternalAccessBindings,
}

impl RenderGraphExecutionResources {
    pub fn new() -> Self {
        Self::default()
    }

    pub(in crate::graphics::scene::scene_renderer) const fn device_epoch(
        &self,
    ) -> Option<RenderPassDeviceEpoch> {
        self.device_epoch
    }
}
