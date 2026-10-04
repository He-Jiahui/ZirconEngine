//! 命令按钮资产必须声明有限宽度下的文字省略策略。
#[test]
/// 用资产声明固定长命令标签的溢出责任，实际宽度仍由运行时父布局决定。
fn workbench_button_declares_runtime_ellipsis_for_constrained_command_labels() {
    let source = include_str!(
        "../../../../assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui"
    );

    assert!(
        source.contains("text_overflow = \"ellipsis\""),
        "workbench buttons must keep adaptive command strips from expanding for long labels"
    );
}
