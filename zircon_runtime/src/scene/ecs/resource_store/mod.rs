#[path = "store.rs"]
mod store;
#[path = "stored_resource.rs"]
mod stored_resource;

pub use store::ResourceStore;
pub(crate) use stored_resource::TransferredResourceRow;
