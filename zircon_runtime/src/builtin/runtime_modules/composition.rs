#[path = "composition/compiler.rs"]
mod compiler;
#[path = "composition/identity.rs"]
mod identity;
#[path = "composition/outcome.rs"]
mod outcome;

pub use compiler::RuntimeModuleCompositionCompiler;
pub use identity::RuntimeModuleCompositionIdentity;
pub use outcome::{
    RuntimeModuleCompositionPlan, RuntimeModuleCompositionRejection, RuntimeModuleCompositionResult,
};

pub(super) use identity::RuntimeModuleCompositionIdentitySeed;
pub(super) use outcome::finish_runtime_module_composition;
