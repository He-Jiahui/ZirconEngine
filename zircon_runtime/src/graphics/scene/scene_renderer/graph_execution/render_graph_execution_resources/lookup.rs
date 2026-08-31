use crate::render_graph::{
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphTextureSubresourceRange,
};
use crate::rhi::{BufferDesc, TextureDesc};

use super::RenderGraphExecutionResources;
use super::texture_views::{
    texture_full_mip_view_descriptor, texture_mip_view_descriptor,
    texture_subresource_view_descriptor, validate_texture_view_descriptor,
};

impl RenderGraphExecutionResources {
    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn texture_view(
        &self,
        name: &str,
    ) -> Option<&wgpu::TextureView> {
        self.imported_texture_views.get(name)
    }

    pub(in crate::graphics::scene::scene_renderer) fn buffer(
        &self,
        name: &str,
    ) -> Option<&wgpu::Buffer> {
        self.buffer_backing(name).and_then(|backing| {
            self.owned_buffers
                .get(backing)
                .map(|allocation| allocation.native())
                .or_else(|| self.buffers.get(backing))
        })
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture(
        &self,
        name: &str,
    ) -> Option<&wgpu::Texture> {
        self.owned_texture_backing(name)
            .and_then(|backing| self.owned_textures.get(backing))
            .map(|allocation| allocation.native())
    }

    pub(in crate::graphics::scene::scene_renderer) fn physical_texture(
        &self,
        name: &str,
    ) -> Option<&wgpu::Texture> {
        self.owned_texture(name)
            .or_else(|| self.imported_textures.get(name))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_desc(
        &self,
        name: &str,
    ) -> Option<&TextureDesc> {
        self.owned_texture_backing(name)
            .and_then(|backing| self.owned_textures.get(backing))
            .map(|allocation| allocation.desc())
    }

    pub(in crate::graphics::scene::scene_renderer) fn physical_texture_desc(
        &self,
        name: &str,
    ) -> Option<&TextureDesc> {
        self.owned_texture_desc(name)
            .or_else(|| self.imported_texture_descs.get(name))
    }

    pub(in crate::graphics::scene::scene_renderer) fn physical_buffer_desc(
        &self,
        name: &str,
    ) -> Option<&BufferDesc> {
        self.owned_buffer_desc(name).or_else(|| {
            self.buffer_backing(name)
                .and_then(|backing| self.imported_buffer_descs.get(backing))
        })
    }

    pub(in crate::graphics::scene::scene_renderer) fn physical_buffer_size(
        &self,
        name: &str,
    ) -> Option<wgpu::BufferAddress> {
        self.buffer(name).map(wgpu::Buffer::size)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_owned_texture_desc(
        &self,
        name: &str,
    ) -> Result<&TextureDesc, String> {
        self.owned_texture_desc(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_mip_view(
        &self,
        name: &str,
        mip_level: u32,
    ) -> Result<wgpu::TextureView, String> {
        let backing = self.owned_texture_backing(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let allocation = self.owned_textures.get(backing).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` backing `{backing}` is missing"
            )
        })?;
        let texture = allocation.native();
        let desc = allocation.desc();
        if mip_level >= desc.mip_levels {
            return Err(format!(
                "render graph execution texture resource `{name}` mip level {mip_level} is outside mip_levels {}",
                desc.mip_levels
            ));
        }
        Ok(texture.create_view(&texture_mip_view_descriptor(mip_level)))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_full_mip_view(
        &self,
        name: &str,
    ) -> Result<wgpu::TextureView, String> {
        let backing = self.owned_texture_backing(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let allocation = self.owned_textures.get(backing).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` backing `{backing}` is missing"
            )
        })?;
        let texture = allocation.native();
        let desc = allocation.desc();
        Ok(texture.create_view(&texture_full_mip_view_descriptor(desc.mip_levels)))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_view_with_descriptor(
        &self,
        name: &str,
        descriptor: &wgpu::TextureViewDescriptor<'_>,
    ) -> Result<wgpu::TextureView, String> {
        let backing = self.owned_texture_backing(name).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` is not an owned transient texture"
            )
        })?;
        let allocation = self.owned_textures.get(backing).ok_or_else(|| {
            format!(
                "render graph execution texture resource `{name}` backing `{backing}` is missing"
            )
        })?;
        let texture = allocation.native();
        let desc = allocation.desc();
        validate_texture_view_descriptor(name, desc, descriptor)?;
        Ok(texture.create_view(descriptor))
    }

    pub(in crate::graphics::scene::scene_renderer) fn owned_texture_mip_level_count(
        &self,
        name: &str,
    ) -> Option<u32> {
        self.owned_textures
            .get(self.owned_texture_backing(name).unwrap_or(name))
            .map(|allocation| allocation.desc().mip_levels)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_texture_view(
        &self,
        name: &str,
    ) -> Result<&wgpu::TextureView, String> {
        self.texture_view(name)
            .ok_or_else(|| format!("render graph execution texture resource `{name}` is not bound"))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_texture_view_for_declaration(
        &self,
        declaration: &RenderGraphResourceDeclaration,
    ) -> Result<&wgpu::TextureView, String> {
        if declaration.kind == RenderGraphResourceKind::TransientBuffer {
            return Err(format!(
                "render graph execution resource `{}` is a buffer declaration, not a texture view",
                declaration.name
            ));
        }
        self.require_texture_view(&declaration.name)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_texture_desc_for_declaration(
        &self,
        declaration: &RenderGraphResourceDeclaration,
    ) -> Result<TextureDesc, String> {
        if declaration.kind == RenderGraphResourceKind::TransientBuffer {
            return Err(format!(
                "render graph execution resource `{}` is a buffer declaration, not a texture descriptor",
                declaration.name
            ));
        }
        match &declaration.desc {
            RenderGraphResourceDesc::Texture(desc) => Ok(desc.clone()),
            RenderGraphResourceDesc::External => self
                .physical_texture_desc(&declaration.name)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "render graph execution external texture resource `{}` is missing its physical descriptor",
                        declaration.name
                    )
                }),
            RenderGraphResourceDesc::Buffer(_) => Err(format!(
                "render graph execution resource `{}` is a buffer declaration, not a texture descriptor",
                declaration.name
            )),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_buffer(
        &self,
        name: &str,
    ) -> Result<&wgpu::Buffer, String> {
        self.buffer(name)
            .ok_or_else(|| format!("render graph execution buffer resource `{name}` is not bound"))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution) fn require_buffer_for_declaration(
        &self,
        declaration: &RenderGraphResourceDeclaration,
    ) -> Result<&wgpu::Buffer, String> {
        if declaration.kind == RenderGraphResourceKind::TransientTexture {
            return Err(format!(
                "render graph execution resource `{}` is a texture declaration, not a buffer",
                declaration.name
            ));
        }
        self.require_buffer(&declaration.name)
    }

    pub(super) fn owned_texture_subresource_view(
        &self,
        name: &str,
        range: RenderGraphTextureSubresourceRange,
    ) -> Result<wgpu::TextureView, String> {
        self.owned_texture_view_with_descriptor(name, &texture_subresource_view_descriptor(range))
    }

    pub(super) fn owned_texture_backing(&self, name: &str) -> Option<&str> {
        self.owned_texture_backings.get(name).map(String::as_str)
    }

    pub(super) fn buffer_backing<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        if let Some(backing) = self.buffer_backings.get(name) {
            return Some(backing);
        }
        (self.owned_buffers.contains_key(name) || self.buffers.contains_key(name)).then_some(name)
    }

    pub(super) fn owned_buffer_desc(&self, name: &str) -> Option<&BufferDesc> {
        self.buffer_backing(name)
            .and_then(|backing| self.owned_buffers.get(backing))
            .map(|allocation| allocation.desc())
    }
}
