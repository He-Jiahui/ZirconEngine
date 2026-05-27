use thiserror::Error;

use crate::scene::ecs::SystemParamError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ScheduleError {
    #[error("system id cannot be empty")]
    EmptySystemId,
    #[error("system {0} already registered")]
    DuplicateSystem(String),
    #[error("system {system_id} failed to initialize params: {source}")]
    SystemParam {
        system_id: String,
        source: SystemParamError,
    },
}
