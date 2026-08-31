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
    for result in results {
        host_requests.extend(result.host_requests.iter().cloned());
        if !redraw_requested
            && result
                .applied_effects
                .iter()
                .any(|applied| matches!(applied.effect, UiDispatchEffect::DirtyRedraw { .. }))
        {
            redraw_requested = true;
        }
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
#[path = "outcome/single_pass_metadata_tests.rs"]
mod single_pass_metadata_tests;
