use super::super::source_assertions::assert_source_order;
use super::sources::{entry_root, runtime_event_loop_policy_source};

#[test]
fn runtime_entry_maps_platform_event_loop_policy_to_winit_control_flow() {
    let event_loop_policy_root_source =
        include_str!("../../runtime_entry_app/event_loop_policy/mod.rs");
    let event_loop_policy_control_flow_source =
        include_str!("../../runtime_entry_app/event_loop_policy/control_flow.rs");
    let event_loop_policy_source = runtime_event_loop_policy_source();
    let runtime_app_source = include_str!("../../runtime_entry_app/mod.rs");
    let root = entry_root();

    for required in [
        "EventLoopPolicy::Game",
        "EventLoopPolicy::Continuous",
        "EventLoopPolicy::DesktopApp",
        "EventLoopPolicy::Mobile",
        "EventLoopPolicy::Headless",
        "ControlFlow::Poll",
        "ControlFlow::Wait",
        "event_loop.set_control_flow",
        "pub(in crate::entry::runtime_entry_app) fn apply_event_loop_policy",
    ] {
        assert!(
            event_loop_policy_source.contains(required),
            "runtime event-loop policy helper should preserve `{required}`"
        );
    }
    assert!(
        runtime_app_source.contains("mod event_loop_policy;"),
        "runtime entry app should keep event-loop policy mapping in a child module"
    );
    assert!(
        event_loop_policy_root_source.contains("mod control_flow;"),
        "runtime event-loop policy root should stay structural and delegate control-flow behavior"
    );
    assert!(
        !root.join("runtime_entry_app/event_loop_policy.rs").exists(),
        "runtime event-loop policy should stay folder-backed instead of returning to an umbrella event_loop_policy.rs file"
    );
    assert_source_order(
        event_loop_policy_control_flow_source,
        &[
            "fn apply_event_loop_policy",
            "event_loop.set_control_flow(winit_control_flow(self.event_loop_policy));",
            "fn winit_control_flow",
            "EventLoopPolicy::Game | EventLoopPolicy::Continuous",
            "ControlFlow::Poll",
            "EventLoopPolicy::DesktopApp | EventLoopPolicy::Mobile | EventLoopPolicy::Headless",
            "ControlFlow::Wait",
        ],
        "event-loop policy control-flow helper should keep the runtime profile to winit ControlFlow mapping source-visible",
    );
}
