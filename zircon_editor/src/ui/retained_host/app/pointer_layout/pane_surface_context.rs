use super::super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn pane_surface_host(
        &self,
    ) -> PaneSurfaceHostContext<'_> {
        self.ui.global::<PaneSurfaceHostContext>()
    }
}
