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
        .filter_map(|node| node.value_text.as_deref());

    input_summary(values)
}

fn input_summary<'a, I>(values: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
    I::IntoIter: Clone,
{
    let mut values = values
        .into_iter()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(3);
    let (value_count, value_length) = values
        .clone()
        .fold((0usize, 0usize), |(count, length), value| {
            (count + 1, length + value.len())
        });
    let first = values.next()?;
    let delimiter_count = value_count.saturating_sub(1);
    let mut summary = String::with_capacity(
        "Inputs: "
            .len()
            .saturating_add(value_length)
            .saturating_add(delimiter_count.saturating_mul(" | ".len())),
    );
    summary.push_str("Inputs: ");
    summary.push_str(first);
    for value in values {
        summary.push_str(" | ");
        summary.push_str(value);
    }
    Some(summary)
}

#[cfg(test)]
#[path = "live_input_summary/tests/single_buffer_tests.rs"]
mod single_buffer_tests;

fn command_namespace(action_id: &str) -> Option<&str> {
    let command_action = action_id.strip_suffix(".invoke")?;
    command_action
        .rsplit_once('.')
        .map(|(namespace, _)| namespace)
}

#[cfg(test)]
#[path = "tests/live_input_summary.rs"]
mod tests;
