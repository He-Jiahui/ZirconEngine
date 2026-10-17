//! 验证仓库内宿主窗口模板文件可加载并实例化。

use crate::ui::template_runtime::EditorUiHostRuntime;
use std::path::PathBuf;

#[test]
fn editor_repository_unreal_shell_template_file_loads_and_instantiates() {
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("editor")
        .join("unreal")
        .join("editor_shell.zui");
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    runtime
        .register_document_file("test://ui/host-window-file", template_path)
        .unwrap();

    let projection = runtime
        .project_document("test://ui/host-window-file")
        .unwrap();
    assert_eq!(projection.root.component, "UiHostWindow");
    assert_eq!(projection.root.children.len(), 1);
    assert_eq!(projection.root.children[0].component, "DocumentHost");
    assert_eq!(projection.root.children[0].children.len(), 1);
    assert_eq!(
        projection.root.children[0].children[0].component,
        "PaneSurface"
    );
}
