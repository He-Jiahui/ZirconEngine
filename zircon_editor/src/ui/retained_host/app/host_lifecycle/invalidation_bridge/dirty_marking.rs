use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn mark_layout_dirty(&mut self) {
        self.invalidate_host(HostInvalidationMask::LAYOUT);
    }

    pub(in crate::ui::retained_host::app) fn mark_presentation_dirty(&mut self) {
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn mark_presentation_dirty_for_view(
        &mut self,
        view: &ViewInstanceId,
    ) {
        self.invalidate_host_for_view(view, HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn mark_presentation_dirty_for_pane(
        &mut self,
        pane_id: &str,
    ) -> bool {
        let Some(scope) = self
            .committed_shell_state
            .as_ref()
            .and_then(|state| state.shell_content_scope_for_pane(pane_id))
        else {
            return false;
        };
        self.invalidate_host_for_shell_content(scope, HostInvalidationMask::SHELL_CONTENT);
        true
    }

    pub(in crate::ui::retained_host::app) fn mark_render_and_presentation_dirty(&mut self) {
        self.invalidate_host(
            HostInvalidationMask::RENDER.union(HostInvalidationMask::PRESENTATION_DATA),
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn pane_presentation_uses_shell_content_invalidation_with_no_global_mask() {
        let source = include_str!("dirty_marking.rs");
        let function = source
            .split("fn mark_presentation_dirty_for_pane")
            .nth(1)
            .and_then(|body| body.split("fn mark_render_and_presentation_dirty").next())
            .expect("pane presentation invalidation implementation");

        assert!(function.contains("shell_content_scope_for_pane(pane_id)"));
        assert!(function.contains("HostInvalidationMask::SHELL_CONTENT"));
        assert!(!function.contains("HostInvalidationMask::PRESENTATION_DATA"));
        assert!(!function.contains("self.invalidate_host("));
    }
}
