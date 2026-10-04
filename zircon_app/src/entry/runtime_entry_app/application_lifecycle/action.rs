#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::entry::runtime_entry_app) enum SurfaceReleaseAction {
    Noop,
    Release,
}

impl SurfaceReleaseAction {
    pub(super) const fn releases_surface(self) -> bool {
        matches!(self, Self::Release)
    }
}
