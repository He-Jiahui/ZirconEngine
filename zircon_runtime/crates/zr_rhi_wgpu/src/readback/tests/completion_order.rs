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
