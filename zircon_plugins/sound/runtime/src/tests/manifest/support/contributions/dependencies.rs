use super::super::values::bool_from_plugin_toml;
use super::StaticDependency;

pub(super) fn dependencies_from_plugin_toml(manifest: &str) -> Vec<StaticDependency> {
    let mut dependencies = Vec::new();
    let mut current_id = None;
    let mut current_required = None;
    let mut current_capability = None;
    let mut inside_dependency = false;

    for line in manifest.lines().map(str::trim) {
        if line == "[[dependencies]]" {
            push_dependency(
                &mut dependencies,
                &mut current_id,
                &mut current_required,
                &mut current_capability,
            );
            inside_dependency = true;
            continue;
        }
        if line.starts_with("[[") {
            push_dependency(
                &mut dependencies,
                &mut current_id,
                &mut current_required,
                &mut current_capability,
            );
            inside_dependency = false;
        }
        if !inside_dependency {
            continue;
        }
        if let Some(value) = line
            .strip_prefix("id = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_id = Some(value.to_string());
            continue;
        }
        if let Some(value) = line.strip_prefix("required = ") {
            current_required = Some(bool_from_plugin_toml(value));
            continue;
        }
        if let Some(value) = line
            .strip_prefix("capability = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_capability = Some(value.to_string());
        }
    }
    push_dependency(
        &mut dependencies,
        &mut current_id,
        &mut current_required,
        &mut current_capability,
    );
    dependencies
}

fn push_dependency(
    dependencies: &mut Vec<StaticDependency>,
    id: &mut Option<String>,
    required: &mut Option<bool>,
    capability: &mut Option<String>,
) {
    let Some(id) = id.take() else {
        return;
    };
    dependencies.push((
        id,
        required
            .take()
            .expect("sound dependency should declare required"),
        capability.take(),
    ));
}
