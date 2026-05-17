use std::collections::BTreeSet;

use crate::ui::template_runtime::EditorUiHostRuntime;

use super::support::*;

#[test]
fn material_component_lab_projects_to_renderable_host_nodes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates()
        .expect("built-in host templates should load");
    let projection = runtime
        .project_document("editor.window.material_component_lab")
        .expect("Material Component Lab document should project");
    let projected_binding_ids = projection
        .bindings
        .iter()
        .map(|binding| binding.binding_id.as_str())
        .collect::<BTreeSet<_>>();
    for binding_id in material_lab_event_ids() {
        assert!(
            projected_binding_ids.contains(binding_id.as_str()),
            "Material Lab projection should retain `{binding_id}`"
        );
    }
    let surface = runtime
        .build_shared_surface("editor.window.material_component_lab")
        .expect("Material Component Lab surface should build");
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .expect("Material Component Lab should build retained host nodes");

    assert!(
        host_model
            .nodes
            .iter()
            .any(|node| node.node_id.contains("appbar")),
        "Material Lab app bar should be present in the retained host model"
    );
    assert!(
        host_model
            .nodes
            .iter()
            .any(|node| node.node_id.contains("drawer")),
        "Material Lab drawer should be present in the retained host model"
    );
    assert!(
        host_model
            .nodes
            .iter()
            .any(|node| node.control_id.as_deref() == Some("buttons_Root")
                || node.control_id.as_deref() == Some("buttons_Sample")),
        "Material Lab prototype nodes should be present in the retained host model"
    );
}
