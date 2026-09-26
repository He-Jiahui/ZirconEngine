use serde::{Deserialize, Serialize};

use crate::ui::template::UiCompiledAssetPackageValidationReport;

pub const UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION: u32 = 3;

// TODO: [CR-UITEMPLATE-0001] 运行时编译器返回独立的 `UiRuntimeCompiledAssetArtifact`；
// 确认该公开形状是否仍需作为受支持的产物入口及二者的转换边界。
/// 接口层可序列化的包产物形状，供同层清单构造器读取报告；产物字节由调用者另行传入。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiCompiledAssetArtifact {
    pub report: UiCompiledAssetPackageValidationReport,
    #[serde(default)]
    pub bytes: Vec<u8>,
}
