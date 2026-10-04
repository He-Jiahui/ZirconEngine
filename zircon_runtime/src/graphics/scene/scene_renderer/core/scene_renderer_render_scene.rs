use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use crate::graphics::scene::gpu_scene::GpuSceneJournalApplyPlan;
use crate::graphics::scene::render_scene::RenderSceneComponentProjectionCommit;
use crate::graphics::scene::resources::RenderSceneWorldReleaseError;
use crate::graphics::types::GraphicsError;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(in crate::graphics::scene::scene_renderer::core) fn admit_render_scene_frame(
        &mut self,
        frame: &RenderFrameExtract,
        frame_generation: u64,
    ) -> Result<(), GraphicsError> {
        let projector = self.render_scene_registry.projector_for_frame(frame);
        let commit = self
            .streamer
            .admit_render_scene_frame(projector, &self.backend, frame, frame_generation)
            .map_err(|error| GraphicsError::RenderSceneAdmission(error.to_string()))?;

        match commit {
            Some(RenderSceneComponentProjectionCommit::Applied { journal, .. }) => {
                let world = journal.world();
                let mut projected_consumer = self
                    .gpu_scene_journal_consumers
                    .entry(world.raw())
                    .or_insert_with(|| {
                        crate::graphics::scene::gpu_scene::GpuSceneJournalConsumer::new(world)
                    })
                    .clone();
                for pending in self
                    .pending_gpu_scene_journals
                    .iter()
                    .filter(|pending| pending.journal().world() == world)
                {
                    projected_consumer
                        .apply_with_staging(pending.journal(), |_| {
                            Ok::<(), std::convert::Infallible>(())
                        })
                        .map_err(|error| GraphicsError::RenderSceneAdmission(error.to_string()))?;
                }
                self.pending_gpu_scene_journals.push_back(
                    projected_consumer
                        .stage_owned(journal, |_| Ok::<(), std::convert::Infallible>(()))
                        .map_err(|error| GraphicsError::RenderSceneAdmission(error.to_string()))?,
                );
                let journal = self
                    .pending_gpu_scene_journals
                    .back()
                    .expect("applied RenderScene journal must retain staged GPUScene ownership")
                    .journal();
                crate::profile_counter!(
                    "render",
                    "render_scene_journal_additions",
                    journal.additions().len() as u64
                );
                crate::profile_counter!(
                    "render",
                    "render_scene_journal_updates",
                    journal.updates().len() as u64
                );
                crate::profile_counter!(
                    "render",
                    "render_scene_journal_removals",
                    journal.removals().len() as u64
                );
                crate::profile_counter!(
                    "render",
                    "render_scene_resource_reference_deltas",
                    journal.resource_reference_deltas().len() as u64
                );
            }
            Some(RenderSceneComponentProjectionCommit::Replayed) => {
                crate::profile_counter!("render", "render_scene_exact_replays", 1);
            }
            None => {}
        }
        crate::profile_counter!(
            "render",
            "render_asset_residency_pending_requests",
            self.streamer.pending_render_asset_residency_request_count() as u64
        );
        crate::profile_counter!(
            "render",
            "render_asset_residency_pending_retirements",
            self.streamer
                .pending_render_asset_residency_retirement_count() as u64
        );
        crate::profile_counter!(
            "render",
            "render_asset_residency_pending_semantic_cancellations",
            self.streamer
                .pending_render_asset_semantic_cancellation_count() as u64
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn stage_pending_gpu_scene_membership(
        &mut self,
    ) -> Result<(), GraphicsError> {
        let mut projected_consumers = self.gpu_scene_journal_consumers.clone();
        for pending in &self.pending_gpu_scene_journals {
            let world = pending.journal().world();
            let consumer = projected_consumers.get_mut(&world.raw()).ok_or_else(|| {
                GraphicsError::RenderSceneAdmission(format!(
                    "GPUScene journal consumer for world {} disappeared before membership staging",
                    world.raw()
                ))
            })?;
            let transaction = consumer
                .stage(pending.journal(), |plan: &GpuSceneJournalApplyPlan<'_>| {
                    self.core
                        .gpu_scene
                        .apply_journal_membership(&self.backend.device, plan);
                    Ok::<(), std::convert::Infallible>(())
                })
                .map_err(|error| GraphicsError::RenderSceneAdmission(error.to_string()))?;
            transaction
                .commit(consumer)
                .map_err(|error| GraphicsError::RenderSceneAdmission(error.to_string()))?;
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn commit_pending_gpu_scene_journals(
        &mut self,
    ) -> Result<(), GraphicsError> {
        let mut consumers = self.gpu_scene_journal_consumers.clone();
        let pending = std::mem::take(&mut self.pending_gpu_scene_journals);
        for staged in pending {
            let world = staged.journal().world();
            let consumer = consumers.get_mut(&world.raw()).ok_or_else(|| {
                GraphicsError::RenderSceneAdmission(format!(
                    "GPUScene journal consumer for world {} disappeared before submission commit",
                    world.raw()
                ))
            })?;
            staged
                .commit(consumer)
                .map_err(|error| GraphicsError::RenderSceneAdmission(error.to_string()))?;
        }
        self.gpu_scene_journal_consumers = consumers;
        Ok(())
    }

    pub(crate) fn release_render_scene_world(
        &mut self,
        world: RenderWorldSnapshotHandle,
        frame_generation: u64,
    ) -> Result<bool, RenderSceneWorldReleaseError> {
        let released = self.streamer.release_render_scene_world(
            &mut self.render_scene_registry,
            &self.backend,
            world,
            frame_generation,
        )?;
        if released {
            if let Some(consumer) = self.gpu_scene_journal_consumers.remove(&world.raw()) {
                for stable_instance_key in consumer.resident_stable_keys() {
                    self.core.gpu_scene.unregister(stable_instance_key);
                }
            }
            self.pending_gpu_scene_journals
                .retain(|pending| pending.journal().world() != world);
        }
        Ok(released)
    }
}

#[cfg(test)]
#[path = "tests/scene_renderer_render_scene.rs"]
mod tests;
