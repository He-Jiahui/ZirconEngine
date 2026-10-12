use super::*;

#[test]
fn command_namespace_excludes_open_row_and_field_actions() {
    assert_eq!(
        command_namespace("workbench.extension.shader_editor.compile.invoke"),
        Some("workbench.extension.shader_editor")
    );
    assert_eq!(
        command_namespace("workbench.extension.shader_editor.open"),
        None
    );
    assert_eq!(
        command_namespace("workbench.extension.shader_editor.target.edit"),
        None
    );
}
