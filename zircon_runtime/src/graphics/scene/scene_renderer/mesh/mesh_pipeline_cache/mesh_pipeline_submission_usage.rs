use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPipelineVariantId;
use crate::rhi::{SubmissionStatus, SubmissionTicket};

use super::{PipelineAdmissionKey, PipelineCreationTarget};

#[derive(Debug)]
struct SubmittedPipelineUsage {
    ticket: SubmissionTicket,
    variants: Vec<PipelineAdmissionKey>,
}

/// Tracks only pipelines that were actually bound into a submitted scene command buffer.
///
/// A variant keeps one last-use ticket per device-generation/queue timeline. Newer tickets
/// subsume older tickets on the same timeline, while independent timelines remain explicit.
/// Terminal collection scans only the bounded in-flight submission set, never the registry.
#[derive(Debug, Default)]
pub(super) struct MeshPipelineSubmissionUsage {
    recording: HashSet<PipelineAdmissionKey>,
    in_flight: Vec<SubmittedPipelineUsage>,
    last_uses: HashMap<PipelineAdmissionKey, Vec<SubmissionTicket>>,
}

impl MeshPipelineSubmissionUsage {
    pub(super) fn begin_recording(&mut self) {
        self.recording.clear();
    }

    pub(super) fn record_bound(
        &mut self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) {
        self.recording
            .insert(PipelineAdmissionKey::new(target, variant_id));
    }

    pub(super) fn bind_recorded_to_submission(&mut self, ticket: SubmissionTicket) {
        let mut submitted_variants = Vec::with_capacity(self.recording.len());
        for key in self.recording.drain() {
            let frontier = self.last_uses.entry(key).or_default();
            if let Some(existing) = frontier
                .iter_mut()
                .find(|existing| same_timeline(**existing, ticket))
            {
                if existing.sequence() >= ticket.sequence() {
                    continue;
                }
                *existing = ticket;
            } else {
                frontier.push(ticket);
            }
            submitted_variants.push(key);
        }
        if !submitted_variants.is_empty() {
            self.in_flight.push(SubmittedPipelineUsage {
                ticket,
                variants: submitted_variants,
            });
        }
    }

    pub(super) fn collect_terminal_submissions(
        &mut self,
        mut status_for: impl FnMut(SubmissionTicket) -> Option<SubmissionStatus>,
    ) {
        let last_uses = &mut self.last_uses;
        self.in_flight.retain(|submission| {
            let Some(status) = status_for(submission.ticket) else {
                return true;
            };
            if !status.is_terminal() {
                return true;
            }

            for key in &submission.variants {
                if let Entry::Occupied(mut entry) = last_uses.entry(*key) {
                    entry
                        .get_mut()
                        .retain(|ticket| *ticket != submission.ticket);
                    if entry.get().is_empty() {
                        entry.remove();
                    }
                }
            }
            false
        });
    }
}

fn same_timeline(left: SubmissionTicket, right: SubmissionTicket) -> bool {
    left.device_id() == right.device_id()
        && left.generation() == right.generation()
        && left.queue_class() == right.queue_class()
}

#[cfg(test)]
mod tests {
    use crate::rhi::{
        DeviceGeneration, DeviceId, RenderQueueClass, SubmissionStatus, SubmissionTicket,
    };

    use super::super::{PipelineAdmissionKey, PipelineCreationTarget};
    use super::MeshPipelineSubmissionUsage;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    fn ticket(
        device_id: u64,
        generation: u64,
        queue_class: RenderQueueClass,
        sequence: u64,
    ) -> SubmissionTicket {
        SubmissionTicket::new(
            DeviceId::new(device_id),
            DeviceGeneration::new(generation),
            queue_class,
            sequence,
        )
    }

    fn key(target: PipelineCreationTarget, variant: u32) -> PipelineAdmissionKey {
        PipelineAdmissionKey::new(target, MeshPipelineVariantId::new(variant))
    }

