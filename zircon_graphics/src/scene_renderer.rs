use crate::render_backend::{read_texture_rgba, OffscreenTarget, RenderBackend};
use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame, ViewportState};
use bytemuck::{Pod, Zeroable};
use crossbeam_channel::{Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use zircon_asset::{
    AssetRequest, CpuAssetPayload, CpuMeshPayload, CpuTexturePayload, MeshSource, MeshVertex,
    TextureSource,
};
use zircon_math::{perspective, view_matrix, Vec3, Vec4};
use zircon_scene::RenderSceneSnapshot;

pub(crate) const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub(crate) struct ResourceStreamer {
    meshes: HashMap<MeshSource, Arc<GpuMeshResource>>,
    textures: HashMap<TextureSource, Arc<GpuTextureResource>>,
    pending_meshes: HashSet<MeshSource>,
    pending_textures: HashSet<TextureSource>,
    fallback_texture: Arc<GpuTextureResource>,
}

impl ResourceStreamer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            pending_meshes: HashSet::new(),
            pending_textures: HashSet::new(),
            fallback_texture: Arc::new(create_fallback_texture(device, queue, texture_layout)),
        }
    }

    pub(crate) fn ensure_scene_resources(
        &mut self,
        frame: &EditorOrRuntimeFrame,
        request_tx: &crossbeam_channel::Sender<AssetRequest>,
    ) {
        if frame.scene.show_grid && !self.textures.contains_key(&TextureSource::BuiltinGrid) {
            self.request_texture(TextureSource::BuiltinGrid, request_tx);
        }

        for mesh in &frame.scene.meshes {
            if !self.meshes.contains_key(&mesh.mesh) {
                self.request_mesh(mesh.mesh.clone(), request_tx);
            }
            if !self.textures.contains_key(&mesh.texture) {
                self.request_texture(mesh.texture.clone(), request_tx);
            }
        }
    }

    fn request_mesh(
        &mut self,
        source: MeshSource,
        request_tx: &crossbeam_channel::Sender<AssetRequest>,
    ) {
        if self.pending_meshes.insert(source.clone()) {
            let _ = request_tx.send(AssetRequest::Mesh(source));
        }
    }

    fn request_texture(
        &mut self,
        source: TextureSource,
        request_tx: &crossbeam_channel::Sender<AssetRequest>,
    ) {
        if self.pending_textures.insert(source.clone()) {
            let _ = request_tx.send(AssetRequest::Texture(source));
        }
    }

    pub(crate) fn process_completion(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        payload: CpuAssetPayload,
    ) {
        match payload {
            CpuAssetPayload::Texture(texture) => {
                self.pending_textures.remove(&texture.source);
                let resource = Arc::new(GpuTextureResource::from_payload(
                    device,
                    queue,
                    texture_layout,
                    texture,
                ));
                self.textures.insert(resource.source.clone(), resource);
            }
            CpuAssetPayload::Mesh(mesh) => {
                self.pending_meshes.remove(&mesh.source);
                let resource = Arc::new(GpuMeshResource::from_payload(device, mesh));
                self.meshes.insert(resource.source.clone(), resource);
            }
            CpuAssetPayload::Failure { request, .. } => match request {
                AssetRequest::Texture(texture) => {
                    self.pending_textures.remove(&texture);
                }
                AssetRequest::Mesh(mesh) => {
                    self.pending_meshes.remove(&mesh);
                }
            },
        }
    }

    pub(crate) fn mesh(&self, source: &MeshSource) -> Option<&Arc<GpuMeshResource>> {
        self.meshes.get(source)
    }

    pub(crate) fn texture(&self, source: &TextureSource) -> Arc<GpuTextureResource> {
        self.textures
            .get(source)
            .cloned()
            .unwrap_or_else(|| self.fallback_texture.clone())
    }
}

pub struct SceneRenderer {
    asset_requests: Sender<AssetRequest>,
    asset_completions: Receiver<CpuAssetPayload>,
    backend: RenderBackend,
    core: SceneRendererCore,
    streamer: ResourceStreamer,
    target: Option<OffscreenTarget>,
    generation: u64,
}

impl SceneRenderer {
    pub fn new(
        asset_requests: Sender<AssetRequest>,
        asset_completions: Receiver<CpuAssetPayload>,
    ) -> Result<Self, GraphicsError> {
        let backend = RenderBackend::new_offscreen()?;
        let core = SceneRendererCore::new(&backend.device, OFFSCREEN_FORMAT);
        let streamer = ResourceStreamer::new(
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
        );

        Ok(Self {
            asset_requests,
            asset_completions,
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
        while let Ok(payload) = self.asset_completions.try_recv() {
            self.streamer.process_completion(
                &self.backend.device,
                &self.backend.queue,
                &self.core.texture_bind_group_layout,
                payload,
            );
        }
        self.streamer
            .ensure_scene_resources(frame, &self.asset_requests);

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
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    scene_uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
    model_bind_group_layout: wgpu::BindGroupLayout,
    mesh_pipeline: wgpu::RenderPipeline,
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

        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-mesh-shader"),
            source: wgpu::ShaderSource::Wgsl(MESH_SHADER.into()),
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
        let mesh_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-mesh-pipeline"),
            layout: Some(&mesh_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[GpuMeshVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
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
            texture_bind_group_layout,
            scene_uniform_buffer,
            scene_bind_group,
            model_bind_group_layout,
            mesh_pipeline,
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

            pass.set_pipeline(&self.mesh_pipeline);
            for draw in &mesh_draws {
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
    frame
        .scene
        .meshes
        .iter()
        .filter_map(|mesh_instance| {
            let mesh = streamer.mesh(&mesh_instance.mesh)?.clone();
            let texture = streamer.texture(&mesh_instance.texture);
            let tint = if mesh_instance.selected {
                mesh_instance.tint * Vec4::new(1.0, 0.92, 0.55, 1.0)
            } else {
                mesh_instance.tint
            };
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
            Some(MeshDraw {
                mesh,
                texture,
                model_buffer,
                model_bind_group,
            })
        })
        .collect()
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

pub(crate) struct GpuMeshResource {
    source: MeshSource,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl GpuMeshResource {
    fn from_payload(device: &wgpu::Device, payload: CpuMeshPayload) -> Self {
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
            source: payload.source,
            vertex_buffer,
            index_buffer,
            index_count: payload.indices.len() as u32,
        }
    }
}

pub(crate) struct GpuTextureResource {
    source: TextureSource,
    #[allow(dead_code)]
    texture: wgpu::Texture,
    #[allow(dead_code)]
    view: wgpu::TextureView,
    #[allow(dead_code)]
    sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl GpuTextureResource {
    fn from_payload(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        payload: CpuTexturePayload,
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
            source: payload.source,
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
        source: TextureSource::BuiltinChecker,
        texture,
        view,
        sampler,
        bind_group,
    }
}

const MESH_SHADER: &str = r#"
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
