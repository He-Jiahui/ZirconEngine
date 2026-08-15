mod environment;
mod export;
mod geometry;
mod schema;

pub(in crate::ui::retained_host::host_contract) use environment::{
    profile_capture_enabled, profile_export_dir, ProfileOutputRootError,
};
pub(in crate::ui::retained_host::host_contract) use export::{
    submit_present_artifacts, ProfileArtifactSubmissionError,
};
pub(in crate::ui::retained_host::host_contract) use schema::{
    UiProfileFrame, UiProfileGeometry, UiProfileHitSample, UiProfileLayout, UiProfileNamedFrame,
    UiProfilePoint, UiProfileSize, UiProfileTabFrame,
};

#[cfg(test)]
mod tests;
