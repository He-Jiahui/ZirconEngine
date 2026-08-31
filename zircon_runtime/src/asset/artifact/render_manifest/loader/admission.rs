use std::collections::HashMap;
use std::sync::Arc;

use super::contract::{RenderArtifactBlockAdmissionError, RenderArtifactBlockRequest};
use super::entry::RenderArtifactBlockEntry;
use super::loader::{
    RenderArtifactBlockLoaderInner, RenderArtifactBlockTicket, RenderArtifactBlockTicketBatch,
};
use super::policy::quote_retained_bytes;
use super::registry::{RenderArtifactDecodeKey, register_ticket};
use super::worker::atomic_add;

struct PreparedGroup {
    key: RenderArtifactDecodeKey,
    descriptor: super::super::RenderArtifactBlockDescriptor,
    retained_bytes: usize,
    request_count: usize,
}

impl RenderArtifactBlockLoaderInner {
    pub(super) fn request_batch(
        self: &Arc<Self>,
        requests: &[RenderArtifactBlockRequest],
    ) -> Result<RenderArtifactBlockTicketBatch, RenderArtifactBlockAdmissionError> {
        if requests.is_empty() {
            return Err(RenderArtifactBlockAdmissionError::EmptyBatch);
        }

        let mut group_indices = HashMap::<RenderArtifactDecodeKey, usize>::new();
        let mut groups = Vec::<PreparedGroup>::new();
        let mut request_group_indices = Vec::with_capacity(requests.len());
        for request in requests {
            let retained_bytes = quote_retained_bytes(request.descriptor(), self.limits)?;
            let key = RenderArtifactDecodeKey::from_descriptor(request.descriptor());
            let group_index = if let Some(index) = group_indices.get(&key).copied() {
                let Some(group) = groups.get_mut(index) else {
                    return Err(RenderArtifactBlockAdmissionError::InternalInvariant {
                        reason: "prepared group index is missing",
                    });
                };
                group.request_count = group.request_count.checked_add(1).ok_or(
                    RenderArtifactBlockAdmissionError::TicketCapacityExceeded {
                        capacity: self.limits.max_total_tickets(),
                    },
                )?;
                index
            } else {
                let index = groups.len();
                group_indices.insert(key, index);
                groups.push(PreparedGroup {
                    key,
                    descriptor: request.descriptor().clone(),
                    retained_bytes,
                    request_count: 1,
                });
                index
            };
            request_group_indices.push(group_index);
        }

        let mut registry = self.lock_registry();
        if !registry.accepting {
            return Err(RenderArtifactBlockAdmissionError::Closed);
        }
        let requested_ticket_count = registry.tickets.len().checked_add(requests.len()).ok_or(
            RenderArtifactBlockAdmissionError::TicketCapacityExceeded {
                capacity: self.limits.max_total_tickets(),
            },
        )?;
        if requested_ticket_count > self.limits.max_total_tickets() {
            return Err(RenderArtifactBlockAdmissionError::TicketCapacityExceeded {
                capacity: self.limits.max_total_tickets(),
            });
        }

        let mut new_entry_count = 0_usize;
        let mut requested_retained_bytes = 0_usize;
        for group in &groups {
            if let Some(entry) = registry.entries.get(&group.key) {
                let requested_entry_tickets = entry
                    .ticket_count()
                    .checked_add(group.request_count)
                    .ok_or(
                        RenderArtifactBlockAdmissionError::EntryTicketCapacityExceeded {
                            capacity: self.limits.max_tickets_per_entry(),
                        },
                    )?;
                if requested_entry_tickets > self.limits.max_tickets_per_entry() {
                    return Err(
                        RenderArtifactBlockAdmissionError::EntryTicketCapacityExceeded {
                            capacity: self.limits.max_tickets_per_entry(),
                        },
                    );
                }
                continue;
            }
            if group.request_count > self.limits.max_tickets_per_entry() {
                return Err(
                    RenderArtifactBlockAdmissionError::EntryTicketCapacityExceeded {
                        capacity: self.limits.max_tickets_per_entry(),
                    },
                );
            }
            new_entry_count = new_entry_count.saturating_add(1);
            requested_retained_bytes =
                requested_retained_bytes
                    .checked_add(group.retained_bytes)
                    .ok_or(RenderArtifactBlockAdmissionError::RetainedBytesOverflow)?;
        }

        let requested_entry_count = registry.entries.len().checked_add(new_entry_count).ok_or(
            RenderArtifactBlockAdmissionError::EntryCapacityExceeded {
                capacity: self.limits.max_entries(),
            },
        )?;
        if requested_entry_count > self.limits.max_entries() {
            return Err(RenderArtifactBlockAdmissionError::EntryCapacityExceeded {
                capacity: self.limits.max_entries(),
            });
        }
        let remaining_retained_bytes = self
            .limits
            .max_retained_bytes()
            .saturating_sub(registry.retained_bytes);
        if requested_retained_bytes > remaining_retained_bytes {
            return Err(
                RenderArtifactBlockAdmissionError::RetainedBytesCapacityExceeded {
                    requested: requested_retained_bytes,
                    remaining: remaining_retained_bytes,
                },
            );
        }

        let ticket_count = u64::try_from(requests.len())
            .map_err(|_| RenderArtifactBlockAdmissionError::TicketIdExhausted)?;
        let next_ticket_id = registry
            .next_ticket_id
            .checked_add(ticket_count)
            .ok_or(RenderArtifactBlockAdmissionError::TicketIdExhausted)?;
        let frontier_count = u64::try_from(new_entry_count)
            .map_err(|_| RenderArtifactBlockAdmissionError::FrontierSequenceExhausted)?;
        let next_frontier_sequence = registry
            .next_frontier_sequence
            .checked_add(frontier_count)
            .ok_or(RenderArtifactBlockAdmissionError::FrontierSequenceExhausted)?;

        let mut resolved_entries = Vec::with_capacity(groups.len());
        let mut new_entries = Vec::with_capacity(groups.len());
        for group in &groups {
            if let Some(entry) = registry.entries.get(&group.key).cloned() {
                resolved_entries.push(entry);
                new_entries.push(false);
            } else {
                resolved_entries.push(Arc::new(RenderArtifactBlockEntry::new(
                    group.descriptor.clone(),
                    group.retained_bytes,
                    group.request_count,
                )));
                new_entries.push(true);
            }
        }
        let ticket_entries = request_group_indices
            .iter()
            .map(|index| resolved_entries.get(*index).cloned())
            .collect::<Option<Vec<_>>>()
            .ok_or(RenderArtifactBlockAdmissionError::InternalInvariant {
                reason: "request group entry is missing",
            })?;

        for ((group, entry), is_new) in groups
            .iter()
            .zip(resolved_entries.iter())
            .zip(new_entries.iter().copied())
        {
            if is_new {
                registry.retained_bytes += group.retained_bytes;
                registry.entries.insert(group.key, Arc::clone(entry));
            } else {
                entry.add_tickets(group.request_count);
            }
        }

        let mut ticket_id = registry.next_ticket_id;
        let mut tickets = Vec::<RenderArtifactBlockTicket>::with_capacity(requests.len());
        for (request, entry) in requests.iter().zip(ticket_entries) {
            let key = RenderArtifactDecodeKey::from_descriptor(request.descriptor());
            tickets.push(register_ticket(
                self,
                &mut registry,
                ticket_id,
                key,
                &entry,
                request.descriptor().clone(),
                request.priority(),
                request.deadline(),
            ));
            ticket_id = ticket_id.saturating_add(1);
        }

        let mut frontier_sequence = registry.next_frontier_sequence;
        for (group, is_new) in groups.iter().zip(new_entries) {
            if is_new {
                registry.io_frontier.enqueue(group.key, frontier_sequence);
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
        Ok(RenderArtifactBlockTicketBatch::new(tickets))
    }
}
