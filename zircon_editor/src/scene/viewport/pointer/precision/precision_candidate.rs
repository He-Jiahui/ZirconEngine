use super::precision_shape::PrecisionShape;
use crate::scene::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;

#[derive(Clone, Debug)]
pub(in crate::scene::viewport::pointer) struct PrecisionCandidate {
    pub(in crate::scene::viewport::pointer) route: ViewportPointerRoute,
    pub(in crate::scene::viewport::pointer) priority: u8,
    pub(in crate::scene::viewport::pointer) shape: PrecisionShape,
}
