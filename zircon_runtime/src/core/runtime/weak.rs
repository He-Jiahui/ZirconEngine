use std::any::Any;
use std::sync::{Arc, Weak};

use super::handle::CoreHandle;
use super::state::CoreRuntimeInner;
use crate::core::CoreError;

#[derive(Clone, Debug)]
pub struct CoreWeak {
    pub(crate) inner: Weak<CoreRuntimeInner>,
}

impl CoreWeak {
    pub fn upgrade(&self) -> Option<CoreHandle> {
        let Some(inner) = self.inner.upgrade() else {
            return None;
        };
        Some(CoreHandle { inner })
    }

    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_driver(name)
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_manager(name)
    }

    pub fn resolve_plugin<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_plugin(name)
    }
}
