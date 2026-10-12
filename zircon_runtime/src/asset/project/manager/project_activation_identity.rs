//! Identity of one ProjectManager activation; it owns no project or service payload.
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::ProjectManager;

/// An opaque, process-local identity for one published project activation.
///
/// Clones retain only the marker. Catalog updates preserve it; opening a replacement
/// packet creates a different marker, including reopening the same physical root.
#[derive(Clone, Debug)]
pub struct ProjectActivationIdentity(Arc<()>);

impl ProjectActivationIdentity {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }
}

impl PartialEq for ProjectActivationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ProjectActivationIdentity {}

impl Hash for ProjectActivationIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl ProjectManager {
    pub fn activation_identity(&self) -> &ProjectActivationIdentity {
        &self.activation_identity
    }

    pub(crate) fn begin_new_activation(&mut self) {
        self.activation_identity = ProjectActivationIdentity::new();
    }
}
