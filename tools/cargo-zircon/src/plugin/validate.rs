use std::collections::HashSet;
use std::fs;
use std::path::Path;

use toml::Value;

use super::diagnostic::PluginDiagnostic;

mod native_artifact;

pub use native_artifact::validate_native_artifact;

const REQUIRED_STRINGS: &[(&str, &str)] = &[
    ("id", "plugin.id.missing"),
    ("version", "plugin.version.missing"),
    ("sdk_api_version", "plugin.sdk_api_version.missing"),
    ("display_name", "plugin.display_name.missing"),
    ("category", "plugin.category.missing"),
    ("description", "plugin.description.missing"),
    ("maturity", "plugin.maturity.missing"),
];
const TARGETS: &[&str] = &["client_runtime", "server_runtime", "editor_host"];
const PLATFORMS: &[&str] = &[
    "windows", "linux", "macos", "android", "ios", "web_gpu", "wasm", "headless",
];
const PACKAGING: &[&str] = &["source_template", "library_embed", "native_dynamic"];
const CATEGORIES: &[&str] = &[
    "asset_importer",
    "authoring",
    "diagnostics",
    "platform",
    "rendering",
    "runtime",
    "sdk",
];
const MATURITY: &[&str] = &["stable", "beta", "experimental"];

pub fn validate_plugin_manifest(
    manifest_text: &str,
    package_root: Option<&Path>,
) -> Vec<PluginDiagnostic> {
    let manifest: Value = match manifest_text.parse() {
        Ok(manifest) => manifest,
        Err(error) => {
            return vec![PluginDiagnostic::new(
                "plugin.toml.parse",
                format!("plugin.toml is not valid TOML: {error}"),
                "Fix the reported TOML syntax before running plugin validation again.",
            )];
        }
    };
    let Some(root) = manifest.as_table() else {
        return vec![PluginDiagnostic::new(
            "plugin.toml.root",
            "plugin.toml root must be a table",
            "Declare package fields at the document root.",
        )];
    };
    let mut diagnostics = Vec::new();

    for (field, code) in REQUIRED_STRINGS {
        match root.get(*field).and_then(Value::as_str) {
            Some(value) if !value.trim().is_empty() => {}
            Some(_) if *field == "display_name" => diagnostics.push(PluginDiagnostic::new(
                "plugin.display_name.empty",
                "display_name must not be empty",
                "Set display_name to the user-facing plugin name from declare_plugin!.",
            )),
            Some(_) => diagnostics.push(PluginDiagnostic::new(
                format!("{code}.empty"),
                format!("{field} must not be empty"),
                format!("Set {field} to a non-empty generated package value."),
            )),
            None => diagnostics.push(PluginDiagnostic::new(
                *code,
                format!("missing required root field `{field}`"),
                format!("Add `{field}` through the Rust declaration manifest projection."),
            )),
        }
    }

    if let Some(id) = root.get("id").and_then(Value::as_str) {
        if !valid_plugin_id(id) {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.id.invalid",
                format!("plugin id `{id}` is not lowercase snake case"),
                "Use a lowercase ASCII identifier such as `demo_probe`.",
            ));
        }
    }
    for field in ["version", "sdk_api_version"] {
        if let Some(version) = root.get(field).and_then(Value::as_str) {
            if !valid_semver(version) {
                diagnostics.push(PluginDiagnostic::new(
                    format!("plugin.{field}.invalid"),
                    format!("{field} `{version}` is not MAJOR.MINOR.PATCH"),
                    "Use three numeric components without leading zeroes.",
                ));
            }
        }
    }
    validate_enum(root, "category", CATEGORIES, &mut diagnostics);
    validate_enum(root, "maturity", MATURITY, &mut diagnostics);
    validate_string_array(
        root,
        "supported_targets",
        TARGETS,
        "plugin.target",
        &mut diagnostics,
    );
    validate_string_array(
        root,
        "supported_platforms",
        PLATFORMS,
        "plugin.platform",
        &mut diagnostics,
    );
    validate_string_array(
        root,
        "default_packaging",
        PACKAGING,
        "plugin.packaging",
        &mut diagnostics,
    );
    validate_capabilities(root, &mut diagnostics);
    validate_modules(root, &mut diagnostics);
    validate_distribution(root, package_root, &mut diagnostics);
    diagnostics
}

