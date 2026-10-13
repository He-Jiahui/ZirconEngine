//! Runtime UI 组件事件生成的动作宿主请求及其有限诊断格式。
//!
//! 动态会话只排队已送达且带有模板动作的组件事件；宿主路由检查视口后，目前将未接入的动作
//! 作为抽样诊断记录。secure text 通过 opaque 引用另行处理，不由这里解析。

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::handles::ZrRuntimeViewportHandle;
use crate::ui::component::UiSecureTextValueRef;
use crate::ui::dispatch::UiTemplateActionInvocation;
use crate::ui::event_ui::{UiNodeId, UiTreeId};

/// A secure-content-free Runtime UI action delivery for an application host.
///
/// Secure text is represented only by an opaque reference. Resolving that reference requires a
/// separate trusted-session contract; this request never carries the underlying text.
///
/// # 使用时机与生命周期
/// `RuntimeUiActionRequestQueue::record_result` 从已送达的组件事件生成请求，并在批次 drain
/// 时把它交给 Runtime ABI。对于带 secure ref 的 Runtime 生产路径，该队列校验引用的 tree/node
/// 与目标一致，且 invocation payload 的值全为 `UiValue::Null`；队列拒绝请求时会将引用交回
/// Runtime 撤销。公开构造器本身不执行这些检查。
///
/// # 参数
/// 路由字段保留视口、surface、输入序号、动作序号、tree 与节点身份；`invocation` 保存 action/route
/// 目标及普通 payload；`secure_value` 可选地携带不透明引用。
///
/// # 约束
/// 直接构造或反序列化时，调用者仍须遵守 secure ref 与空值 payload 的生产契约。Serde 只省略
/// `None` 的 `secure_value` 字段，不负责从普通 payload 中识别或清除秘密。自定义 `Debug` 保留
/// 路由身份和 invocation target ID，只省略 invocation payload 与引用正文；它不是序列化脱敏器。
///
/// # 参考
/// `zircon_runtime::dynamic_api::session::runtime_ui::action_requests::RuntimeUiActionRequestQueue`；
/// `zircon_app::entry::runtime_entry_app::host_requests::ui_action::report_unhandled_runtime_ui_action`。
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeUiActionHostRequestV1 {
    /// 负责接收此动作的 Runtime 视口；App 在记录诊断前会与当前窗口视口比较。
    pub target_viewport: ZrRuntimeViewportHandle,
    /// 生成该请求的 Runtime UI surface 索引。
    pub target_surface: u32,
    /// 产生组件事件的 Runtime 输入序号。
    pub input_sequence: u64,
    /// 同一派发结果中该动作事件的位置；转换超出 `u32` 时 Runtime 生产者使用 `u32::MAX`。
    pub action_index: u32,
    /// 组件事件所属 UI tree 的身份。
    pub tree_id: UiTreeId,
    /// 收到组件事件的节点身份。
    pub target: UiNodeId,
    /// 要交给宿主的 action/route 调用及其普通参数；该字段不会由本载体自动执行。
    pub invocation: UiTemplateActionInvocation,
    /// 可选 secure text opaque ref；Runtime 生产队列只在 invocation 参数全为空值时携带它。
    /// 缺省时 Serde 不输出该键，反序列化缺少该键时补为 `None`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secure_value: Option<UiSecureTextValueRef>,
}

impl ZrRuntimeUiActionHostRequestV1 {
    // EXEMPT(GEN-Q7): host action request construction mirrors the fixed ABI payload fields.
    #[allow(clippy::too_many_arguments)]
    /// 按 ABI 字段组装动作请求；不会校验 route、secure ref 与 invocation payload 的一致性。
    ///
    /// # 参数
    /// 参数依次给出视口、surface、输入/动作序号、tree、目标节点、动作调用及可选 secure ref。
    ///
    /// # 返回值
    /// 返回序列化载体；Runtime 的队列生产路径另行执行 secure payload、大小和嵌套深度检查。
    pub fn new(
        target_viewport: ZrRuntimeViewportHandle,
        target_surface: u32,
        input_sequence: u64,
        action_index: u32,
        tree_id: UiTreeId,
        target: UiNodeId,
        invocation: UiTemplateActionInvocation,
        secure_value: Option<UiSecureTextValueRef>,
    ) -> Self {
        Self {
            target_viewport,
            target_surface,
            input_sequence,
            action_index,
            tree_id,
            target,
            invocation,
            secure_value,
        }
    }
}

// Debug 为日志保留可关联请求的路由字段和动作 target ID，但不展开普通 payload 或 secure ref 正文。
// target/tree/viewport 等身份仍可见，因此该格式不是对所有动态标识的隐藏承诺。
impl fmt::Debug for ZrRuntimeUiActionHostRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZrRuntimeUiActionHostRequestV1")
            .field("target_viewport", &self.target_viewport)
            .field("target_surface", &self.target_surface)
            .field("input_sequence", &self.input_sequence)
            .field("action_index", &self.action_index)
            .field("tree_id", &self.tree_id)
            .field("target", &self.target)
            .field("action_target", &self.invocation.target_id())
            .field("secure_value", &self.secure_value.is_some())
            .finish()
    }
}

#[cfg(test)]
#[path = "tests/ui_action.rs"]
mod tests;
