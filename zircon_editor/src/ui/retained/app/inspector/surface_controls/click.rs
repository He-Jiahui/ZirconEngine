use super::super::super::RetainedEditorHost;
use zircon_runtime_interface::ui::binding::UiEventKind;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_inspector_control_clicked(
        &mut self,
        control_id: &str,
    ) {
        let arguments = match control_id {
            "ApplyBatchButton" => match self.inspector_apply_arguments() {
                Ok(arguments) => arguments,
                Err(error) => {
                    self.set_status_line(error);
                    return;
                }
            },
            "DeleteSelected" => Vec::new(),
            _ => {
                self.set_status_line(format!("Unknown inspector click control {control_id}"));
                return;
            }
        };

        self.dispatch_inspector_surface_control(control_id, UiEventKind::Click, arguments);
    }
}
