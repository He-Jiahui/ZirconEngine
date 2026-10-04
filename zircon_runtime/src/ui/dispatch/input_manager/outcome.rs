use zircon_runtime_interface::ui::dispatch::{
    UiDispatchEffect, UiDispatchHostRequest, UiInputDispatchResult,
};

use crate::ui::surface::UiSurface;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiInputDispatchOutcome {
    pub results: Vec<UiInputDispatchResult>,
    pub host_requests: Vec<UiDispatchHostRequest>,
    pub redraw_requested: bool,
}

fn collect_dispatch_metadata(
    results: &[UiInputDispatchResult],
    initial_redraw_requested: bool,
) -> (Vec<UiDispatchHostRequest>, bool) {
    let mut host_requests = Vec::new();
    let mut redraw_requested = initial_redraw_requested;
    let mut remaining_results = results.len();
    for result in results {
        if host_requests.is_empty() && !result.host_requests.is_empty() {
            host_requests.reserve(remaining_results.max(result.host_requests.len()));
        }
        host_requests.extend(result.host_requests.iter().cloned());
        if !redraw_requested
            && result
                .applied_effects
                .iter()
                .any(|applied| matches!(applied.effect, UiDispatchEffect::DirtyRedraw { .. }))
        {
            redraw_requested = true;
        }
        remaining_results = remaining_results.saturating_sub(1);
    }
    (host_requests, redraw_requested)
}

impl UiInputDispatchOutcome {
    pub(crate) fn from_results(surface: &UiSurface, results: Vec<UiInputDispatchResult>) -> Self {
        let (host_requests, redraw_requested) =
            collect_dispatch_metadata(&results, surface.window_state.redraw_requested);

        Self {
            results,
            host_requests,
            redraw_requested,
        }
    }
}

#[cfg(test)]
#[path = "outcome/tests/single_pass_metadata_tests.rs"]
mod single_pass_metadata_tests;

#[cfg(test)]
#[path = "outcome/tests/host_request_capacity_tests.rs"]
mod host_request_capacity_tests;
