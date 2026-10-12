#[path = "invalidation/mask.rs"]
mod mask;
#[path = "invalidation/root.rs"]
mod root;

pub(crate) use mask::HostInvalidationMask;
pub(super) use root::HostInvalidationRoot;
