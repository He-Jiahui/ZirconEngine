use std::collections::HashMap;
use std::ops::Range;

use crate::core::framework::render::RenderGraphExecutionAccessBindingReport;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphPhysicalAllocationId, RenderGraphResourceAccessId,
    RenderGraphResourceAccessRange, RenderGraphResourceKind, RenderGraphVersionedAccessKey,
};

use super::RenderGraphExecutionResources;

/// Device-local backing for one exact, live transient graph access.
///
/// The compiler owns the logical key and physical allocation identity. This
/// product table is deliberately frame-scoped and owns only the WGPU view,
/// physical texture backing, or buffer window that materialization proved to
/// match that key.
#[derive(Debug)]
enum RenderGraphExecutionAccessBinding {
    Texture {
        key: RenderGraphVersionedAccessKey,
        physical_allocation: RenderGraphPhysicalAllocationId,
        view: wgpu::TextureView,
    },
    Buffer {
        key: RenderGraphVersionedAccessKey,
        physical_allocation: RenderGraphPhysicalAllocationId,
        buffer: wgpu::Buffer,
        range: Range<wgpu::BufferAddress>,
    },
}

impl RenderGraphExecutionAccessBinding {
    const fn key(&self) -> RenderGraphVersionedAccessKey {
        match self {
            Self::Texture { key, .. } | Self::Buffer { key, .. } => *key,
        }
    }

    const fn physical_allocation(&self) -> RenderGraphPhysicalAllocationId {
        match self {
            Self::Texture {
                physical_allocation,
                ..
            }
            | Self::Buffer {
                physical_allocation,
                ..
            } => *physical_allocation,
        }
    }
}

/// Exact access-ID lookup for transient WGPU bindings.
#[derive(Debug, Default)]
pub(super) struct RenderGraphExecutionAccessBindingTable {
    bindings: HashMap<RenderGraphResourceAccessId, RenderGraphExecutionAccessBinding>,
    texture_backings: HashMap<RenderGraphPhysicalAllocationId, wgpu::Texture>,
    report: RenderGraphExecutionAccessBindingReport,
}

impl RenderGraphExecutionAccessBindingTable {
    fn from_compiled_graph(
        resources: &RenderGraphExecutionResources,
        graph: &CompiledRenderGraph,
    ) -> Result<Self, String> {
        let mut bindings = HashMap::with_capacity(graph.access_allocation_bindings().len());
        let mut texture_backings = HashMap::new();
        let mut texture_views: HashMap<
            (
                RenderGraphPhysicalAllocationId,
                crate::render_graph::RenderGraphTextureSubresourceRange,
            ),
            wgpu::TextureView,
        > = HashMap::with_capacity(graph.access_allocation_bindings().len());
        let mut transient_texture_access_binding_count = 0;
        let mut transient_buffer_access_binding_count = 0;
        let mut reused_texture_view_count = 0;

        for compiled_binding in graph.access_allocation_bindings() {
            let Some(physical_allocation) = compiled_binding.physical_allocation else {
                continue;
            };
            let key = compiled_binding.key;
            let declaration = graph.resource_declaration(key.resource).ok_or_else(|| {
                format!(
                    "render graph execution access {:?} references an undeclared resource {:?}",
                    key.access_id, key.resource
                )
            })?;
            if physical_allocation.kind() != declaration.kind {
                return Err(format!(
                    "render graph execution access {:?} physical allocation kind {:?} does not match declared resource `{}` kind {:?}",
                    key.access_id,
                    physical_allocation.kind(),
                    declaration.name,
                    declaration.kind
                ));
            }

            let binding = match (declaration.kind, key.range) {
                (
                    RenderGraphResourceKind::TransientTexture,
                    RenderGraphResourceAccessRange::Texture(range),
                ) => {
                    transient_texture_access_binding_count += 1;
                    if let std::collections::hash_map::Entry::Vacant(entry) =
                        texture_backings.entry(physical_allocation)
                    {
                        let texture = resources
                            .owned_texture(&declaration.name)
                            .cloned()
                            .ok_or_else(|| {
                                format!(
                                    "render graph execution transient texture resource `{}` is not materialized for access {:?}",
                                    declaration.name, key.access_id
                                )
                            })?;
                        entry.insert(texture);
                    }
                    let view_key = (physical_allocation, range);
                    let view = match texture_views.get(&view_key) {
                        Some(view) => {
                            reused_texture_view_count += 1;
                            view.clone()
                        }
                        None => {
                            let view = resources
                                .owned_texture_subresource_view(&declaration.name, range)?;
                            texture_views.insert(view_key, view.clone());
                            view
                        }
                    };
                    RenderGraphExecutionAccessBinding::Texture {
                        key,
                        physical_allocation,
                        view,
                    }
                }
                (
                    RenderGraphResourceKind::TransientBuffer,
                    RenderGraphResourceAccessRange::Buffer(range),
                ) => {
                    transient_buffer_access_binding_count += 1;
                    let size = range.size.ok_or_else(|| {
                        format!(
                            "render graph execution transient buffer access {:?} has unresolved byte range",
                            key.access_id
                        )
                    })?;
                    let end = range.offset.checked_add(size).ok_or_else(|| {
                        format!(
                            "render graph execution transient buffer access {:?} byte range overflows",
                            key.access_id
                        )
                    })?;
                    let buffer = resources.buffer(&declaration.name).cloned().ok_or_else(|| {
                        format!(
                            "render graph execution transient buffer resource `{}` is not materialized for access {:?}",
                            declaration.name, key.access_id
                        )
                    })?;
                    if end > buffer.size() {
                        return Err(format!(
                            "render graph execution transient buffer access {:?} range [{}..{}) exceeds materialized resource `{}` size {}",
                            key.access_id,
                            range.offset,
                            end,
                            declaration.name,
                            buffer.size()
                        ));
                    }
                    RenderGraphExecutionAccessBinding::Buffer {
                        key,
                        physical_allocation,
                        buffer,
                        range: range.offset..end,
                    }
                }
                (kind, range) => {
                    return Err(format!(
                        "render graph execution access {:?} declared resource `{}` kind {:?} cannot materialize range {:?}",
                        key.access_id, declaration.name, kind, range
                    ));
                }
            };
            if bindings.insert(key.access_id, binding).is_some() {
                return Err(format!(
                    "render graph execution materialization encountered duplicate exact access binding {:?}",
                    key.access_id
                ));
            }
        }

        Ok(Self {
            report: RenderGraphExecutionAccessBindingReport::new(
                bindings.len(),
                transient_texture_access_binding_count,
                transient_buffer_access_binding_count,
                texture_views.len(),
                reused_texture_view_count,
            ),
            bindings,
            texture_backings,
        })
    }

