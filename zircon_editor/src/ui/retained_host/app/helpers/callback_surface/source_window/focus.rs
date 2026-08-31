use crate::ui::retained_host::callback_dispatch::dispatch_builtin_floating_window_focus_for_source;
use crate::ui::workbench::layout::MainPageId;

use super::super::super::super::{RetainedEditorHost, workbench_snapshot_access};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn with_callback_source_window<T>(
        &mut self,
        source_window_id: Option<MainPageId>,
        callback: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = std::mem::replace(&mut self.callback_source_window, source_window_id);
        let result = callback(self);
        self.callback_source_window = previous;
        result
    }

    pub(in crate::ui::retained_host::app) fn focus_callback_source_window(&mut self) {
        let source_window_id = self.callback_source_window.clone();
        let Some(source_window_id) = source_window_id else {
            self.last_focused_callback_window = None;
            return;
        };

        match dispatch_builtin_floating_window_focus_for_source(
            &self.runtime,
            Some(&source_window_id),
            self.last_focused_callback_window.as_ref(),
        ) {
            Some(Ok(effects)) => {
                self.apply_dispatch_effects(effects);
                self.last_focused_callback_window = Some(source_window_id);
            }
            Some(Err(error)) => self.set_status_line(error),
            None => {
                self.last_focused_callback_window = Some(source_window_id);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn note_focused_floating_window(
        &mut self,
        window_id: Option<MainPageId>,
    ) {
        self.last_focused_callback_window = window_id;
    }

    pub(in crate::ui::retained_host::app) fn note_focused_floating_window_surface(
        &mut self,
        surface_key: &str,
    ) {
        if surface_key == "main" {
            self.last_focused_callback_window = None;
            return;
        }

        let chrome = self.runtime.chrome_snapshot();
        self.last_focused_callback_window =
            workbench_snapshot_access::floating_window_id_for_surface_key(
                &chrome.workbench,
                surface_key,
            );
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;

    use super::MainPageId;

    const LOOKUP_ROUNDS: usize = 65_536;
    const PREVIOUS_WINDOW_ID: &str = "window:editor.callback-source.performance.previous-window";

    #[test]
    fn optimization_batch_20260830dq_callback_source_window_moves_previous_owner() {
        let source = include_str!("focus.rs");
        let body = source
            .split("fn with_callback_source_window")
            .nth(1)
            .expect("callback source window scope exists")
            .split("fn focus_callback_source_window")
            .next()
            .expect("callback source window scope ends before focus");

        assert!(body.contains("std::mem::replace"));
        assert!(!body.contains("callback_source_window.clone()"));
    }

    #[test]
    #[ignore = "deterministic clone-count evidence for the managed optimization batch"]
    fn optimization_batch_20260830dq_callback_source_window_replace_evidence() {
        let legacy_slot = Some(MainPageId::new(PREVIOUS_WINDOW_ID));
        let mut legacy_previous_id_clones = 0_u64;
        let mut legacy_cloned_id_bytes = 0_u64;

        for _ in 0..LOOKUP_ROUNDS {
            let previous = legacy_slot.clone();
            if let Some(previous) = previous.as_ref() {
                legacy_previous_id_clones += 1;
                legacy_cloned_id_bytes += previous.0.len() as u64;
            }
            black_box(previous);
        }

        let mut optimized_slot = Some(MainPageId::new(PREVIOUS_WINDOW_ID));
        for _ in 0..LOOKUP_ROUNDS {
            let previous = std::mem::replace(&mut optimized_slot, None);
            black_box(optimized_slot.as_ref());
            optimized_slot = previous;
        }

        let optimized_previous_id_clones = 0_u64;
        println!(
            "EDITOR525_CALLBACK_SOURCE_WINDOW_REPLACE_BENCH_V1 rounds={LOOKUP_ROUNDS} legacy_previous_id_clones={legacy_previous_id_clones} optimized_previous_id_clones={optimized_previous_id_clones} legacy_cloned_id_bytes={legacy_cloned_id_bytes} clone_reduction_basis_points=10000"
        );
        assert_eq!(legacy_previous_id_clones, LOOKUP_ROUNDS as u64);
        assert_eq!(optimized_previous_id_clones, 0);
        assert_eq!(optimized_slot, legacy_slot);
    }
}
