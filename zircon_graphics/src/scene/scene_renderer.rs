use crate::backend::{read_texture_rgba, OffscreenTarget, RenderBackend};
use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame, ViewportState};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use zircon_asset::{AlphaMode, MaterialAsset, MeshVertex, ProjectAssetManager, TextureAsset};
use zircon_math::{perspective, view_matrix, Vec3, Vec4};
use zircon_resource::{
    AssetReference, MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceLocator,
};
use zircon_scene::RenderSceneSnapshot;

pub(crate) const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[derive(Clone, Debug)]
pub(crate) struct MaterialRuntime {
    base_color: Vec4,
    base_color_texture: Option<ResourceId>,
    pipeline_key: PipelineKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PipelineKey {
    shader_id: ResourceId,
    shader_revision: u64,
    double_sided: bool,
    alpha_blend: bool,
}

#[derive(Clone, Debug)]
struct ShaderRuntime {
    source: String,
}

pub(crate) struct ResourceStreamer {
    asset_manager: Arc<ProjectAssetManager>,
    models: HashMap<ResourceId, PreparedModel>,
    materials: HashMap<ResourceId, PreparedMaterial>,
    textures: HashMap<ResourceId, PreparedTexture>,
    shaders: HashMap<ResourceId, PreparedShader>,
    fallback_texture: Arc<GpuTextureResource>,
}

struct PreparedModel {
    revision: u64,
    resource: Arc<GpuModelResource>,
}

struct PreparedMaterial {
    revision: u64,
    runtime: MaterialRuntime,
}

struct PreparedTexture {
    revision: u64,
    resource: Arc<GpuTextureResource>,
}

struct PreparedShader {
    revision: u64,
    runtime: ShaderRuntime,
}

impl ResourceStreamer {
    pub(crate) fn new(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            asset_manager,
            models: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            fallback_texture: Arc::new(create_fallback_texture(device, queue, texture_layout)),
        }
    }

    pub(crate) fn ensure_scene_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<(), GraphicsError> {
        for mesh in &frame.scene.meshes {
            self.ensure_model(device, mesh.model)?;
            self.ensure_material(device, queue, texture_layout, mesh.material)?;
        }
        Ok(())
    }

