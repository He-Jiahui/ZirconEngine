use std::sync::Weak;

use super::handle::CoreHandle;
use super::state::CoreRuntimeInner;

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
}
