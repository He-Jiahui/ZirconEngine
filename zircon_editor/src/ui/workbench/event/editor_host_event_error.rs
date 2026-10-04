use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
/// 宿主绑定适配失败的边界诊断；与命令已识别但当前不可执行的注册表错误分开传播。
pub enum EditorHostEventError {
    #[error("unsupported editor host binding payload")]
    UnsupportedPayload,
    #[error("unknown menu action id {0}")]
    UnknownMenuAction(String),
}
