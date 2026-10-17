#[path = "backend.rs"]
mod backend;
#[path = "catalog.rs"]
mod catalog;
#[path = "configuration.rs"]
mod configuration;
#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "status.rs"]
mod status;

#[cfg(test)]
pub(crate) use lifecycle::{
    signal_owner_gain_writer_admitted, wait_for_owner_gain_writer_admission_attempt,
};