fn validate_modules(root: &toml::map::Map<String, Value>, diagnostics: &mut Vec<PluginDiagnostic>) {
    let Some(modules) = root.get("modules").and_then(Value::as_array) else {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.modules.missing",
            "plugin manifest has no [[modules]] entries",
            "Project at least one runtime or editor module from declare_plugin!.",
        ));
        return;
    };
    if modules.is_empty() {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.modules.empty",
            "plugin manifest modules list is empty",
            "Add the declaration-owned runtime or editor module.",
        ));
    }
    let declared_targets = root
        .get("supported_targets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<HashSet<_>>();
    let plugin_id = root.get("id").and_then(Value::as_str);
    let mut seen_module_names = HashSet::new();
    for (index, module) in modules.iter().enumerate() {
        let Some(module) = module.as_table() else {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.module.invalid",
                format!("module #{index} is not a table"),
                "Use a [[modules]] table with name, kind, crate_name, target_modes, and capabilities.",
            ));
            continue;
        };
        for field in ["name", "kind", "crate_name"] {
            match module.get(field) {
                Some(Value::String(value)) if !value.trim().is_empty() => {}
                Some(Value::String(_)) => diagnostics.push(PluginDiagnostic::new(
                    format!("plugin.module.{field}.empty"),
                    format!("module #{index} has an empty `{field}`"),
                    format!("Project a non-empty `{field}` from declare_plugin!."),
                )),
                Some(_) => diagnostics.push(PluginDiagnostic::new(
                    format!("plugin.module.{field}.invalid_type"),
                    format!("module #{index} `{field}` is not a string"),
                    format!("Project `{field}` as a string from declare_plugin!."),
                )),
                None => diagnostics.push(PluginDiagnostic::new(
                    format!("plugin.module.{field}.missing"),
                    format!("module #{index} is missing `{field}`"),
                    format!("Add `{field}` to the generated module projection."),
                )),
            }
        }
        let module_name = module.get("name").and_then(Value::as_str);
        let module_kind = module.get("kind").and_then(Value::as_str);
        if let Some(module_name) = module_name.filter(|name| !name.trim().is_empty()) {
            if !valid_module_name(module_name) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.module.name.invalid",
                    format!("module #{index} name `{module_name}` is not a dot namespace"),
                    "Use lowercase ASCII package.module notation without empty segments.",
                ));
            }
            if plugin_id.is_some_and(|plugin_id| {
                !module_name
                    .strip_prefix(plugin_id)
                    .is_some_and(|suffix| suffix.starts_with('.'))
            }) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.module.name.outside_namespace",
                    format!(
                        "module #{index} name `{module_name}` is outside the plugin `{}` namespace",
                        plugin_id.unwrap_or_default()
                    ),
                    "Keep generated module names under the package id namespace.",
                ));
            }
            if !seen_module_names.insert(module_name) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.module.name.duplicate",
                    format!("module #{index} duplicates module name `{module_name}`"),
                    "Project each declaration-owned module name exactly once.",
                ));
            }
            if let Some(expected_suffix) = match module_kind {
                Some("runtime") => Some(".runtime"),
                Some("editor") => Some(".editor"),
                _ => None,
            } {
                if !module_name.ends_with(expected_suffix) {
                    diagnostics.push(PluginDiagnostic::new(
                        "plugin.module.name.invalid_for_kind",
                        format!(
                            "module #{index} name `{module_name}` does not end with `{expected_suffix}`"
                        ),
                        "Use the declaration-owned runtime or editor module name.",
                    ));
                }
            }
        }
        if let Some(crate_name) = module
            .get("crate_name")
            .and_then(Value::as_str)
            .filter(|crate_name| !crate_name.trim().is_empty())
        {
            if !valid_plugin_crate_name(crate_name) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.module.crate_name.invalid",
                    format!("module #{index} crate_name `{crate_name}` is invalid"),
                    "Use a zircon_plugin_ prefixed lowercase Cargo package name.",
                ));
            }
        }
        if let Some(kind) = module_kind {
            if !["runtime", "editor", "native", "vm"].contains(&kind) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.module.kind.unknown",
                    format!("module #{index} has unknown kind `{kind}`"),
                    "Use runtime, editor, native, or vm.",
                ));
            }
        }
        match module.get("target_modes").and_then(Value::as_array) {
            Some(target_modes) => {
                if target_modes.is_empty() {
                    diagnostics.push(PluginDiagnostic::new(
                        "plugin.module.target_modes.empty",
                        format!("module #{index} has no target modes"),
                        "Project at least one module target from declare_plugin! metadata.",
                    ));
                }
                for (target_index, target_mode) in target_modes.iter().enumerate() {
                    let Some(target_mode) = target_mode.as_str() else {
                        diagnostics.push(PluginDiagnostic::new(
                            "plugin.module.target_mode.invalid_type",
                            format!(
                                "module #{index} target_modes entry #{target_index} is not a string"
                            ),
                            "Project target_modes as strings from declare_plugin! metadata.",
                        ));
                        continue;
                    };
                    if !TARGETS.contains(&target_mode) {
                        diagnostics.push(PluginDiagnostic::new(
                            "plugin.module.target_mode.unknown",
                            format!("module #{index} has unknown target mode `{target_mode}`"),
                            format!("Use one of: {}.", TARGETS.join(", ")),
                        ));
                    } else if !declared_targets.contains(target_mode) {
                        diagnostics.push(PluginDiagnostic::new(
                            "plugin.module.target_mode.undeclared",
                            format!(
                                "module #{index} target mode `{target_mode}` is absent from root supported_targets"
                            ),
                            "Add the target to declare_plugin! or remove the stale module projection.",
                        ));
                    }
                    if module_kind == Some("editor") && target_mode != "editor_host" {
                        diagnostics.push(PluginDiagnostic::new(
                            "plugin.module.target_mode.invalid_for_kind",
                            format!("editor module #{index} cannot target `{target_mode}`"),
                            "Editor modules may target only editor_host.",
                        ));
                    }
                }
            }
            None => diagnostics.push(PluginDiagnostic::new(
                "plugin.module.target_modes.missing",
                format!("module #{index} is missing `target_modes`"),
                "Project `target_modes` from declare_plugin! metadata.",
            )),
        }
        match module.get("capabilities").and_then(Value::as_array) {
            Some(capabilities) => {
                if capabilities.is_empty() {
                    diagnostics.push(PluginDiagnostic::new(
                        "plugin.module.capabilities.empty",
                        format!("module #{index} has no capabilities"),
                        "Project at least one module capability from declare_plugin! metadata.",
                    ));
                }
                for (capability_index, capability) in capabilities.iter().enumerate() {
                    let Some(capability) = capability.as_str() else {
                        diagnostics.push(PluginDiagnostic::new(
                            "plugin.module.capability.invalid_type",
                            format!(
                                "module #{index} capabilities entry #{capability_index} is not a string"
                            ),
                            "Project module capabilities as strings from declare_plugin! metadata.",
                        ));
                        continue;
                    };
                    let expected_prefix = match module.get("kind").and_then(Value::as_str) {
                        Some("runtime") => Some("runtime."),
                        Some("editor") => Some("editor."),
                        _ => None,
                    };
                    if expected_prefix.is_some_and(|prefix| !capability.starts_with(prefix)) {
                        diagnostics.push(PluginDiagnostic::new(
                            "plugin.module.capability.invalid_prefix",
                            format!(
                                "module #{index} capability `{capability}` does not match its module kind"
                            ),
                            "Use runtime.* capabilities for runtime modules and editor.* capabilities for editor modules.",
                        ));
                    }
                }
            }
            None => diagnostics.push(PluginDiagnostic::new(
                "plugin.module.capabilities.missing",
                format!("module #{index} is missing `capabilities`"),
                "Project `capabilities` from declare_plugin! metadata.",
            )),
        }
    }
}

