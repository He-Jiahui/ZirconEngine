//! A single, temporary visibility gesture owned by the existing PreviewHost.

mod contract;
mod target;
mod transaction;

pub(super) use contract::PreviewVisibilityState;
pub(crate) use contract::{
    UiPreviewVisibilityError, UiPreviewVisibilityGesture, UiPreviewVisibilityOutcome,
    UiPreviewVisibilityPublication, UiPreviewVisibilityResetError, UiPreviewVisibilitySample,
    UiPreviewVisibilitySource, UiPreviewVisibilityTarget,
};

#[cfg(test)]
#[path = "tests/clone.rs"]
mod preview_visibility_clone_tests;
