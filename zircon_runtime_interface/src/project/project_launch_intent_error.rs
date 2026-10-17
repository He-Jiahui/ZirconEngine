use std::path::PathBuf;

use thiserror::Error;

use super::ProjectNameError;

/// Rejects a launch request that cannot safely enter project preflight.
/// 启动意图在协议版本、名称或路径输入校验阶段被拒绝时返回的原因。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ProjectLaunchIntentError {
    /// 线协议版本与当前实现支持的版本不匹配。
    #[error("project launch intent schema version {actual} is unsupported; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, actual: u32 },
    /// 打开请求的路径为空或仅含空白。
    #[error("project launch intent open path must not be empty")]
    EmptyOpenPath,
    /// 创建请求的目标位置为空或仅含空白。
    #[error("project launch intent create location must not be empty")]
    EmptyCreateLocation,
    /// 创建请求中的项目名称未通过项目名称校验；保留底层错误供调用方查看。
    #[error("project launch intent create name is invalid: {source}")]
    ProjectName {
        #[from]
        #[source]
        source: ProjectNameError,
    },
    /// 路径不能表示为 Unicode 文本；携带原路径供诊断，不尝试有损转换。
    #[error("project launch intent path {path:?} contains no textual path input")]
    NonTextPath { path: PathBuf },
}