    fn ensure_model(
        &mut self,
        device: &wgpu::Device,
        handle: ResourceHandle<ModelMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let revision = self.resource_revision(id)?;
        if self
            .models
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }
        let model = self
            .asset_manager
            .load_model_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let resource = Arc::new(GpuModelResource::from_asset(device, id, model));
        self.models.insert(
            id,
            PreparedModel {
                revision,
                resource,
            },
        );
        Ok(())
    }

    fn ensure_material(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        handle: ResourceHandle<MaterialMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let revision = self.resource_revision(id)?;
        if self
            .materials
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }
        let material = self
            .asset_manager
            .load_material_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let texture_id = self.resolve_texture_id(&material);
        if let Some(texture_id) = texture_id {
            self.ensure_texture(device, queue, texture_layout, texture_id)?;
        }
        let (shader_id, shader_revision) = self.ensure_shader_source(&material.shader)?;
        self.materials.insert(
            id,
            PreparedMaterial {
                revision,
                runtime: MaterialRuntime {
                    base_color: Vec4::from_array(material.base_color),
                    base_color_texture: texture_id,
                    pipeline_key: PipelineKey {
                        shader_id,
                        shader_revision,
                        double_sided: material.double_sided,
                        alpha_blend: matches!(material.alpha_mode, AlphaMode::Blend),
                    },
                },
            },
        );
        Ok(())
    }

    fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
    ) -> Result<(), GraphicsError> {
        let revision = self.resource_revision(id)?;
        if self
            .textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }
        let texture = self
            .asset_manager
            .load_texture_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let resource = Arc::new(GpuTextureResource::from_asset(
            device,
            queue,
            texture_layout,
            id,
            texture,
        ));
        self.textures.insert(
            id,
            PreparedTexture {
                revision,
                resource,
            },
        );
        Ok(())
    }

    fn resolve_texture_id(&self, material: &MaterialAsset) -> Option<ResourceId> {
        material
            .base_color_texture
            .as_ref()
            .and_then(|reference| {
                self.asset_manager
                    .resource_manager()
                    .registry()
                    .get_by_locator(&reference.locator)
                    .map(|record| record.id())
            })
    }

    fn ensure_shader_source(
        &mut self,
        reference: &AssetReference,
    ) -> Result<(ResourceId, u64), GraphicsError> {
        let uri = &reference.locator;
        let shader_id = self
            .asset_manager
            .resolve_asset_id(uri)
            .or_else(|| {
                self.asset_manager
                    .resolve_asset_id(&fallback_shader_uri())
            })
            .ok_or_else(|| GraphicsError::Asset(format!("missing shader resource for {uri}")))?;
        let revision = self.resource_revision(shader_id)?;

        if self
            .shaders
            .get(&shader_id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok((shader_id, revision));
        }

        let shader = self
            .asset_manager
            .load_shader_asset(shader_id)
            .or_else(|_| {
                self.asset_manager
                    .load_shader_asset_by_uri(&fallback_shader_uri())
            })
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        self.shaders.insert(
            shader_id,
            PreparedShader {
                revision,
                runtime: ShaderRuntime {
                    source: shader.source,
                },
            },
        );
        Ok((shader_id, revision))
    }

    pub(crate) fn model(&self, id: &ResourceId) -> Option<&Arc<GpuModelResource>> {
        self.models.get(id).map(|prepared| &prepared.resource)
    }

    pub(crate) fn material(&self, id: &ResourceId) -> Option<&MaterialRuntime> {
        self.materials.get(id).map(|prepared| &prepared.runtime)
    }

    pub(crate) fn texture(&self, id: Option<ResourceId>) -> Arc<GpuTextureResource> {
        id.and_then(|texture_id| self.textures.get(&texture_id).map(|prepared| prepared.resource.clone()))
            .unwrap_or_else(|| self.fallback_texture.clone())
    }

    fn shader_source(&self, shader_id: &ResourceId) -> Option<&str> {
        self.shaders
            .get(shader_id)
            .map(|shader| shader.runtime.source.as_str())
    }

    fn resource_revision(&self, id: ResourceId) -> Result<u64, GraphicsError> {
        self.asset_manager
            .resource_manager()
            .registry()
            .get(id)
            .map(|record| record.revision)
            .ok_or_else(|| GraphicsError::Asset(format!("missing resource record {id}")))
    }
}

pub struct SceneRenderer {
    backend: RenderBackend,
    core: SceneRendererCore,
    streamer: ResourceStreamer,
    target: Option<OffscreenTarget>,
    generation: u64,
}

impl SceneRenderer {
    pub fn new(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        let backend = RenderBackend::new_offscreen()?;
        let core = SceneRendererCore::new(&backend.device, OFFSCREEN_FORMAT);
        let streamer = ResourceStreamer::new(
            asset_manager,
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
        );

        Ok(Self {
            backend,
            core,
            streamer,
            target: None,
            generation: 0,
            })
    }

    pub fn render(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport: ViewportState,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame(&EditorOrRuntimeFrame {
            scene: snapshot,
            viewport,
        })
    }

    pub fn render_frame(
        &mut self,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.streamer
            .ensure_scene_resources(
                &self.backend.device,
                &self.backend.queue,
                &self.core.texture_bind_group_layout,
                frame,
            )?;

        let size =
            zircon_math::UVec2::new(frame.viewport.size.x.max(1), frame.viewport.size.y.max(1));
        if self
            .target
            .as_ref()
            .is_none_or(|target| target.size != size)
        {
            self.target = Some(OffscreenTarget::new(&self.backend.device, size));
        }
        let target = self.target.as_ref().unwrap();

        self.core.render_scene(
            &self.backend.device,
            &self.backend.queue,
            &self.streamer,
            frame,
            &target.color_view,
            &target.depth_view,
        );
        self.generation += 1;
        let rgba = read_texture_rgba(
            &self.backend.device,
            &self.backend.queue,
            &target.color,
            target.size,
        )?;

        Ok(ViewportFrame {
            width: target.size.x,
            height: target.size.y,
            rgba,
            generation: self.generation,
        })
    }
}

