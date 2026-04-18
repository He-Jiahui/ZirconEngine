use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportToolbarPointerDispatch {
    pub route: Option<ViewportToolbarPointerRoute>,
}
