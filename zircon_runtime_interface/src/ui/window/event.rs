use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{
    UiDispatchEffect, UiTransientDismissalReason, UiTransientDismissalTarget, UiWindowId,
};
use crate::ui::layout::UiPoint;

use super::{UiWindowEventImpact, UiWindowEventMetadata, UiWindowMetrics, UiWindowPixelPosition};

/// 与输入事件分开的窗口生命周期和度量事件；窗口泵据此更新布局、输入状态与重绘需求。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWindowEvent {
    /// 由事件源提供的窗口身份、时间序号和合成标记。
    pub metadata: UiWindowEventMetadata,
    /// 生命周期、窗口度量或重绘通知；构造事件不会执行相应宿主操作。
    pub kind: UiWindowEventKind,
}

impl UiWindowEvent {
    /// 接收拥有的元数据和事件种类，返回待窗口泵处理的记录，不触发分发。
    pub const fn new(metadata: UiWindowEventMetadata, kind: UiWindowEventKind) -> Self {
        Self { metadata, kind }
    }

    /// 记录窗口焦点状态；失焦时可由瞬态效果投影产生关闭命令。
    pub const fn window_focused(metadata: UiWindowEventMetadata, focused: bool) -> Self {
        Self::new(metadata, UiWindowEventKind::Focused { focused })
    }

    /// 将两种激活原因都折为获焦，将停用折为失焦；不保留激活原因。
    pub const fn window_activation_changed(
        metadata: UiWindowEventMetadata,
        activation: UiWindowActivation,
    ) -> Self {
        Self::window_focused(metadata, activation.is_active())
    }

    /// 记录应用级激活状态，与单个窗口的焦点事件分开。
    pub const fn application_activation_changed(
        metadata: UiWindowEventMetadata,
        is_active: bool,
    ) -> Self {
        Self::new(
            metadata,
            UiWindowEventKind::ApplicationActivation { is_active },
        )
    }

    /// 携带完整度量快照的尺寸通知，不校验逻辑尺寸与物理尺寸的比例。
    pub const fn size_changed(metadata: UiWindowEventMetadata, metrics: UiWindowMetrics) -> Self {
        Self::new(metadata, UiWindowEventKind::Resized { metrics })
    }

    /// 将系统绘制请求表示为 Paint 重绘原因。
    pub const fn os_paint(metadata: UiWindowEventMetadata) -> Self {
        Self::request_redraw(metadata, UiWindowRedrawReason::Paint)
    }

    /// 调整窗口大小期间只请求 Paint 重绘；没有尺寸参数，不生成尺寸变更。
    pub const fn resizing_window(metadata: UiWindowEventMetadata) -> Self {
        Self::request_redraw(metadata, UiWindowRedrawReason::Paint)
    }

    /// 记录已报告的窗口操作，不调用最大化、还原或菜单操作。
    pub const fn window_action(metadata: UiWindowEventMetadata, action: UiWindowAction) -> Self {
        Self::new(metadata, UiWindowEventKind::WindowAction { action })
    }

    /// 记录整数物理像素位置，不执行窗口移动。
    pub const fn moved_window(
        metadata: UiWindowEventMetadata,
        position: UiWindowPixelPosition,
    ) -> Self {
        Self::new(metadata, UiWindowEventKind::Moved { position })
    }

    /// 只生成关闭请求；关闭完成和销毁须由各自事件报告。
    pub const fn window_close(metadata: UiWindowEventMetadata) -> Self {
        Self::new(metadata, UiWindowEventKind::CloseRequested)
    }

    /// 保存重绘原因，实际调度与绘制由消费者决定。
    pub const fn request_redraw(
        metadata: UiWindowEventMetadata,
        reason: UiWindowRedrawReason,
    ) -> Self {
        Self::new(metadata, UiWindowEventKind::RequestRedraw { reason })
    }

    /// 借用非空窗口身份；空字符串返回 `None`，借用不超过本事件。
    pub fn window_id(&self) -> Option<&UiWindowId> {
        (!self.metadata.window_id.0.is_empty()).then_some(&self.metadata.window_id)
    }

    /// 返回种类对应的影响提示；提示的实际应用见宿主窗口泵。
    pub const fn impact(&self) -> UiWindowEventImpact {
        self.kind.impact()
    }

    /// 仅识别显式重绘种类，不把其他带重绘影响的事件算作重绘请求。
    pub const fn is_redraw_request(&self) -> bool {
        matches!(self.kind, UiWindowEventKind::RequestRedraw { .. })
    }

