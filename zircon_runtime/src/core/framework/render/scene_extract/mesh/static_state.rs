/// 静态绘制和阴影缓存的资格声明；仅标记静态并不足以证明资源未变化。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderMeshStaticState {
    pub transform_static: bool,
    pub geometry_revision: u64,
    pub material_revision: u64,
}

impl RenderMeshStaticState {
    pub const fn new(
        transform_static: bool,
        geometry_revision: u64,
        material_revision: u64,
    ) -> Self {
        Self {
            transform_static,
            geometry_revision,
            material_revision,
        }
    }

    /// 只知道变换是否静态时使用；修订号保持未知，缓存路径会保守地重建。
    pub const fn from_transform_static(transform_static: bool) -> Self {
        Self {
            transform_static,
            geometry_revision: 0,
            material_revision: 0,
        }
    }

    /// 仅在静态变换和几何、材质修订都可信时允许跨帧复用。
    pub const fn has_authoritative_revisions(self) -> bool {
        self.transform_static && self.geometry_revision != 0 && self.material_revision != 0
    }
}
