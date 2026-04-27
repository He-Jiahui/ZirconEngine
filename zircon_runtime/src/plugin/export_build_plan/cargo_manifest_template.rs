use crate::{ExportProfile, RuntimeTargetMode};

pub(super) fn cargo_manifest_template(
    profile: &ExportProfile,
    linked_runtime_crates: &[String],
) -> String {
    let package_name = sanitize_package_name(&format!("zircon_export_{}", profile.output_name));
    let target_feature = target_feature(profile.target_mode);
    let mut contents = format!(
        "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nzircon_app = {{ path = \"../../zircon_app\", default-features = false, features = [\"{target_feature}\"] }}\nzircon_runtime = {{ path = \"../../zircon_runtime\", default-features = false }}\n"
    );
    for crate_name in linked_runtime_crates {
        contents.push_str(&format!(
            "{crate_name} = {{ path = \"../../zircon_plugins/{}/runtime\" }}\n",
            plugin_path_for_runtime_crate(crate_name)
        ));
    }
    contents
}

fn target_feature(target_mode: RuntimeTargetMode) -> &'static str {
    match target_mode {
        RuntimeTargetMode::ClientRuntime => "target-client",
        RuntimeTargetMode::ServerRuntime => "target-server",
        RuntimeTargetMode::EditorHost => "target-editor-host",
    }
}

fn sanitize_package_name(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' | '_' | '-' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            _ => '_',
        })
        .collect()
}

fn plugin_path_for_runtime_crate(crate_name: &str) -> String {
    crate_name
        .strip_prefix("zircon_plugin_")
        .and_then(|value| value.strip_suffix("_runtime"))
        .unwrap_or(crate_name)
        .to_string()
}
