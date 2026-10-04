use std::collections::{hash_map::Entry, HashMap, HashSet};

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
#[path = "tests/mesh_pipeline_submission_usage.rs"]
mod tests;
