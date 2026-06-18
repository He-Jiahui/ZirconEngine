use super::super::{callback_dispatch, HostShellPointerRoute, RetainedEditorHost};
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn floating_window_header_pointer_clicked(
        &mut self,
        x: f32,
        y: f32,
    ) {
        self.use_committed_pointer_layout();
        let Some(window_id) = self
            .shell_pointer_bridge
            .drag_route_at(UiPoint::new(x, y))
            .and_then(|route| match route {
                HostShellPointerRoute::FloatingWindow(window_id)
                | HostShellPointerRoute::FloatingWindowEdge { window_id, .. } => Some(window_id),
                HostShellPointerRoute::DragTarget(_)
                | HostShellPointerRoute::DocumentEdge(_)
                | HostShellPointerRoute::Resize(_) => None,
            })
        else {
            return;
        };

        if let Some(result) =
            callback_dispatch::dispatch_builtin_floating_window_focus(&self.runtime, &window_id)
        {
            self.apply_dispatch_result(result);
            self.note_focused_floating_window(Some(window_id));
        }
    }
}
