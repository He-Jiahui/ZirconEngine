#[path = "profiling_artifacts/environment.rs"]
mod environment;
#[path = "profiling_artifacts/export.rs"]
mod export;
#[path = "profiling_artifacts/geometry.rs"]
mod geometry;
#[path = "../../contract/profiling_artifacts/native_hierarchy_draws.rs"]
pub(in crate::ui::retained_host::host_contract) mod native_hierarchy_draws;
#[path = "../../contract/profiling_artifacts/normal_capture_hold.rs"]
pub(in crate::ui::retained_host::host_contract) mod normal_capture_hold;
#[path = "../../contract/profiling_artifacts/normal_frame_readback.rs"]
pub(in crate::ui::retained_host::host_contract) mod normal_frame_readback;
#[path = "profiling_artifacts/normal_present.rs"]
pub(in crate::ui::retained_host::host_contract) mod normal_present;
#[path = "profiling_artifacts/schema.rs"]
mod schema;
#[path = "profiling_artifacts/submitted_text.rs"]
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
