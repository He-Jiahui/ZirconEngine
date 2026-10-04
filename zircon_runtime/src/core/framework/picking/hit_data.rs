use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

#[derive(Clone, Debug, PartialEq)]
/// 后端提供的命中空间数据；深度参与统一排序，位置和法线可缺省以容纳屏幕空间候选。
pub struct HitData {
    pub camera: EntityId,
    pub depth: Real,
    pub position: Option<Vec3>,
    pub normal: Option<Vec3>,
}

impl HitData {
    pub fn new(
        camera: EntityId,
        depth: Real,
        position: Option<Vec3>,
        normal: Option<Vec3>,
    ) -> Self {
        Self {
            camera,
            depth,
            position,
            normal,
        }
    }
}
