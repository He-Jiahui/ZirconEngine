use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::notifications::NotificationId;

use super::{
    DecisionCenterInstanceId, DecisionNotification, DecisionNotificationError,
    DecisionNotificationSnapshot, DecisionOptionId, DecisionReceipt, DecisionReceiptBatch,
    DecisionReceiptCursor, DecisionReceiptSequence, DecisionResolveReport, DecisionTicket,
};

const DEFAULT_PENDING_CAPACITY: usize = 128;
const DEFAULT_RECEIPT_CAPACITY: usize = 256;
static NEXT_CENTER_INSTANCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecisionCenterConfig {
    pending_capacity: usize,
    receipt_capacity: usize,
}

impl Default for DecisionCenterConfig {
    fn default() -> Self {
        Self {
            pending_capacity: DEFAULT_PENDING_CAPACITY,
            receipt_capacity: DEFAULT_RECEIPT_CAPACITY,
        }
    }
}

impl DecisionCenterConfig {
    pub fn new(
        pending_capacity: usize,
        receipt_capacity: usize,
    ) -> Result<Self, DecisionNotificationError> {
        if pending_capacity == 0 {
            return Err(DecisionNotificationError::InvalidCapacity {
                field: "pending_capacity",
            });
        }
        if receipt_capacity == 0 {
            return Err(DecisionNotificationError::InvalidCapacity {
                field: "receipt_capacity",
            });
        }
        Ok(Self {
            pending_capacity,
            receipt_capacity,
        })
    }

    pub const fn pending_capacity(self) -> usize {
        self.pending_capacity
    }

    pub const fn receipt_capacity(self) -> usize {
        self.receipt_capacity
    }
}

#[derive(Debug)]
pub struct DecisionNotificationCenter {
    instance_id: DecisionCenterInstanceId,
    config: DecisionCenterConfig,
    state: Mutex<DecisionCenterState>,
}

#[derive(Debug)]
struct DecisionCenterState {
    entries: BTreeMap<NotificationId, DecisionEntry>,
    pending_order: VecDeque<NotificationId>,
    receipts: VecDeque<DecisionReceipt>,
    pending_count: usize,
    next_ticket_incarnation: u64,
    next_receipt_sequence: u64,
}

#[derive(Debug)]
struct DecisionEntry {
    ticket: DecisionTicket,
    notification: DecisionNotification,
    resolved: Option<DecisionReceipt>,
}

impl DecisionNotificationCenter {
    pub fn new(config: DecisionCenterConfig) -> Result<Self, DecisionNotificationError> {
        let instance_id = NEXT_CENTER_INSTANCE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map(DecisionCenterInstanceId::new)
            .map_err(|_| DecisionNotificationError::CenterInstanceExhausted)?;
        Ok(Self {
            instance_id,
            config,
            state: Mutex::new(DecisionCenterState {
                entries: BTreeMap::new(),
                pending_order: VecDeque::new(),
                receipts: VecDeque::new(),
                pending_count: 0,
                next_ticket_incarnation: 1,
                next_receipt_sequence: 1,
            }),
        })
    }

    pub const fn instance_id(&self) -> DecisionCenterInstanceId {
        self.instance_id
    }

    pub const fn initial_cursor(&self) -> DecisionReceiptCursor {
        DecisionReceiptCursor::start(self.instance_id)
    }