pub(crate) struct SceneRendererCore {
    target_format: wgpu::TextureFormat,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    scene_uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
    model_bind_group_layout: wgpu::BindGroupLayout,
    mesh_pipeline_layout: wgpu::PipelineLayout,
    shader_modules: HashMap<String, wgpu::ShaderModule>,
    mesh_pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
    line_pipeline: wgpu::RenderPipeline,
    grid_vertex_buffer: wgpu::Buffer,
    grid_vertex_count: u32,
}

impl SceneRendererCore {
    pub(crate) fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-scene-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let scene_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-scene-uniform"),
            size: std::mem::size_of::<SceneUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });
        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-model-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-texture-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-line-shader"),
            source: wgpu::ShaderSource::Wgsl(LINE_SHADER.into()),
        });
        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-mesh-layout"),
            bind_group_layouts: &[
                &scene_bind_group_layout,
                &model_bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let line_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-line-layout"),
            bind_group_layouts: &[&scene_bind_group_layout],
            push_constant_ranges: &[],
        });
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-line-pipeline"),
            layout: Some(&line_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &line_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[LineVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &line_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        let grid_vertices = build_grid_vertices();
        let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-grid-buffer"),
            contents: bytemuck::cast_slice(&grid_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            target_format,
            texture_bind_group_layout,
            scene_uniform_buffer,
            scene_bind_group,
            model_bind_group_layout,
            mesh_pipeline_layout,
            shader_modules: HashMap::new(),
            mesh_pipelines: HashMap::new(),
            line_pipeline,
            grid_vertex_buffer,
            grid_vertex_count: grid_vertices.len() as u32,
        }
    }

    pub(crate) fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let aspect = frame.viewport.size.x as f32 / frame.viewport.size.y.max(1) as f32;
        let scene_uniform = SceneUniform::from_frame(frame, aspect);
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&scene_uniform),
        );

        let mesh_draws = build_mesh_draws(device, &self.model_bind_group_layout, streamer, frame);
        let gizmo = build_gizmo_buffer(device, frame);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("zircon-scene-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.09,
                            b: 0.11,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_bind_group(0, &self.scene_bind_group, &[]);

            for draw in &mesh_draws {
                let pipeline = self.ensure_mesh_pipeline(device, streamer, &draw.pipeline_key);
                pass.set_pipeline(pipeline);
                pass.set_bind_group(1, &draw.model_bind_group, &[]);
                pass.set_bind_group(2, &draw.texture.bind_group, &[]);
                pass.set_vertex_buffer(0, draw.mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(draw.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..draw.mesh.index_count, 0, 0..1);
            }

            pass.set_pipeline(&self.line_pipeline);
            if frame.scene.show_grid {
                pass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
                pass.draw(0..self.grid_vertex_count, 0..1);
            }
            if let Some((buffer, count)) = gizmo.as_ref() {
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..*count, 0..1);
            }
        }
        queue.submit([encoder.finish()]);
    }

    fn ensure_mesh_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        key: &PipelineKey,
    ) -> &'a wgpu::RenderPipeline {
        let shader_key = format!("{}@{}", key.shader_id, key.shader_revision);
        if !self.shader_modules.contains_key(&shader_key) {
            let source = streamer
                .shader_source(&key.shader_id)
                .unwrap_or(FALLBACK_MESH_SHADER);
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self.mesh_pipelines.contains_key(key) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let pipeline = create_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                self.target_format,
                key,
            );
            self.mesh_pipelines.insert(key.clone(), pipeline);
        }
        self.mesh_pipelines
            .get(key)
            .expect("mesh pipeline cached")
    }
}

