//! 在线资源变化的命令与回执；命令可离线组合，实际身份校验与发布权限始终归管理器提交边界。

mod batch;
mod operation;
mod receipt;

pub use batch::ResourceMutationBatch;
pub use receipt::ResourceMutationReceipt;

pub(crate) use operation::ResourceMutationOperation;