    fn assert_all_ready_lookups_record_usage(
        source: &str,
        ready_lookup: &str,
        usage_recorder: &str,
    ) {
        let mut lookup_count = 0usize;
        for (offset, _) in source.match_indices(ready_lookup) {
            lookup_count += 1;
            let nearby_prefix_has_recorder = source[..offset]
                .lines()
                .rev()
                .take(16)
                .any(|line| line.contains(usage_recorder));
            assert!(
                nearby_prefix_has_recorder,
                "`{ready_lookup}` must record actual usage before returning the pipeline"
            );
        }
        assert!(
            lookup_count > 0,
            "expected at least one `{ready_lookup}` call"
        );
    }

    #[test]
    fn recording_deduplicates_actual_pipeline_bindings_before_submission() {
        let mut usage = MeshPipelineSubmissionUsage::default();
        let key = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            7,
        );
        let submission = ticket(1, 1, RenderQueueClass::Graphics, 3);

        usage.begin_recording();
        usage.record_bound(key.target, key.variant_id);
        usage.record_bound(key.target, key.variant_id);
        usage.bind_recorded_to_submission(submission);

        assert!(usage.recording.is_empty());
        assert_eq!(usage.in_flight.len(), 1);
        assert_eq!(usage.in_flight[0].variants, vec![key]);
        assert_eq!(usage.last_uses.get(&key), Some(&vec![submission]));
    }

    #[test]
    fn newer_ticket_replaces_only_the_same_timeline_frontier() {
        let mut usage = MeshPipelineSubmissionUsage::default();
        let key = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            9,
        );
        let first = ticket(1, 1, RenderQueueClass::Graphics, 4);
        let newer = ticket(1, 1, RenderQueueClass::Graphics, 5);
        let compute = ticket(1, 1, RenderQueueClass::Compute, 2);

        for submission in [first, newer, compute] {
            usage.begin_recording();
            usage.record_bound(key.target, key.variant_id);
            usage.bind_recorded_to_submission(submission);
        }

        let mut frontier = usage.last_uses[&key].clone();
        frontier.sort_by_key(|submission| (submission.queue_class() as u8, submission.sequence()));
        assert_eq!(frontier, vec![newer, compute]);

        usage.collect_terminal_submissions(|submission| {
            (submission == first).then_some(SubmissionStatus::Completed)
        });
        assert_eq!(usage.last_uses[&key].len(), 2);
    }

    #[test]
    fn a_new_device_generation_keeps_an_independent_last_use() {
        let mut usage = MeshPipelineSubmissionUsage::default();
        let key = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer),
            11,
        );
        let old_generation = ticket(1, 1, RenderQueueClass::Graphics, 8);
        let new_generation = ticket(1, 2, RenderQueueClass::Graphics, 1);

        for submission in [old_generation, new_generation] {
            usage.begin_recording();
            usage.record_bound(key.target, key.variant_id);
            usage.bind_recorded_to_submission(submission);
        }

        assert_eq!(usage.last_uses[&key].len(), 2);
    }

    #[test]
    fn every_terminal_status_releases_only_its_exact_frontier() {
        let mut usage = MeshPipelineSubmissionUsage::default();
        let base = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            13,
        );
        let oit = key(PipelineCreationTarget::Oit, 13);
        let submission = ticket(1, 1, RenderQueueClass::Graphics, 12);

        usage.begin_recording();
        usage.record_bound(base.target, base.variant_id);
        usage.record_bound(oit.target, oit.variant_id);
        usage.bind_recorded_to_submission(submission);
        usage.collect_terminal_submissions(|candidate| {
            (candidate == submission).then_some(SubmissionStatus::Failed)
        });

        assert!(usage.in_flight.is_empty());
        assert!(usage.last_uses.is_empty());
    }

    #[test]
    fn unknown_or_nonterminal_status_is_fail_closed() {
        let mut usage = MeshPipelineSubmissionUsage::default();
        let key = key(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity),
            17,
        );
        let submission = ticket(1, 1, RenderQueueClass::Graphics, 20);

        usage.begin_recording();
        usage.record_bound(key.target, key.variant_id);
        usage.bind_recorded_to_submission(submission);
        usage.collect_terminal_submissions(|_| None);
        usage.collect_terminal_submissions(|_| Some(SubmissionStatus::Submitted));

        assert_eq!(usage.in_flight.len(), 1);
        assert_eq!(usage.last_uses.get(&key), Some(&vec![submission]));
    }

    #[test]
    fn every_mesh_ready_lookup_records_the_exact_pipeline_target() {
        let base = include_str!("../../overlay/passes/base_scene_pass.rs");
        let gbuffer =
            include_str!("../../deferred/deferred_scene_resources/record_gbuffer_geometry.rs");
        let graph_mesh = include_str!(
            "../../graph_execution/render_pass_execution_context/gpu/mesh_recording.rs"
        );
        let oit = include_str!("../../graph_execution/render_pass_execution_context/gpu/oit.rs");
        let shadow = include_str!("../../shadow/shadow_map_renderer.rs");
        let velocity = include_str!("../../temporal/velocity/execute_velocity_object.rs");

        assert_all_ready_lookups_record_usage(
            base,
            "base_pipeline_for_ready_variant(",
            "record_bound_mesh_pass_pipeline(",
        );
        assert_all_ready_lookups_record_usage(
            gbuffer,
            "gbuffer_pipeline_for_ready_variant(",
            "record_bound_mesh_pass_pipeline(",
        );
        assert_all_ready_lookups_record_usage(
            graph_mesh,
            "depth_prepass_pipeline_for_ready_variant(",
            "record_bound_mesh_pass_pipeline(",
        );
        assert_all_ready_lookups_record_usage(
            graph_mesh,
            "taa_reactive_pipeline_for_ready_variant(",
            "record_bound_mesh_pass_pipeline(",
        );
        assert_all_ready_lookups_record_usage(
            oit,
            "oit_pipeline_for_ready_base_variant(",
            "record_bound_oit_pipeline(",
        );
        assert_all_ready_lookups_record_usage(
            shadow,
            "shadow_pipeline_for_ready_variant(",
            "record_bound_mesh_pass_pipeline(",
        );
        assert_all_ready_lookups_record_usage(
            velocity,
            "velocity_pipeline_for_ready_variant(",
            "record_bound_mesh_pass_pipeline(",
        );
    }

    #[test]
    fn direct_and_compiled_paths_bind_usage_after_their_scene_submit() {
        let direct = include_str!("../../core/scene_renderer_core_render_scene/render_scene.rs");
        let direct_begin = direct
            .find("begin_submission_usage_recording()")
            .expect("direct recording begin");
        let direct_record = direct
            .find("record_scene_content(")
            .expect("direct mesh recording");
        let direct_submit = direct
            .find("let scene_submission = match backend.submit_graphics_command_buffers")
            .expect("direct scene submit");
        let direct_bind = direct
            .find("bind_recorded_pipeline_usage_to_submission(scene_submission)")
            .expect("direct usage ticket binding");
        assert!(direct_begin < direct_record);
        assert!(direct_record < direct_submit);
        assert!(direct_submit < direct_bind);

        let compiled =
            include_str!("../../core/scene_renderer_core_render_compiled_scene/render/render.rs");
        let compiled_begin = compiled
            .find("begin_submission_usage_recording()")
            .expect("compiled recording begin");
        let compiled_execute = compiled
            .find("execute_compiled_scene_graph_stages(")
            .expect("compiled mesh graph recording");
        assert!(compiled_begin < compiled_execute);

        let compiled_submit = include_str!(
            "../../core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs"
        );
        let submit = compiled_submit
            .find("let submission_ticket = backend.submit_graphics_command_buffers")
            .expect("compiled scene submit");
        let bind = compiled_submit
            .find("bind_recorded_pipeline_usage_to_submission(submission_ticket)")
            .expect("compiled usage ticket binding");
        assert!(submit < bind);
    }
}
