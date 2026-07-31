//! Document dirty-state projection backed by Editor03 saved-top authority.

mod error;
mod external_effect_id;
mod registry;

pub use error::DirtyRegistryError;
pub use external_effect_id::{DirtyExternalEffectId, DirtyExternalEffectIdError};
pub use registry::{
    DirtyDocumentSnapshot, DirtyExternalEffectRevision, DirtyRegistry, DirtyRegistryCursor,
    DirtyRegistryDelta,
};

#[cfg(test)]
mod tests;
