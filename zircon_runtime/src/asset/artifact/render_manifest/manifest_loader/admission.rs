use std::collections::HashMap;
use std::sync::Arc;

use super::contract::{
    RenderArtifactManifestAdmissionError, RenderArtifactManifestRequest,
    RenderArtifactManifestRequestKey,
};
use super::loader::{
    RenderArtifactManifestLoaderInner, RenderArtifactManifestTicket,
    RenderArtifactManifestTicketBatch,
};
use super::state::{RenderArtifactManifestEntry, register_ticket};
use super::worker::atomic_add;

struct PreparedGroup {
    key: RenderArtifactManifestRequestKey,
    request_count: usize,
}

impl RenderArtifactManifestLoaderInner {
    pub(super) fn request_batch(
        self: &Arc<Self>,
        requests: &[RenderArtifactManifestRequest],
    ) -> Result<RenderArtifactManifestTicketBatch, RenderArtifactManifestAdmissionError> {
        if requests.is_empty() {
            return Err(RenderArtifactManifestAdmissionError::EmptyBatch);
        }

        let mut group_indices = HashMap::<RenderArtifactManifestRequestKey, usize>::new();
        let mut groups = Vec::<PreparedGroup>::new();
        let mut request_group_indices = Vec::with_capacity(requests.len());
        for request in requests {
            if request.key().target_platform().trim().is_empty() {
                return Err(RenderArtifactManifestAdmissionError::EmptyTargetPlatform);
            }
            let key = request.key().clone();
            let group_index = if let Some(index) = group_indices.get(&key).copied() {
                let Some(group) = groups.get_mut(index) else {
                    return Err(RenderArtifactManifestAdmissionError::InternalInvariant {
                        reason: "prepared group index is missing",
                    });
                };
                group.request_count = group.request_count.checked_add(1).ok_or(
                    RenderArtifactManifestAdmissionError::TicketCapacityExceeded {
                        capacity: self.limits.max_total_tickets(),
                    },
                )?;
                index
            } else {
                let index = groups.len();
                group_indices.insert(key.clone(), index);
                groups.push(PreparedGroup {
                    key,
                    request_count: 1,
                });
                index
            };
            request_group_indices.push(group_index);
        }

        let mut registry = self.lock_registry();
        if !registry.accepting {
            return Err(RenderArtifactManifestAdmissionError::Closed);
        }
        let requested_ticket_count = registry.tickets.len().checked_add(requests.len()).ok_or(
            RenderArtifactManifestAdmissionError::TicketCapacityExceeded {
                capacity: self.limits.max_total_tickets(),
            },
        )?;
        if requested_ticket_count > self.limits.max_total_tickets() {
            return Err(
                RenderArtifactManifestAdmissionError::TicketCapacityExceeded {
                    capacity: self.limits.max_total_tickets(),
                },
            );
        }

        let mut new_entry_count = 0_usize;
        for group in &groups {
            if let Some(entry) = registry.entries.get(&group.key) {
                let requested_entry_tickets = entry
                    .ticket_count()
                    .checked_add(group.request_count)
                    .ok_or(
                        RenderArtifactManifestAdmissionError::EntryTicketCapacityExceeded {
                            capacity: self.limits.max_tickets_per_entry(),
                        },
                    )?;
                if requested_entry_tickets > self.limits.max_tickets_per_entry() {
                    return Err(
                        RenderArtifactManifestAdmissionError::EntryTicketCapacityExceeded {
                            capacity: self.limits.max_tickets_per_entry(),
                        },
                    );
                }
            } else {
                if group.request_count > self.limits.max_tickets_per_entry() {
                    return Err(
                        RenderArtifactManifestAdmissionError::EntryTicketCapacityExceeded {
                            capacity: self.limits.max_tickets_per_entry(),
                        },
                    );
                }
                new_entry_count = new_entry_count.saturating_add(1);
            }
        }

        let requested_entry_count = registry.entries.len().checked_add(new_entry_count).ok_or(
            RenderArtifactManifestAdmissionError::EntryCapacityExceeded {
                capacity: self.limits.max_entries(),
            },
        )?;
        if requested_entry_count > self.limits.max_entries() {
            return Err(
                RenderArtifactManifestAdmissionError::EntryCapacityExceeded {
                    capacity: self.limits.max_entries(),
                },
            );
        }
        let requested_retained_bytes = self
            .entry_retained_bytes
            .checked_mul(new_entry_count)
            .ok_or(RenderArtifactManifestAdmissionError::RetainedBytesRequestOverflow)?;
        let remaining_retained_bytes = self
            .limits
            .max_retained_bytes()
            .saturating_sub(registry.reserved_retained_bytes);
        if requested_retained_bytes > remaining_retained_bytes {
            return Err(
                RenderArtifactManifestAdmissionError::RetainedBytesCapacityExceeded {
                    requested: requested_retained_bytes,
                    remaining: remaining_retained_bytes,
                },
            );
        }

        let ticket_count = u64::try_from(requests.len())
            .map_err(|_| RenderArtifactManifestAdmissionError::TicketIdExhausted)?;
        let next_ticket_id = registry
            .next_ticket_id
            .checked_add(ticket_count)
            .ok_or(RenderArtifactManifestAdmissionError::TicketIdExhausted)?;
        let frontier_count = u64::try_from(new_entry_count)
            .map_err(|_| RenderArtifactManifestAdmissionError::FrontierSequenceExhausted)?;
        let next_frontier_sequence = registry
            .next_frontier_sequence
            .checked_add(frontier_count)
            .ok_or(RenderArtifactManifestAdmissionError::FrontierSequenceExhausted)?;

        let mut resolved_entries = Vec::with_capacity(groups.len());
        let mut new_entries = Vec::with_capacity(groups.len());
        for group in &groups {
            if let Some(entry) = registry.entries.get(&group.key).cloned() {
                resolved_entries.push(entry);
                new_entries.push(false);
            } else {
                resolved_entries.push(Arc::new(RenderArtifactManifestEntry::new(
                    group.key.clone(),
                    self.entry_retained_bytes,
                    group.request_count,
                )));
                new_entries.push(true);
            }
        }
        let ticket_entries = request_group_indices
            .iter()
            .map(|index| resolved_entries.get(*index).cloned())
            .collect::<Option<Vec<_>>>()
            .ok_or(RenderArtifactManifestAdmissionError::InternalInvariant {
                reason: "request group entry is missing",
            })?;

        for ((group, entry), is_new) in groups
            .iter()
            .zip(resolved_entries.iter())
            .zip(new_entries.iter().copied())
        {
            if is_new {
                registry.reserved_retained_bytes += self.entry_retained_bytes;
                registry
                    .entries
                    .insert(group.key.clone(), Arc::clone(entry));
            } else {
                entry.add_tickets(group.request_count);
            }
        }

        let mut ticket_id = registry.next_ticket_id;
        let mut tickets = Vec::<RenderArtifactManifestTicket>::with_capacity(requests.len());
        for (request, entry) in requests.iter().zip(ticket_entries) {
            tickets.push(register_ticket(
                self,
                &mut registry,
                ticket_id,
                request.key().clone(),
                &entry,
                request.priority(),
                request.deadline(),
            ));
            ticket_id = ticket_id.saturating_add(1);
        }

        let mut frontier_sequence = registry.next_frontier_sequence;
        for (group, is_new) in groups.iter().zip(new_entries) {
            if is_new {
                registry
                    .io_frontier
                    .enqueue(group.key.clone(), frontier_sequence);
                frontier_sequence = frontier_sequence.saturating_add(1);
            }
        }
        registry.next_ticket_id = next_ticket_id;
        registry.next_frontier_sequence = next_frontier_sequence;
        let merged_requests = requests.len().saturating_sub(new_entry_count);
        atomic_add(
            &self.metrics.merged_requests,
            u64::try_from(merged_requests).map_or(u64::MAX, |count| count),
        );
        drop(registry);
        Ok(RenderArtifactManifestTicketBatch::new(tickets))
    }
}
