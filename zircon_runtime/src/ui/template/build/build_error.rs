use thiserror::Error;

use zircon_runtime_interface::ui::tree::UiTreeError;

/// 模板实例无法形成运行时树时的错误，供 Editor 模板服务和预览宿主保留原始节点上下文。
/// 构建返回错误前不会发布 surface；它与资产加载、编译和后续布局计算的错误分开报告。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiTemplateBuildError {
    /// 保留共享树契约的失败原因，调用方可沿错误链判断父子身份等结构问题。
    #[error(transparent)]
    Tree(#[from] UiTreeError),
    /// 控件查找要求实例内身份唯一；同时保留首次声明和冲突位置以便定位资源。
    #[error(
        "duplicate control id {control_id:?} at {duplicate_node_path}; first declared at {first_node_path}"
    )]
    DuplicateControlId {
        control_id: String,
        first_node_path: String,
        duplicate_node_path: String,
    },
    /// 已展开模板的布局输入无法转换为共享契约，路径指向实际实例节点。
    #[error("invalid layout contract at {node_path}: {detail}")]
    InvalidLayoutContract { node_path: String, detail: String },
}
