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

    // 只有已提交 shell 能识别 pane 时才请求局部内容刷新；调用方据返回值决定是否走全局回退。
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
#[path = "tests/dirty_marking.rs"]
mod tests;
