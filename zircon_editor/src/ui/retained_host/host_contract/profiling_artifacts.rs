mod environment;
mod export;
mod geometry;
mod schema;

pub(in crate::ui::retained_host::host_contract) use environment::profile_capture_enabled;
pub(in crate::ui::retained_host::host_contract) use export::queue_present_artifacts;
pub(in crate::ui::retained_host::host_contract) use schema::{
    UiProfileFrame, UiProfileGeometry, UiProfileHitSample, UiProfileLayout, UiProfileNamedFrame,
    UiProfilePoint, UiProfileSize, UiProfileTabFrame,
};

#[cfg(test)]
mod tests;
