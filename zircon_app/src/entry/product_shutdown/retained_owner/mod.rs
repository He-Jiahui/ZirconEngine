#[path = "failure.rs"]
mod failure;
#[path = "owner.rs"]
mod owner;
#[path = "packet.rs"]
mod packet;
#[path = "registry.rs"]
mod registry;

pub use failure::{ProductCloseError, ProductCompositionFailure};
pub(in crate::entry) use owner::RetainedOwner;
pub(in crate::entry) use packet::{RetainedPacket, RetainedRuntime};
pub use registry::retry_product_cleanup_until;
pub(in crate::entry) use registry::{
    ensure_product_admission, pending_owner_count, runtime_owners,
};
