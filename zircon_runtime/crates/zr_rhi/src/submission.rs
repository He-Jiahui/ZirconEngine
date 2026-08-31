use std::collections::{BTreeMap, HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use super::{DeviceGeneration, DeviceId, RenderQueueClass};

const DEFAULT_MAX_UNRESOLVED_SUBMISSIONS: usize = 4_096;
const DEFAULT_MAX_TERMINAL_STATUSES: usize = 4_096;

/// A globally correlatable point on one device generation's logical queue timeline.
///
/// Backends must verify a ticket against their issued-submission table before
/// treating it as valid. The public constructor supports serialization and
/// diagnostics; constructing matching fields cannot manufacture a submission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubmissionTicket {
    device_id: DeviceId,
    generation: DeviceGeneration,
    queue_class: RenderQueueClass,
    sequence: u64,
}

impl SubmissionTicket {
    pub const fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        queue_class: RenderQueueClass,
        sequence: u64,
    ) -> Self {
        Self {
            device_id,
            generation,
            queue_class,
            sequence,
        }
    }

    pub const fn device_id(self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(self) -> DeviceGeneration {
        self.generation
    }

    pub const fn queue_class(self) -> RenderQueueClass {
        self.queue_class
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

/// Evidence that one device generation completed a nonblocking completion pump.
///
/// The sequence is local to the device generation and advances only after a
/// successful backend poll. Consumers can reject stale or replayed post-poll
/// maintenance without treating completion observation as a destructive queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubmissionPollReceipt {
    device_id: DeviceId,
    generation: DeviceGeneration,
    sequence: u64,
}

impl SubmissionPollReceipt {
    pub const fn new(device_id: DeviceId, generation: DeviceGeneration, sequence: u64) -> Self {
        Self {
            device_id,
            generation,
            sequence,
        }
    }

    pub const fn device_id(self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(self) -> DeviceGeneration {
        self.generation
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

/// The observable lifecycle of a submission ticket.
///
/// `Accepted` means the backend validated and queued a packet. `Submitted`
/// means the native queue accepted it. Only terminal states permit dependent
/// resource retirement or completion consumers to progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmissionStatus {
    Accepted,
    Submitted,
    Completed,
    Failed,
    Cancelled,
    DeviceLost,
}

impl SubmissionStatus {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::DeviceLost
        )
    }
}

/// Bounded per-device submission policy.
///
/// The unresolved limit caps command contexts, staged packets, and the gaps
/// that can exist in the terminal sequence index. The terminal history limit
/// caps caller-visible receipt retention without weakening native retirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionLimits {
    max_unresolved_submissions: usize,
    max_terminal_statuses: usize,
}

impl SubmissionLimits {
    pub const fn new(max_unresolved_submissions: usize, max_terminal_statuses: usize) -> Self {
        Self {
            max_unresolved_submissions,
            max_terminal_statuses,
        }
    }

    pub const fn max_unresolved_submissions(self) -> usize {
        self.max_unresolved_submissions
    }

    pub const fn max_terminal_statuses(self) -> usize {
        self.max_terminal_statuses
    }
}

impl Default for SubmissionLimits {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_UNRESOLVED_SUBMISSIONS,
            DEFAULT_MAX_TERMINAL_STATUSES,
        )
    }
}

/// Bounded status and retirement index for one device generation.
///
/// A ticket can leave the caller-visible status history once its terminal
/// receipt ages out. Retirement remains safe because terminal sequence ranges
/// retain compressed completion facts until lower unresolved tickets close the
/// gap. Therefore a long-running early packet cannot make later cancellations
/// grow one entry per ticket.
#[derive(Debug)]
pub struct SubmissionHistory {
    limits: SubmissionLimits,
    statuses: HashMap<SubmissionTicket, SubmissionStatus>,
    terminal_status_order: VecDeque<SubmissionTicket>,
    terminal_ranges: BTreeMap<u64, u64>,
    terminal_prefix: u64,
    unresolved_count: usize,
}

