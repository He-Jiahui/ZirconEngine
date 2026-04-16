use super::precision_shape::PrecisionShape;
use crate::editing::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;

#[derive(Clone, Debug)]
pub(in crate::editing::viewport::pointer) struct PrecisionCandidate {
    pub(in crate::editing::viewport::pointer) route: ViewportPointerRoute,
    pub(in crate::editing::viewport::pointer) priority: u8,
    pub(in crate::editing::viewport::pointer) shape: PrecisionShape,
}
