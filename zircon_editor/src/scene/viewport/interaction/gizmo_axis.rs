//! 轴身份在工具几何、拾取路由和事务规格之间传递，颜色或屏幕方向不能代替稳定轴标识。

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}
