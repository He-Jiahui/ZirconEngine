#[path = "runtime/capabilities.rs"]
mod capabilities;
#[path = "runtime/defaults.rs"]
mod defaults;
#[path = "runtime/dependencies.rs"]
mod dependencies;
#[path = "runtime/identity.rs"]
mod identity;
#[path = "runtime/modules.rs"]
mod modules;
#[path = "runtime/signature.rs"]
mod signature;

pub(super) use signature::optional_feature_signature;
