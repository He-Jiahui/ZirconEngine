use zircon_runtime_interface::ui::dispatch::UiClipboardRequest;

use crate::core::framework::input::ImeHostRequest;

use super::RuntimeUiSurfaceSet;

impl RuntimeUiSurfaceSet {
    pub(in crate::dynamic_api::session) fn drain_ime_host_requests_into(
        &mut self,
        output: &mut Vec<ImeHostRequest>,
    ) {
        for runtime_surface in &mut self.surfaces {
            output.extend(runtime_surface.input.drain_ime_host_requests());
        }
    }

    pub(in crate::dynamic_api::session) fn drain_clipboard_host_requests_into(
        &mut self,
        output: &mut Vec<(u32, UiClipboardRequest)>,
    ) {
        let mut requests = Vec::new();
        for (surface_index, runtime_surface) in self.surfaces.iter_mut().enumerate() {
            let Ok(target_surface) = u32::try_from(surface_index) else {
                continue;
            };
            requests.clear();
            runtime_surface
                .input
                .drain_clipboard_host_requests_into(&mut requests);
            output.extend(requests.drain(..).map(|request| (target_surface, request)));
        }
    }

    pub(in crate::dynamic_api::session) fn drain_action_host_requests_into(
        &mut self,
        output: &mut Vec<zircon_runtime_interface::ZrRuntimeUiActionHostRequestV1>,
    ) {
        self.action_requests.drain_into(output);
    }

    pub(in crate::dynamic_api::session) fn drain_ui_host_requests_into(
        &mut self,
        output: &mut Vec<zircon_runtime_interface::ZrRuntimeUiHostRequestV1>,
    ) {
        self.host_requests.drain_into(output);
    }
}

#[cfg(test)]
#[path = "tests/host_request_drain_optimization_batch_20260830cp_runtime_tests.rs"]
mod optimization_batch_20260830cp_runtime_tests;
