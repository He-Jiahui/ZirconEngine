use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_surface_event_routing_is_child_owner() {
    let parent = read_runtime_src("ui/surface/surface.rs");
    let event_routing = read_runtime_src("ui/surface/surface/event_routing.rs");
    let pointer_component_events =
        read_runtime_src("ui/surface/surface/pointer_component_events.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "UI surface parent keeps surface state, frame snapshots, and property mutation entry points",
        &parent,
        &[
            "mod event_routing;",
            "mod pointer_component_events;",
            "pub struct UiSurface",
            "pub fn surface_frame(&self) -> UiSurfaceFrame",
            "pub fn mutate_property(",
            "fn focus_reconcile_reason(",
        ],
    );
    for moved_owner in [
        "pub fn capture_pointer(",
        "fn dispatch_pointer_event_with_query_and_modifiers(",
        "fn pointer_component_events(",
        "fn route_pointer_event_with_details(",
        "fn activation_phase(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/surface/surface.rs should delegate event-routing owner `{moved_owner}` to child modules"
        );
    }

    assert_contains_all(
        "event routing child owns input dispatch, pointer routing, and navigation dispatch",
        &event_routing,
        &[
            "pub fn capture_pointer(",
            "pub fn dispatch_input_event(",
            "fn dispatch_pointer_event_with_query_and_modifiers(",
            "fn route_pointer_event_with_details(",
            "pub fn route_navigation_event(",
            "pub fn dispatch_navigation_event(",
            "fn diff_nodes(",
            "fn activation_phase(",
        ],
    );
    assert_contains_all(
        "pointer component events child owns route-derived component state and event reports",
        &pointer_component_events,
        &[
            "pub(super) fn apply_pointer_component_state(",
            "pub(super) fn apply_pointer_transient_state_dirty(",
            "pub(crate) fn mark_component_state_render_dirty(",
            "pub(super) fn pointer_component_events(",
            "pub(super) fn push_focus_component_events(",
            "pub(super) fn push_pointer_component_events(",
            "UiPointerComponentEventReason",
            "UiDirtyFlags",
        ],
    );

    for (path, source) in [
        ("ui/surface/surface.rs", parent.as_str()),
        (
            "ui/surface/surface/event_routing.rs",
            event_routing.as_str(),
        ),
        (
            "ui/surface/surface/pointer_component_events.rs",
            pointer_component_events.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 UI surface event-routing owner split",
                "runtime_15_ui_surface_event_routing_owner_split_static_passed_cargo_deferred",
                "ui/surface/surface.rs",
                "ui/surface/surface/event_routing.rs",
                "ui/surface/surface/pointer_component_events.rs",
                "runtime_15_ui_surface_event_routing_is_child_owner",
            ],
        );
    }
}
