use super::super::*;
use crate::ui::retained_host::app::showcase_event_inputs::demo_input_for_showcase_action as static_demo_input_for_showcase_action;
use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

impl RetainedEditorHost {
    pub(super) fn demo_input_for_showcase_action(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) -> UiComponentShowcaseDemoEventInput {
        if let Some(payload) = self.take_active_reference_drag_payload_for_drop(action_id) {
            return UiComponentShowcaseDemoEventInput::DropReference { payload };
        }
        if action_id.contains("VirtualListScrolled") {
            return self.next_showcase_virtual_list_range(control_id);
        }
        if action_id.contains("PagedListNextPage") {
            return self.next_showcase_page(control_id);
        }
        static_demo_input_for_showcase_action(control_id, action_id)
    }

    fn next_showcase_virtual_list_range(
        &self,
        control_id: &str,
    ) -> UiComponentShowcaseDemoEventInput {
        let current_start = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "viewport_start")
            .unwrap_or(0);
        let visible_count = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "viewport_count")
            .unwrap_or(25)
            .max(1);
        let total_count = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "total_count")
            .unwrap_or(current_start + visible_count)
            .max(0);
        let max_start = total_count.saturating_sub(visible_count).max(0);
        let start = (current_start + visible_count).min(max_start);
        UiComponentShowcaseDemoEventInput::SetVisibleRange {
            start,
            count: visible_count,
        }
    }

    fn next_showcase_page(&self, control_id: &str) -> UiComponentShowcaseDemoEventInput {
        let page_index = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "page_index")
            .unwrap_or(0);
        let page_size = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "page_size")
            .unwrap_or(100)
            .max(1);
        let page_count = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "page_count")
            .unwrap_or(page_index + 2)
            .max(1);
        UiComponentShowcaseDemoEventInput::SetPage {
            page_index: (page_index + 1).min(page_count - 1),
            page_size,
        }
    }
}
