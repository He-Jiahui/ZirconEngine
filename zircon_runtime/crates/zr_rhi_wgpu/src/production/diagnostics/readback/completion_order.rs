use std::collections::{HashMap, VecDeque};

use zr_rhi::{DiagnosticReadbackTerminal, SubmissionTicket};

/// Holds completed maps behind the oldest outstanding diagnostic ticket.
///
/// Native map callbacks may arrive out of order. Retaining an already-mapped
/// staging buffer until its predecessors finish preserves delivery order
/// without copying its payload into an unbounded CPU-side queue.
pub(crate) struct TicketOrderedDiagnosticCompletions<T> {
    tickets: VecDeque<SubmissionTicket>,
    completed: HashMap<SubmissionTicket, T>,
}

impl<T> Default for TicketOrderedDiagnosticCompletions<T> {
    fn default() -> Self {
        Self {
            tickets: VecDeque::new(),
            completed: HashMap::new(),
        }
    }
}

impl<T: Copy> TicketOrderedDiagnosticCompletions<T> {
    pub(crate) fn register(&mut self, ticket: SubmissionTicket) {
        debug_assert!(!self.tickets.contains(&ticket));
        self.tickets.push_back(ticket);
    }

    pub(crate) fn is_completed(&self, ticket: SubmissionTicket) -> bool {
        self.completed.contains_key(&ticket)
    }

    pub(crate) fn complete(&mut self, ticket: SubmissionTicket, completion: T) -> bool {
        if self.completed.contains_key(&ticket) {
            return false;
        }
        self.completed.insert(ticket, completion);
        true
    }

    pub(crate) fn take_next_ready(&mut self) -> Option<(SubmissionTicket, T)> {
        let ticket = *self.tickets.front()?;
        let completion = *self.completed.get(&ticket)?;
        self.tickets.pop_front();
        self.completed.remove(&ticket);
        Some((ticket, completion))
    }

    pub(crate) fn replace_all(&mut self, completion: T) {
        self.completed.clear();
        self.completed.extend(
            self.tickets
                .iter()
                .copied()
                .map(|ticket| (ticket, completion)),
        );
    }

    pub(crate) fn clear(&mut self) {
        self.tickets.clear();
        self.completed.clear();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiagnosticBatchCompletion {
    Mapped,
    MapFailed,
    Terminal(DiagnosticReadbackTerminal),
}

#[cfg(test)]
mod tests {
    use zr_rhi::{
        DeviceGeneration, DeviceId, DiagnosticReadbackTerminal, RenderQueueClass, SubmissionTicket,
    };

    use super::{DiagnosticBatchCompletion, TicketOrderedDiagnosticCompletions};

    fn ticket(sequence: u64) -> SubmissionTicket {
        SubmissionTicket::new(
            DeviceId::new(17),
            DeviceGeneration::initial(),
            RenderQueueClass::Copy,
            sequence,
        )
    }

    #[test]
    fn later_map_completion_waits_for_the_oldest_diagnostic_ticket() {
        let first = ticket(4);
        let later = ticket(7);
        let mut completions = TicketOrderedDiagnosticCompletions::default();
        completions.register(first);
        completions.register(later);

        assert!(completions.complete(later, DiagnosticBatchCompletion::Mapped));
        assert_eq!(completions.take_next_ready(), None);

        assert!(completions.complete(first, DiagnosticBatchCompletion::MapFailed));
        assert_eq!(
            completions.take_next_ready(),
            Some((first, DiagnosticBatchCompletion::MapFailed))
        );
        assert_eq!(
            completions.take_next_ready(),
            Some((later, DiagnosticBatchCompletion::Mapped))
        );
    }

    #[test]
    fn replacement_terminalizes_every_ticket_in_registration_order() {
        let first = ticket(4);
        let later = ticket(7);
        let mut completions = TicketOrderedDiagnosticCompletions::default();
        completions.register(first);
        completions.register(later);
        assert!(completions.complete(later, DiagnosticBatchCompletion::Mapped));

        let terminal = DiagnosticBatchCompletion::Terminal(DiagnosticReadbackTerminal::DeviceLost);
        completions.replace_all(terminal);

        assert_eq!(completions.take_next_ready(), Some((first, terminal)));
        assert_eq!(completions.take_next_ready(), Some((later, terminal)));
        assert_eq!(completions.take_next_ready(), None);
    }
}
