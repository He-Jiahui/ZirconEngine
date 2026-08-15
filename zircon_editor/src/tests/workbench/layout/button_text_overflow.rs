#[test]
fn workbench_button_declares_runtime_ellipsis_for_constrained_command_labels() {
    let source = include_str!(
        "../../../../assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui"
    );

    assert!(
        source.contains("text_overflow = \"ellipsis\""),
        "workbench buttons must keep adaptive command strips from expanding for long labels"
    );
}
