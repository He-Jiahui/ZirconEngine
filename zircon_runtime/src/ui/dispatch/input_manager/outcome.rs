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

impl UiInputDispatchOutcome {
    pub(crate) fn from_results(surface: &UiSurface, results: Vec<UiInputDispatchResult>) -> Self {
        let host_requests = results
            .iter()
            .flat_map(|result| result.host_requests.iter().cloned())
            .collect();
        let redraw_requested = surface.window_state.redraw_requested
            || results.iter().any(|result| {
                result
                    .applied_effects
                    .iter()
                    .any(|applied| matches!(applied.effect, UiDispatchEffect::DirtyRedraw { .. }))
            });

        Self {
            results,
            host_requests,
            redraw_requested,
        }
    }
}
