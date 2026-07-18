mod acoustics;
mod automation_timeline;
mod clip_assets;
mod dynamic_event_executors;
mod dynamic_events;
mod external_sources;
mod hrtf_profiles;
mod impulse_responses;
mod manager_state;
mod manager_trait;
mod mixer_graph;
mod mixer_presets;
mod output_device;
mod output_render;
mod parameters;
mod playback;
mod playback_controls;
mod playback_status;
mod playback_validation;
mod ray_tracing_convolution;
mod runtime_settings;
mod source_controls;
mod source_seek;
mod source_status;
mod sources;
mod timeline_sequences;

pub use manager_state::{DefaultSoundManager, SoundDriver};

#[cfg(test)]
pub(crate) use mixer_graph::sync::{
    last_graph_commit_lock_hold_for_test, ActiveGraphCommitHarness,
};
pub(crate) use playback_controls::kira_slice_position_for_absolute_frame;
pub(crate) use playback_status::absolute_position_from_kira_slice;
pub(crate) use source_controls::{
    mute_bound_source, pause_bound_source, resume_bound_source, set_bound_source_gain,
    set_bound_source_speed,
};
pub(crate) use source_seek::seek_bound_source;
pub(crate) use sources::{stop_bound_source, sync_source_voice};
