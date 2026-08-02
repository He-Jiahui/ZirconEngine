use toml::Value;

use super::support::{collect_zui_files, editor_asset_root, load_zui_document};

const MODULES_ROOT: &str = "ui/editor/components/workbench/modules";

#[test]
fn workbench_module_buttons_inherit_the_shared_semantic_palette() {
    let module_root = editor_asset_root().join(MODULES_ROOT);
    let mut button_count = 0usize;
    let mut offenders = Vec::new();

    for path in collect_zui_files(&module_root) {
        let document = load_zui_document(&path);
        for (node_id, node) in &document.nodes {
            if node.component != "WorkbenchButton" {
                continue;
            }

            button_count += 1;
            for prop in ["background_color", "border_color"] {
                let has_raw_color = node
                    .props
                    .get(prop)
                    .and_then(Value::as_str)
                    .is_some_and(|value| value.starts_with('#'));
                if has_raw_color {
                    offenders.push(format!(
                        "{} node `{node_id}` overrides shared WorkbenchButton `{prop}`",
                        path.display()
                    ));
                }
            }
        }
    }

    assert!(
        button_count >= 140,
        "Workbench modules should retain their shared Button component coverage"
    );
    assert!(
        offenders.is_empty(),
        "Workbench buttons must inherit the shared palette instead of raw color overrides: {offenders:#?}"
    );
}

#[test]
fn workbench_module_canvases_inherit_the_shared_inset_surface() {
    let module_root = editor_asset_root().join(MODULES_ROOT);
    let mut canvas_count = 0usize;
    let mut offenders = Vec::new();

    for path in collect_zui_files(&module_root) {
        let document = load_zui_document(&path);
        for (node_id, node) in &document.nodes {
            let is_module_canvas = node.component == "VerticalGroup"
                && node
                    .classes
                    .iter()
                    .any(|class| class == "workbench-module-canvas");
            if !is_module_canvas {
                continue;
            }

            canvas_count += 1;
            for prop in [
                "background_color",
                "border_color",
                "border_width",
                "corner_radius",
            ] {
                if node.props.contains_key(prop) {
                    offenders.push(format!(
                        "{} node `{node_id}` overrides shared Workbench module canvas `{prop}`",
                        path.display()
                    ));
                }
            }
        }
    }

    assert!(
        canvas_count >= 52,
        "Workbench modules should retain their shared canvas coverage"
    );
    assert!(
        offenders.is_empty(),
        "Workbench module canvases must inherit their shared inset surface: {offenders:#?}"
    );
}
