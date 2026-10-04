use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
// TODO: [CR-MATERIAL-0001] 确认 None/ErrorMaterial 的执行语义；当前资产和 streamer 仅生成 DefaultMaterial，未见策略分派或覆盖测试。
/// 记录材质缺失或不可用时预期的替代策略；当前准备链仅生产 DefaultMaterial。
pub enum RenderMaterialFallbackPolicy {
    None,
    #[default]
    DefaultMaterial,
    ErrorMaterial,
}
