//! Runtime UI 剪贴板事务的宿主请求与结果载体。
//!
//! Runtime 把请求放入有界宿主请求批次；App 在目标视口执行平台读写后，通过事件把结果交还
//! Runtime UI surface。这里的构造器只组装数据，不访问系统剪贴板。

use serde::{Deserialize, Serialize};

use crate::handles::ZrRuntimeViewportHandle;
use crate::ui::dispatch::{UiClipboardRequest, UiClipboardTransferId, UiClipboardTransferOutcome};
use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 运行时送往平台宿主的剪贴板事务，携带视口和 UI surface 归属。
///
/// # 使用时机与生命周期
/// Runtime UI 在派发结果中排队后，由动态会话收集到 [`crate::runtime_api::ZrRuntimeHostRequestV1::Clipboard`]
/// 并随请求批次输出。App 完成平台操作后使用同一 surface、transfer ID 与 owner 回送结果；
/// 这些标识用于 Runtime 再次定位原 UI 输入状态。
///
/// # 参数
/// `target_viewport` 选择宿主窗口，`target_surface` 标识会话内的 UI surface，`request` 携带
/// 读/写意图、transfer ID、节点 owner、编辑修订和可选写入文本。
///
/// # 约束
/// App 会核对目标视口，并只接受与意图、请求种类及文本载荷相符的剪贴板操作；本构造器
/// 不执行这些检查。请求对象的 JSON 编解码由调用边界负责。
///
/// # 参考
/// [`ZrRuntimeClipboardResultV1`]、`zircon_app::entry::runtime_entry_app::host_requests::clipboard`。
pub struct ZrRuntimeClipboardHostRequestV1 {
    /// App 应处理的 Runtime 视口；结果事件本身由 App 当前视口发回。
    pub target_viewport: ZrRuntimeViewportHandle,
    /// Runtime UI surface 索引；App 原样放入完成结果，Runtime 用它选回该 surface。
    pub target_surface: u32,
    /// 携带 transfer ID、owner、编辑修订、读写种类与可选文本；宿主按其组合执行平台事务。
    pub request: UiClipboardRequest,
}

impl ZrRuntimeClipboardHostRequestV1 {
    /// 组装带视口和 surface 路由信息的请求，不执行剪贴板操作或校验 request 内部组合。
    ///
    /// # 参数
    /// `target_viewport` 与 `target_surface` 指定 Runtime UI 所属窗口和 surface；`request` 是待完成的事务。
    ///
    /// # 返回值
    /// 返回可放入 [`crate::runtime_api::ZrRuntimeHostRequestV1::Clipboard`] 的载体。
    pub fn new(
        target_viewport: ZrRuntimeViewportHandle,
        target_surface: u32,
        request: UiClipboardRequest,
    ) -> Self {
        Self {
            target_viewport,
            target_surface,
            request,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 宿主完成事务后回送的结果；运行时按 surface、传输 ID 和节点 owner 关联原请求。
///
/// # 使用时机与生命周期
/// App 在目标窗口线程完成读写后创建结果，并以剪贴板结果事件交回同一动态会话。Runtime
/// 校验 transfer ID 和结果字节预算，再由 surface 的输入队列匹配 transfer ID 与 owner。
///
/// # 参数
/// `target_surface`、`transfer_id` 和 `owner` 回指请求；`outcome` 携带读取文本、写入完成或失败原因。
///
/// # 约束
/// 结果构造器不验证这些关联字段。Runtime 会拒绝有界 JSON 解码失败和无效 transfer ID；ReadText 文本超预算时
/// 转为 PayloadTooLarge outcome 后继续派发，是否匹配请求由 UI 输入层判断。
///
/// # 参考
/// [`ZrRuntimeClipboardHostRequestV1`]、`zircon_runtime::dynamic_api::session::events::handle_clipboard_result`。
pub struct ZrRuntimeClipboardResultV1 {
    /// 发起事务的 Runtime UI surface 索引。
    pub target_surface: u32,
    /// 请求签发的事务 ID，用于拒绝旧结果或重复匹配。
    pub transfer_id: UiClipboardTransferId,
    /// 原请求节点 owner；Runtime 据此把结果交给相应输入状态。
    pub owner: UiNodeId,
    /// 平台操作的 typed 结果；读取文本仍受 Runtime 结果载荷上限约束。
    pub outcome: UiClipboardTransferOutcome,
}

impl ZrRuntimeClipboardResultV1 {
    /// 组装一次已完成或失败的剪贴板事务回执，不派发 Runtime 事件。
    ///
    /// # 参数
    /// `target_surface`、`transfer_id` 和 `owner` 必须与原请求关联；`outcome` 描述平台操作结果。
    ///
    /// # 返回值
    /// 返回供 `ZrRuntimeEventV1::clipboard_result` JSON 编码的 typed result。
    pub fn new(
        target_surface: u32,
        transfer_id: UiClipboardTransferId,
        owner: UiNodeId,
        outcome: UiClipboardTransferOutcome,
    ) -> Self {
        Self {
            target_surface,
            transfer_id,
            owner,
            outcome,
        }
    }
}

#[cfg(test)]
#[path = "tests/clipboard.rs"]
mod tests;
