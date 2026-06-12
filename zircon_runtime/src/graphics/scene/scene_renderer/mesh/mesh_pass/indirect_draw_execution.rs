use wgpu::util::DeviceExt;

use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;

use super::{IndirectDrawBatch, IndirectDrawBatcher, MeshDrawCommand, MeshPassCommandBuffers};

pub(crate) const INDEXED_INDIRECT_ARGS_STRIDE_BYTES: wgpu::BufferAddress =
    std::mem::size_of::<IndexedIndirectArgs>() as wgpu::BufferAddress;

#[derive(Clone, Copy)]
pub(crate) struct MeshDrawCommandStream<'a> {
    commands: &'a [MeshDrawCommand],
    indirect: Option<&'a MeshIndirectDrawExecution>,
}

impl<'a> MeshDrawCommandStream<'a> {
    pub(crate) fn new(
        commands: &'a [MeshDrawCommand],
        indirect: Option<&'a MeshIndirectDrawExecution>,
    ) -> Self {
        Self { commands, indirect }
    }

    pub(crate) fn empty() -> Self {
        Self {
            commands: &[],
            indirect: None,
        }
    }

    pub(crate) fn commands(self) -> &'a [MeshDrawCommand] {
        self.commands
    }

    pub(crate) fn indirect(self) -> Option<&'a MeshIndirectDrawExecution> {
        self.indirect
    }

    pub(crate) fn is_empty(self) -> bool {
        self.commands.is_empty()
    }
}

pub(crate) struct MeshIndirectDrawExecution {
    args_buffer: wgpu::Buffer,
    batches: Vec<IndirectDrawBatch>,
}

impl MeshIndirectDrawExecution {
    pub(crate) fn build(
        device: &wgpu::Device,
        label: &'static str,
        commands: &[MeshDrawCommand],
        capabilities: &RenderCapabilitySummary,
    ) -> Option<Self> {
        let batcher = IndirectDrawBatcher::build(commands, capabilities);
        if batcher.args_cpu().is_empty() || batcher.batches().is_empty() {
            return None;
        }

        let args_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(batcher.args_cpu()),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        Some(Self {
            args_buffer,
            batches: batcher.batches().to_vec(),
        })
    }

    pub(crate) fn args_buffer(&self) -> &wgpu::Buffer {
        &self.args_buffer
    }

    pub(crate) fn batches(&self) -> &[IndirectDrawBatch] {
        &self.batches
    }
}

#[derive(Default)]
pub(crate) struct MeshPassIndirectDrawExecutions {
    depth_prepass: Option<MeshIndirectDrawExecution>,
    shadow: Option<MeshIndirectDrawExecution>,
    opaque: Option<MeshIndirectDrawExecution>,
    alpha_mask: Option<MeshIndirectDrawExecution>,
    transparent: Option<MeshIndirectDrawExecution>,
    velocity: Option<MeshIndirectDrawExecution>,
}

impl MeshPassIndirectDrawExecutions {
    pub(crate) fn build(
        device: &wgpu::Device,
        capabilities: &RenderCapabilitySummary,
        command_buffers: &MeshPassCommandBuffers,
    ) -> Self {
        Self {
            depth_prepass: MeshIndirectDrawExecution::build(
                device,
                "zircon-depth-prepass-indirect-args",
                command_buffers.depth_prepass().commands(),
                capabilities,
            ),
            shadow: MeshIndirectDrawExecution::build(
                device,
                "zircon-shadow-indirect-args",
                command_buffers.shadow().commands(),
                capabilities,
            ),
            opaque: MeshIndirectDrawExecution::build(
                device,
                "zircon-opaque-indirect-args",
                command_buffers.opaque().commands(),
                capabilities,
            ),
            alpha_mask: MeshIndirectDrawExecution::build(
                device,
                "zircon-alpha-mask-indirect-args",
                command_buffers.alpha_mask().commands(),
                capabilities,
            ),
            transparent: MeshIndirectDrawExecution::build(
                device,
                "zircon-transparent-indirect-args",
                command_buffers.transparent().commands(),
                capabilities,
            ),
            velocity: MeshIndirectDrawExecution::build(
                device,
                "zircon-velocity-indirect-args",
                command_buffers.velocity().commands(),
                capabilities,
            ),
        }
    }

    pub(crate) fn depth_prepass(&self) -> Option<&MeshIndirectDrawExecution> {
        self.depth_prepass.as_ref()
    }

    pub(crate) fn shadow(&self) -> Option<&MeshIndirectDrawExecution> {
        self.shadow.as_ref()
    }

    pub(crate) fn opaque(&self) -> Option<&MeshIndirectDrawExecution> {
        self.opaque.as_ref()
    }

    pub(crate) fn alpha_mask(&self) -> Option<&MeshIndirectDrawExecution> {
        self.alpha_mask.as_ref()
    }

    pub(crate) fn transparent(&self) -> Option<&MeshIndirectDrawExecution> {
        self.transparent.as_ref()
    }

    pub(crate) fn velocity(&self) -> Option<&MeshIndirectDrawExecution> {
        self.velocity.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::INDEXED_INDIRECT_ARGS_STRIDE_BYTES;

    #[test]
    fn mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer() {
        let source = include_str!("indirect_draw_execution.rs");

        assert_eq!(INDEXED_INDIRECT_ARGS_STRIDE_BYTES, 20);
        assert!(source.contains("create_buffer_init"));
        assert!(source.contains("wgpu::BufferUsages::INDIRECT"));
        assert!(source.contains("bytemuck::cast_slice(batcher.args_cpu())"));
    }
}
