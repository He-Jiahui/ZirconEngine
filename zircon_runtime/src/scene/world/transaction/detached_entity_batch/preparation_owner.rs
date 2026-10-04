use std::sync::Arc;

/// A runtime-only identity retained by outstanding preparations across World moves.
#[derive(Clone, Debug, Default)]
pub(in crate::scene::world) struct DetachPreparationOwner(Arc<()>);

impl DetachPreparationOwner {
    pub(super) fn is_same_world(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

// Runtime identity does not participate in persistent World equality.
impl PartialEq for DetachPreparationOwner {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