    pub fn publish(
        &self,
        notification: DecisionNotification,
    ) -> Result<DecisionTicket, DecisionNotificationError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.entries.contains_key(notification.id()) {
            return Err(DecisionNotificationError::DuplicateNotification {
                notification: notification.id().clone(),
            });
        }
        if state.pending_count >= self.config.pending_capacity {
            return Err(DecisionNotificationError::PendingCapacityReached {
                capacity: self.config.pending_capacity,
            });
        }
        let ticket = DecisionTicket::new(
            self.instance_id,
            notification.id().clone(),
            state.next_ticket_incarnation,
        );
        state.next_ticket_incarnation = state
            .next_ticket_incarnation
            .checked_add(1)
            .ok_or(DecisionNotificationError::TicketSequenceExhausted)?;
        state.entries.insert(
            notification.id().clone(),
            DecisionEntry {
                ticket: ticket.clone(),
                notification,
                resolved: None,
            },
        );
        state.pending_order.push_back(notification.id().clone());
        state.pending_count += 1;
        Ok(ticket)
    }

    /// Returns unresolved Decisions in publication order.
    ///
    /// Consumers present one complete Decision at a time, so identifier sorting must not decide
    /// which operator action blocks the next one.
    pub fn pending_snapshot(&self) -> Vec<DecisionNotificationSnapshot> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .pending_order
            .iter()
            .filter_map(|notification_id| state.entries.get(notification_id))
            .filter(|entry| entry.resolved.is_none())
            .map(|entry| {
                DecisionNotificationSnapshot::new(
                    entry.ticket.clone(),
                    entry.notification.clone(),
                    None,
                )
            })
            .collect()
    }

    /// Returns the exact unresolved Decision count without cloning the pending presentation
    /// snapshot. Producers use this as a backpressure change signal before retrying publication.
    pub fn pending_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pending_count
    }

    pub fn snapshot(&self) -> Vec<DecisionNotificationSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entries
            .values()
            .map(|entry| {
                DecisionNotificationSnapshot::new(
                    entry.ticket.clone(),
                    entry.notification.clone(),
                    entry.resolved.clone(),
                )
            })
            .collect()
    }

    pub fn resolve(
        &self,
        ticket: &DecisionTicket,
        option_id: &DecisionOptionId,
    ) -> Result<DecisionResolveReport, DecisionNotificationError> {
        self.validate_ticket_center(ticket)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let notification_id = ticket.notification_id();
        let entry = state.entries.get(notification_id).ok_or_else(|| {
            DecisionNotificationError::NotificationNotFound {
                notification: notification_id.clone(),
            }
        })?;
        if &entry.ticket != ticket {
            return Err(DecisionNotificationError::StaleTicket {
                notification: notification_id.clone(),
                expected_incarnation: entry.ticket.incarnation(),
                received_incarnation: ticket.incarnation(),
            });
        }
        if let Some(receipt) = entry.resolved.clone() {
            if receipt.option_id() == option_id {
                return Ok(DecisionResolveReport {
                    receipt,
                    newly_resolved: false,
                });
            }
            return Err(DecisionNotificationError::AlreadyResolved {
                notification: notification_id.clone(),
                selected: receipt.option_id().clone(),
                requested: option_id.clone(),
            });
        }

        if !entry.notification.has_option(option_id) {
            return Err(DecisionNotificationError::OptionNotFound {
                notification: notification_id.clone(),
                option: option_id.clone(),
            });
        }

        let sequence = DecisionReceiptSequence::new(state.next_receipt_sequence);
        state.next_receipt_sequence = state
            .next_receipt_sequence
            .checked_add(1)
            .ok_or(DecisionNotificationError::ReceiptSequenceExhausted)?;
        let receipt = DecisionReceipt {
            sequence,
            ticket: ticket.clone(),
            option_id: option_id.clone(),
        };
        let Some(entry) = state.entries.get_mut(notification_id) else {
            return Err(DecisionNotificationError::NotificationNotFound {
                notification: notification_id.clone(),
            });
        };
        entry.resolved = Some(receipt.clone());
        state.pending_count = state.pending_count.saturating_sub(1);
        state
            .pending_order
            .retain(|pending_id| pending_id != notification_id);
        state.receipts.push_back(receipt.clone());
        while state.receipts.len() > self.config.receipt_capacity {
            let Some(evicted) = state.receipts.pop_front() else {
                break;
            };
            let remove_entry = state
                .entries
                .get(evicted.ticket().notification_id())
                .and_then(|entry| entry.resolved.as_ref())
                .is_some_and(|resolved| resolved.ticket() == evicted.ticket());
            if remove_entry {
                state.entries.remove(evicted.ticket().notification_id());
            }
        }
        Ok(DecisionResolveReport {
            receipt,
            newly_resolved: true,
        })
    }

    pub fn cancel(
        &self,
        ticket: &DecisionTicket,
    ) -> Result<DecisionResolveReport, DecisionNotificationError> {
        self.validate_ticket_center(ticket)?;
        let notification_id = ticket.notification_id();
        let cancel_option = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.entries.get(notification_id).and_then(|entry| {
                (&entry.ticket == ticket)
                    .then(|| entry.notification.cancel_option().cloned())
                    .flatten()
            })
        }
        .ok_or_else(|| {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match state.entries.get(notification_id) {
                Some(entry) if &entry.ticket != ticket => DecisionNotificationError::StaleTicket {
                    notification: notification_id.clone(),
                    expected_incarnation: entry.ticket.incarnation(),
                    received_incarnation: ticket.incarnation(),
                },
                Some(_) => DecisionNotificationError::CancellationNotAllowed {
                    notification: notification_id.clone(),
                },
                None => DecisionNotificationError::NotificationNotFound {
                    notification: notification_id.clone(),
                },
            }
        })?;
        self.resolve(ticket, &cancel_option)
    }

    pub fn receipts_since(
        &self,
        cursor: DecisionReceiptCursor,
    ) -> Result<DecisionReceiptBatch, DecisionNotificationError> {
        if cursor.center_instance() != self.instance_id {
            return Err(DecisionNotificationError::ForeignCursor {
                expected_center: self.instance_id,
                received_center: cursor.center_instance(),
            });
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(oldest) = state.receipts.front().map(DecisionReceipt::sequence) {
            if cursor.value() < oldest.value().saturating_sub(1) {
                return Err(DecisionNotificationError::CursorExpired {
                    requested: cursor.value(),
                    oldest_available: oldest,
                    resume_cursor: DecisionReceiptCursor::before(self.instance_id, oldest),
                });
            }
        }
        if state
            .receipts
            .back()
            .is_none_or(|receipt| cursor.value() >= receipt.sequence().value())
        {
            return Ok(DecisionReceiptBatch {
                receipts: Vec::new(),
                next_cursor: cursor,
            });
        }
        let receipts = state
            .receipts
            .iter()
            .filter(|receipt| receipt.sequence().value() > cursor.value())
            .cloned()
            .collect::<Vec<_>>();
        let next_cursor = receipts
            .last()
            .map(|receipt| DecisionReceiptCursor::after(self.instance_id, receipt.sequence()))
            .unwrap_or(cursor);
        Ok(DecisionReceiptBatch {
            receipts,
            next_cursor,
        })
    }

    fn validate_ticket_center(
        &self,
        ticket: &DecisionTicket,
    ) -> Result<(), DecisionNotificationError> {
        if ticket.center_instance() == self.instance_id {
            Ok(())
        } else {
            Err(DecisionNotificationError::ForeignTicket {
                notification: ticket.notification_id().clone(),
                expected_center: self.instance_id,
                received_center: ticket.center_instance(),
            })
        }
    }
}
