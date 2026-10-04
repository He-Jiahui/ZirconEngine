mod catalog;
mod cloud;
mod config;
mod error;
mod http;
mod identity;
mod lifecycle;
mod organization;
mod storage;

#[cfg(test)]
#[path = "tests/test_support.rs"]
pub(crate) mod test_support;

pub use lifecycle::run;
