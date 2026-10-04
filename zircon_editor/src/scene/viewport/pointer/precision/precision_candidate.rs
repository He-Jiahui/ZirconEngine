//! 精确候选同时绑定路由、类别和屏幕形状，最终解析时必须保留这一身份，不能只按 UI 节点编号选择实体。

use super::precision_shape::PrecisionShape;
use crate::scene::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;

#[derive(Clone, Debug)]
pub(in crate::scene::viewport::pointer) struct PrecisionCandidate {
    pub(in crate::scene::viewport::pointer) route: ViewportPointerRoute,
    pub(in crate::scene::viewport::pointer) priority: u8,
    pub(in crate::scene::viewport::pointer) shape: PrecisionShape,
}
