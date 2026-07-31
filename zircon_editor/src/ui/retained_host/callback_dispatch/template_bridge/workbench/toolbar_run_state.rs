use crate::ui::workbench::model::WorkbenchViewModel;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const PLAY_CONTROL_ID: &str = "WorkbenchRunPlay";
const STOP_CONTROL_ID: &str = "WorkbenchRunStop";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_toolbar_run_state(
        &mut self,
        model: &WorkbenchViewModel,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_visible(PLAY_CONTROL_ID, !model.is_playing)?;
        self.set_visible(STOP_CONTROL_ID, model.is_playing)?;
        Ok(())
    }
}
