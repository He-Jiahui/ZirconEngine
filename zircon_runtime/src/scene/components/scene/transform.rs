use crate::core::math::{Mat4, Transform};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect,
)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::LocalTransform",
    script_visibility = "public",
    field(
        name = "translation",
        value_type_path = "Vec3",
        editor_hint = "Vec3",
        read = "super::reflection::local_transform::read_translation",
        write = "super::reflection::local_transform::write_translation"
    ),
    field(
        name = "rotation",
        value_type_path = "Vec4",
        editor_hint = "Vec4",
        read = "super::reflection::local_transform::read_rotation",
        readonly
    ),
    field(
        name = "scale",
        value_type_path = "Vec3",
        editor_hint = "Vec3",
        read = "super::reflection::local_transform::read_scale",
        write = "super::reflection::local_transform::write_scale"
    )
)]
/// 节点作者编辑的局部变换；World 由它和层级关系计算全局矩阵，不能直接把全局缓存当作源数据保存。
pub struct LocalTransform {
    #[zr_reflect(skip)]
    pub transform: Transform,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// World 内部传播维护的全局矩阵缓存；通用组件写入与可变查询不得把它当作作者输入修改。
pub struct WorldMatrix(pub Mat4);

impl Default for WorldMatrix {
    fn default() -> Self {
        Self(Mat4::IDENTITY)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldTransform {
    pub transform: Transform,
}

impl Default for WorldTransform {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}
