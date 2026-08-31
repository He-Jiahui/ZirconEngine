use crate::graphics::resource_identity::SampledTextureIdentity;
use crate::render_graph::RenderGraphTextureSubresourceRange;
use crate::rhi::{BufferDesc, TextureDesc};

use super::texture_views::texture_mip_view_descriptor;
use super::{RenderGraphExecutionResources, TransientBufferAllocation, TransientTextureAllocation};

impl RenderGraphExecutionResources {
    pub(in crate::graphics::scene::scene_renderer) fn import_texture_view(
        &mut self,
        name: impl Into<String>,
        view: wgpu::TextureView,
    ) -> Option<wgpu::TextureView> {
        let name = name.into();
        self.imported_textures.remove(&name);
        self.imported_texture_descs.remove(&name);
        self.replace_imported_texture_view(name, view)
    }

    fn replace_imported_texture_view(
        &mut self,
        name: String,
        view: wgpu::TextureView,
    ) -> Option<wgpu::TextureView> {
        self.sampled_texture_identities.remove(&name);
        self.imported_texture_views.insert(name, view)
    }

    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_texture_view(
        &mut self,
        name: impl Into<String>,
        view: &wgpu::TextureView,
    ) -> Option<wgpu::TextureView> {
        self.import_texture_view(name, view.clone())
    }

    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_texture(
        &mut self,
        name: impl Into<String>,
        texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        desc: TextureDesc,
    ) -> Option<wgpu::TextureView> {
        let name = name.into();
        self.imported_textures.insert(name.clone(), texture.clone());
        self.imported_texture_descs.insert(name.clone(), desc);
        self.replace_imported_texture_view(name, view.clone())
    }

    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_texture_with_identity(
        &mut self,
        name: impl Into<String>,
        texture: &wgpu::Texture,
        view: &wgpu::TextureView,
        desc: TextureDesc,
        identity: SampledTextureIdentity,
    ) -> Option<wgpu::TextureView> {
        let name = name.into();
        let previous = self.import_borrowed_texture(name.clone(), texture, view, desc);
        self.set_texture_identity(name, identity);
        previous
    }

    /// Binds an externally owned view with the producer-supplied physical
    /// descriptor. WGPU does not expose a descriptor from `TextureView`, so a
    /// schema-backed graph must reject a view-only lease instead of guessing.
    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_texture_view_with_physical_desc(
        &mut self,
        name: impl Into<String>,
        view: &wgpu::TextureView,
        desc: TextureDesc,
    ) -> Option<wgpu::TextureView> {
        let name = name.into();
        self.imported_textures.remove(&name);
        self.imported_texture_descs.insert(name.clone(), desc);
        self.replace_imported_texture_view(name, view.clone())
    }

    pub(in crate::graphics::scene::scene_renderer) fn insert_buffer(
        &mut self,
        name: impl Into<String>,
        buffer: wgpu::Buffer,
    ) -> Option<wgpu::Buffer> {
        let name = name.into();
        self.imported_buffer_descs.remove(&name);
        self.owned_buffers.remove(&name);
        self.buffer_backings.insert(name.clone(), name.clone());
        self.buffers.insert(name, buffer)
    }

    /// Binds an externally owned buffer with its producer-supplied physical
    /// descriptor. WGPU does not expose buffer descriptor metadata at encoding
    /// time, so a schema-backed graph must reject descriptor-less leases.
    pub(in crate::graphics::scene::scene_renderer) fn import_borrowed_buffer_with_physical_desc(
        &mut self,
        name: impl Into<String>,
        buffer: &wgpu::Buffer,
        desc: BufferDesc,
    ) -> Option<wgpu::Buffer> {
        let name = name.into();
        self.owned_buffers.remove(&name);
        self.imported_buffer_descs.insert(name.clone(), desc);
        self.buffer_backings.insert(name.clone(), name.clone());
        self.buffers.insert(name, buffer.clone())
    }

