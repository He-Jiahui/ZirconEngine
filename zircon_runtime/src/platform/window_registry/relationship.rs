use crate::core::framework::window::WindowId;

/// Ownership semantics for one window's relationship to its parent.
///
/// Platform backends may map these to native transient/modal/owner behavior,
/// but the registry owns the generation-qualified graph and destruction
/// ordering independent of any backend handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowParentKind {
    Transient,
    Modal,
    OwnerShutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowParentRelation {
    pub(super) window: WindowId,
    pub(super) kind: WindowParentKind,
}

impl WindowParentRelation {
    pub(super) const fn new(window: WindowId, kind: WindowParentKind) -> Self {
        Self { window, kind }
    }
}
