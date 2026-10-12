use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const GENERATED_BOTTOM_DRAWER_HOST_CONTROL_ID: &str = "WorkbenchGeneratedBottomDrawerHost";
const GENERATED_BOTTOM_DRAWER_CONTROL_ID: &str = "WorkbenchGeneratedBottomDrawer";
const GENERATED_BOTTOM_PANEL_CONTROL_ID: &str = "WorkbenchGeneratedBottomPanel";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn open_workbench_generated_bottom_drawer(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_visible(GENERATED_BOTTOM_DRAWER_HOST_CONTROL_ID, true)?;
        self.set_visible(GENERATED_BOTTOM_DRAWER_CONTROL_ID, true)?;
        self.set_visible(GENERATED_BOTTOM_PANEL_CONTROL_ID, true)
    }

    pub(super) fn close_workbench_generated_bottom_drawer(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        // Preserve generated-bottom selection/input state while the collapsed drawer owns visibility.
        self.set_visible(GENERATED_BOTTOM_DRAWER_HOST_CONTROL_ID, false)?;
        self.set_visible(GENERATED_BOTTOM_DRAWER_CONTROL_ID, false)
    }
}
