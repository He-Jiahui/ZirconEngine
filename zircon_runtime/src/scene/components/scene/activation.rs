use crate::core::framework::render::DEFAULT_RENDER_LAYER_MASK;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect,
)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::ActiveSelf",
    script_visibility = "public"
)]
/// 节点作者设置的局部启用开关；层级有效状态由 World 传播计算，消费者不应把它当成最终可见性。
pub struct ActiveSelf(
    #[zr_reflect(name = "value", value_type_path = "Bool", editor_hint = "Bool")] pub bool,
);

impl Default for ActiveSelf {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect,
)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::ActiveInHierarchy",
    serialization = "none",
    serializable = false,
    script_visibility = "public"
)]
/// World 传播得到的有效启用状态；通用组件写入与可变查询不得修改它，作者应修改局部开关。
pub struct ActiveInHierarchy(
    #[zr_reflect(
        name = "value",
        value_type_path = "Bool",
        editor_hint = "Bool",
        readonly,
        serializable = false
    )]
    pub bool,
);

impl Default for ActiveInHierarchy {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect,
)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::RenderLayerMask",
    script_visibility = "public"
)]
pub struct RenderLayerMask(
    #[zr_reflect(name = "mask", value_type_path = "Unsigned", editor_hint = "Unsigned")] pub u32,
);

impl Default for RenderLayerMask {
    fn default() -> Self {
        Self(default_render_layer_mask())
    }
}

pub const fn default_render_layer_mask() -> u32 {
    DEFAULT_RENDER_LAYER_MASK
}
