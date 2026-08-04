use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_surface_render_feedback_commands_are_child_owners() {
    let parent = read_runtime_src("ui/surface/render/feedback.rs");
    let colors = read_runtime_src("ui/surface/render/feedback/colors.rs");
    let commands = read_runtime_src("ui/surface/render/feedback/commands.rs");
    let state = read_runtime_src("ui/surface/render/feedback/state.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "feedback parent keeps component semantics, layout, metadata parsing, and child mounts",
        &parent,
        &[
            "mod colors;",
            "mod commands;",
            "mod state;",
            "use self::colors::{",
            "use self::commands::{icon_command, quad_command, text_command};",
            "pub(super) fn feedback_suppresses_owner_text(",
            "pub(super) fn feedback_suppresses_owner_image(",
            "pub(super) fn feedback_render_commands(",
            "fn alert_commands(",
            "fn tooltip_commands(",
            "fn toast_commands(",
            "fn first_string(",
            "pub(super) fn color_attribute<",
        ],
    );
    for moved_owner in [
        "fn quad_command(",
        "fn text_command(",
        "fn icon_command(",
        "fn alert_surface_color",
        "fn tooltip_surface_color",
        "fn toast_surface_color",
        "const ALERT_INFO_SURFACE",
        "const TOAST_SURFACE",
        "UiRenderCommandKind",
        "UiVisualAssetRef",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/surface/render/feedback.rs should delegate feedback render helper `{moved_owner}` to child owners"
        );
    }

    assert_contains_all(
        "feedback color child owns visual-state color constants and resolution",
        &colors,
        &[
            "const TOOLTIP_SURFACE",
            "const ALERT_INFO_SURFACE",
            "const TOAST_SURFACE",
            "const DISABLED_SURFACE",
            "pub(super) enum AlertTone",
            "pub(super) fn alert_surface_color<",
            "pub(super) fn tooltip_surface_color<",
            "pub(super) fn toast_surface_color<",
            "fn alert_tone_surface(",
            "color_attribute(",
            "FeedbackRenderState",
        ],
    );
    assert_contains_all(
        "feedback command child owns primitive render-command DTO construction",
        &commands,
        &[
            "pub(super) fn quad_command(",
            "pub(super) fn text_command(",
            "pub(super) fn icon_command(",
            "UiRenderCommandKind::Quad",
            "UiRenderCommandKind::Text",
            "UiRenderCommandKind::Image",
            "UiResolvedStyle",
            "UiVisualAssetRef::Icon",
            "FeedbackRenderState",
        ],
    );
    assert_contains_all(
        "feedback state child continues to own painter family state resolution",
        &state,
        &[
            "pub(super) enum FeedbackKind",
            "pub(super) struct FeedbackRenderState",
            "pub(super) fn resolve(",
            "UiPainterFamily::Alert",
            "UiPainterFamily::Tooltip",
            "UiPainterFamily::Toast",
        ],
    );

    for (path, source) in [
        ("ui/surface/render/feedback.rs", parent.as_str()),
        ("ui/surface/render/feedback/colors.rs", colors.as_str()),
        ("ui/surface/render/feedback/commands.rs", commands.as_str()),
        ("ui/surface/render/feedback/state.rs", state.as_str()),
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
                "Runtime 15 M4 UI surface render feedback command/color owner split",
                "runtime_15_ui_surface_render_feedback_command_color_owner_split_static_passed_cargo_deferred",
                "ui/surface/render/feedback.rs",
                "ui/surface/render/feedback/colors.rs",
                "ui/surface/render/feedback/commands.rs",
                "runtime_15_ui_surface_render_feedback_commands_are_child_owners",
            ],
        );
    }
}
