use crate::ui::host::editor_activity_log::parse_activity_log_jump_action_id;
use crate::ui::retained_host::event_bridge::{apply_record_effects, UiHostEventEffects};

use super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_activity_log_jump_action(
        &mut self,
        action_id: &str,
    ) -> bool {
        let Some(sequence) = parse_activity_log_jump_action_id(action_id) else {
            return false;
        };
        let result = self
            .runtime
            .dispatch_activity_log_jump(sequence)
            .map(|record| {
                let mut effects = UiHostEventEffects::default();
                if let Some(record) = record.as_ref() {
                    apply_record_effects(&mut effects, record);
                }
                effects
            });
        self.apply_dispatch_result(result);
        true
    }
}