    fn binding(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&RenderGraphExecutionAccessBinding, String> {
        self.bindings.get(&access).ok_or_else(|| {
            format!(
                "render graph execution access {:?} has no transient physical binding",
                access
            )
        })
    }

    pub(super) fn contains(&self, access: RenderGraphResourceAccessId) -> bool {
        self.bindings.contains_key(&access)
    }
}

impl RenderGraphExecutionResources {
    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn materialize_transient_access_bindings(
        &mut self,
        graph: &CompiledRenderGraph,
    ) -> Result<(), String> {
        self.access_bindings =
            RenderGraphExecutionAccessBindingTable::from_compiled_graph(self, graph)?;
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn clear_transient_access_bindings(
        &mut self,
    ) {
        self.access_bindings = RenderGraphExecutionAccessBindingTable::default();
    }

    pub(super) fn access_binding_report(&self) -> RenderGraphExecutionAccessBindingReport {
        self.access_bindings.report
    }

    pub(in crate::graphics::scene::scene_renderer) fn transient_access_binding_count(
        &self,
    ) -> usize {
        self.access_bindings.bindings.len()
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn transient_texture_backing_count(
        &self,
    ) -> usize {
        self.access_bindings.texture_backings.len()
    }

    pub(in crate::graphics::scene::scene_renderer) fn transient_physical_allocation_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphPhysicalAllocationId> {
        self.access_bindings
            .bindings
            .get(&access)
            .map(RenderGraphExecutionAccessBinding::physical_allocation)
    }

    pub(in crate::graphics::scene::scene_renderer) fn transient_texture_view_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::TextureView, String> {
        match self.access_bindings.binding(access)? {
            RenderGraphExecutionAccessBinding::Texture { view, .. } => Ok(view),
            RenderGraphExecutionAccessBinding::Buffer { .. } => Err(format!(
                "render graph execution access {:?} is a transient buffer binding, not a texture view",
                access
            )),
        }
    }

    /// Returns the full WGPU texture backing proven for an exact transient access.
    ///
    /// Copy/readback epilogues use this when a subresource view is insufficient.
    pub(in crate::graphics::scene::scene_renderer) fn transient_texture_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<&wgpu::Texture, String> {
        match self.access_bindings.binding(access)? {
            RenderGraphExecutionAccessBinding::Texture {
                physical_allocation,
                ..
            } => self
                .access_bindings
                .texture_backings
                .get(physical_allocation)
                .ok_or_else(|| {
                    format!(
                        "render graph execution texture access {:?} has no physical texture backing",
                        access
                    )
                }),
            RenderGraphExecutionAccessBinding::Buffer { .. } => Err(format!(
                "render graph execution access {:?} is a transient buffer binding, not a texture backing",
                access
            )),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn transient_buffer_slice_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<wgpu::BufferSlice<'_>, String> {
        match self.access_bindings.binding(access)? {
            RenderGraphExecutionAccessBinding::Buffer { buffer, range, .. } => {
                Ok(buffer.slice(range.clone()))
            }
            RenderGraphExecutionAccessBinding::Texture { .. } => Err(format!(
                "render graph execution access {:?} is a transient texture binding, not a buffer slice",
                access
            )),
        }
    }

    /// Returns the WGPU buffer and the compiler-proven byte window for an exact access.
    ///
    /// Bind-group lowering uses this instead of rediscovering a transient buffer by name.
    pub(in crate::graphics::scene::scene_renderer) fn transient_buffer_binding_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Result<(&wgpu::Buffer, Range<wgpu::BufferAddress>), String> {
        match self.access_bindings.binding(access)? {
            RenderGraphExecutionAccessBinding::Buffer { buffer, range, .. } => {
                Ok((buffer, range.clone()))
            }
            RenderGraphExecutionAccessBinding::Texture { .. } => Err(format!(
                "render graph execution access {:?} is a transient texture binding, not a buffer binding",
                access
            )),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn transient_access_key(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphVersionedAccessKey> {
        self.access_bindings
            .bindings
            .get(&access)
            .map(RenderGraphExecutionAccessBinding::key)
    }
}
