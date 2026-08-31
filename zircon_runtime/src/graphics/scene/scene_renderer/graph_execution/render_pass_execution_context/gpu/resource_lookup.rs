use std::num::NonZeroU64;

use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::{RenderGraphResourceAccessKind, RenderGraphResourceKind};
use crate::rhi::TextureDesc;

use super::super::RgResourceResolver;
use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn with_resource_resolver(
        mut self,
        resource_resolver: Option<RgResourceResolver<'a>>,
    ) -> Self {
        self.resource_resolver =
            resource_resolver.map(|resolver| resolver.with_physical_resources(self.resources));
        self
    }

    pub fn require_texture_view(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::TextureView, String> {
        Self::require_texture_view_by_name(
            self.resources,
            self.resource_resolver,
            resource_name,
            access,
        )
    }

    pub fn optional_texture_view(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<&'a wgpu::TextureView>, String> {
        Self::optional_texture_view_by_name(
            self.resources,
            self.resource_resolver,
            resource_name,
            access,
        )
    }

    pub fn require_buffer(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::Buffer, String> {
        Self::require_buffer_by_name(
            self.resources,
            self.resource_resolver,
            resource_name,
            access,
        )
    }

    pub fn require_buffer_binding(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<wgpu::BufferBinding<'a>, String> {
        Self::require_buffer_binding_by_name(
            self.resources,
            self.resource_resolver,
            resource_name,
            access,
        )
    }

    /// Resolves a graph buffer into the compiler-proven WGPU binding window.
    ///
    /// Transient and typed external accesses retain their exact byte range;
    /// persistent and unknown report-only declarations use the full native
    /// buffer as an explicit compatibility path.
    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn require_buffer_binding_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'resources>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<wgpu::BufferBinding<'resources>, String> {
        if let Some(resolver) = resource_resolver {
            if let Some(access_id) =
                resolver.exact_transient_access_by_name(resource_name, access)?
            {
                let (buffer, range) = resources.transient_buffer_binding_for_access(access_id)?;
                let size = range.end.checked_sub(range.start).ok_or_else(|| {
                    format!("render graph buffer `{resource_name}` exact binding range is inverted")
                })?;
                let size = NonZeroU64::new(size).ok_or_else(|| {
                    format!(
                        "render graph buffer `{resource_name}` exact binding range must not be empty"
                    )
                })?;
                return Ok(wgpu::BufferBinding {
                    buffer,
                    offset: range.start,
                    size: Some(size),
                });
            }
            if let Some(access_id) =
                resolver.exact_external_access_by_name(resource_name, access)?
            {
                let (buffer, range) = resources.external_buffer_binding_for_access(access_id)?;
                let size = range.end.checked_sub(range.start).ok_or_else(|| {
                    format!("render graph external buffer `{resource_name}` exact binding range is inverted")
                })?;
                let size = NonZeroU64::new(size).ok_or_else(|| {
                    format!(
                        "render graph external buffer `{resource_name}` exact binding range must not be empty"
                    )
                })?;
                return Ok(wgpu::BufferBinding {
                    buffer,
                    offset: range.start,
                    size: Some(size),
                });
            }
            let buffer = resolver.buffer_by_name(resource_name, access)?;
            return Ok(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: None,
            });
        }
        let buffer = resources.require_buffer(resource_name)?;
        Ok(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: None,
        })
    }

    pub fn require_texture_desc(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<TextureDesc, String> {
        Self::require_texture_desc_by_name(
            self.resources,
            self.resource_resolver,
            resource_name,
            access,
        )
    }

    pub fn require_owned_texture_full_mip_view(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<wgpu::TextureView, String> {
        if let Some(resolver) = self.resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            if declaration.kind != RenderGraphResourceKind::TransientTexture {
                return Err(format!(
                    "render graph resource `{resource_name}` must be a transient texture before a full-mip view can be requested"
                ));
            }
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                // Validate the compiler-selected access before exposing the
                // explicit full-mip compatibility view below. The access view
                // may cover only one mip and cannot satisfy this API's
                // full-chain contract.
                self.resources
                    .graph_owned_texture_view_for_access(access_id)?;
            }
            self.resources
                .require_texture_view_for_declaration(declaration)?;
        }
        self.resources.owned_texture_full_mip_view(resource_name)
    }

    pub fn texture_view_with_full_mip_fallback(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<wgpu::TextureView, String> {
        if let Some(view) = Self::optional_owned_texture_full_mip_view_by_name(
            self.resources,
            self.resource_resolver,
            resource_name,
            access,
        )? {
            return Ok(view);
        }
        self.require_texture_view(resource_name, access).cloned()
    }

    pub fn require_owned_texture_mip_view(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
        mip_level: u32,
    ) -> Result<wgpu::TextureView, String> {
        if let Some(resolver) = self.resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            if declaration.kind != RenderGraphResourceKind::TransientTexture {
                return Err(format!(
                    "render graph resource `{resource_name}` must be a transient texture before a mip view can be requested"
                ));
            }
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                self.resources
                    .graph_owned_texture_view_for_access(access_id)?;
            } else {
                self.resources
                    .require_texture_view_for_declaration(declaration)?;
            }
        }
        self.resources
            .owned_texture_mip_view(resource_name, mip_level)
    }

    pub(in crate::graphics::scene::scene_renderer) fn require_texture_view_by_name(
        resources: &'a RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::TextureView, String> {
        if let Some(resolver) = resource_resolver {
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                return resources.graph_owned_texture_view_for_access(access_id);
            }
            return resolver.texture_view_by_name(resource_name, access);
        } else {
            resources.require_texture_view(resource_name)
        }
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn require_buffer_by_name(
        resources: &'a RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<&'a wgpu::Buffer, String> {
        if let Some(resolver) = resource_resolver {
            if let Some(access_id) =
                resolver.exact_transient_access_by_name(resource_name, access)?
            {
                // The returned buffer remains the native WGPU object for API
                // compatibility; the exact compiler-proven byte window is
                // still required before exposing it to an executor.
                let (buffer, _) = resources.transient_buffer_binding_for_access(access_id)?;
                return Ok(buffer);
            }
            return resolver.buffer_by_name(resource_name, access);
        } else {
            resources.require_buffer(resource_name)
        }
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn require_texture_desc_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'a>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<TextureDesc, String> {
        if let Some(resolver) = resource_resolver {
            let declaration =
                resolver.require_pass_resource_declaration_by_name(resource_name, access)?;
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                resources.graph_owned_texture_view_for_access(access_id)?;
            }
            if let Some(access_id) =
                resolver.exact_external_access_by_name(resource_name, access)?
            {
                return resources.external_texture_desc_for_access(access_id);
            }
            resources.require_texture_desc_for_declaration(declaration)
        } else {
            resources.require_owned_texture_desc(resource_name).cloned()
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
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                return resources
                    .graph_owned_texture_view_for_access(access_id)
                    .map(Some);
            }
            if let Some(access_id) =
                resolver.exact_external_access_by_name(resource_name, access)?
            {
                return resources.optional_external_texture_view_for_access(access_id);
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
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                return resources
                    .graph_owned_texture_view_for_access(access_id)
                    .map(Some);
            }
            if let Some(access_id) =
                resolver.exact_external_access_by_name(resource_name, access)?
            {
                return resources.optional_external_texture_view_for_access(access_id);
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
            if let Some(access_id) =
                resolver.exact_external_access_by_name(resource_name, access)?
            {
                return resources
                    .optional_external_buffer_binding_for_access(access_id)
                    .map(|binding| binding.map(|(buffer, _)| buffer));
            }
            return Ok(resources.buffer(&declaration.name));
        }
        Ok(resources.buffer(resource_name))
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn optional_buffer_binding_by_name<
        'resources,
    >(
        resources: &'resources RenderGraphExecutionResources,
        resource_resolver: Option<RgResourceResolver<'resources>>,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> Result<Option<wgpu::BufferBinding<'resources>>, String> {
        if let Some(resolver) = resource_resolver {
            let Some(declaration) =
                resolver.pass_resource_declaration_by_name(resource_name, access)
            else {
                return Ok(None);
            };
            if declaration.kind == RenderGraphResourceKind::TransientTexture {
                return Err(format!(
                    "render graph resource `{resource_name}` is a texture but a buffer binding was requested"
                ));
            }
            if let Some(access_id) =
                resolver.exact_transient_access_by_name(resource_name, access)?
            {
                let (buffer, range) = resources.transient_buffer_binding_for_access(access_id)?;
                let size = range.end.checked_sub(range.start).ok_or_else(|| {
                    format!("render graph buffer `{resource_name}` exact binding range is inverted")
                })?;
                let size = NonZeroU64::new(size).ok_or_else(|| {
                    format!(
                        "render graph buffer `{resource_name}` exact binding range must not be empty"
                    )
                })?;
                return Ok(Some(wgpu::BufferBinding {
                    buffer,
                    offset: range.start,
                    size: Some(size),
                }));
            }
            if let Some(access_id) =
                resolver.exact_external_access_by_name(resource_name, access)?
            {
                let Some((buffer, range)) =
                    resources.optional_external_buffer_binding_for_access(access_id)?
                else {
                    return Ok(None);
                };
                let size = range.end.checked_sub(range.start).ok_or_else(|| {
                    format!("render graph external buffer `{resource_name}` exact binding range is inverted")
                })?;
                let size = NonZeroU64::new(size).ok_or_else(|| {
                    format!(
                        "render graph external buffer `{resource_name}` exact binding range must not be empty"
                    )
                })?;
                return Ok(Some(wgpu::BufferBinding {
                    buffer,
                    offset: range.start,
                    size: Some(size),
                }));
            }
            return Ok(resources
                .buffer(&declaration.name)
                .map(|buffer| wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: None,
                }));
        }
        Ok(resources
            .buffer(resource_name)
            .map(|buffer| wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: None,
            }))
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
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                return resources.graph_owned_texture_for_access(access_id);
            } else {
                resources.require_texture_view_for_declaration(declaration)?;
            }
        }
        resources.owned_texture(resource_name).ok_or_else(|| {
            format!("render graph execution owned texture resource `{resource_name}` is not bound")
        })
    }

    pub(in crate::graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu) fn require_physical_texture_by_name<
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
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                return resources.graph_owned_texture_for_access(access_id);
            } else {
                resources.require_texture_view_for_declaration(declaration)?;
            }
        }
        resources.physical_texture(resource_name).ok_or_else(|| {
            format!(
                "render graph execution physical texture resource `{resource_name}` is not bound"
            )
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
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                resources.graph_owned_texture_view_for_access(access_id)?;
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
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(declared_resource_name, access)?
            {
                resources.graph_owned_texture_view_for_access(access_id)?;
            } else {
                resources.require_texture_view_for_declaration(declaration)?;
            }
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
            if let Some(access_id) =
                resolver.exact_graph_owned_texture_access_by_name(resource_name, access)?
            {
                resources.graph_owned_texture_view_for_access(access_id)?;
            } else {
                resources.require_texture_view_for_declaration(declaration)?;
            }
        }
        Ok(resources
            .owned_texture_mip_level_count(resource_name)
            .unwrap_or(1))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::asset::ProjectAssetManager;
    use crate::core::framework::render::{
        RenderFrameExtract, RenderPluginRendererOutputs, RenderWorldSnapshotHandle,
    };
    use crate::core::math::UVec2;
    use crate::graphics::ViewportRenderFrame;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
        TransientResourcePool,
    };
    use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
    use crate::render_graph::{QueueLane, RenderGraphBuilder, RenderGraphResourceAccessKind};
    use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};
    use crate::scene::world::World;

    use super::RenderPassGpuExecutionContext;

    #[test]
    fn public_gpu_resource_lookup_requires_compiled_pass_declaration_access() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let mut builder = RenderGraphBuilder::new("gpu-buffer-public-lookup");
        let scene_depth = builder.create_texture(
            TextureDesc::new(
                "scene-depth",
                16,
                16,
                TextureFormat::Depth32Float,
                TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
            )
            .with_sample_count(4),
        );
        let hybrid_gi_scene = builder.create_buffer(BufferDesc::new(
            "hybrid-gi-scene",
            256,
            BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ));
        let hzb = builder.create_texture(
            TextureDesc::new(
                "hzb-furthest",
                8,
                8,
                TextureFormat::Rgba16Float,
                TextureUsage::SAMPLED | TextureUsage::STORAGE,
            )
            .with_mip_levels(4),
        );
        let depth_prepass = builder.add_pass("depth-prepass", QueueLane::Graphics);
        builder.write_texture(depth_prepass, scene_depth).unwrap();
        let hzb_build = builder.add_pass("hzb-build", QueueLane::AsyncCompute);
        builder.read_texture(hzb_build, scene_depth).unwrap();
        builder.write_texture(hzb_build, hzb).unwrap();
        let pass = builder.add_pass("hybrid-gi-scene-prepare", QueueLane::Graphics);
        builder.read_texture(pass, scene_depth).unwrap();
        builder.read_texture(pass, hzb).unwrap();
        builder.write_buffer(pass, hybrid_gi_scene).unwrap();
        let output = builder.import_present_external_resource("viewport-output");
        let present = builder.add_pass("present", QueueLane::Graphics);
        builder.read_buffer(present, hybrid_gi_scene).unwrap();
        builder.write_external(present, output).unwrap();
        let graph = builder.compile().unwrap();
        let pass = graph
            .passes()
            .iter()
            .find(|pass| pass.name == "hybrid-gi-scene-prepare")
            .unwrap();
        let mut resources = RenderGraphExecutionResources::new();
        let mut transient_pool = TransientResourcePool::default();
        transient_pool.begin_frame(backend.device_profile());
        resources
            .materialize_transient_resources_with_pool(
                &backend.device,
                backend.device_profile(),
                &graph,
                &mut transient_pool,
            )
            .unwrap();
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu-buffer-public-lookup-test"),
            });
        let scene_bind_group_layout =
            backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("gpu-buffer-public-lookup-empty-layout"),
                    entries: &[],
                });
        let scene_bind_group = backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("gpu-buffer-public-lookup-empty-bind-group"),
                layout: &scene_bind_group_layout,
                entries: &[],
            });
        let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
        let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new_for_test(
            Arc::new(ProjectAssetManager::default()),
            &backend.device,
            &backend.queue,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let mut plugin_outputs = RenderPluginRendererOutputs::default();
        let gpu = RenderPassGpuExecutionContext::new_for_test(
            &backend.device,
            &backend.queue,
            &mut encoder,
            &frame,
            &scene_bind_group_layout,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Depth32Float,
            &scene_bind_group,
            &mut resources,
            &mut plugin_outputs,
            &mut screen_space_ui_renderer,
        );
        let context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                pass.resources.clone(),
            )
            .with_resource_resolver(&graph, pass.id)
            .with_gpu(gpu);

        assert!(
            context
                .resource_resolver()
                .is_some_and(|resolver| resolver.has_physical_resources())
        );
        assert!(
            context
                .gpu()
                .and_then(RenderPassGpuExecutionContext::resource_resolver)
                .is_some_and(|resolver| resolver.has_physical_resources())
        );
        context
            .gpu()
            .unwrap()
            .require_texture_desc("scene-depth", RenderGraphResourceAccessKind::Read)
            .map(|desc| assert_eq!(desc.sample_count, 4))
            .expect("declared read texture descriptor should expose MSAA sample count");
        context
            .gpu()
            .unwrap()
            .require_buffer("hybrid-gi-scene", RenderGraphResourceAccessKind::Write)
            .expect("declared write buffer should resolve through the public GPU facade");
        context
            .gpu()
            .unwrap()
            .require_owned_texture_full_mip_view(
                "hzb-furthest",
                RenderGraphResourceAccessKind::Read,
            )
            .expect("declared transient HZB read should expose its full mip chain");
        let error = context
            .gpu()
            .unwrap()
            .require_buffer("hybrid-gi-scene", RenderGraphResourceAccessKind::Read)
            .unwrap_err();

        assert!(
            error.contains("did not declare Read access for resource `hybrid-gi-scene`"),
            "{error}"
        );
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }
}
use std::num::NonZeroU64;
