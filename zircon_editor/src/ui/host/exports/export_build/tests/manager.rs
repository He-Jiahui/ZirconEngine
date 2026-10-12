use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
fn exported_native_probe_uses_target_mode_specific_loader() {
    let root = temp_export_root("editor-export-native-target-mode-probe");
    let package_root = root.join("plugins/split_tool");
    fs::create_dir_all(&package_root).unwrap();
    fs::write(
        package_root.join("plugin.toml"),
        split_native_plugin_manifest(),
    )
    .unwrap();
    fs::write(
        root.join("plugins/native_plugins.toml"),
        r#"
[[plugins]]
id = "split_tool"
path = "plugins/split_tool"
manifest = "plugins/split_tool/plugin.toml"
"#,
    )
    .unwrap();

    let runtime_report =
        exported_native_load_report_for_profile(&root, RuntimeTargetMode::ClientRuntime);
    assert!(runtime_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));
    assert!(!runtime_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));

    let editor_report =
        exported_native_load_report_for_profile(&root, RuntimeTargetMode::EditorHost);
    assert!(editor_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));
    assert!(!editor_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));

    let _ = fs::remove_dir_all(root);
}

fn split_native_plugin_manifest() -> &'static str {
    r#"
id = "split_tool"
version = "0.1.0"
display_name = "Split Tool"

[[modules]]
name = "split_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_split_tool_runtime"

[[modules]]
name = "split_tool.editor"
kind = "editor"
crate_name = "zircon_plugin_split_tool_editor"
"#
}

fn temp_export_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
}

fn platform_library_file_name(crate_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{crate_name}.dll")
    }
    #[cfg(target_os = "macos")]
    {
        format!("lib{crate_name}.dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{crate_name}.so")
    }
}
