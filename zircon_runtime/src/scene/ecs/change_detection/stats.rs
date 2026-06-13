use crate::core::diagnostics::DiagnosticStore;

use super::{ChangeTickWindow, ComponentTicks};

pub const ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC: &str =
    "ecs.change_detection.scanned_marks";
pub const ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC: &str =
    "ecs.change_detection.added_matches";
pub const ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC: &str =
    "ecs.change_detection.changed_matches";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChangeDetectionScanStats {
    pub scanned_marks: u64,
    pub added_matches: u64,
    pub changed_matches: u64,
}

impl ChangeDetectionScanStats {
    pub fn scan_added(&mut self, ticks: ComponentTicks, window: ChangeTickWindow) -> bool {
        self.scanned_marks = self.scanned_marks.saturating_add(1);
        let matched = ticks.is_added(window);
        if matched {
            self.added_matches = self.added_matches.saturating_add(1);
        }
        matched
    }

    pub fn scan_changed(&mut self, ticks: ComponentTicks, window: ChangeTickWindow) -> bool {
        self.scanned_marks = self.scanned_marks.saturating_add(1);
        let matched = ticks.is_changed(window);
        if matched {
            self.changed_matches = self.changed_matches.saturating_add(1);
        }
        matched
    }

    pub fn merge(&mut self, other: Self) {
        self.scanned_marks = self.scanned_marks.saturating_add(other.scanned_marks);
        self.added_matches = self.added_matches.saturating_add(other.added_matches);
        self.changed_matches = self.changed_matches.saturating_add(other.changed_matches);
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        record_count(
            store,
            ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC,
            frame_index,
            self.scanned_marks,
        );
        record_count(
            store,
            ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC,
            frame_index,
            self.added_matches,
        );
        record_count(
            store,
            ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC,
            frame_index,
            self.changed_matches,
        );
    }
}

fn record_count(store: &mut DiagnosticStore, path: &'static str, frame_index: u64, value: u64) {
    store.record(
        path,
        frame_index,
        value as f64,
        Some("count"),
        ["ecs", "change_detection"],
    );
}