    /// Preserves an external producer's stable backing identity while carrying
    /// the descriptor required by a schema-backed external graph resource.
    pub(in crate::graphics::scene::scene_renderer) fn bind_borrowed_buffer_with_physical_desc(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
        desc: BufferDesc,
    ) -> Result<Option<String>, String> {
        let logical_name = logical_name.into();
        let backing_name = backing_name.into();
        if self.buffers.contains_key(&backing_name) {
            return Err(format!(
                "render graph execution typed external buffer `{logical_name}` reuses backing `{backing_name}`; register each physical backing once until explicit external buffer aliasing is implemented"
            ));
        }
        self.owned_buffers.remove(&backing_name);
        self.imported_buffer_descs
            .insert(backing_name.clone(), desc);
        self.buffers.insert(backing_name.clone(), buffer.clone());
        Ok(self.buffer_backings.insert(logical_name, backing_name))
    }

    pub(in crate::graphics::scene::scene_renderer) fn bind_execution_owned_buffer(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: impl Into<String>,
        buffer: &wgpu::Buffer,
    ) -> Option<String> {
        let logical_name = logical_name.into();
        let backing_name = backing_name.into();
        self.imported_buffer_descs.remove(&logical_name);
        self.buffers.insert(backing_name.clone(), buffer.clone());
        self.buffer_backings.insert(logical_name, backing_name)
    }

    pub(in crate::graphics::scene::scene_renderer) fn import_texture_alias(
        &mut self,
        alias: impl Into<String>,
        source: &wgpu::Texture,
    ) -> Option<wgpu::TextureView> {
        self.import_texture_view(
            alias,
            source.create_view(&wgpu::TextureViewDescriptor::default()),
        )
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn insert_owned_texture_backing(
        &mut self,
        backing_name: impl Into<String>,
        allocation: TransientTextureAllocation,
    ) -> Option<TransientTextureAllocation> {
        let backing_name = backing_name.into();
        self.owned_textures.insert(backing_name, allocation)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn bind_owned_texture_view(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: &str,
    ) -> Result<Option<wgpu::TextureView>, String> {
        let logical_name = logical_name.into();
        let allocation = self.owned_textures.get(backing_name).ok_or_else(|| {
            format!("render graph execution texture backing `{backing_name}` is missing")
        })?;
        let view = allocation
            .native()
            .create_view(&texture_mip_view_descriptor(0));
        self.owned_texture_backings
            .insert(logical_name.clone(), backing_name.to_string());
        let identity = allocation.identity();
        let previous = self.import_texture_view(logical_name.clone(), view);
        self.set_texture_identity(logical_name, identity);
        Ok(previous)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn bind_owned_texture_subresource_view(
        &mut self,
        logical_name: impl Into<String>,
        parent_name: &str,
        range: RenderGraphTextureSubresourceRange,
    ) -> Result<Option<wgpu::TextureView>, String> {
        let logical_name = logical_name.into();
        let view = self.owned_texture_subresource_view(parent_name, range)?;
        let backing_name = self
            .owned_texture_backing(parent_name)
            .ok_or_else(|| {
                format!(
                    "render graph execution texture view alias `{logical_name}` parent `{parent_name}` is not an owned transient texture"
                )
            })?
            .to_owned();
        let identity = self
            .owned_textures
            .get(&backing_name)
            .map(TransientTextureAllocation::identity)
            .ok_or_else(|| {
                format!(
                    "render graph execution texture view alias `{logical_name}` parent backing `{backing_name}` is missing allocation"
                )
            })?;
        self.owned_texture_backings
            .insert(logical_name.clone(), backing_name);
        self.texture_view_aliases
            .insert(logical_name.clone(), (parent_name.to_owned(), range));
        let previous = self.import_texture_view(logical_name.clone(), view);
        self.set_texture_identity(logical_name, identity);
        Ok(previous)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn insert_buffer_backing(
        &mut self,
        backing_name: impl Into<String>,
        allocation: TransientBufferAllocation,
    ) -> Option<TransientBufferAllocation> {
        let backing_name = backing_name.into();
        self.imported_buffer_descs.remove(&backing_name);
        self.buffers.remove(&backing_name);
        self.owned_buffers.insert(backing_name, allocation)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn bind_buffer(
        &mut self,
        logical_name: impl Into<String>,
        backing_name: &str,
    ) -> Option<String> {
        self.buffer_backings
            .insert(logical_name.into(), backing_name.to_string())
    }
}
