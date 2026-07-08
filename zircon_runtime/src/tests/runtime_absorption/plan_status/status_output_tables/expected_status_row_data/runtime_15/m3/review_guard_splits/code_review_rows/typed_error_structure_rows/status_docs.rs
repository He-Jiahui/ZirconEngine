#[path = "status_docs/delegation.rs"]
mod delegation;
#[path = "status_docs/foundation.rs"]
mod foundation;
#[path = "status_docs/paths.rs"]
mod paths;
#[path = "status_docs/status_maps.rs"]
mod status_maps;
#[path = "status_docs/status_mirrors.rs"]
mod status_mirrors;

pub(super) use delegation::*;
pub(super) use foundation::*;
pub(super) use paths::*;
pub(super) use status_maps::*;
pub(super) use status_mirrors::*;
