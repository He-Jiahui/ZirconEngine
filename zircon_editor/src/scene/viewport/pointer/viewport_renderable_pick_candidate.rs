//! 渲染物的 owner、世界位置和近似半径用于呈现拾取，值本身不证明该实体在当前渲染产品中可见。

use zircon_runtime_interface::math::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportRenderablePickCandidate {
    pub owner: u64,
    pub position: Vec3,
    pub radius_world: f32,
}
