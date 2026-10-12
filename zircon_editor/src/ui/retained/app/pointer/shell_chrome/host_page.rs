use super::super::super::{callback_dispatch, RetainedEditorHost};
use crate::ui::retained_host::{
    host_page_pointer::HOST_PAGE_OVERFLOW_POINTER_INDEX, HostPageOverflowMenuStateData,
    UiHostContext,
};
impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn host_page_pointer_clicked(
        &mut self,
        tab_index: i32,
        close: bool,
    ) {
        self.use_committed_pointer_layout();
        if tab_index == HOST_PAGE_OVERFLOW_POINTER_INDEX {
            self.host_page_overflow_pointer_clicked();
            return;
        }
        if tab_index < 0 {
            self.set_status_line(format!("Invalid host page tab index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_host_page_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &self.host_page_pointer_bridge,
            tab_index as usize,
            close,
        ) {
            Ok(dispatch) => {
                if dispatch.pointer.route.is_some() {
                    self.set_host_page_overflow_menu_open(false);
                }
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    fn host_page_overflow_pointer_clicked(&mut self) {
        zircon_runtime::profile_counter!("editor", "ui.host_page.native_overflow_receipt_count", 1);
        let open = !self
            .ui
            .get_host_presentation_generation()
            .page_overflow_menu_state()
            .open;
        self.set_host_page_overflow_menu_open(open);
        let mut effects = crate::ui::retained_host::event_bridge::UiHostEventEffects::default();
        effects.request_paint_only();
        self.apply_dispatch_effects(effects);
    }

    fn set_host_page_overflow_menu_open(&self, open: bool) {
        self.ui
            .global::<UiHostContext>()
            .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData {
                open,
                hovered_page_index: -1,
                scroll_offset: 0.0,
            });
    }
}