impl SubmissionHistory {
    pub fn new(limits: SubmissionLimits) -> Self {
        Self {
            limits,
            statuses: HashMap::new(),
            terminal_status_order: VecDeque::new(),
            terminal_ranges: BTreeMap::new(),
            terminal_prefix: 0,
            unresolved_count: 0,
        }
    }

    pub const fn limits(&self) -> SubmissionLimits {
        self.limits
    }

    pub const fn unresolved_count(&self) -> usize {
        self.unresolved_count
    }

    pub fn can_accept(&self) -> bool {
        self.unresolved_count < self.limits.max_unresolved_submissions
    }

    /// Records an accepted ticket only when unresolved admission has capacity.
    /// Backends retain ownership of ticket uniqueness and identity validation.
    pub fn record_accepted(&mut self, ticket: SubmissionTicket) -> bool {
        if !self.can_accept() || self.statuses.contains_key(&ticket) {
            return false;
        }
        self.statuses.insert(ticket, SubmissionStatus::Accepted);
        self.unresolved_count = self.unresolved_count.saturating_add(1);
        true
    }

    pub fn status(&self, ticket: SubmissionTicket) -> Option<SubmissionStatus> {
        self.statuses.get(&ticket).copied()
    }

    pub fn transition(
        &mut self,
        ticket: SubmissionTicket,
        next: SubmissionStatus,
    ) -> Option<SubmissionStatus> {
        let previous = self.status(ticket)?;
        if previous.is_terminal() {
            return Some(previous);
        }

        self.statuses.insert(ticket, next);
        if next.is_terminal() {
            self.unresolved_count = self.unresolved_count.saturating_sub(1);
            self.record_terminal(ticket);
        }
        Some(previous)
    }

    pub fn unresolved_tickets(&self) -> Vec<SubmissionTicket> {
        self.statuses
            .iter()
            .filter_map(|(ticket, status)| (!status.is_terminal()).then_some(*ticket))
            .collect()
    }

    /// Returns whether a ticket is terminal even after its display status was
    /// evicted. Callers must validate ticket ownership before using this for
    /// resource retirement.
    pub fn is_terminal(&self, ticket: SubmissionTicket) -> bool {
        self.status(ticket)
            .is_some_and(SubmissionStatus::is_terminal)
            || ticket.sequence() <= self.terminal_prefix
            || self
                .terminal_ranges
                .range(..=ticket.sequence())
                .next_back()
                .is_some_and(|(_, end)| ticket.sequence() <= *end)
    }

    #[cfg(test)]
    pub fn terminal_range_count(&self) -> usize {
        self.terminal_ranges.len()
    }

    fn record_terminal(&mut self, ticket: SubmissionTicket) {
        self.terminal_status_order.push_back(ticket);
        while self.terminal_status_order.len() > self.limits.max_terminal_statuses {
            if let Some(expired) = self.terminal_status_order.pop_front() {
                self.statuses.remove(&expired);
            }
        }
        self.record_terminal_sequence(ticket.sequence());
    }

    fn record_terminal_sequence(&mut self, sequence: u64) {
        if sequence <= self.terminal_prefix {
            return;
        }

        let mut start = sequence;
        let mut end = sequence;
        if let Some((previous_start, previous_end)) = self
            .terminal_ranges
            .range(..=sequence)
            .next_back()
            .map(|(start, end)| (*start, *end))
        {
            if previous_end.saturating_add(1) >= sequence {
                start = previous_start;
                end = previous_end.max(sequence);
                self.terminal_ranges.remove(&previous_start);
            }
        }

        while let Some((next_start, next_end)) = self
            .terminal_ranges
            .range(start..)
            .next()
            .map(|(start, end)| (*start, *end))
        {
            if next_start > end.saturating_add(1) {
                break;
            }
            end = end.max(next_end);
            self.terminal_ranges.remove(&next_start);
        }
        self.terminal_ranges.insert(start, end);

        while let Some((next_start, next_end)) = self
            .terminal_ranges
            .iter()
            .next()
            .map(|(start, end)| (*start, *end))
        {
            if next_start != self.terminal_prefix.saturating_add(1) {
                break;
            }
            self.terminal_prefix = next_end;
            self.terminal_ranges.remove(&next_start);
        }
    }
}
