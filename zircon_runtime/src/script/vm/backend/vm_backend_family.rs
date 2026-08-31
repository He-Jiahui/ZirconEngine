use std::sync::Arc;

use super::{VmBackend, VmError};

pub trait VmBackendFamily: Send + Sync {
    fn family_name(&self) -> &str;

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError>;

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str));
}
