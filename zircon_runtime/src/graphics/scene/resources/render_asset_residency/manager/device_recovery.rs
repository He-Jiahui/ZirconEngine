use crate::core::resource::{
    ResourceManagementGeneration, ResourceReadinessGeneration, UntypedResourceHandle,
};

use super::{
    PendingResidency, RenderAssetResidencyManager, release_active, release_pending,
    resolve_ticket_seed,
};
use crate::graphics::scene::resources::render_asset_residency::{
    RenderAssetDemandGeneration, RenderAssetDeviceEpoch, RenderAssetResidencyAdmissionError,
    RenderAssetResidencyMutation, RenderAssetResidencyMutationStats, RenderAssetResidencyRelease,
    RenderAssetResidencyState, RenderAssetResidencyTicket, RenderAssetResidencyTicketId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetDeviceRecoveryError {
    UnchangedEpoch {
        epoch: RenderAssetDeviceEpoch,
    },
    NonAdvancingGeneration {
        failed: RenderAssetDeviceEpoch,
        replacement: RenderAssetDeviceEpoch,
    },
    ResidencyEpochMismatch {
        resource: UntypedResourceHandle,
        expected: RenderAssetDeviceEpoch,
        actual: RenderAssetDeviceEpoch,
    },
    GpuEpochMismatch {
        expected: RenderAssetDeviceEpoch,
        actual: RenderAssetDeviceEpoch,
    },
    MissingResidencyEntry {
        resource: UntypedResourceHandle,
    },
    Admission(RenderAssetResidencyAdmissionError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderAssetDeviceRecoveryReport {
    failed_epoch: RenderAssetDeviceEpoch,
    replacement_epoch: RenderAssetDeviceEpoch,
    mutation: RenderAssetResidencyMutation,
    abandoned_tracked_submissions: usize,
    abandoned_active_artifacts: usize,
    abandoned_pending_uploads: usize,
    abandoned_detached_uploads: usize,
    abandoned_ready_retirements: usize,
    abandoned_allocation_bytes: u64,
}

impl RenderAssetDeviceRecoveryReport {
    pub(crate) const fn failed_epoch(&self) -> RenderAssetDeviceEpoch {
        self.failed_epoch
    }

    pub(crate) const fn replacement_epoch(&self) -> RenderAssetDeviceEpoch {
        self.replacement_epoch
    }

    pub(crate) const fn mutation(&self) -> &RenderAssetResidencyMutation {
        &self.mutation
    }

    pub(crate) const fn abandoned_tracked_submissions(&self) -> usize {
        self.abandoned_tracked_submissions
    }

    pub(crate) const fn abandoned_active_artifacts(&self) -> usize {
        self.abandoned_active_artifacts
    }

    pub(crate) const fn abandoned_pending_uploads(&self) -> usize {
        self.abandoned_pending_uploads
    }

    pub(crate) const fn abandoned_detached_uploads(&self) -> usize {
        self.abandoned_detached_uploads
    }

    pub(crate) const fn abandoned_ready_retirements(&self) -> usize {
        self.abandoned_ready_retirements
    }

    pub(crate) const fn abandoned_allocation_bytes(&self) -> u64 {
        self.abandoned_allocation_bytes
    }
}

#[derive(Clone, Copy)]
struct PreparedDeviceRecovery {
    resource: UntypedResourceHandle,
    ticket: RenderAssetResidencyTicket,
}

impl RenderAssetResidencyManager {
    pub(super) fn ensure_device_epoch_admission(
        &self,
        actual: RenderAssetDeviceEpoch,
    ) -> Result<(), RenderAssetResidencyAdmissionError> {
        let Some(expected) = self.gpu.bound_device_epoch() else {
            return Ok(());
        };
        if actual == expected {
            return Ok(());
        }
        Err(RenderAssetResidencyAdmissionError::DeviceEpochMismatch { expected, actual })
    }

    /// Invalidates one failed GPU generation and reissues every live resource
    /// against its replacement after a complete, non-mutating preflight.
    pub(crate) fn recover_device_epoch(
        &mut self,
        failed: RenderAssetDeviceEpoch,
        replacement: RenderAssetDeviceEpoch,
        management: &ResourceManagementGeneration,
        readiness: &ResourceReadinessGeneration,
        demand_generation: RenderAssetDemandGeneration,
    ) -> Result<RenderAssetDeviceRecoveryReport, RenderAssetDeviceRecoveryError> {
        if failed == replacement {
            return Err(RenderAssetDeviceRecoveryError::UnchangedEpoch { epoch: failed });
        }
        if failed.device_id() == replacement.device_id()
            && replacement.generation().raw() <= failed.generation().raw()
        {
            return Err(RenderAssetDeviceRecoveryError::NonAdvancingGeneration {
                failed,
                replacement,
            });
        }
        if let Some(actual) = self.gpu.bound_device_epoch() {
            if actual != failed {
                return Err(RenderAssetDeviceRecoveryError::GpuEpochMismatch {
                    expected: failed,
                    actual,
                });
            }
        }

        let mut resources = self.entries.keys().copied().collect::<Vec<_>>();
        resources.sort_by_key(|resource| resource.id());
        let mut seeds = Vec::with_capacity(resources.len());
        for resource in resources.iter().copied() {
            let Some(entry) = self.entries.get(&resource) else {
                return Err(RenderAssetDeviceRecoveryError::MissingResidencyEntry { resource });
            };
            for actual in [
                entry.pending.map(|pending| pending.ticket.device()),
                entry.active.map(|active| active.ticket.device()),
            ]
            .into_iter()
            .flatten()
            {
                if actual != failed {
                    return Err(RenderAssetDeviceRecoveryError::ResidencyEpochMismatch {
                        resource,
                        expected: failed,
                        actual,
                    });
                }
            }
            let seed = resolve_ticket_seed(
                resource,
                management,
                readiness,
                replacement,
                demand_generation,
            )
            .map_err(RenderAssetDeviceRecoveryError::Admission)?;
            seeds.push((resource, seed));
        }

        let mut next_ticket_id = self
            .reserve_ticket_ids(seeds.len())
            .map_err(RenderAssetDeviceRecoveryError::Admission)?;
        let mut prepared = Vec::with_capacity(seeds.len());
        for (resource, seed) in seeds {
            let exhausted = RenderAssetDeviceRecoveryError::Admission(
                RenderAssetResidencyAdmissionError::TicketIdExhausted,
            );
            let id = RenderAssetResidencyTicketId::new(next_ticket_id).ok_or(exhausted)?;
            prepared.push(PreparedDeviceRecovery {
                resource,
                ticket: seed.issue(id),
            });
            next_ticket_id = next_ticket_id.checked_add(1).ok_or(exhausted)?;
        }

        self.next_ticket_id = next_ticket_id;
        let gpu = self.gpu.abandon_for_device_recovery(replacement);
        let mut requests = Vec::with_capacity(prepared.len());
        let mut releases = Vec::<RenderAssetResidencyRelease>::with_capacity(prepared.len());
        let mut abandoned_active_artifacts = 0_usize;
        let mut abandoned_pending_uploads = 0_usize;
        let mut abandoned_entry_bytes = 0_u64;
        for recovery in prepared {
            let Some(entry) = self.entries.get_mut(&recovery.resource) else {
                debug_assert!(
                    false,
                    "preflighted residency entry disappeared during commit"
                );
                continue;
            };
            if let Some(pending) = entry.pending.take() {
                releases.push(release_pending(pending));
            }
            if let Some(active) = entry.active.take() {
                releases.push(release_active(active));
            }
            if let Some(upload) = entry.pending_upload.take() {
                abandoned_pending_uploads = abandoned_pending_uploads.saturating_add(1);
                abandoned_entry_bytes =
                    abandoned_entry_bytes.saturating_add(upload.artifact().allocation_bytes());
            }
            if let Some(artifact) = entry.active_artifact.take() {
                abandoned_active_artifacts = abandoned_active_artifacts.saturating_add(1);
                abandoned_entry_bytes =
                    abandoned_entry_bytes.saturating_add(artifact.allocation_bytes());
            }
            entry.pending = Some(PendingResidency {
                ticket: recovery.ticket,
                state: RenderAssetResidencyState::QueuedIo,
                submission: None,
            });
            requests.push(recovery.ticket);
        }

        let stats = RenderAssetResidencyMutationStats {
            input_delta_count: resources.len(),
            preflight_entry_lookup_count: resources.len(),
            catalog_lookup_count: resources.len(),
            ..RenderAssetResidencyMutationStats::default()
        };
        Ok(RenderAssetDeviceRecoveryReport {
            failed_epoch: failed,
            replacement_epoch: replacement,
            mutation: RenderAssetResidencyMutation::from_parts(requests, releases, stats),
            abandoned_tracked_submissions: gpu.tracked_submissions,
            abandoned_active_artifacts,
            abandoned_pending_uploads,
            abandoned_detached_uploads: gpu.detached_uploads,
            abandoned_ready_retirements: gpu.ready_retirements,
            abandoned_allocation_bytes: abandoned_entry_bytes.saturating_add(gpu.allocation_bytes),
        })
    }
}
