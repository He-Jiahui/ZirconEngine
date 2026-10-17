use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;

const DRAWER: &str = "UnrealContentDrawerRegion";
const SEARCH: &str = "SearchEdited";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn content_drawer_is_open(&self) -> bool {
        self.content_drawer_open
    }

    pub(crate) fn toggle_content_drawer(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if self.content_drawer_open {
            return self.dismiss_content_drawer();
        }
        self.content_drawer_previous_focus = self.template_surface.surface.focus.focused;
        self.content_drawer_open = true;
        self.set_visible(DRAWER, true)?;
        self.set_control_active("UnrealContentDrawerToggle", true)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        let assets = self.content_drawer_assets.clone();
        self.prepare_content_drawer_assets(&assets)?;
        self.focus_content_drawer_search()
    }

    pub(crate) fn focus_content_drawer_search(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if !self.content_drawer_open {
            return self.toggle_content_drawer();
        }
        if let Some(search) = self.control_node_id(SEARCH) {
            self.template_surface.surface.focus_node(search)?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }

    pub(crate) fn dismiss_content_drawer_for_pointer(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.content_drawer_previous_focus = None;
        self.content_drawer_native_previous_focus = None;
        self.dismiss_content_drawer()
    }

    pub(crate) fn dismiss_content_drawer(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if !self.content_drawer_open {
            return Ok(());
        }
        let focused = self.template_surface.surface.focus.focused;
        let drawer_focus = focused.is_some_and(|node| self.content_drawer_contains(node));
        if drawer_focus {
            self.template_surface.surface.clear_focus();
        }
        self.content_drawer_open = false;
        self.set_visible(DRAWER, false)?;
        self.set_control_active("UnrealContentDrawerToggle", false)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        let previous = self.content_drawer_previous_focus.take();
        // A real focus transfer to another pane takes precedence over restoration.
        if self.template_surface.surface.focus.focused.is_none() {
            if let Some(previous) = previous {
                let _ = self.template_surface.surface.focus_node(previous);
            }
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }

    pub(super) fn content_drawer_contains(&self, node: UiNodeId) -> bool {
        self.control_node_ids_with_descendants(DRAWER)
            .contains(&node)
    }
}
