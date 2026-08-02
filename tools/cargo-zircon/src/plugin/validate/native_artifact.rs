use std::ffi::{c_char, CStr};
use std::path::Path;

use toml::Value;

use super::super::diagnostic::PluginDiagnostic;

#[repr(C)]
struct NativePluginAbiV3Projection {
    abi_version: u32,
    plugin_id: *const c_char,
    package_manifest_toml: *const c_char,
    runtime_entry_name: *const c_char,
    editor_entry_name: *const c_char,
    requested_capabilities: *const c_char,
}

pub fn validate_native_artifact(
    manifest_text: &str,
    artifact_path: &Path,
) -> Vec<PluginDiagnostic> {
    if !artifact_path.is_file() {
        return vec![PluginDiagnostic::new(
            "plugin.native_artifact.missing",
            format!(
                "native plugin artifact `{}` does not exist",
                artifact_path.display()
            ),
            "Build the manifest's dist_crate, then pass its target profile directory through --artifact-root or the file through --artifact.",
        )];
    }
    let manifest: Value = match manifest_text.parse() {
        Ok(manifest) => manifest,
        Err(error) => {
            return vec![PluginDiagnostic::new(
                "plugin.native_artifact.manifest_invalid",
                format!("cannot validate a native artifact against invalid plugin.toml: {error}"),
                "Fix plugin.toml before probing its native artifact.",
            )];
        }
    };
    let Some(root) = manifest.as_table() else {
        return vec![PluginDiagnostic::new(
            "plugin.native_artifact.manifest_invalid",
            "cannot validate a native artifact without a plugin manifest table",
            "Fix plugin.toml before probing its native artifact.",
        )];
    };
    let Some(distribution) = root.get("distribution").and_then(Value::as_table) else {
        return vec![PluginDiagnostic::new(
            "plugin.native_artifact.distribution_missing",
            "native artifact validation requires [distribution] metadata",
            "Generate the native distribution projection from declare_plugin!.",
        )];
    };
    let Some(descriptor_symbol) = distribution
        .get("descriptor_symbol")
        .and_then(Value::as_str)
    else {
        return vec![PluginDiagnostic::new(
            "plugin.native_artifact.descriptor_symbol_missing",
            "native distribution does not declare descriptor_symbol",
            "Set descriptor_symbol to zircon_native_plugin_descriptor_v3 through the generated manifest projection.",
        )];
    };

    let library = match unsafe { libloading::Library::new(artifact_path) } {
        Ok(library) => library,
        Err(error) => {
            return vec![PluginDiagnostic::new(
                "plugin.native_artifact.load_failed",
                format!(
                    "native artifact `{}` could not be loaded: {error}",
                    artifact_path.display()
                ),
                "Build the artifact for the current host platform and ensure all dynamic dependencies are available.",
            )];
        }
    };
    let descriptor = match unsafe {
        library.get::<unsafe extern "C" fn() -> *const NativePluginAbiV3Projection>(
            nul_terminated_symbol(descriptor_symbol).as_bytes(),
        )
    } {
        Ok(descriptor) => descriptor,
        Err(error) => {
            return vec![PluginDiagnostic::new(
                "plugin.native_artifact.descriptor_symbol_missing",
                format!(
                    "native artifact `{}` does not export `{descriptor_symbol}`: {error}",
                    artifact_path.display()
                ),
                "Use zircon_plugin_sdk's native_dist_*_plugin_v3! macro in the dist crate.",
            )];
        }
    };
    let descriptor = unsafe { descriptor() };
    if descriptor.is_null() {
        return vec![PluginDiagnostic::new(
            "plugin.native_artifact.descriptor_null",
            format!("native descriptor `{descriptor_symbol}` returned null"),
            "Return the SDK-generated static ABI v3 descriptor.",
        )];
    }
    let descriptor = unsafe { &*descriptor };
    let mut diagnostics = Vec::new();
    if descriptor.abi_version != 3 {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.native_artifact.abi_version_mismatch",
            format!(
                "native descriptor ABI is {}, expected 3",
                descriptor.abi_version
            ),
            "Build the dist crate with the SDK ABI v3 macro.",
        ));
    }
    let plugin_id = read_descriptor_string(descriptor.plugin_id, "plugin_id", &mut diagnostics);
    if let (Some(expected), Some(actual)) = (root.get("id").and_then(Value::as_str), plugin_id) {
        if actual != expected {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.native_artifact.plugin_id_mismatch",
                format!(
                    "native descriptor plugin_id `{actual}` differs from manifest id `{expected}`"
                ),
                "Regenerate both the manifest and dist ABI projection from the same declare_plugin! block.",
            ));
        }
    }
    if let Some(embedded) = read_descriptor_string(
        descriptor.package_manifest_toml,
        "package_manifest_toml",
        &mut diagnostics,
    ) {
        match embedded.parse::<Value>() {
            Ok(embedded) if embedded == manifest => {}
            Ok(_) => diagnostics.push(PluginDiagnostic::new(
                "plugin.native_artifact.embedded_manifest_drift",
                "native artifact embeds a plugin manifest that differs from plugin.toml",
                "Run plugin sync-manifest, rebuild the dist crate, and validate the rebuilt artifact.",
            )),
            Err(error) => diagnostics.push(PluginDiagnostic::new(
                "plugin.native_artifact.embedded_manifest_invalid",
                format!("native artifact embeds invalid plugin TOML: {error}"),
                "Embed the generated plugin.toml snapshot through the SDK dist macro.",
            )),
        }
    }
    validate_descriptor_capabilities(root, descriptor, &mut diagnostics);
    validate_entry_symbol(
        &library,
        distribution,
        "runtime_entry",
        descriptor.runtime_entry_name,
        &mut diagnostics,
    );
    validate_entry_symbol(
        &library,
        distribution,
        "editor_entry",
        descriptor.editor_entry_name,
        &mut diagnostics,
    );
    diagnostics
}

