use serde::{Deserialize, Serialize};

/// 包验证视角。运行时据此选择动作权限，并在报告中声明保留或剥离的区段。
/// 产物实际携带的字段由运行时产物类型决定，不能仅由 profile 推断。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCompiledAssetPackageProfile {
    #[default]
    Runtime,
    Editor,
}
