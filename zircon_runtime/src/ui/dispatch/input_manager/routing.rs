//! 把输入路由策略投影为有序阶段名称，供诊断与权限说明使用。
//! 本模块不执行命中、回调或默认动作；运行时顺序由专用入口实现。

use zircon_runtime_interface::ui::dispatch::UiInputRoutePolicy;

#[cfg(test)]
#[path = "routing/tests/route_stage_names.rs"]
mod route_stage_names_tests;

/// 路由报告中的逻辑阶段；枚举值描述职责，不持有树节点或回调。
/// # 使用与约束
/// 策略可关联多个阶段，名称投影按固定表排序；不能据此假定每条输入都会执行全部阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputRouteStage {
    /// 由捕获所有者接管指针路由。
    PointerCapture,
    /// 先考虑弹出层对路由的影响。
    PopupStack,
    /// 从根向目标的预览阶段；实际执行由专属路由入口决定。
    PreviewTunnel,
    /// 向当前目标投递。
    DirectTarget,
    /// 沿祖先冒泡。
    BubblePath,
    /// 沿焦点相关路径路由。
    FocusPath,
    /// 执行事件族定义的默认处理。
    DefaultAction,
}

/// 固定报告的阶段排序；本表不驱动管理器调度循环。
/// Slate-style authority order used by the manager. Concrete event dispatchers
/// still own their leaf behavior; this list fixes the cross-cutting route order.
pub const UI_INPUT_ROUTE_ORDER: [UiInputRouteStage; 7] = [
    UiInputRouteStage::PointerCapture,
    UiInputRouteStage::PopupStack,
    UiInputRouteStage::PreviewTunnel,
    UiInputRouteStage::DirectTarget,
    UiInputRouteStage::BubblePath,
    UiInputRouteStage::FocusPath,
    UiInputRouteStage::DefaultAction,
];

/// **参数与返回**：输入一个策略，返回拥有容器的静态名称序列；未路由策略得到空序列。
/// 给诊断与路由权限报告投影策略涉及的阶段；不执行阶段、命中测试或默认动作。
pub fn route_stage_names_for_policy(policy: UiInputRoutePolicy) -> Vec<&'static str> {
    let mut names = Vec::with_capacity(4);
    for stage in UI_INPUT_ROUTE_ORDER {
        if route_policy_uses_stage(policy, stage) {
            names.push(route_stage_name(stage));
        }
    }
    names
}

/// 查询策略报告是否包含某个逻辑阶段；无状态、无路由副作用。
/// 返回值只表达诊断归属，实际事件是否走到该阶段由分发过程决定。
pub const fn route_policy_uses_stage(policy: UiInputRoutePolicy, stage: UiInputRouteStage) -> bool {
    match policy {
        UiInputRoutePolicy::Unrouted => false,
        UiInputRoutePolicy::PreviewTunnel => matches!(
            stage,
            UiInputRouteStage::PopupStack | UiInputRouteStage::PreviewTunnel
        ),
        UiInputRoutePolicy::Bubble => matches!(
            stage,
            UiInputRouteStage::PopupStack
                | UiInputRouteStage::PreviewTunnel
                | UiInputRouteStage::DirectTarget
                | UiInputRouteStage::BubblePath
        ),
        UiInputRoutePolicy::Direct => matches!(stage, UiInputRouteStage::DirectTarget),
        UiInputRoutePolicy::FocusPath => matches!(
            stage,
            UiInputRouteStage::PopupStack | UiInputRouteStage::FocusPath
        ),
        UiInputRoutePolicy::PointerCapture => {
            matches!(stage, UiInputRouteStage::PointerCapture)
        }
        UiInputRoutePolicy::DefaultAction => matches!(
            stage,
            UiInputRouteStage::PopupStack | UiInputRouteStage::DefaultAction
        ),
    }
}

/// 返回阶段的稳定诊断标签；字符串为静态借用，无需由调用方释放。
pub const fn route_stage_name(stage: UiInputRouteStage) -> &'static str {
    match stage {
        UiInputRouteStage::PointerCapture => "pointer_capture",
        UiInputRouteStage::PopupStack => "popup_stack",
        UiInputRouteStage::PreviewTunnel => "preview_tunnel",
        UiInputRouteStage::DirectTarget => "direct_target",
        UiInputRouteStage::BubblePath => "bubble_path",
        UiInputRouteStage::FocusPath => "focus_path",
        UiInputRouteStage::DefaultAction => "default_action",
    }
}
