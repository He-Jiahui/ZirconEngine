use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::workbench::page_tabs::main_page_tab_close_frame;

use super::HostPagePointerError;
use super::host_page_pointer_bridge::HostPagePointerBridge;

impl HostPagePointerBridge {
    pub(super) fn update_measured_frame(
        &mut self,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
    ) -> Result<Option<UiFrame>, HostPagePointerError> {
        if !tab_x.is_finite() || !tab_width.is_finite() || tab_width <= 0.0 {
            return Err(HostPagePointerError::InvalidTabFrame {
                item_index,
                x: tab_x,
                width: tab_width,
            });
        }
        let Some(tab_position) = self
            .layout
            .tabs
            .iter()
            .position(|tab| tab.page_index == item_index)
        else {
            return Ok(None);
        };
        let measured_frame = {
            let tab = &self.layout.tabs[tab_position];
            UiFrame::new(
                self.layout.strip_frame.x + tab_x,
                tab.frame.y,
                tab_width,
                tab.frame.height,
            )
        };
        let closeable = self
            .layout
            .items
            .get(item_index)
            .is_some_and(|item| item.close_instance_id.is_some());
        let measured_close_frame = closeable.then(|| main_page_tab_close_frame(measured_frame));
        let tab = &mut self.layout.tabs[tab_position];
        if tab.frame == measured_frame && tab.close_frame == measured_close_frame {
            return Ok(Some(measured_frame));
        }
        tab.frame = measured_frame;
        tab.close_frame = measured_close_frame;
        self.rebuild_surface();
        Ok(Some(measured_frame))
    }
}
