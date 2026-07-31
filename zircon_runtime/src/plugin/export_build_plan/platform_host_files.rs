use crate::core::framework::project::ExportProfile;

use self::{browser::browser_host_files, mobile::mobile_host_files};
use super::ExportGeneratedFile;

mod browser;
mod mobile;

pub(super) fn platform_host_files(
    profile: &ExportProfile,
    has_native_dynamic_plugins: bool,
) -> Vec<ExportGeneratedFile> {
    let policy = profile.target_platform.policy();
    match policy.host_kind {
        crate::core::framework::project::ExportPlatformHostKind::Desktop => {
            vec![ExportGeneratedFile {
                path: "src/main.rs".to_string(),
                purpose: "generated desktop runtime entry point".to_string(),
                contents: super::main_template::main_template(profile, has_native_dynamic_plugins),
            }]
        }
        crate::core::framework::project::ExportPlatformHostKind::Headless => {
            vec![ExportGeneratedFile {
                path: "src/main.rs".to_string(),
                purpose: "generated headless runtime entry point".to_string(),
                contents: super::main_template::main_template(profile, has_native_dynamic_plugins),
            }]
        }
        crate::core::framework::project::ExportPlatformHostKind::MobileApp => {
            mobile_host_files(profile)
        }
        crate::core::framework::project::ExportPlatformHostKind::Browser => {
            browser_host_files(profile)
        }
    }
}

fn runtime_library_file(profile: &ExportProfile, host_label: &str) -> ExportGeneratedFile {
    ExportGeneratedFile {
        path: "src/lib.rs".to_string(),
        purpose: format!("generated {host_label} runtime library entry point"),
        contents: runtime_library_template(profile, host_label),
    }
}

fn runtime_library_template(profile: &ExportProfile, host_label: &str) -> String {
    let target_platform = profile.target_platform.as_str();
    let mut output = format!(
        "mod zircon_plugins;\n\n/// Starts the Zircon runtime from a generated {host_label} scaffold.\npub fn zircon_export_bootstrap() -> Result<(), Box<dyn std::error::Error>> {{\n    let _core = zircon_app::bootstrap_export_runtime(\n        zircon_plugins::export_runtime_bootstrap_config(),\n    )?;\n    Ok(())\n}}\n\n#[no_mangle]\npub extern \"C\" fn zircon_export_start() -> bool {{\n    zircon_export_bootstrap().is_ok()\n}}\n\n#[no_mangle]\npub extern \"C\" fn zircon_export_handle_lifecycle(_state: u32) -> bool {{ true }}\n\n#[no_mangle]\npub extern \"C\" fn zircon_export_handle_touch(\n    _pointer_id: u64,\n    _phase: u32,\n    _x: f32,\n    _y: f32,\n) -> bool {{ true }}\n\n#[no_mangle]\npub extern \"C\" fn zircon_export_handle_keyboard(\n    _action: u32,\n    _key_code: u32,\n    _scan_code: u32,\n    _text: *const u8,\n    _text_len: usize,\n) -> bool {{ true }}\n\n#[no_mangle]\npub extern \"C\" fn zircon_export_handle_viewport_metrics(\n    _logical_width: u32,\n    _logical_height: u32,\n    _scale: f32,\n) -> bool {{ true }}\n\n#[cfg(target_os = \"android\")]\n#[no_mangle]\npub extern \"system\" fn Java_dev_zircon_export_ZirconRuntime_start(\n    _env: *mut core::ffi::c_void,\n    _class: *mut core::ffi::c_void,\n) -> bool {{\n    zircon_export_start()\n}}\n\n#[cfg(target_os = \"android\")]\n#[no_mangle]\npub extern \"system\" fn Java_dev_zircon_export_ZirconRuntime_dispatchLifecycle(\n    _env: *mut core::ffi::c_void,\n    _class: *mut core::ffi::c_void,\n    state: i32,\n) -> bool {{\n    zircon_export_handle_lifecycle(state as u32)\n}}\n\n#[cfg(target_os = \"android\")]\n#[no_mangle]\npub extern \"system\" fn Java_dev_zircon_export_ZirconRuntime_dispatchTouch(\n    _env: *mut core::ffi::c_void,\n    _class: *mut core::ffi::c_void,\n    pointer_id: i64,\n    phase: i32,\n    x: f32,\n    y: f32,\n) -> bool {{\n    zircon_export_handle_touch(pointer_id as u64, phase as u32, x, y)\n}}\n\n#[cfg(target_os = \"android\")]\n#[no_mangle]\npub extern \"system\" fn Java_dev_zircon_export_ZirconRuntime_dispatchKeyboard(\n    _env: *mut core::ffi::c_void,\n    _class: *mut core::ffi::c_void,\n    action: i32,\n    key_code: i32,\n    scan_code: i32,\n    _text: *mut core::ffi::c_void,\n) -> bool {{\n    zircon_export_handle_keyboard(action as u32, key_code as u32, scan_code as u32, core::ptr::null(), 0)\n}}\n\n#[cfg(target_os = \"android\")]\n#[no_mangle]\npub extern \"system\" fn Java_dev_zircon_export_ZirconRuntime_dispatchViewportMetrics(\n    _env: *mut core::ffi::c_void,\n    _class: *mut core::ffi::c_void,\n    width: i32,\n    height: i32,\n    scale: f32,\n) -> bool {{\n    zircon_export_handle_viewport_metrics(width.max(0) as u32, height.max(0) as u32, scale)\n}}\n\npub const ZIRCON_EXPORT_TARGET_PLATFORM: &str = \"{target_platform}\";\n"
    );
    output.push_str(
        "\n#[no_mangle]\npub extern \"C\" fn zircon_export_fetch_resource(\n    _uri: *const core::ffi::c_char,\n    _flags: u32,\n) -> bool { true }\n",
    );
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_library_template_does_not_rescan_the_completed_source() {
        let source = include_str!("platform_host_files.rs");
        let replacement_pass = ["    .rep", "lace("].concat();
        let template_body = source
            .split("fn runtime_library_template")
            .nth(1)
            .and_then(|body| body.split("fn native_library_stem").next())
            .expect("runtime library template body should remain available");

        assert!(!template_body.contains(&replacement_pass));
    }
}

fn native_library_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' | '_' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' => '_',
            _ => '_',
        })
        .collect()
}

fn bundle_identifier_suffix(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' | '_' => '.',
            _ => '-',
        })
        .collect()
}

fn android_identifier_suffix(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' | '_' => '.',
            _ => '.',
        })
        .collect::<String>()
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(".")
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_escape(value: &str) -> String {
    xml_escape(value)
}

fn swift_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn javascript_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn json_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn gradle_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn powershell_string_escape(value: &str) -> String {
    value.replace('`', "``").replace('\'', "''")
}

fn properties_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n")
}

fn toml_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