fn validate_distribution(
    root: &toml::map::Map<String, Value>,
    package_root: Option<&Path>,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let native_requested = root
        .get("default_packaging")
        .and_then(Value::as_array)
        .is_some_and(|values| {
            values
                .iter()
                .any(|value| value.as_str() == Some("native_dynamic"))
        });
    if !native_requested {
        if root.contains_key("distribution") {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.distribution.unexpected",
                "plugin manifest has [distribution] metadata without native_dynamic packaging",
                "Remove the stale distribution projection or declare native_dynamic packaging.",
            ));
        }
        return;
    }
    let Some(distribution) = root.get("distribution").and_then(Value::as_table) else {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.distribution.missing",
            "native_dynamic packaging requires [distribution]",
            "Run `cargo zircon plugin sync-manifest` or scaffold with `--native`.",
        ));
        return;
    };
    let has_runtime = root
        .get("modules")
        .and_then(Value::as_array)
        .is_some_and(|modules| {
            modules
                .iter()
                .any(|module| module.get("kind").and_then(Value::as_str) == Some("runtime"))
        });
    let has_editor = root
        .get("modules")
        .and_then(Value::as_array)
        .is_some_and(|modules| {
            modules
                .iter()
                .any(|module| module.get("kind").and_then(Value::as_str) == Some("editor"))
        });
    if has_runtime
        && distribution
            .get("runtime_entry")
            .and_then(Value::as_str)
            .is_none()
    {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.distribution.runtime_entry.missing",
            "runtime native distribution is missing runtime_entry",
            "Declare native_projection.runtime.entry in declare_plugin!.",
        ));
    }
    if has_editor
        && distribution
            .get("editor_entry")
            .and_then(Value::as_str)
            .is_none()
    {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.distribution.editor_entry.missing",
            "editor native distribution is missing editor_entry",
            "Declare native_projection.editor.entry in declare_plugin!.",
        ));
    }
    for field in ["dist_crate", "descriptor_symbol", "engine_compat"] {
        if distribution.get(field).and_then(Value::as_str).is_none() {
            diagnostics.push(PluginDiagnostic::new(
                format!("plugin.distribution.{field}.missing"),
                format!("native distribution is missing `{field}`"),
                format!("Add `{field}` to the generated distribution contract."),
            ));
        }
    }
    if let Some(dist_crate) = distribution.get("dist_crate").and_then(Value::as_str) {
        let projected = root
            .get("modules")
            .and_then(Value::as_array)
            .is_some_and(|modules| {
                modules.iter().any(|module| {
                    module.get("crate_name").and_then(Value::as_str) == Some(dist_crate)
                })
            });
        if !projected {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.distribution.module.missing",
                format!("distribution crate `{dist_crate}` is absent from [[modules]]"),
                "Project the declaration-owned dist or inline native module.",
            ));
        }
    }
    if distribution.get("abi_version").and_then(Value::as_integer) != Some(3) {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.distribution.abi_version.invalid",
            "native distribution abi_version must be 3",
            "Use the SDK v3 descriptor projection.",
        ));
    }
    if let (Some(package_root), Some(dist_crate)) = (
        package_root,
        distribution.get("dist_crate").and_then(Value::as_str),
    ) {
        if !package_contains_crate(package_root, dist_crate) {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.distribution.crate.missing",
                format!(
                    "distribution crate `{dist_crate}` has no matching package Cargo.toml"
                ),
                "Generate the native dist/ or inline native/ crate, or remove native_dynamic packaging.",
            ));
        }
    }
}