fn validate_descriptor_capabilities(
    root: &toml::map::Map<String, Value>,
    descriptor: &NativePluginAbiV3Projection,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let Some(actual) = read_descriptor_string(
        descriptor.requested_capabilities,
        "requested_capabilities",
        diagnostics,
    ) else {
        return;
    };
    let mut actual = actual
        .lines()
        .filter(|capability| !capability.is_empty())
        .collect::<Vec<_>>();
    let mut expected = root
        .get("capabilities")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    actual.sort_unstable();
    actual.dedup();
    expected.sort_unstable();
    expected.dedup();
    if actual != expected {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.native_artifact.capability_drift",
            format!(
                "native descriptor requested capabilities {:?}, manifest declares {:?}",
                actual, expected
            ),
            "Regenerate NATIVE_REQUESTED_CAPABILITIES and plugin.toml from the same declare_plugin! block.",
        ));
    }
}

fn validate_entry_symbol(
    library: &libloading::Library,
    distribution: &toml::map::Map<String, Value>,
    field: &str,
    descriptor_entry: *const c_char,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let Some(expected) = validate_entry_name(distribution, field, descriptor_entry, diagnostics)
    else {
        return;
    };
    if let Err(error) = unsafe {
        library.get::<unsafe extern "C" fn()>(nul_terminated_symbol(&expected).as_bytes())
    } {
        diagnostics.push(PluginDiagnostic::new(
            format!("plugin.native_artifact.{field}_symbol_missing"),
            format!("native artifact does not export {field} symbol `{expected}`: {error}"),
            "Use the generated entry identifier in the SDK native dist macro.",
        ));
    }
}

fn validate_entry_name(
    distribution: &toml::map::Map<String, Value>,
    field: &str,
    descriptor_entry: *const c_char,
    diagnostics: &mut Vec<PluginDiagnostic>,
) -> Option<String> {
    let expected = distribution.get(field).and_then(Value::as_str);
    if expected.is_none() && descriptor_entry.is_null() {
        return None;
    }
    let actual = read_descriptor_string(descriptor_entry, field, diagnostics)?;
    let Some(expected) = expected else {
        diagnostics.push(PluginDiagnostic::new(
            format!("plugin.native_artifact.{field}_unexpected"),
            format!("native descriptor exposes {field} `{actual}`, but the manifest omits it"),
            "Regenerate the descriptor and plugin.toml from the same declare_plugin! block.",
        ));
        return None;
    };
    if actual != expected {
        diagnostics.push(PluginDiagnostic::new(
            format!("plugin.native_artifact.{field}_mismatch"),
            format!("native descriptor {field} `{actual}` differs from manifest `{expected}"),
            "Regenerate the native entry name and plugin.toml from the same declare_plugin! block.",
        ));
        return None;
    }
    Some(expected.to_string())
}

fn read_descriptor_string(
    pointer: *const c_char,
    field: &str,
    diagnostics: &mut Vec<PluginDiagnostic>,
) -> Option<String> {
    if pointer.is_null() {
        diagnostics.push(PluginDiagnostic::new(
            format!("plugin.native_artifact.{field}_null"),
            format!("native descriptor field `{field}` is null"),
            "Use the SDK-generated nul-terminated ABI projection.",
        ));
        return None;
    }
    match unsafe { CStr::from_ptr(pointer) }.to_str() {
        Ok(value) => Some(value.to_string()),
        Err(error) => {
            diagnostics.push(PluginDiagnostic::new(
                format!("plugin.native_artifact.{field}_invalid_utf8"),
                format!("native descriptor field `{field}` is not UTF-8: {error}"),
                "Project UTF-8 metadata through declare_plugin! and rebuild the dist crate.",
            ));
            None
        }
    }
}

fn nul_terminated_symbol(symbol: &str) -> String {
    let mut symbol = symbol.to_string();
    symbol.push('\0');
    symbol
}
