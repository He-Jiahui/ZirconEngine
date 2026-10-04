use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
/// 资产透明度进入可见性、渲染队列与材质准备阶段的共同契约；遮罩阈值只对 Mask 生效。
pub enum RenderMaterialAlphaMode {
    #[default]
    Opaque,
    Mask {
        cutoff: f32,
    },
    Blend,
}
