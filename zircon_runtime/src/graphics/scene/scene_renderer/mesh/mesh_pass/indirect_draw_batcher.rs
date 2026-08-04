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

        let mut batcher = Self::default();
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
mod tests {
    use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::*;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    #[test]
    fn render_gpu_scene_indirect_batcher_groups_by_pipeline_geometry_material() {
        let commands = vec![
            command(10, 1, 2, 3),
            command(20, 3, 3, 2),
            command(30, 6, 1, 1).with_material(MeshBindHandle::test(901)),
        ];

        let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

        assert_eq!(batcher.fallback_draw_count(), 0);
        assert_eq!(batcher.args_cpu().len(), 3);
        assert_eq!(batcher.batches().len(), 2);
        assert_eq!(batcher.batches()[0].first_command_index, 0);
        assert_eq!(batcher.batches()[0].first_args, 0);
        assert_eq!(batcher.batches()[0].args_count, 2);
        assert_eq!(batcher.batches()[0].draw_count_index, 0);
        assert_eq!(batcher.batches()[0].total_instances, 5);
        assert_eq!(batcher.batches()[1].first_command_index, 2);
        assert_eq!(batcher.batches()[1].first_args, 2);
        assert_eq!(batcher.batches()[1].args_count, 1);
        assert_eq!(batcher.batches()[1].draw_count_index, 1);
        assert_eq!(batcher.batches()[1].total_instances, 1);
        assert_eq!(
            batcher.args_cpu()[1],
            IndexedIndirectArgs {
                index_count: 20,
                instance_count: 2,
                first_index: 3,
                base_vertex: 0,
                first_instance: 3,
            }
        );
        assert_eq!(
            batcher.stats(),
            IndirectDrawBatcherStats {
                batch_count: 2,
                batched_draw_count: 3,
                fallback_draw_count: 0,
                indirect_args_count: 3,
            }
        );
    }

    #[test]
    fn render_gpu_scene_indirect_batcher_keeps_per_draw_indirect_without_multi_draw() {
        let commands = vec![command(10, 1, 2, 1), command(20, 2, 1, 1)];

        let batcher = IndirectDrawBatcher::build(
            &commands,
            &RenderCapabilitySummary {
                supports_indirect_draw: true,
                supports_indirect_first_instance: true,
                ..RenderCapabilitySummary::default()
            },
        );

        assert_eq!(batcher.args_cpu().len(), 2);
        assert_eq!(batcher.batches().len(), 1);
        assert_eq!(batcher.fallback_draw_count(), 0);
        assert_eq!(
            batcher.stats(),
            IndirectDrawBatcherStats {
                batch_count: 1,
                batched_draw_count: 2,
                fallback_draw_count: 0,
                indirect_args_count: 2,
            }
        );
    }

    #[test]
    fn render_gpu_scene_indirect_batcher_uses_direct_draw_when_first_instance_is_unavailable() {
        let commands = vec![command(10, 1, 2, 1)];
        let batcher = IndirectDrawBatcher::build(
            &commands,
            &RenderCapabilitySummary {
                supports_indirect_draw: true,
                ..RenderCapabilitySummary::default()
            },
        );

        assert!(batcher.args_cpu().is_empty());
        assert!(batcher.batches().is_empty());
        assert_eq!(batcher.fallback_draw_count(), 1);
    }

    #[test]
    fn render_gpu_scene_indirect_batcher_keeps_existing_indirect_commands_on_fallback_path() {
        let commands = vec![
            command(10, 1, 1, 1),
            MeshDrawCommand::new(
                RenderPhase::Opaque3d,
                MeshPassPipelineKind::Base,
                default_pipeline_key(),
                MeshPipelineVariantId::new(1),
                20,
                DrawInstanceSource::GpuSceneInstance {
                    first_instance_index: 4,
                    instance_count: 1,
                },
                MeshGeometryHandle::test(7),
                MeshDrawArgs::test_indexed_indirect(99, 32),
            ),
        ];

        let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

        assert_eq!(batcher.args_cpu().len(), 1);
        assert_eq!(batcher.batches().len(), 1);
        assert_eq!(batcher.fallback_draw_count(), 1);
    }

    #[test]
    fn render_gpu_scene_indirect_batcher_keeps_command_local_gpu_scene_groups_on_direct_path() {
        let commands = vec![
            command(10, 1, 2, 1).with_gpu_scene_bind_group(MeshBindHandle::test(401)),
            command(20, 2, 3, 1),
        ];

        let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

        assert_eq!(batcher.args_cpu().len(), 1);
        assert_eq!(batcher.batches().len(), 1);
        assert_eq!(batcher.fallback_draw_count(), 1);
        assert_eq!(batcher.batches()[0].first_command_index, 1);
        assert_eq!(batcher.batches()[0].draw_count_index, 0);
    }

    #[test]
    fn render_gpu_scene_indirect_batcher_splits_velocity_draws_by_previous_geometry() {
        let commands = vec![
            velocity_command(10, 1),
            velocity_command(10, 2),
            velocity_command(20, 3),
        ];

        let batcher = IndirectDrawBatcher::build(&commands, &gpu_driven_capabilities());

        assert_eq!(batcher.fallback_draw_count(), 0);
        assert_eq!(batcher.args_cpu().len(), 3);
        assert_eq!(batcher.batches().len(), 2);
        assert_eq!(batcher.batches()[0].args_count, 2);
        assert_eq!(batcher.batches()[1].args_count, 1);
    }

    fn command(
        index_count: u32,
        first_index: u32,
        first_instance: u32,
        instance_count: u32,
    ) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            u64::from(first_instance),
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: first_instance,
                instance_count,
            },
            MeshGeometryHandle::test(7),
            MeshDrawArgs::DirectIndexed {
                first_index,
                index_count,
                first_instance,
                instance_count,
            },
        )
        .with_material_textures(MeshBindHandle::test(101))
        .with_material(MeshBindHandle::test(201))
        .with_standard_material(MeshBindHandle::test(301))
    }

    fn velocity_command(previous_geometry_id: u64, first_instance: u32) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Velocity,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            u64::from(first_instance),
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: first_instance,
                instance_count: 1,
            },
            MeshGeometryHandle::test(7),
            MeshDrawArgs::DirectIndexed {
                first_index: 0,
                index_count: 3,
                first_instance,
                instance_count: 1,
            },
        )
        .with_previous_velocity_geometry(MeshGeometryHandle::test(previous_geometry_id))
    }

    fn gpu_driven_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        }
    }
}
