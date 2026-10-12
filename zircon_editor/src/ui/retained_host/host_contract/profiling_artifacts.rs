mod environment;
mod export;
mod geometry;
pub(in crate::ui::retained_host::host_contract) mod native_hierarchy_draws;
pub(in crate::ui::retained_host::host_contract) mod normal_capture_hold;
pub(in crate::ui::retained_host::host_contract) mod normal_frame_readback;
pub(in crate::ui::retained_host::host_contract) mod normal_present;
mod schema;
mod submitted_text;
pub(in crate::ui::retained_host::host_contract) use normal_present::{
    certify_image_residency, stream_observations,
};
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
