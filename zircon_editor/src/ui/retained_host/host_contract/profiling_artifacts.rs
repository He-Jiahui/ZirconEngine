mod environment;
mod export;
mod geometry;
mod schema;
mod submitted_text;
pub(in crate::ui::retained_host::host_contract) use submitted_text::{bind_submitted, source_rows};

pub(in crate::ui::retained_host::host_contract) use environment::{
    profile_capture_enabled, profile_dynamic_capture_enabled, profile_export_dir,
    ProfileOutputRootError,
};
pub(in crate::ui::retained_host::host_contract) use export::{
    submit_present_artifacts, ProfileArtifactSubmissionError,
};
pub(in crate::ui::retained_host::host_contract) use schema::{
    UiProfileFrame, UiProfileGeometry, UiProfileHitSample, UiProfileLayout, UiProfileLogicalSize,
    UiProfileNamedFrame, UiProfileNativeHierarchy, UiProfileNativeHierarchyRename,
    UiProfileNativeHierarchyRow, UiProfileNativePopupFocus, UiProfilePoint, UiProfileSize,
    UiProfileTabFrame,
};

#[cfg(test)]
#[path = "profiling_artifacts/tests/cases.rs"]
mod tests;
