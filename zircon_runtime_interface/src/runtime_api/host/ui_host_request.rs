//! Runtime UI effect 转换成的平台宿主请求载体。
//!
//! UI 派发层把已应用的 effect 转为 request；本模块只做 typed projection 与 Serde 编码，不执行
//! OS 操作。IME 和 clipboard 继续走各自的事务请求。

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::handles::ZrRuntimeViewportHandle;
use crate::ui::{
    dispatch::{
        UiDispatchHostRequestKind, UiPointerLockPolicy, UiPopupEffectKind, UiTooltipEffectKind,
        UiTransientDismissalReason, UiTransientDismissalTarget,
    },
    event_ui::{UiNodeId, UiTreeId},
    layout::UiPoint,
    text::UiRichLinkTarget,
};

/// One platform-facing operation carried by a handled Runtime UI reply.
///
/// Input-method and clipboard requests keep their dedicated transaction contracts and never enter
/// this generic channel. Dynamic identifiers remain serializable but are omitted from `Debug`.
///
/// 此处省略的是 `kind` 中的 popup/tooltip ID 与链接目标；视口、surface、tree、序号等路由字段仍会
/// 出现在 `Debug`。`kind.as_str()` 仅是诊断名称，并非 Serde 的 wire tag。
///
/// # 使用时机与生命周期
/// Runtime 从 handled UI reply 投影通用宿主 effect 后，将请求放进批次。App 当前先核对视口，随后
/// 对尚未接入的通用 effect 计数并抽样记日志；本类型不保证消费者已实现每一种 kind。
///
/// # 参数
/// 前六个字段关联原 UI 输入与 effect；`kind` 携带被投影的具体操作数据。
///
/// # 约束
/// `from_dispatch_request` 只复制/克隆 request 数据，不验证目标节点归属，也不触发平台副作用。
/// `Debug` 仍可见路由身份；Serde 会正常序列化 variant payload。
///
/// # 参考
/// `zircon_runtime::ui::surface::input::effect::host_request::host_request_for_effect`；
/// `zircon_app::entry::runtime_entry_app::host_requests::ui_host_request::report_unhandled_runtime_ui_host_request`。
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeUiHostRequestV1 {
    /// 目标平台窗口对应的 Runtime viewport。
    pub target_viewport: ZrRuntimeViewportHandle,
    /// 生成请求的 Runtime UI surface 索引。
    pub target_surface: u32,
    /// 产生原 UI 派发回复的输入序号。
    pub input_sequence: u64,
    /// 原派发 host request 在回复中的序号。
    pub request_index: u32,
    /// 原请求所属 UI tree。
    pub tree_id: UiTreeId,
    /// 原回复中该 effect 的序号。
    pub effect_index: u32,
    /// 从 Runtime UI dispatch request 搬运来的通用操作；IME/clipboard 不会由投影构造器产出。
    pub kind: ZrRuntimeUiHostRequestKindV1,
}

impl ZrRuntimeUiHostRequestV1 {
    // EXEMPT(GEN-Q7): host request projection mirrors the fixed ABI payload fields.
    #[allow(clippy::too_many_arguments)]
    /// 将通用 dispatch request 投影为带回复身份的宿主载体。
    ///
    /// # 参数
    /// `target_viewport`、`target_surface`、两个序号与 `tree_id` 复制回复上下文；`request` 提供 kind。
    ///
    /// # 返回值
    /// 对通用 request 返回 `Some`；输入法与剪贴板返回 `None`，由专用 request ABI 承载。
    ///
    /// # 约束
    /// 方法只复制标量并克隆动态字符串/链接目标，不验证节点是否属于该 tree，也不执行平台操作。
    pub fn from_dispatch_request(
        target_viewport: ZrRuntimeViewportHandle,
        target_surface: u32,
        input_sequence: u64,
        request_index: u32,
        tree_id: UiTreeId,
        effect_index: u32,
        request: &UiDispatchHostRequestKind,
    ) -> Option<Self> {
        Some(Self {
            target_viewport,
            target_surface,
            input_sequence,
            request_index,
            tree_id,
            effect_index,
            kind: ZrRuntimeUiHostRequestKindV1::from_dispatch_request(request)?,
        })
    }
}

// 只格式化静态 kind 名和路由身份；嵌套变体中的 popup_id、tooltip_id、link_target 不进入日志。
impl fmt::Debug for ZrRuntimeUiHostRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZrRuntimeUiHostRequestV1")
            .field("target_viewport", &self.target_viewport)
            .field("target_surface", &self.target_surface)
            .field("input_sequence", &self.input_sequence)
            .field("request_index", &self.request_index)
            .field("tree_id", &self.tree_id)
            .field("effect_index", &self.effect_index)
            .field("kind", &self.kind.as_str())
            .finish()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
