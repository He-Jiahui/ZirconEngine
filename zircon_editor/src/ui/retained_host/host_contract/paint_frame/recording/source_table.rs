use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::ui::retained_host::host_contract) struct HostRenderSourceKey(
    pub(in crate::ui::retained_host::host_contract) u32,
);

#[derive(Clone, Debug, Default)]
pub(in crate::ui::retained_host::host_contract) struct HostRenderSourceTable {
    frames: Vec<Arc<UiSurfaceFrame>>,
    keys_by_identity: HashMap<usize, HostRenderSourceKey>,
}

impl HostRenderSourceTable {
    pub(in crate::ui::retained_host::host_contract) fn register(
        &mut self,
        frame: &Arc<UiSurfaceFrame>,
    ) -> Option<HostRenderSourceKey> {
        let identity = Arc::as_ptr(frame) as usize;
        if let Some(key) = self.keys_by_identity.get(&identity).copied() {
            debug_assert!(self
                .resolve(key)
                .is_some_and(|candidate| Arc::ptr_eq(candidate, frame)));
            return Some(key);
        }

        let key = HostRenderSourceKey(u32::try_from(self.frames.len()).ok()?);
        self.frames.push(Arc::clone(frame));
        self.keys_by_identity.insert(identity, key);
        Some(key)
    }

    pub(in crate::ui::retained_host::host_contract) fn resolve(
        &self,
        key: HostRenderSourceKey,
    ) -> Option<&Arc<UiSurfaceFrame>> {
        self.frames.get(key.0 as usize)
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.frames.len()
    }
}

impl PartialEq for HostRenderSourceTable {
    fn eq(&self, other: &Self) -> bool {
        self.frames.len() == other.frames.len()
            && self
                .frames
                .iter()
                .zip(other.frames.iter())
                .all(|(left, right)| Arc::ptr_eq(left, right))
    }
}

#[cfg(test)]
#[path = "tests/source_table.rs"]
mod tests;
