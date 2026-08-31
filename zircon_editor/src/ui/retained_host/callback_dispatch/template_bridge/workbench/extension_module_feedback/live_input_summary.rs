use super::super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;

const SCAN_ONLY_NAMESPACE: &str = "workbench.extension.ui_asset_editor";

pub(super) fn for_command(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    action_id: &str,
) -> Option<String> {
    let namespace = command_namespace(action_id)?;
    if namespace == SCAN_ONLY_NAMESPACE {
        return None;
    }
    let field_action_prefix = format!("{namespace}.");
    let values = bridge
        .host_projection()
        .nodes
        .iter()
        .filter(|node| {
            node.routes.iter().any(|route| {
                route.action_id.starts_with(&field_action_prefix)
                    && route.action_id.ends_with(".edit")
            })
        })
        .filter_map(|node| node.value_text.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(3)
        .collect::<Vec<_>>();

    (!values.is_empty()).then(|| format!("Inputs: {}", values.join(" | ")))
}

fn command_namespace(action_id: &str) -> Option<&str> {
    let command_action = action_id.strip_suffix(".invoke")?;
    command_action
        .rsplit_once('.')
        .map(|(namespace, _)| namespace)
}

#[cfg(test)]
mod tests {
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
}
