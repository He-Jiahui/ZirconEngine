use std::num::{NonZeroU32, NonZeroU64};
use std::time::{Duration, Instant};

use super::{
    WindowCommand, WindowCommandAccepted, WindowCommandHeader, WindowCommandId,
    WindowCommandReceipt, WindowCommandTerminal, WindowObservedGeneration,
};
use crate::core::framework::window::{WindowId, WindowRegistryId};

fn target_window() -> WindowId {
    WindowId::new(
        WindowRegistryId::new(23).expect("fixture registry identity is nonzero"),
        4,
        NonZeroU32::new(9).expect("fixture window generation is nonzero"),
    )
}

fn header() -> WindowCommandHeader {
    WindowCommandHeader::new(
        target_window(),
        WindowCommandId::new(41).expect("fixture request identity is nonzero"),
        Instant::now() + Duration::from_secs(1),
    )
}

fn observed_generation() -> WindowObservedGeneration {
    WindowObservedGeneration::new(NonZeroU64::new(12).expect("fixture generation is nonzero"))
}

#[test]
fn command_keeps_target_generation_request_identity_deadline_and_desired_state_together() {
    let header = header();
    let command = WindowCommand::new(header, "title: Zircon".to_string());

    assert_eq!(command.header(), header);
    assert_eq!(command.target(), target_window());
    assert_eq!(command.request_id().raw(), 41);
    assert_eq!(command.desired(), "title: Zircon");
    assert_eq!(command.deadline(), header.deadline());
}

#[test]
fn accepted_receipt_is_distinct_from_the_single_terminal_receipt() {
    let header = header();
    let accepted_at = Instant::now();
    let accepted = WindowCommandAccepted::new(header, accepted_at);
    let receipt = WindowCommandReceipt::new(
        header,
        observed_generation(),
        "effective title: Zircon".to_string(),
        WindowCommandTerminal::<String>::Applied,
    );

    assert_eq!(accepted.header(), header);
    assert_eq!(accepted.accepted_at(), accepted_at);
    assert_eq!(receipt.header(), header);
    assert_eq!(receipt.observed_generation(), observed_generation());
    assert_eq!(receipt.effective(), "effective title: Zircon");
    assert_eq!(receipt.terminal(), &WindowCommandTerminal::Applied);
}

#[test]
fn rejected_canceled_and_failed_terminals_retain_exact_effective_state() {
    let header = header();
    let observed = observed_generation();
    let rejected = WindowCommandReceipt::new(
        header,
        observed,
        1280_u32,
        WindowCommandTerminal::Rejected {
            reason: "unsupported mode",
        },
    );
    let canceled = WindowCommandReceipt::new(
        header,
        observed,
        1280_u32,
        WindowCommandTerminal::<&str>::Canceled,
    );
    let failed = WindowCommandReceipt::new(
        header,
        observed,
        1280_u32,
        WindowCommandTerminal::Failed {
            reason: "backend disconnected",
        },
    );

    assert_eq!(rejected.effective(), &1280);
    assert_eq!(
        rejected.terminal(),
        &WindowCommandTerminal::Rejected {
            reason: "unsupported mode"
        }
    );
    assert_eq!(canceled.effective(), &1280);
    assert_eq!(canceled.terminal(), &WindowCommandTerminal::Canceled);
    assert_eq!(failed.effective(), &1280);
    assert_eq!(
        failed.terminal(),
        &WindowCommandTerminal::Failed {
            reason: "backend disconnected"
        }
    );
}
