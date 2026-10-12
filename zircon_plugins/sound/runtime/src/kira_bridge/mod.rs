#[path = "cpal_owner.rs"]
mod cpal_owner;
#[path = "device.rs"]
mod device;
#[path = "graph_compile.rs"]
mod graph_compile;
#[path = "graph_validation/mod.rs"]
mod graph_validation;
#[path = "manager.rs"]
mod manager;
#[path = "playback_data.rs"]
mod playback_data;

pub(crate) use cpal_owner::OwnerCpalBackend;
pub(crate) use device::admit_output_descriptor;
#[cfg(test)]
pub(crate) use device::{admit_exact_output_config, device_identity_matches, device_info_for_test};
pub(crate) use device::{available_backends, available_devices, KIRA_CPAL_BACKEND};
pub(crate) use graph_compile::{
    compile_graph, compile_graph_update, diff_graphs, GraphSyncAction, GraphSyncPlan,
    PARAMETER_TWEEN_DURATION,
};
#[cfg(test)]
pub(crate) use graph_compile::{graph_compile_invocations, reset_graph_compile_invocations};
pub(crate) use graph_validation::{validate_effect, validate_graph, validate_track_controls};
#[cfg(test)]
pub(crate) use manager::fail_next_retirement_reservation_for_test;
pub(crate) use manager::{DefaultKiraEngine, KiraEngine};
pub(crate) use playback_data::{cached_static_sound_data, static_sound_data};
