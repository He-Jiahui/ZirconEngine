#[path = "core_statuses/base_statuses.rs"]
mod base_statuses;
#[path = "core_statuses/inventory_statuses.rs"]
mod inventory_statuses;
#[path = "core_statuses/production_statuses.rs"]
mod production_statuses;

pub(in super::super) use base_statuses::*;
pub(in super::super) use inventory_statuses::*;
pub(in super::super) use production_statuses::*;
