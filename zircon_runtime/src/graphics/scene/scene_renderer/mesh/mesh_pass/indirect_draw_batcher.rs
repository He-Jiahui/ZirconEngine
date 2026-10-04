use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;

use super::{MeshDrawArgs, MeshDrawCommand, MeshPassPipelineKind, MeshPipelineVariantId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IndirectDrawBatcher {
    args_cpu: Vec<IndexedIndirectArgs>,
    batches: Vec<IndirectDrawBatch>,
    fallback_draw_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IndirectDrawBatch {
    pub(crate) phase: RenderPhase,
    pub(crate) pipeline_kind: MeshPassPipelineKind,
    pub(crate) pipeline_variant_id: MeshPipelineVariantId,
    pub(crate) pipeline_key: PipelineKey,
    pub(crate) geometry_id: u64,
    pub(crate) first_command_index: usize,
    pub(crate) first_args: u32,
    pub(crate) args_count: u32,
    pub(crate) draw_count_index: u32,
    pub(crate) total_instances: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct IndirectDrawBatcherStats {
    pub(crate) batch_count: usize,
    pub(crate) batched_draw_count: usize,
    pub(crate) fallback_draw_count: usize,
    pub(crate) indirect_args_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IndirectDrawBatchKey {
    phase: RenderPhase,
    pipeline_kind: MeshPassPipelineKind,
    pipeline_variant_id: MeshPipelineVariantId,
    pipeline_key: PipelineKey,
    geometry_bind_key: (u64, u64),
    material_textures_id: Option<u64>,
    base_color_texture_id: Option<u64>,
    material_id: Option<u64>,
    standard_material_id: Option<u64>,
    gpu_scene_bind_group_id: Option<u64>,
}

impl IndirectDrawBatcher {
    /// 按原命令顺序聚合相邻且绑定身份相同的直接 indexed draw。
    /// 设备不支持 indirect first-instance 或命令带局部 GPUScene 绑定时走直接回退。
    pub(crate) fn build(
        commands: &[MeshDrawCommand],
        capabilities: &RenderCapabilitySummary,
    ) -> Self {
        if !capabilities.indirect_draw_submission_supported() {
            return Self {
                fallback_draw_count: commands.len(),
                ..Self::default()
            };
        }

        let mut batcher = Self {
            args_cpu: Vec::with_capacity(commands.len()),
            ..Self::default()
        };
        let mut active_key = None::<IndirectDrawBatchKey>;

        for (command_index, command) in commands.iter().enumerate() {
            let Some(args) = indirect_args_for_command(command) else {
                batcher.fallback_draw_count += 1;
                active_key = None;
                continue;
            };
            let key = IndirectDrawBatchKey::from_command(command);
            let next_arg_index = batcher.args_cpu.len() as u32;
            batcher.args_cpu.push(args);

            if active_key.as_ref() == Some(&key) {
                let batch = batcher
                    .batches
                    .last_mut()
                    .expect("active indirect key must have a matching batch");
                batch.args_count += 1;
                batch.total_instances = batch.total_instances.saturating_add(args.instance_count);
            } else {
                let draw_count_index = batcher.batches.len() as u32;
                batcher.batches.push(IndirectDrawBatch {
                    phase: key.phase,
                    pipeline_kind: key.pipeline_kind,
                    pipeline_variant_id: key.pipeline_variant_id,
                    pipeline_key: key.pipeline_key.clone(),
                    geometry_id: key.geometry_bind_key.0,
                    first_command_index: command_index,
                    first_args: next_arg_index,
                    args_count: 1,
                    draw_count_index,
                    total_instances: args.instance_count,
                });
                active_key = Some(key);
            }
        }

        batcher
    }

    pub(crate) fn args_cpu(&self) -> &[IndexedIndirectArgs] {
        &self.args_cpu
    }

    pub(crate) fn batches(&self) -> &[IndirectDrawBatch] {
        &self.batches
    }

    pub(crate) fn into_execution_parts(self) -> (Vec<IndexedIndirectArgs>, Vec<IndirectDrawBatch>) {
        (self.args_cpu, self.batches)
    }

    pub(crate) const fn fallback_draw_count(&self) -> usize {
        self.fallback_draw_count
    }

    pub(crate) fn stats(&self) -> IndirectDrawBatcherStats {
        IndirectDrawBatcherStats {
            batch_count: self.batches.len(),
            batched_draw_count: self.args_cpu.len(),
            fallback_draw_count: self.fallback_draw_count,
            indirect_args_count: self.args_cpu.len(),
        }
    }
}

impl IndirectDrawBatchKey {
    fn from_command(command: &MeshDrawCommand) -> Self {
        Self {
            phase: command.phase,
            pipeline_kind: command.pipeline_kind,
            pipeline_variant_id: command.pipeline_variant_id,
            pipeline_key: command.pipeline_key().clone(),
            geometry_bind_key: command.geometry_bind_key(),
            material_textures_id: command.material_textures.as_ref().map(|handle| handle.id()),
            base_color_texture_id: command
                .base_color_texture
                .as_ref()
                .map(|handle| handle.id()),
            material_id: command.material.as_ref().map(|handle| handle.id()),
            standard_material_id: command.standard_material.as_ref().map(|handle| handle.id()),
            gpu_scene_bind_group_id: command
                .gpu_scene_bind_group
                .as_ref()
                .map(|handle| handle.id()),
        }
    }
}

fn indirect_args_for_command(command: &MeshDrawCommand) -> Option<IndexedIndirectArgs> {
    if command.gpu_scene_bind_group.is_some() {
        return None;
    }

    match command.draw_args {
        MeshDrawArgs::DirectIndexed {
            first_index,
            index_count,
            first_instance,
            instance_count,
        } => Some(IndexedIndirectArgs {
            index_count,
            instance_count,
            first_index,
            base_vertex: 0,
            first_instance,
        }),
        MeshDrawArgs::IndexedIndirect { .. } => None,
    }
}

#[cfg(test)]
#[path = "tests/indirect_draw_batcher.rs"]
mod tests;
