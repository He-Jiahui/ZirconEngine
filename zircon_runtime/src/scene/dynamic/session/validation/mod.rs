#[path = "lookup.rs"]
mod lookup;
#[path = "metadata.rs"]
mod metadata;
#[path = "support.rs"]
mod support;

pub(super) use lookup::require_slot;
pub(super) use metadata::normalize_slot_metadata;
pub(super) use support::ensure_supported;