fn package_contains_crate(package_root: &Path, crate_name: &str) -> bool {
    ["dist", "native", "runtime", "editor"]
        .into_iter()
        .filter_map(|owner| fs::read_to_string(package_root.join(owner).join("Cargo.toml")).ok())
        .filter_map(|cargo| cargo.parse::<Value>().ok())
        .any(|cargo| {
            cargo
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(Value::as_str)
                == Some(crate_name)
        })
}

fn validate_enum(
    root: &toml::map::Map<String, Value>,
    field: &str,
    allowed: &[&str],
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    if let Some(value) = root.get(field).and_then(Value::as_str) {
        if !allowed.contains(&value) {
            diagnostics.push(PluginDiagnostic::new(
                format!("plugin.{field}.unknown"),
                format!("unknown {field} `{value}`"),
                format!("Use one of: {}.", allowed.join(", ")),
            ));
        }
    }
}

fn validate_string_array(
    root: &toml::map::Map<String, Value>,
    field: &str,
    allowed: &[&str],
    code_prefix: &str,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let Some(values) = root.get(field).and_then(Value::as_array) else {
        diagnostics.push(PluginDiagnostic::new(
            format!("{code_prefix}.missing"),
            format!("missing `{field}` array"),
            format!("Project `{field}` from declare_plugin!.",),
        ));
        return;
    };
    if values.is_empty() {
        diagnostics.push(PluginDiagnostic::new(
            format!("{code_prefix}.empty"),
            format!("`{field}` must not be empty"),
            format!("Declare at least one supported {field} value."),
        ));
    }
    for (index, value) in values.iter().enumerate() {
        match value.as_str() {
            Some(value) if !allowed.contains(&value) => diagnostics.push(PluginDiagnostic::new(
                format!("{code_prefix}.unknown"),
                format!("unknown {field} value `{value}`"),
                format!("Use one of: {}.", allowed.join(", ")),
            )),
            Some(_) => {}
            None => diagnostics.push(PluginDiagnostic::new(
                format!("{code_prefix}.invalid_type"),
                format!("`{field}` entry #{index} is not a string"),
                format!("Project `{field}` as strings from declare_plugin!."),
            )),
        }
    }
}

