use super::super::super::RetainedEditorHost;

#[cfg(feature = "profiling")]
use super::super::super::HostInvalidationMask;
#[cfg(feature = "profiling")]
use super::{commands::profile_command_for_action, status::performance_timeline_action_status};
#[cfg(feature = "profiling")]
use zircon_runtime_interface::ProfileControlRequest;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_performance_timeline_action(
        &mut self,
        action_id: &str,
    ) {
        #[cfg(feature = "profiling")]
        {
            self.dispatch_performance_timeline_action_enabled(action_id);
        }

        #[cfg(not(feature = "profiling"))]
        {
            let _ = action_id;
            self.set_status_line("Profiling controls require a profiling build");
        }
    }

    #[cfg(feature = "profiling")]
    fn dispatch_performance_timeline_action_enabled(&mut self, action_id: &str) {
        let Some(command) = profile_command_for_action(action_id) else {
            self.set_status_line(format!("Unknown performance timeline action {action_id}"));
            return;
        };
        let request = ProfileControlRequest {
            command,
            config: None,
        };
        let editor_response =
            zircon_runtime::core::diagnostics::profiling::control(request.clone());
        let runtime_response = self.runtime_client.profile_control(&request);

        self.set_status_line(performance_timeline_action_status(
            &editor_response,
            runtime_response,
        ));
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }
}
