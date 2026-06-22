use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::{RenderGraphResourceAccessKind, RenderGraphResourceKind};

use super::super::RgResourceResolver;
use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn with_resource_resolver(
        mut self,
        resource_resolver: Option<RgResourceResolver<'a>>,
    ) -> Self {
        self.resource_resolver = resource_resolver;
        self
    }

    pub fn require_texture_view(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&wgpu::TextureView, String> {
        Self::require_texture_view_by_name(
            self.resources,
            self.resource_resolver,
            resource_name,
            access,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn require_texture_view_by_name<'resources>(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'resources wgpu::TextureView, String> {
        if let Some(resolver) = resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            resources.require_texture_view_for_declaration(declaration)
        } else {
            resources.require_texture_view(resource_name)
        }
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn require_buffer_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'resources wgpu::Buffer, String> {
        if let Some(resolver) = resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            resources.require_buffer_for_declaration(declaration)
        } else {
            resources.require_buffer(resource_name)
        }
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn optional_texture_view_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<&'resources wgpu::TextureView>, String> {
        if let Some(resolver) = resource_resolver {
            let Some(declaration) =
                resolver.pass_resource_declaration_by_name(resource_name, access)
            else {
                return Ok(None);
            };
            if declaration.kind == RenderGraphResourceKind::TransientBuffer {
                return Err(format!(
                    "render graph resource `{resource_name}` is a buffer but a texture view was requested"
                ));
            }
            return Ok(resources.texture_view(&declaration.name));
        }
        Ok(resources.texture_view(resource_name))
    }

    pub(in crate::graphics::scene::scene_renderer) fn declared_optional_texture_view_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<&'resources wgpu::TextureView>, String> {
        if let Some(resolver) = resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            if declaration.kind == RenderGraphResourceKind::TransientBuffer {
                return Err(format!(
                    "render graph resource `{resource_name}` is a buffer but a texture view was requested"
                ));
            }
            return Ok(resources.texture_view(&declaration.name));
        }
        Ok(resources.texture_view(resource_name))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn optional_buffer_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<&'resources wgpu::Buffer>, String> {
        if let Some(resolver) = resource_resolver {
            let Some(declaration) =
                resolver.pass_resource_declaration_by_name(resource_name, access)
            else {
                return Ok(None);
            };
            if declaration.kind == RenderGraphResourceKind::TransientTexture {
                return Err(format!(
                    "render graph resource `{resource_name}` is a texture but a buffer was requested"
                ));
            }
            return Ok(resources.buffer(&declaration.name));
        }
        Ok(resources.buffer(resource_name))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn require_owned_texture_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'resources wgpu::Texture, String> {
        if let Some(resolver) = resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            resources.require_texture_view_for_declaration(declaration)?;
        }
        resources.owned_texture(resource_name).ok_or_else(|| {
            format!("render graph execution owned texture resource `{resource_name}` is not bound")
        })
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn optional_owned_texture_full_mip_view_by_name(
        resources: &RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<wgpu::TextureView>, String> {
        if let Some(resolver) = resource_resolver {
            let Some(declaration) =
                resolver.pass_resource_declaration_by_name(resource_name, access)
            else {
                return Ok(None);
            };
            if declaration.kind == RenderGraphResourceKind::TransientBuffer {
                return Err(format!(
                    "render graph resource `{resource_name}` is a buffer but an owned texture view was requested"
                ));
            }
            if resources.texture_view(&declaration.name).is_none() {
                return Ok(None);
            }
        }
        Ok(resources.owned_texture_full_mip_view(resource_name).ok())
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn require_owned_texture_mip_view_by_name(
        resources: &RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        declared_resource_name: &str,
        physical_texture_name: &str,
        access: RenderGraphResourceAccessKind,
        mip_level: u32,
    ) -> Result<wgpu::TextureView, String> {
        if let Some(resolver) = resource_resolver {
            let declaration = resolver
                .require_pass_resource_declaration_by_name(declared_resource_name, access)?;
            resources.require_texture_view_for_declaration(declaration)?;
        }
        resources.owned_texture_mip_view(physical_texture_name, mip_level)
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn owned_texture_mip_level_count_by_name(
        resources: &RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<u32, String> {
        if let Some(resolver) = resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            resources.require_texture_view_for_declaration(declaration)?;
        }
        Ok(resources
            .owned_texture_mip_level_count(resource_name)
            .unwrap_or(1))
    }
}
