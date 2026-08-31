use zircon_runtime_interface::ui::dispatch::UiInputRoutePolicy;

#[cfg(test)]
mod route_stage_names_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputRouteStage {
    PointerCapture,
    PopupStack,
    PreviewTunnel,
    DirectTarget,
    BubblePath,
    FocusPath,
    DefaultAction,
}

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

pub fn route_stage_names_for_policy(policy: UiInputRoutePolicy) -> Vec<&'static str> {
    let mut names = Vec::with_capacity(4);
    for stage in UI_INPUT_ROUTE_ORDER {
        if route_policy_uses_stage(policy, stage) {
            names.push(route_stage_name(stage));
        }
    }
    names
}

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
