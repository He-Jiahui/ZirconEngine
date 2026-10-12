#[path = "catalog/mod.rs"]
mod catalog;
#[path = "cloud/mod.rs"]
mod cloud;
#[path = "config.rs"]
mod config;
#[path = "error.rs"]
mod error;
#[path = "http/mod.rs"]
mod http;
#[path = "identity/mod.rs"]
mod identity;
#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "organization/mod.rs"]
mod organization;
#[path = "storage/mod.rs"]
mod storage;

#[cfg(test)]
#[path = "tests/test_support.rs"]
pub(crate) mod test_support;

pub use lifecycle::run;
