use std::sync::mpsc;

use super::drain_progress_for_active;
use crate::ui::retained_host::app::build_export_actions::{
    job_queue::worker::DesktopExportJobProgress, DesktopExportProgressSnapshot,
};

#[test]
fn terminal_poll_can_drain_progress_sent_after_the_initial_drain() {
    let (sender, receiver) = mpsc::channel();
    let mut progress = None;
    assert!(!drain_progress_for_active(&receiver, 7, &mut progress));

    sender
        .send(DesktopExportJobProgress {
            id: 7,
            progress: DesktopExportProgressSnapshot {
                stage: "complete".to_string(),
                percent: 100,
                message: "Desktop export build finished".to_string(),
            },
        })
        .expect("test progress channel should remain connected");

    assert!(drain_progress_for_active(&receiver, 7, &mut progress));
    assert_eq!(progress.map(|snapshot| snapshot.percent), Some(100));
}
