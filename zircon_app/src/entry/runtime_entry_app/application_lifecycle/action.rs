#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SurfaceReleaseAction {
    Noop,
    Release,
}

impl SurfaceReleaseAction {
    pub(super) const fn releases_surface(self) -> bool {
        matches!(self, Self::Release)
    }
}
