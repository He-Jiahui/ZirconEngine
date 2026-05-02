use std::fmt;
use std::sync::mpsc;

use crate::ParticleSystemAsset;

use super::planner::ParticleGpuFrameParams;
use super::program::{
    compile_particle_gpu_program, ParticleGpuProgram, PARTICLE_GPU_COUNTER_WORDS_BASE,
    PARTICLE_GPU_WORKGROUP_SIZE,
};
use super::readback::{ParticleGpuCounterReadback, ParticleGpuReadbackDecodeError};
use super::transparent::{
    ParticleGpuTransparentRenderConfig, ParticleGpuTransparentRenderParams,
    ParticleGpuTransparentRenderer,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ParticleGpuReadbackRequest {
    #[default]
    None,
    Counters,
}

#[derive(Debug)]
pub enum ParticleGpuBackendError {
    EmptyProgram,
    FrameEmitterCountMismatch { expected: u32, actual: usize },
    TransparentRendererUnavailable,
    ReadbackMap(String),
    ReadbackDecode(ParticleGpuReadbackDecodeError),
}

impl fmt::Display for ParticleGpuBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProgram => write!(f, "particle GPU program has zero particle capacity"),
            Self::FrameEmitterCountMismatch { expected, actual } => write!(
                f,
                "particle GPU frame has {actual} emitters but layout expects {expected}"
            ),
            Self::TransparentRendererUnavailable => {
                write!(f, "particle GPU transparent renderer has not been created")
            }
            Self::ReadbackMap(error) => {
                write!(f, "particle GPU readback map failed: {error}")
            }
            Self::ReadbackDecode(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ParticleGpuBackendError {}

impl From<ParticleGpuReadbackDecodeError> for ParticleGpuBackendError {
    fn from(value: ParticleGpuReadbackDecodeError) -> Self {
        Self::ReadbackDecode(value)
    }
}

pub struct ParticleGpuBuffers<'a> {
    pub particles: &'a wgpu::Buffer,
    pub alive_indices: &'a wgpu::Buffer,
    pub indirect_draw_args: &'a wgpu::Buffer,
    pub counters: &'a wgpu::Buffer,
}

pub struct ParticleGpuBackend {
    program: ParticleGpuProgram,
    bind_group_layout: wgpu::BindGroupLayout,
    spawn_update_pipeline: wgpu::ComputePipeline,
    compact_alive_pipeline: wgpu::ComputePipeline,
    indirect_args_pipeline: wgpu::ComputePipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    emitter_params_buffer: wgpu::Buffer,
    counters_buffer: wgpu::Buffer,
    alive_indices_buffer: wgpu::Buffer,
    indirect_draw_args_buffer: wgpu::Buffer,
    debug_readback_buffer: wgpu::Buffer,
    update_bind_groups: Vec<wgpu::BindGroup>,
    compact_bind_groups: Vec<wgpu::BindGroup>,
    active_buffer_index: usize,
    transparent_renderer: Option<ParticleGpuTransparentRenderer>,
}

impl ParticleGpuBackend {
    pub fn new(
        device: &wgpu::Device,
        asset: &ParticleSystemAsset,
    ) -> Result<Self, ParticleGpuBackendError> {
        let program = compile_particle_gpu_program(asset);
        if program.layout.capacity == 0 {
            return Err(ParticleGpuBackendError::EmptyProgram);
        }

        let bind_group_layout = create_bind_group_layout(device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-particle-gpu-simulation-shader"),
            source: wgpu::ShaderSource::Wgsl(program.shader.wgsl.clone().into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-particle-gpu-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let spawn_update_pipeline = create_compute_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "zircon-particle-gpu-spawn-update",
            program.shader.entries.spawn_update,
        );
        let compact_alive_pipeline = create_compute_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "zircon-particle-gpu-compact-alive",
            program.shader.entries.compact_alive,
        );
        let indirect_args_pipeline = create_compute_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "zircon-particle-gpu-indirect-args",
            program.shader.entries.build_indirect_args,
        );

        let particle_buffers = vec![
            create_storage_buffer(
                device,
                "zircon-particle-gpu-particles-a",
                program.resources.particle_buffer_bytes,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            ),
            create_storage_buffer(
                device,
                "zircon-particle-gpu-particles-b",
                program.resources.particle_buffer_bytes,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            ),
        ];
        let emitter_params_buffer = create_storage_buffer(
            device,
            "zircon-particle-gpu-emitter-params",
            program.resources.emitter_params_bytes,
            wgpu::BufferUsages::COPY_DST,
        );
        let counters_buffer = create_storage_buffer(
            device,
            "zircon-particle-gpu-counters",
            program.resources.counter_bytes,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        );
        let alive_indices_buffer = create_storage_buffer(
            device,
            "zircon-particle-gpu-alive-indices",
            program.resources.alive_indices_bytes,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        );
        let indirect_draw_args_buffer = create_storage_buffer(
            device,
            "zircon-particle-gpu-indirect-draw-args",
            program.resources.indirect_draw_args_bytes,
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC,
        );
        let debug_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-particle-gpu-debug-readback"),
            size: program.resources.debug_readback_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let update_bind_groups = vec![
            create_bind_group(
                device,
                &bind_group_layout,
                "zircon-particle-gpu-update-a-to-b",
                &particle_buffers[0],
                &particle_buffers[1],
                &emitter_params_buffer,
                &counters_buffer,
                &alive_indices_buffer,
                &indirect_draw_args_buffer,
            ),
            create_bind_group(
                device,
                &bind_group_layout,
                "zircon-particle-gpu-update-b-to-a",
                &particle_buffers[1],
                &particle_buffers[0],
                &emitter_params_buffer,
                &counters_buffer,
                &alive_indices_buffer,
                &indirect_draw_args_buffer,
            ),
        ];
        let compact_bind_groups = vec![
            create_bind_group(
                device,
                &bind_group_layout,
                "zircon-particle-gpu-compact-a",
                &particle_buffers[0],
                &particle_buffers[0],
                &emitter_params_buffer,
                &counters_buffer,
                &alive_indices_buffer,
                &indirect_draw_args_buffer,
            ),
            create_bind_group(
                device,
                &bind_group_layout,
                "zircon-particle-gpu-compact-b",
                &particle_buffers[1],
                &particle_buffers[1],
                &emitter_params_buffer,
                &counters_buffer,
                &alive_indices_buffer,
                &indirect_draw_args_buffer,
            ),
        ];

        Ok(Self {
            program,
            bind_group_layout,
            spawn_update_pipeline,
            compact_alive_pipeline,
            indirect_args_pipeline,
            particle_buffers,
            emitter_params_buffer,
            counters_buffer,
            alive_indices_buffer,
            indirect_draw_args_buffer,
            debug_readback_buffer,
            update_bind_groups,
            compact_bind_groups,
            active_buffer_index: 0,
            transparent_renderer: None,
        })
    }

    pub fn new_with_transparent_rendering(
        device: &wgpu::Device,
        asset: &ParticleSystemAsset,
        scene_layout: &wgpu::BindGroupLayout,
        config: ParticleGpuTransparentRenderConfig,
    ) -> Result<Self, ParticleGpuBackendError> {
        let mut backend = Self::new(device, asset)?;
        backend.enable_transparent_rendering(device, scene_layout, config);
        Ok(backend)
    }

    pub fn enable_transparent_rendering(
        &mut self,
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        config: ParticleGpuTransparentRenderConfig,
    ) {
        self.transparent_renderer = Some(ParticleGpuTransparentRenderer::new(
            device,
            scene_layout,
            config,
            &self.program.shader.transparent_wgsl,
            &self.program.shader.transparent_entries,
            &self.particle_buffers,
            &self.alive_indices_buffer,
        ));
    }

    pub fn program(&self) -> &ParticleGpuProgram {
        &self.program
    }

    pub fn active_buffers(&self) -> ParticleGpuBuffers<'_> {
        ParticleGpuBuffers {
            particles: &self.particle_buffers[self.active_buffer_index],
            alive_indices: &self.alive_indices_buffer,
            indirect_draw_args: &self.indirect_draw_args_buffer,
            counters: &self.counters_buffer,
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn transparent_render_enabled(&self) -> bool {
        self.transparent_renderer.is_some()
    }

    pub fn transparent_render_bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
        self.transparent_renderer
            .as_ref()
            .map(|renderer| renderer.bind_group_layout())
    }

    pub fn execute_frame(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        frame: &ParticleGpuFrameParams,
        readback: ParticleGpuReadbackRequest,
    ) -> Result<(), ParticleGpuBackendError> {
        let expected = self.program.layout.emitter_count;
        if frame.emitters.len() != expected as usize {
            return Err(ParticleGpuBackendError::FrameEmitterCountMismatch {
                expected,
                actual: frame.emitters.len(),
            });
        }

        let emitter_bytes = frame.encode_emitters(&self.program.layout);
        queue.write_buffer(&self.emitter_params_buffer, 0, &emitter_bytes);
        queue.write_buffer(&self.counters_buffer, 0, &zeroed_counters(expected));

        let workgroups = self
            .program
            .layout
            .dispatch_workgroups(PARTICLE_GPU_WORKGROUP_SIZE);
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ParticleGpuSpawnUpdatePass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.spawn_update_pipeline);
            pass.set_bind_group(0, &self.update_bind_groups[self.active_buffer_index], &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
        self.active_buffer_index = 1 - self.active_buffer_index;
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ParticleGpuCompactAndIndirectPass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.compact_alive_pipeline);
            pass.set_bind_group(0, &self.compact_bind_groups[self.active_buffer_index], &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
            pass.set_pipeline(&self.indirect_args_pipeline);
            pass.dispatch_workgroups(1, 1, 1);
        }

        if readback == ParticleGpuReadbackRequest::Counters {
            encoder.copy_buffer_to_buffer(
                &self.counters_buffer,
                0,
                &self.debug_readback_buffer,
                0,
                self.program.resources.counter_bytes,
            );
            encoder.copy_buffer_to_buffer(
                &self.indirect_draw_args_buffer,
                0,
                &self.debug_readback_buffer,
                self.program.resources.counter_bytes,
                self.program.resources.indirect_draw_args_bytes,
            );
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_transparent_render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        params: ParticleGpuTransparentRenderParams,
    ) -> Result<(), ParticleGpuBackendError> {
        let renderer = self
            .transparent_renderer
            .as_ref()
            .ok_or(ParticleGpuBackendError::TransparentRendererUnavailable)?;
        renderer.record(
            queue,
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            self.active_buffer_index,
            &self.indirect_draw_args_buffer,
            params,
        );
        Ok(())
    }

    pub fn read_counter_readback(
        &self,
        device: &wgpu::Device,
    ) -> Result<ParticleGpuCounterReadback, ParticleGpuBackendError> {
        let word_count =
            (self.program.resources.counter_bytes / std::mem::size_of::<u32>() as u64) as usize;
        let words = read_buffer_u32s_at(device, &self.debug_readback_buffer, 0, word_count)?;
        ParticleGpuCounterReadback::from_words(&words, self.program.layout.emitter_count)
            .map_err(ParticleGpuBackendError::from)
    }

    pub fn read_indirect_draw_args_readback(
        &self,
        device: &wgpu::Device,
    ) -> Result<[u32; 4], ParticleGpuBackendError> {
        let words = read_buffer_u32s_at(
            device,
            &self.debug_readback_buffer,
            self.program.resources.counter_bytes,
            4,
        )?;
        Ok([words[0], words[1], words[2], words[3]])
    }

    pub fn read_render_outputs_readback(
        &self,
        device: &wgpu::Device,
    ) -> Result<
        zircon_runtime::core::framework::render::RenderParticleGpuReadbackOutputs,
        ParticleGpuBackendError,
    > {
        let counters = self.read_counter_readback(device)?;
        let indirect_draw_args = self.read_indirect_draw_args_readback(device)?;
        Ok(counters.to_render_outputs(indirect_draw_args))
    }
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-particle-gpu-bind-group-layout"),
        entries: &[
            storage_entry(0, true),
            storage_entry(1, false),
            storage_entry(2, true),
            storage_entry(3, false),
            storage_entry(4, false),
            storage_entry(5, false),
        ],
    })
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn create_compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    label: &'static str,
    entry_point: &'static str,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module: shader,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_storage_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
    extra_usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(std::mem::size_of::<u32>() as u64),
        usage: wgpu::BufferUsages::STORAGE | extra_usage,
        mapped_at_creation: false,
    })
}

