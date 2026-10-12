#[path = "error.rs"]
mod error;
#[path = "plan.rs"]
mod plan;
#[path = "registration.rs"]
mod registration;

pub use error::WorldRuntimeExtensionError;
pub use plan::WorldRuntimeExtensionPlan;
pub use registration::WorldRuntimeExtensionRegistration;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
