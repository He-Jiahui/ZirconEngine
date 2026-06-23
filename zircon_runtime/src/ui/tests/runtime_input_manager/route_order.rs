use super::*;

#[test]
fn input_manager_route_order_matches_slate_style_authority_order() {
    use crate::ui::dispatch::UiInputRouteStage::*;

    assert_eq!(
        UI_INPUT_ROUTE_ORDER,
        [
            PointerCapture,
            PopupStack,
            PreviewTunnel,
            DirectTarget,
            BubblePath,
            FocusPath,
            DefaultAction,
        ]
    );
}

#[test]
fn input_manager_route_policy_stage_names_follow_authority_order() {
    use crate::ui::dispatch::{
        route_policy_uses_stage, route_stage_name, route_stage_names_for_policy, UiInputRouteStage,
    };

    assert_eq!(
        route_stage_names_for_policy(UiInputRoutePolicy::Bubble),
        vec![
            "popup_stack",
            "preview_tunnel",
            "direct_target",
            "bubble_path"
        ]
    );
    assert_eq!(
        route_stage_names_for_policy(UiInputRoutePolicy::FocusPath),
        vec!["popup_stack", "focus_path"]
    );
    assert_eq!(
        route_stage_names_for_policy(UiInputRoutePolicy::PointerCapture),
        vec!["pointer_capture"]
    );
    assert!(route_policy_uses_stage(
        UiInputRoutePolicy::Bubble,
        UiInputRouteStage::DirectTarget
    ));
    assert!(!route_policy_uses_stage(
        UiInputRoutePolicy::Bubble,
        UiInputRouteStage::FocusPath
    ));
    assert_eq!(
        route_stage_name(UiInputRouteStage::DefaultAction),
        "default_action"
    );
}