pub(crate) fn create_depth_texture(
    device: &wgpu::Device,
    size: zircon_math::UVec2,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-depth"),
        size: wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

struct MeshDraw {
    mesh: Arc<GpuMeshResource>,
    texture: Arc<GpuTextureResource>,
    pipeline_key: PipelineKey,
    #[allow(dead_code)]
    model_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
}

fn build_mesh_draws(
    device: &wgpu::Device,
    model_layout: &wgpu::BindGroupLayout,
    streamer: &ResourceStreamer,
    frame: &EditorOrRuntimeFrame,
) -> Vec<MeshDraw> {
    let mut draws = Vec::new();
    for mesh_instance in &frame.scene.meshes {
        let Some(model) = streamer.model(&mesh_instance.model.id()) else {
            continue;
        };
        let material = streamer.material(&mesh_instance.material.id());
        let texture = streamer.texture(material.and_then(|material| material.base_color_texture));
        let material_tint = material
            .map(|material| material.base_color)
            .unwrap_or(Vec4::ONE);
        let pipeline_key = material
            .map(|material| material.pipeline_key.clone())
            .unwrap_or_else(default_pipeline_key);
        let tint = if mesh_instance.selected {
            mesh_instance.tint * material_tint * Vec4::new(1.0, 0.92, 0.55, 1.0)
        } else {
            mesh_instance.tint * material_tint
        };
        for mesh in &model.meshes {
            let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-model-buffer"),
                contents: bytemuck::bytes_of(&ModelUniform {
                    model: mesh_instance.transform.matrix().to_cols_array_2d(),
                    tint: tint.to_array(),
                }),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("zircon-model-bind-group"),
                layout: model_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_buffer.as_entire_binding(),
                }],
            });
            draws.push(MeshDraw {
                mesh: mesh.clone(),
                texture: texture.clone(),
                pipeline_key: pipeline_key.clone(),
                model_buffer,
                model_bind_group,
            });
        }
    }
    draws
}

fn build_gizmo_buffer(
    device: &wgpu::Device,
    frame: &EditorOrRuntimeFrame,
) -> Option<(wgpu::Buffer, u32)> {
    let gizmo = frame.scene.gizmo.as_ref()?;
    let vertices = vec![
        LineVertex::new(gizmo.origin, Vec4::new(1.0, 0.25, 0.25, 1.0)),
        LineVertex::new(gizmo.origin + Vec3::X, Vec4::new(1.0, 0.25, 0.25, 1.0)),
        LineVertex::new(gizmo.origin, Vec4::new(0.25, 1.0, 0.25, 1.0)),
        LineVertex::new(gizmo.origin + Vec3::Y, Vec4::new(0.25, 1.0, 0.25, 1.0)),
        LineVertex::new(gizmo.origin, Vec4::new(0.25, 0.5, 1.0, 1.0)),
        LineVertex::new(gizmo.origin + Vec3::Z, Vec4::new(0.25, 0.5, 1.0, 1.0)),
    ];
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-gizmo-buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    Some((buffer, vertices.len() as u32))
}

fn build_grid_vertices() -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    for index in -10..=10 {
        let color = if index == 0 {
            Vec4::new(0.24, 0.36, 0.88, 1.0)
        } else if index % 5 == 0 {
            Vec4::new(0.22, 0.24, 0.3, 1.0)
        } else {
            Vec4::new(0.16, 0.17, 0.2, 1.0)
        };
        let z = index as f32;
        vertices.push(LineVertex::new(Vec3::new(-10.0, 0.0, z), color));
        vertices.push(LineVertex::new(Vec3::new(10.0, 0.0, z), color));
        vertices.push(LineVertex::new(Vec3::new(z, 0.0, -10.0), color));
        vertices.push(LineVertex::new(Vec3::new(z, 0.0, 10.0), color));
    }
    vertices
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneUniform {
    view_proj: [[f32; 4]; 4],
    light_dir: [f32; 4],
    light_color: [f32; 4],
}

