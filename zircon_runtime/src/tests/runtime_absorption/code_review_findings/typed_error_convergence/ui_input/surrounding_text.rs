#[test]
fn review_f5_ui_input_surrounding_text_error_implements_std_error() {
    let interface_effect = include_str!(
        "../../../../../../../zircon_runtime_interface/src/ui/dispatch/input/effect.rs"
    );
    let interface_test = include_str!(
        "../../../../../../../zircon_runtime_interface/src/tests/ui_dispatch_error_contracts.rs"
    );
    let runtime_error = include_str!("../../../../../ui/surface/input/error.rs");
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
    let platform_input =
        include_str!("../../../../../../../docs/zircon_runtime/ui/platform_input.md");
    let interface_doc =
        include_str!("../../../../../../../docs/zircon_runtime_interface/ui/mod.md");

    assert!(
        interface_effect
            .contains("impl std::error::Error for UiInputMethodSurroundingTextError {}"),
        "interface surrounding-text validation error should implement StdError for typed source composition"
    );
    assert!(
        interface_test.contains("assert_std_error::<UiInputMethodSurroundingTextError>();"),
        "interface tests should compile-check the surrounding-text validation error source contract"
    );
    assert!(
        runtime_error.contains("InvalidInputMethodSurroundingText")
            && runtime_error.contains("#[source]")
            && runtime_error.contains("validation_error: UiInputMethodSurroundingTextError"),
        "runtime UI input error should preserve the typed surrounding-text validation error payload"
    );
}
