#[path = "apply.rs"]
mod apply;
#[path = "effect/mod.rs"]
mod effect;
#[path = "listener.rs"]
mod listener;
#[path = "parameter_values.rs"]
mod parameter_values;
#[path = "source.rs"]
mod source;
#[path = "track.rs"]
mod track;
#[path = "volume.rs"]
mod volume;

pub(crate) use apply::apply_automation_target;
#[cfg(test)]
pub(crate) use apply::ensure_automation_execution_available;
