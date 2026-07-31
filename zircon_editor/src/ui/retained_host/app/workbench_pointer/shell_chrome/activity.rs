use super::super::super::{HostActivityRailPointerSide, RetainedEditorHost, callback_dispatch};
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn activity_rail_pointer_clicked(
        &mut self,
        side: &str,
        x: f32,
        y: f32,
    ) {
        self.use_committed_pointer_layout();
        let side = match HostActivityRailPointerSide::parse(side) {
            Ok(side) => side,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        match callback_dispatch::dispatch_shared_activity_rail_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &mut self.activity_rail_pointer_bridge,
            side,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