fn validate_capabilities(
    root: &toml::map::Map<String, Value>,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    match root.get("capabilities").and_then(Value::as_array) {
        Some(values) if !values.is_empty() => {
            for (index, capability) in values.iter().enumerate() {
                match capability.as_str() {
                    Some(capability) if capability.trim().is_empty() => {
                        diagnostics.push(PluginDiagnostic::new(
                            "plugin.capability.empty",
                            format!("capabilities entry #{index} is empty"),
                            "Declare a non-empty capability identifier in declare_plugin!.",
                        ));
                    }
                    Some(_) => {}
                    None => diagnostics.push(PluginDiagnostic::new(
                        "plugin.capability.invalid_type",
                        format!("capabilities entry #{index} is not a string"),
                        "Project capabilities as strings from declare_plugin!.",
                    )),
                }
            }
        }
        Some(_) => diagnostics.push(PluginDiagnostic::new(
            "plugin.capabilities.empty",
            "`capabilities` must not be empty",
            "Declare at least one capability value in declare_plugin!.",
        )),
        None => diagnostics.push(PluginDiagnostic::new(
            "plugin.capabilities.missing",
            "missing `capabilities` array",
            "Project `capabilities` from declare_plugin!.",
        )),
    }
}

fn valid_plugin_id(id: &str) -> bool {
    !id.is_empty()
        && id.as_bytes()[0].is_ascii_lowercase()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && !id.ends_with('_')
        && !id.contains("__")
}

fn valid_module_name(name: &str) -> bool {
    let segments = name.split('.').collect::<Vec<_>>();
    segments.len() >= 2
        && segments.iter().all(|segment| !segment.is_empty())
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'.'
        })
}

fn valid_plugin_crate_name(crate_name: &str) -> bool {
    crate_name.starts_with("zircon_plugin_")
        && crate_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && !crate_name.ends_with('_')
        && !crate_name.contains("__")
}

fn valid_semver(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == &"0" || !part.starts_with('0'))
        })
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::ptr;

    use toml::Value;

    use super::validate_entry_name;

    #[test]
    fn native_entry_name_validation_covers_absent_unexpected_missing_drift_and_match() {
        let empty_distribution = toml::map::Map::new();
        let mut diagnostics = Vec::new();
        assert_eq!(
            validate_entry_name(
                &empty_distribution,
                "editor_entry",
                ptr::null(),
                &mut diagnostics,
            ),
            None
        );
        assert!(diagnostics.is_empty());

        let unexpected = CString::new("unexpected_editor_entry").unwrap();
        assert_eq!(
            validate_entry_name(
                &empty_distribution,
                "editor_entry",
                unexpected.as_ptr(),
                &mut diagnostics,
            ),
            None
        );
        assert_eq!(
            diagnostics.last().unwrap().code,
            "plugin.native_artifact.editor_entry_unexpected"
        );

        let mut runtime_distribution = toml::map::Map::new();
        runtime_distribution.insert(
            "runtime_entry".to_string(),
            Value::String("expected_runtime_entry".to_string()),
        );
        assert_eq!(
            validate_entry_name(
                &runtime_distribution,
                "runtime_entry",
                ptr::null(),
                &mut diagnostics,
            ),
            None
        );
        assert_eq!(
            diagnostics.last().unwrap().code,
            "plugin.native_artifact.runtime_entry_null"
        );

        let drifted = CString::new("drifted_runtime_entry").unwrap();
        assert_eq!(
            validate_entry_name(
                &runtime_distribution,
                "runtime_entry",
                drifted.as_ptr(),
                &mut diagnostics,
            ),
            None
        );
        assert_eq!(
            diagnostics.last().unwrap().code,
            "plugin.native_artifact.runtime_entry_mismatch"
        );

        let expected = CString::new("expected_runtime_entry").unwrap();
        assert_eq!(
            validate_entry_name(
                &runtime_distribution,
                "runtime_entry",
                expected.as_ptr(),
                &mut diagnostics,
            ),
            Some("expected_runtime_entry".to_string())
        );
    }
}
