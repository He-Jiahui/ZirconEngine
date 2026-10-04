use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderMaterialAlphaMode;
use crate::core::math::Vec4;
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 场景持有的二维网格资源引用；项目序列化保留该配置，渲染提取再按相机层与材质模式决定是否提交。
// TODO: [CR-R02-runtime_scene_authored_components-0001] 确认旧帧提取说明对应的二维网格契约；当前只见 MeshRenderer/Sprite2dComponent 提取，M6A 又排除材质化 Mesh2d。下一步核对后续渲染契约并验证单组件提取。
pub struct Mesh2dComponent {
    pub mesh: ResourceHandle<ModelMarker>,
    pub material: ResourceHandle<MaterialMarker>,
    pub color: Vec4,
    pub z_order: i32,
    #[serde(default)]
    pub material_alpha_mode: RenderMaterialAlphaMode,
}

impl Default for Mesh2dComponent {
    fn default() -> Self {
        Self {
            mesh: ResourceHandle::new(ResourceId::from_stable_label("builtin://quad")),
            material: ResourceHandle::new(ResourceId::from_stable_label(
                "builtin://material/default",
            )),
            color: Vec4::ONE,
            z_order: 0,
            material_alpha_mode: RenderMaterialAlphaMode::Opaque,
        }
    }
}
