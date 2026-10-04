use serde::{Deserialize, Serialize};

// TODO: [CR-RENDER-IMAGE-0001] 确认 MainWorld/RenderWorld 何时实际控制纹理驻留；目前描述符只传递该列表，GPU 上传路径未读取它。
/// Where image asset data is expected to stay resident after render preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderImageAssetUsage {
    MainWorld,
    RenderWorld,
}
