#[test]
fn review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary() {
    let input_mod = include_str!("../../../../../ui/surface/input/mod.rs");
    let surface_mod = include_str!("../../../../../ui/surface/mod.rs");
    let error = include_str!("../../../../../ui/surface/input/error.rs");
    let effect = include_str!("../../../../../ui/surface/input/effect.rs");
    let validation = include_str!("../../../../../ui/surface/input/validation.rs");
    let state_drag_drop = include_str!("../../../../../ui/surface/input/state/drag_drop.rs");
    let effect_sources = [
        (
            "component event effect",
            include_str!("../../../../../ui/surface/input/effect/component_event.rs"),
        ),
        (
            "drag/drop effect",
            include_str!("../../../../../ui/surface/input/effect/drag_drop.rs"),
        ),
        (
            "focus/pointer effect",
            include_str!("../../../../../ui/surface/input/effect/focus_pointer.rs"),
        ),
        (
            "navigation effect",
            include_str!("../../../../../ui/surface/input/effect/navigation.rs"),
        ),
        (
            "node effect helper",
            include_str!("../../../../../ui/surface/input/effect/node.rs"),
        ),
        (
            "popup/tooltip effect",
            include_str!("../../../../../ui/surface/input/effect/popup_tooltip.rs"),
        ),
        (
            "redraw effect",
            include_str!("../../../../../ui/surface/input/effect/redraw.rs"),
        ),
        (
            "text-service effect",
            include_str!("../../../../../ui/surface/input/effect/text_services.rs"),
        ),
    ];
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = include_str!("../../../../../../../docs/zircon_runtime/ui/platform_input.md");

    for required in [
        "mod error;",
        "pub use error::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult};",
    ] {
        assert!(
            input_mod.contains(required),
            "input module should expose typed input effect error anchor `{required}`"
        );
    }
    assert!(
        surface_mod.contains(
            "pub use input::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult, UiSurfaceInputState};",
        ),
        "surface module should publicly name the typed input effect result beside UiSurfaceInputState"
    );
    for required in [
        "pub type UiSurfaceInputEffectResult<T>",
        "pub enum UiSurfaceInputEffectError",
        "InvalidInputOwner",
        "MissingNode",
        "MissingDirtyTarget",
        "UnexpectedEffect",
        "FocusRejected",
        "PointerCaptureOwnerMismatch",
        "DragSessionOwnerMismatch",
        "InvalidInputMethodSurroundingText",
        "ClipboardWriteRequestMissingText",
    ] {
        assert!(
            error.contains(required),
            "input effect error owner should contain `{required}`"
        );
    }

    assert!(
        effect.contains(") -> UiSurfaceInputEffectResult<Option<UiNodeId>>"),
        "effect dispatcher should return the typed input effect result internally"
    );
    assert_eq!(
        effect.matches("reason: error.to_string()").count(),
        1,
        "only the UiDispatchRejectedEffect.reason boundary should stringify typed input effect errors"
    );
    assert!(
        validation.contains("UiSurfaceInputEffectError::InvalidInputOwner"),
        "input owner validation should return the typed invalid-owner variant"
    );
    for required in [
        "UiSurfaceInputEffectResult<()>",
        "UiSurfaceInputEffectResult<Option<UiNodeId>>",
        "UiSurfaceInputEffectError::DragSessionInactive",
        "UiSurfaceInputEffectError::DragPointerOwnerMismatch",
        "UiSurfaceInputEffectError::DragSessionOwnerMismatch",
    ] {
        assert!(
            state_drag_drop.contains(required),
            "drag/drop state should contain typed error anchor `{required}`"
        );
    }
    for (label, source) in effect_sources.into_iter().chain([
        ("validation", validation),
        ("drag/drop state", state_drag_drop),
    ]) {
        for forbidden in [
            "Result<Option<UiNodeId>, String>",
            "Result<(), String>",
            "Err(format!(",
            "Err(\"",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep String-error transport `{forbidden}`"
            );
        }
    }
}
