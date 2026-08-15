use super::super::super::{callback_dispatch, RetainedEditorHost};
use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventKind};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_inspector_surface_control(
        &mut self,
        control_id: &str,
        event_kind: UiEventKind,
        arguments: Vec<UiBindingValue>,
    ) {
        self.focus_callback_source_window();
        let Some(result) = callback_dispatch::dispatch_builtin_inspector_surface_control(
            &self.runtime,
            &self.inspector_surface_bridge,
            control_id,
            event_kind,
            arguments,
        ) else {
            self.set_status_line(format!("Unknown inspector surface control {control_id}"));
            return;
        };

        self.apply_dispatch_result(result);
    }
}
