use crate::process::editor_child_receipt::{EditorChildDisposition, EditorChildTerminalReceipt};

use super::{EditorLaunchAttempt, EditorProcessLedger};

#[test]
fn terminal_before_launch_completion_is_reconciled_once() {
    let mut ledger = EditorProcessLedger::default();
    let receipt = exited_receipt(41, 913);
    assert!(ledger.observe_terminal(receipt.clone()).is_none());

    let terminal = ledger
        .observe_launch(EditorLaunchAttempt::new(41, 913, "Game"))
        .expect("early exit must be delivered when launch completes");
    assert_eq!(terminal.attempt_id, 41);
    assert_eq!(terminal.process_id, 913);
    assert_eq!(terminal.target, "Game");
    assert_eq!(terminal.receipt, receipt);
    assert!(ledger.observe_terminal(exited_receipt(41, 913)).is_none());
    assert!(ledger
        .observe_launch(EditorLaunchAttempt::new(41, 913, "Game"))
        .is_none());
}

#[test]
fn terminal_after_launch_completion_keeps_attempt_identity() {
    let mut ledger = EditorProcessLedger::default();
    assert!(ledger
        .observe_launch(EditorLaunchAttempt::new(41, 913, "Old Game"))
        .is_none());
    assert!(ledger
        .observe_launch(EditorLaunchAttempt::new(42, 914, "New Game"))
        .is_none());

    let terminal = ledger
        .observe_terminal(exited_receipt(41, 913))
        .expect("late exit must be delivered");
    assert_eq!(terminal.attempt_id, 41);
    assert_eq!(terminal.process_id, 913);
    assert_eq!(terminal.target, "Old Game");
    assert!(ledger.observe_terminal(exited_receipt(41, 913)).is_none());
}

#[test]
fn receipt_for_another_process_cannot_complete_attempt() {
    let mut ledger = EditorProcessLedger::default();
    assert!(ledger
        .observe_launch(EditorLaunchAttempt::new(41, 913, "Game"))
        .is_none());
    assert!(ledger.observe_terminal(exited_receipt(41, 914)).is_none());
    let terminal = ledger
        .observe_terminal(exited_receipt(41, 913))
        .expect("matching child terminal must be delivered");
    assert_eq!(terminal.process_id, 913);
}

#[test]
fn early_wrong_process_receipt_cannot_hide_matching_early_exit() {
    let mut ledger = EditorProcessLedger::default();
    assert!(ledger.observe_terminal(exited_receipt(41, 914)).is_none());
    assert!(ledger.observe_terminal(exited_receipt(41, 913)).is_none());

    let terminal = ledger
        .observe_launch(EditorLaunchAttempt::new(41, 913, "Game"))
        .expect("matching early child exit must survive a mismatched receipt");
    assert_eq!(terminal.process_id, 913);
    assert!(ledger.observe_terminal(exited_receipt(41, 913)).is_none());
}

fn exited_receipt(attempt_id: u64, process_id: u32) -> EditorChildTerminalReceipt {
    EditorChildTerminalReceipt {
        attempt_id,
        process_id,
        disposition: EditorChildDisposition::Exited {
            code: Some(7),
            signal: None,
        },
        cleanup_error: None,
    }
}