    /// 仅在失焦、应用停用或点击非客户区时发出瞬态 UI 关闭命令，供窗口泵应用。
    /// 返回独立的命令值，不在此修改弹窗、提示框或事件本身。
    pub fn transient_dismissal_effect(&self) -> Option<UiDispatchEffect> {
        match &self.kind {
            UiWindowEventKind::Focused { focused: false } => {
                Some(UiDispatchEffect::DismissTransientUi {
                    target: UiTransientDismissalTarget::All,
                    reason: UiTransientDismissalReason::FocusLost,
                })
            }
            UiWindowEventKind::ApplicationActivation { is_active: false } => {
                Some(UiDispatchEffect::DismissTransientUi {
                    target: UiTransientDismissalTarget::All,
                    reason: UiTransientDismissalReason::ApplicationDeactivated,
                })
            }
            UiWindowEventKind::WindowAction {
                action: UiWindowAction::ClickedNonClientArea,
            } => Some(UiDispatchEffect::DismissTransientUi {
                target: UiTransientDismissalTarget::PopupStack,
                reason: UiTransientDismissalReason::WindowAction,
            }),
            _ => None,
        }
    }
}

/// 可序列化的窗口通知种类；度量数值由来源提供，本枚举不执行状态迁移或验证。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiWindowEventKind {
    /// 创建通知；缺省度量由 `UiWindowMetrics::default` 补入。
    Created {
        #[serde(default)]
        metrics: UiWindowMetrics,
    },
    /// 请求宿主关闭窗口，尚不代表已经关闭。
    CloseRequested,
    /// 窗口关闭通知。
    Closed,
    /// 窗口资源销毁通知。
    Destroyed,
    /// 光标位置和可选位移；规范化移动输入时仅保留位置。
    CursorMoved {
        position: UiPoint,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delta: Option<UiPoint>,
    },
    /// 光标进入窗口，尚无位置数据。
    CursorEntered,
    /// 光标离开窗口，供消费者清除悬停与取消指针状态。
    CursorLeft,
    Focused {
        /// `focused` 表示窗口是否获焦。
        focused: bool,
    },
    ApplicationActivation {
        /// `is_active` 表示应用是否激活，不等同于单窗口焦点。
        is_active: bool,
    },
    Occluded {
        /// `occluded` 表示遮挡状态；影响提示本身不申请重绘。
        occluded: bool,
    },
    Resized {
        /// `metrics` 是来源提交的逻辑/物理尺寸与缩放快照。
        metrics: UiWindowMetrics,
    },
    ScaleFactorChanged {
        /// 宿主缩放因子原值，不在此校验。
        scale_factor: f64,
    },
    BackendScaleFactorChanged {
        /// 后端报告的缩放因子原值；与宿主缩放种类分别保留。
        scale_factor: f64,
    },
    Moved {
        /// `position` 为整数物理像素位置。
        position: UiWindowPixelPosition,
    },
    WindowAction {
        /// `action` 记录窗口操作原因；非客户区点击可关闭弹窗栈。
        action: UiWindowAction,
    },
    RequestRedraw {
        /// `reason` 说明重绘来源，不规定优先级或合并策略。
        reason: UiWindowRedrawReason,
    },
}

impl UiWindowEventKind {
    /// 按当前种类生成提示位；不会修改布局、焦点或宿主关闭状态。
    pub const fn impact(&self) -> UiWindowEventImpact {
        match self {
            Self::Created { .. } | Self::Resized { .. } => {
                UiWindowEventImpact::layout_metrics().with_redraw()
            }
            Self::ScaleFactorChanged { .. } | Self::BackendScaleFactorChanged { .. } => {
                UiWindowEventImpact::layout_metrics()
            }
            Self::CursorMoved { .. }
            | Self::CursorEntered
            | Self::Focused { .. }
            | Self::ApplicationActivation { .. } => UiWindowEventImpact::input_state(),
            Self::CursorLeft => UiWindowEventImpact::input_state()
                .with_hover_clear()
                .with_redraw(),
            Self::RequestRedraw { .. } => UiWindowEventImpact::redraw(),
            Self::CloseRequested => UiWindowEventImpact::close_requested(),
            Self::Closed | Self::Destroyed => UiWindowEventImpact::input_state().with_hover_clear(),
            Self::Occluded { .. } | Self::Moved { .. } | Self::WindowAction { .. } => {
                UiWindowEventImpact::clean()
            }
        }
    }
}

/// 宿主报告的窗口操作类别；枚举值本身不会执行操作。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowAction {
    #[default]
    ClickedNonClientArea,
    Maximize,
    Restore,
    WindowMenu,
}

/// 平台激活原因；转换为焦点通知时两种激活原因合并为 `true`。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowActivation {
    #[default]
    Activate,
    ActivateByMouse,
    Deactivate,
}

impl UiWindowActivation {
    /// 两种激活返回 `true`，停用返回 `false`。
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Activate | Self::ActivateByMouse)
    }
}

/// 重绘来源标签；排队、合并和实际绘制由泵及消费者处理。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowRedrawReason {
    #[default]
    Host,
    Input,
    Animation,
    Layout,
    Paint,
    Diagnostics,
}
