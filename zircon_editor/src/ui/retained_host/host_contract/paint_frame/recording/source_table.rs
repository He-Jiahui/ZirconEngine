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
mod tests {
    use super::*;

    #[test]
    fn registration_deduplicates_only_the_same_published_frame_arc() {
        let frame = Arc::new(UiSurfaceFrame::default());
        let equivalent_but_distinct = Arc::new(frame.as_ref().clone());
        let mut table = HostRenderSourceTable::default();

        let first = table.register(&frame).expect("first source key");
        let repeated = table.register(&frame).expect("repeated source key");
        let distinct = table
            .register(&equivalent_but_distinct)
            .expect("distinct source key");

        assert_eq!(first, repeated);
        assert_ne!(first, distinct);
        assert_eq!(table.len(), 2);
        assert!(table
            .resolve(first)
            .is_some_and(|resolved| Arc::ptr_eq(resolved, &frame)));
    }
}