impl SceneUniform {
    fn from_frame(frame: &EditorOrRuntimeFrame, aspect: f32) -> Self {
        let projection = perspective(
            frame.scene.camera.fov_y_radians,
            aspect,
            frame.scene.camera.z_near,
            frame.scene.camera.z_far,
        );
        let view = view_matrix(frame.scene.camera.transform);
        let light = &frame.scene.light;
        Self {
            view_proj: (projection * view).to_cols_array_2d(),
            light_dir: light.direction.extend(0.0).to_array(),
            light_color: (light.color * light.intensity).extend(1.0).to_array(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
    tint: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuMeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

impl GpuMeshVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 2,
                },
            ],
        }
    }
}

impl From<MeshVertex> for GpuMeshVertex {
    fn from(value: MeshVertex) -> Self {
        Self {
            position: value.position,
            normal: value.normal,
            uv: value.uv,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LineVertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl LineVertex {
    fn new(position: Vec3, color: Vec4) -> Self {
        Self {
            position: position.to_array(),
            color: color.to_array(),
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub(crate) struct GpuModelResource {
    #[allow(dead_code)]
    id: ResourceId,
    meshes: Vec<Arc<GpuMeshResource>>,
}

impl GpuModelResource {
    fn from_asset(device: &wgpu::Device, id: ResourceId, asset: zircon_asset::ModelAsset) -> Self {
        Self {
            id,
            meshes: asset
                .primitives
                .into_iter()
                .map(|primitive| Arc::new(GpuMeshResource::from_asset(device, primitive)))
                .collect(),
        }
    }
}

pub(crate) struct GpuMeshResource {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl GpuMeshResource {
    fn from_asset(
        device: &wgpu::Device,
        payload: zircon_asset::ModelPrimitiveAsset,
    ) -> Self {
        let vertices: Vec<GpuMeshVertex> = payload.vertices.into_iter().map(Into::into).collect();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-mesh-vertex-buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-mesh-index-buffer"),
            contents: bytemuck::cast_slice(&payload.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            index_count: payload.indices.len() as u32,
        }
    }
}

pub(crate) struct GpuTextureResource {
    #[allow(dead_code)]
    id: Option<ResourceId>,
    #[allow(dead_code)]
    texture: wgpu::Texture,
    #[allow(dead_code)]
    view: wgpu::TextureView,
    #[allow(dead_code)]
    sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl GpuTextureResource {
    fn from_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-texture"),
            size: wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &payload.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * payload.width),
                rows_per_image: Some(payload.height),
            },
            wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-texture-bind-group"),
            layout: texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            id: Some(id),
            texture,
            view,
            sampler,
            bind_group,
        }
    }
}

fn create_fallback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout,
) -> GpuTextureResource {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-fallback-texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OFFSCREEN_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &[255, 255, 255, 255],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-fallback-bind-group"),
        layout: texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    GpuTextureResource {
        id: None,
        texture,
        view,
        sampler,
        bind_group,
    }
}

fn create_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[GpuMeshVertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: (!key.double_sided).then_some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: !key.alpha_blend,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(if key.alpha_blend {
                    wgpu::BlendState::ALPHA_BLENDING
                } else {
                    wgpu::BlendState::REPLACE
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}

fn default_pipeline_key() -> PipelineKey {
    PipelineKey {
        shader_id: ResourceId::from_locator(&fallback_shader_uri()),
        shader_revision: 1,
        double_sided: false,
        alpha_blend: false,
    }
}

fn fallback_shader_uri() -> ResourceLocator {
    ResourceLocator::parse("builtin://shader/pbr.wgsl").expect("builtin fallback shader uri")
}

const FALLBACK_MESH_SHADER: &str = r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.world_normal = normalize((model_data.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(-scene.light_dir.xyz);
    let lighting = max(dot(light_dir, normalize(input.world_normal)), 0.18);
    let albedo = textureSample(albedo_tex, albedo_sampler, input.uv).rgba * model_data.tint;
    return vec4<f32>(albedo.rgb * scene.light_color.rgb * lighting, albedo.a);
}
"#;

const LINE_SHADER: &str = r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = scene.view_proj * vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;
