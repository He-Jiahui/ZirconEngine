use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;

impl UiHostWindow {
    pub(in crate::ui::retained_host::host_contract) fn begin_template_button_press(
        &self,
        hit: &TemplateNodePointerHit,
        x: f32,
        y: f32,
    ) {
        self.state
            .borrow_mut()
            .begin_template_button_press(hit, x, y);
    }

    pub(in crate::ui::retained_host::host_contract) fn clear_template_button_press(
        &self,
    ) -> NativePointerDispatchResult {
        let mut state = self.state.borrow_mut();
        let damage = state.clear_template_button_press();
        if state.exit_requested {
            return NativePointerDispatchResult::idle();
        }
        damage
            .map(NativePointerDispatchResult::region_with_frame_update)
            .unwrap_or_else(NativePointerDispatchResult::idle)
    }
}
