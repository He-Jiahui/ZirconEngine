#[path = "acoustics.rs"]
mod acoustics;
#[path = "automation_timeline.rs"]
mod automation_timeline;
#[path = "clip_assets.rs"]
mod clip_assets;
#[path = "dynamic_event_executors/mod.rs"]
mod dynamic_event_executors;
#[path = "dynamic_events/mod.rs"]
mod dynamic_events;
#[path = "external_sources.rs"]
mod external_sources;
#[path = "hrtf_profiles.rs"]
mod hrtf_profiles;
#[path = "impulse_responses.rs"]
mod impulse_responses;
#[path = "manager_state.rs"]
mod manager_state;
#[path = "manager_trait.rs"]
mod manager_trait;
#[path = "mixer_graph/mod.rs"]
mod mixer_graph;
#[path = "mixer_presets.rs"]
mod mixer_presets;
#[path = "output_device/mod.rs"]
mod output_device;
#[path = "output_render.rs"]
mod output_render;
#[path = "parameters.rs"]
mod parameters;
#[path = "playback.rs"]
mod playback;
#[path = "playback_controls.rs"]
mod playback_controls;
#[path = "playback_status.rs"]
mod playback_status;
#[path = "playback_validation.rs"]
mod playback_validation;
#[path = "ray_tracing_convolution.rs"]
mod ray_tracing_convolution;
#[path = "runtime_settings.rs"]
mod runtime_settings;
#[path = "scene.rs"]
mod scene;
#[path = "source_controls.rs"]
mod source_controls;
#[path = "source_seek.rs"]
mod source_seek;
#[path = "source_status.rs"]
mod source_status;
#[path = "sources.rs"]
mod sources;
#[path = "timeline_sequences.rs"]
mod timeline_sequences;

pub use manager_state::{DefaultSoundManager, SoundDriver};
pub(crate) use scene::SceneSource;

#[cfg(test)]
pub(crate) use mixer_graph::sync::{
    last_graph_commit_lock_hold_for_test, ActiveGraphCommitHarness,
};
#[cfg(test)]
pub(crate) use output_device::{
    signal_owner_gain_writer_admitted, wait_for_owner_gain_writer_admission_attempt,
};
pub(crate) use playback_controls::kira_slice_position_for_absolute_frame;
pub(crate) use playback_status::absolute_position_from_kira_slice;
pub(crate) use source_controls::{
    mute_bound_source, pause_bound_source, resume_bound_source, set_bound_source_gain,
    set_bound_source_speed,
};
pub(crate) use source_seek::seek_bound_source;
pub(crate) use sources::{prepare_source_generation, stop_bound_source, sync_source_voice};
