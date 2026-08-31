use std::fmt::Write as _;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ExportProfile;

use super::ExportLinkedRuntimeCrate;

pub(super) fn cargo_manifest_template(
    profile: &ExportProfile,
    linked_runtime_crates: &[ExportLinkedRuntimeCrate],
) -> String {
    let package_name = export_package_name(&profile.output_name);
    let target_feature = target_feature(profile.target_mode);
    let mut contents = format!(
        "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nzircon_app = {{ path = \"../../zircon_app\", default-features = false, features = [\"{target_feature}\"] }}\nzircon_runtime = {{ path = \"../../zircon_runtime\", default-features = false }}\n"
    );
    for linked_crate in linked_runtime_crates {
        writeln!(
            contents,
            "{} = {{ path = \"../../zircon_plugins/{}\" }}",
            linked_crate.crate_name, linked_crate.path
        )
        .expect("writing Cargo manifest to String cannot fail");
    }
    match profile.target_platform.policy().host_kind {
        crate::core::framework::project::ExportPlatformHostKind::Desktop
        | crate::core::framework::project::ExportPlatformHostKind::Headless => {}
        crate::core::framework::project::ExportPlatformHostKind::MobileApp => {
            contents.push_str("\n[lib]\ncrate-type = [\"cdylib\", \"staticlib\"]\n");
        }
        crate::core::framework::project::ExportPlatformHostKind::Browser => {
            contents.push_str("\n[lib]\ncrate-type = [\"cdylib\"]\n");
        }
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

fn export_package_name(output_name: &str) -> String {
    const PREFIX: &str = "zircon_export_";

    let mut package_name = String::with_capacity(PREFIX.len().saturating_add(output_name.len()));
    package_name.push_str(PREFIX);
    for character in output_name.chars() {
        package_name.push(match character {
            'a'..='z' | '0'..='9' | '_' | '-' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            _ => '_',
        });
    }
    package_name
}

pub(super) fn plugin_path_for_runtime_crate(crate_name: &str) -> String {
    crate_name
        .strip_prefix("zircon_plugin_")
        .and_then(|value| value.strip_suffix("_runtime"))
        .unwrap_or(crate_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{cargo_manifest_template, ExportLinkedRuntimeCrate, ExportProfile};

    #[test]
    fn streaming_cargo_manifest_preserves_contract() {
        let mut profile = ExportProfile::default();
        profile.output_name = "Demo Game++".to_string();
        let linked_crate = ExportLinkedRuntimeCrate::runtime_plugin(
            "zircon_plugin_test_runtime".to_string(),
            "test".to_string(),
        );

        assert_eq!(
            cargo_manifest_template(&profile, &[linked_crate]),
            concat!(
                "[package]\n",
                "name = \"zircon_export_demo_game__\"\n",
                "version = \"0.1.0\"\n",
                "edition = \"2021\"\n",
                "\n[dependencies]\n",
                "zircon_app = { path = \"../../zircon_app\", default-features = false, features = [\"target-client\"] }\n",
                "zircon_runtime = { path = \"../../zircon_runtime\", default-features = false }\n",
                "zircon_plugin_test_runtime = { path = \"../../zircon_plugins/test\" }\n",
            )
        );
    }
}