/// 通用 UI 平台副作用的数据形状。字段与来源 dispatch kind 一一对应；App 当前仅记录有限诊断。
///
/// # 使用时机与生命周期
/// 由 [`ZrRuntimeUiHostRequestV1::from_dispatch_request`] 从一次 UI 派发请求投影而来，随对应 Runtime
/// 宿主批次传出。容器不会执行其中的行为，也不保证目标节点仍属于当前 tree。
///
/// # 参考
/// 输入来源为 `UiDispatchHostRequestKind`；类型名及动态 payload 由 `Debug` 省略/保留规则见外层载体。
pub enum ZrRuntimeUiHostRequestKindV1 {
    /// 请求对目标节点执行指针锁定。
    PointerLock {
        /// 产生 effect 的 UI 节点身份。
        target: UiNodeId,
        /// 从 dispatch 层原样传递的锁定策略。
        policy: UiPointerLockPolicy,
    },
    /// 请求解除指针锁定；策略保留原 dispatch effect 的路由上下文。
    PointerUnlock {
        /// 与原解锁 effect 一同给出的指针锁定策略。
        policy: UiPointerLockPolicy,
    },
    /// 请求切换目标节点的高精度指针模式。
    HighPrecisionPointer {
        /// 产生 effect 的 UI 节点身份。
        target: UiNodeId,
        /// `true` 请求启用，`false` 请求关闭。
        enabled: bool,
    },
    /// 请求打开、关闭或切换一个 popup；这只是宿主意图，不代表 Runtime 已创建平台窗口。
    Popup {
        /// `Open`、`Close` 或 `Toggle` 的原始 effect 类型。
        kind: UiPopupEffectKind,
        /// Runtime UI popup 的动态标识；Serde 保留，外层自定义 Debug 不输出。
        popup_id: String,
        /// 来自 UI effect 的可选锚点提示；缺省时 JSON 不写该字段，读取缺失值时设为 `None`。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<UiPoint>,
    },
    /// 请求对 tooltip 做 arm/show/hide/cancel 操作；此载体不运行 tooltip timer。
    Tooltip {
        /// 原 UI dispatch effect 的阶段。
        kind: UiTooltipEffectKind,
        /// Runtime UI tooltip 的动态标识；Serde 保留，外层自定义 Debug 不输出。
        tooltip_id: String,
    },
    /// 按给定原因请求关闭指定范围的 transient UI。
    DismissTransientUi {
        /// 关闭范围，如全部 transient UI、popup stack 或 tooltip。
        target: UiTransientDismissalTarget,
        /// 发起关闭请求的 UI 生命周期或交互原因。
        reason: UiTransientDismissalReason,
    },
    /// 承载 Runtime 富文本命中测试生成的链接激活意图；本载体不校验请求来源。
    ActivateLink {
        /// 链接所在的 Runtime UI 节点。
        target: UiNodeId,
        /// 由 Runtime parser 批准的 engine-local destination；SerDe 字段名保持兼容 JSON `href`。
        /// App 当前记录未处理诊断，不会在此处打开或解析资源。
        #[serde(rename = "href")]
        link_target: UiRichLinkTarget,
    },
}

impl ZrRuntimeUiHostRequestKindV1 {
    /// 只投影通用宿主副作用；输入法和剪贴板返回 `None`，由各自的事务 ABI 继续处理。
    ///
    /// # 参数
    /// `request` 是 UI dispatch reply 中的一项 host request；动态 ID 与 link target 会被克隆。
    ///
    /// # 返回值
    /// 七种通用操作返回对应 variant；`InputMethod`、`Clipboard` 返回 `None`。
    ///
    /// # 约束
    /// 本转换不核对目标节点归属、不做权限/资源检查，也不调用平台 API。
    pub fn from_dispatch_request(request: &UiDispatchHostRequestKind) -> Option<Self> {
        match request {
            UiDispatchHostRequestKind::InputMethod(_) | UiDispatchHostRequestKind::Clipboard(_) => {
                None
            }
            UiDispatchHostRequestKind::PointerLock { target, policy } => Some(Self::PointerLock {
                target: *target,
                policy: *policy,
            }),
            UiDispatchHostRequestKind::PointerUnlock { policy } => {
                Some(Self::PointerUnlock { policy: *policy })
            }
            UiDispatchHostRequestKind::HighPrecisionPointer { target, enabled } => {
                Some(Self::HighPrecisionPointer {
                    target: *target,
                    enabled: *enabled,
                })
            }
            UiDispatchHostRequestKind::Popup {
                kind,
                popup_id,
                anchor,
            } => Some(Self::Popup {
                kind: *kind,
                popup_id: popup_id.clone(),
                anchor: *anchor,
            }),
            UiDispatchHostRequestKind::Tooltip { kind, tooltip_id } => Some(Self::Tooltip {
                kind: *kind,
                tooltip_id: tooltip_id.clone(),
            }),
            UiDispatchHostRequestKind::DismissTransientUi { target, reason } => {
                Some(Self::DismissTransientUi {
                    target: *target,
                    reason: *reason,
                })
            }
            UiDispatchHostRequestKind::ActivateLink {
                target,
                link_target,
            } => Some(Self::ActivateLink {
                target: *target,
                link_target: link_target.clone(),
            }),
        }
    }

    /// 返回用于日志的小写蛇形标识；此名称不决定 Serde 的 wire tag。
    ///
    /// # 返回值
    /// 对七种 variant 返回对应固定的小写蛇形标识。
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PointerLock { .. } => "pointer_lock",
            Self::PointerUnlock { .. } => "pointer_unlock",
            Self::HighPrecisionPointer { .. } => "high_precision_pointer",
            Self::Popup { .. } => "popup",
            Self::Tooltip { .. } => "tooltip",
            Self::DismissTransientUi { .. } => "dismiss_transient_ui",
            Self::ActivateLink { .. } => "activate_link",
        }
    }
}

#[cfg(test)]
#[path = "tests/ui_host_request.rs"]
mod tests;
