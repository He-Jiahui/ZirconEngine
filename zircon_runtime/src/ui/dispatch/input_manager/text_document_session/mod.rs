//! 管理输入管理器持有的有界文本源、owner 绑定和差量撤销历史。
//!
//! 管理器先按 Surface 会话身份与树节点同步 owner，再由 Surface 编辑或模型更新准备文档事务；
//! 属性与文档提交成功后更新绑定及历史。绑定同时保存文档 revision 与 Surface 源 epoch，
//! 两者服务于不同的一致性检查，不能互相替代。
//!
//! IME 可见预编辑不是已提交源；首次绑定按恢复文本还原源，已有同 epoch 绑定继续复用。
//! 切换树或 Surface 身份会关闭旧文档并清空历史；失焦清除历史由相邻生命周期模块调用。

#[path = "binding.rs"]
mod binding;
#[path = "error.rs"]
mod error;
#[path = "history.rs"]
mod history;
#[path = "limits.rs"]
mod limits;
#[path = "session.rs"]
mod session;

pub(in crate::ui) use error::UiTextDocumentSessionError;
pub(in crate::ui) use history::{UiTextHistoryCommit, UiTextHistoryDirection};
pub(in crate::ui) use session::UiTextDocumentSession;
