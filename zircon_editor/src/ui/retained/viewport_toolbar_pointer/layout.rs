use super::viewport_toolbar_pointer_surface::ViewportToolbarPointerSurface;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewportToolbarPointerLayout {
    pub surfaces: Vec<ViewportToolbarPointerSurface>,
}