#[allow(clippy::too_many_arguments)]
fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    label: &'static str,
    current_particles: &wgpu::Buffer,
    next_particles: &wgpu::Buffer,
    emitter_params: &wgpu::Buffer,
    counters: &wgpu::Buffer,
    alive_indices: &wgpu::Buffer,
    indirect_draw_args: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: current_particles.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: next_particles.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: emitter_params.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: counters.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: alive_indices.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: indirect_draw_args.as_entire_binding(),
            },
        ],
    })
}

fn zeroed_counters(emitter_count: u32) -> Vec<u8> {
    let counter_words = PARTICLE_GPU_COUNTER_WORDS_BASE + emitter_count;
    vec![0; counter_words as usize * std::mem::size_of::<u32>()]
}

fn read_buffer_u32s_at(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    byte_offset: u64,
    word_count: usize,
) -> Result<Vec<u32>, ParticleGpuBackendError> {
    if word_count == 0 {
        return Ok(Vec::new());
    }

    let byte_count = word_count * std::mem::size_of::<u32>();
    let slice = buffer.slice(byte_offset..byte_offset + byte_count as u64);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| ParticleGpuBackendError::ReadbackMap(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| ParticleGpuBackendError::ReadbackMap(error.to_string()))?
        .map_err(|error| ParticleGpuBackendError::ReadbackMap(error.to_string()))?;

    let mapped = slice.get_mapped_range();
    let words = mapped
        .chunks_exact(std::mem::size_of::<u32>())
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();
    drop(mapped);
    buffer.unmap();

    Ok(words)
}
