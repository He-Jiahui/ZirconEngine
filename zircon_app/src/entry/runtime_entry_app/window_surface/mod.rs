#[path = "native_target.rs"]
mod native_target;

pub(in crate::entry::runtime_entry_app) use native_target::{
    runtime_native_surface_target, NativeSurfaceTargetUnavailable,
};
