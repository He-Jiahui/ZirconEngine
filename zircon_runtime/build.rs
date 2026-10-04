use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[path = "build/asset_import_context.rs"]
mod asset_import_context;

const PROFILE_PRESET_FILE: &str = "runtime-feature-presets.toml";
const GENERATED_PROFILE_FEATURE_PRESET_FILE: &str = "runtime_profile_feature_presets_generated.rs";
const GENERATED_PROFILE_ASSEMBLY_PRESET_FILE: &str =
    "runtime_profile_assembly_presets_generated.rs";
const EXPECTED_BUILTIN_MODULES: [(&str, &str, Option<&str>); 13] = [
    ("foundation", "Foundation", None),
    ("log", "Log", None),
    ("tasks", "Tasks", None),
    ("time", "Time", None),
    ("frame_count", "FrameCount", None),
    ("diagnostics_core", "DiagnosticsCore", None),
    ("platform", "Platform", None),
    ("input", "Input", None),
    ("asset", "Asset", None),
    ("scene", "Scene", None),
    ("text", "Text", Some("text")),
    ("graphics", "Graphics", Some("graphics")),
    ("script", "Script", Some("script")),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeFeaturePresetDocument {
    schema_version: u32,
    builtin_modules: Vec<BuiltinRuntimeModuleRow>,
    profiles: Vec<RuntimeFeaturePresetRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuiltinRuntimeModuleRow {
    id: String,
    rust_variant: String,
    required_feature: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeFeaturePresetRow {
    id: String,
    rust_variant: String,
    cargo_feature: String,
    runtime_features: Vec<String>,
    app_features: Vec<String>,
    descriptor_name: String,
    target_mode: String,
    builtin_modules: Vec<String>,
    minimum_maturity: String,
    default_plugins: Vec<RuntimeProfilePluginRow>,
    optional_plugins: Vec<String>,
    required_capabilities: Vec<String>,
    allow_externalized_required_plugins: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeProfilePluginRow {
    id: String,
    required: bool,
}

fn main() {
    asset_import_context::emit();
    println!("cargo:rerun-if-changed={PROFILE_PRESET_FILE}");
    generate_runtime_profile_presets();

    let profiling_enabled = std::env::var_os("CARGO_FEATURE_PROFILING").is_some()
        || std::env::var_os("CARGO_FEATURE_PROFILING_CHROME").is_some()
        || std::env::var_os("CARGO_FEATURE_PROFILING_TRACY").is_some()
        || std::env::var_os("CARGO_FEATURE_PROFILING_MEMORY").is_some();
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let profile_dir = std::env::var_os("OUT_DIR")
        .and_then(|out_dir| active_profile_dir(std::path::Path::new(&out_dir)));
    let using_profiling_profile =
        profile == "profiling" || profile_dir.as_deref() == Some("profiling");

    if profiling_enabled && profile == "release" && !using_profiling_profile {
        panic!(
            "Zircon profiling features are disabled for ordinary release builds; use `cargo build --profile profiling --features profiling ...` instead"
        );
    }
}

fn generate_runtime_profile_presets() {
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo should define CARGO_MANIFEST_DIR"),
    );
    let source_path = manifest_dir.join(PROFILE_PRESET_FILE);
    let document_text = std::fs::read_to_string(&source_path).unwrap_or_else(|error| {
        panic!(
            "read runtime profile preset source {}: {error}",
            source_path.display()
        )
    });
    let document: RuntimeFeaturePresetDocument = toml::from_str(&document_text)
        .unwrap_or_else(|error| panic!("parse {}: {error}", source_path.display()));
    validate_runtime_profile_presets(&document)
        .unwrap_or_else(|error| panic!("validate {}: {error}", source_path.display()));

    let out_dir = PathBuf::from(
        std::env::var_os("OUT_DIR").expect("Cargo should define OUT_DIR for build scripts"),
    );
    write_generated_file(
        &out_dir.join(GENERATED_PROFILE_FEATURE_PRESET_FILE),
        render_runtime_profile_feature_presets(&document),
    );
    write_generated_file(
        &out_dir.join(GENERATED_PROFILE_ASSEMBLY_PRESET_FILE),
        render_runtime_profile_assembly_presets(&document),
    );
}

fn write_generated_file(output_path: &Path, generated: String) {
    std::fs::write(output_path, generated).unwrap_or_else(|error| {
        panic!(
            "write generated runtime profile presets {}: {error}",
            output_path.display()
        )
    });
}

fn validate_runtime_profile_presets(document: &RuntimeFeaturePresetDocument) -> Result<(), String> {
    if document.schema_version != 2 {
        return Err(format!(
            "unsupported runtime profile preset schema version {}",
            document.schema_version
        ));
    }

    if document.builtin_modules.len() != EXPECTED_BUILTIN_MODULES.len() {
        return Err(format!(
            "runtime profile preset source must declare every builtin module exactly once; expected {}, found {}",
            EXPECTED_BUILTIN_MODULES.len(),
            document.builtin_modules.len()
        ));
    }

    let mut module_by_id = HashMap::new();
    let mut module_variants = HashSet::new();
    for (module, (expected_id, expected_variant, expected_feature)) in document
        .builtin_modules
        .iter()
        .zip(EXPECTED_BUILTIN_MODULES)
    {
        if module.id != expected_id {
            return Err(format!(
                "builtin module order drift: expected {expected_id}, found {}",
                module.id
            ));
        }
        if module.rust_variant != expected_variant {
            return Err(format!(
                "builtin module variant drift for {}: expected {expected_variant}, found {}",
                module.id, module.rust_variant
            ));
        }
        if module.required_feature.as_deref() != expected_feature {
            return Err(format!(
                "builtin module feature gate drift for {}: expected {expected_feature:?}, found {:?}",
                module.id, module.required_feature
            ));
        }
        validate_canonical_key(&module.id, "builtin module id")?;
        validate_rust_variant(&module.rust_variant, "builtin module")?;
        if module_by_id.insert(module.id.as_str(), module).is_some() {
            return Err(format!("duplicate builtin module id {}", module.id));
        }
        if !module_variants.insert(module.rust_variant.as_str()) {
            return Err(format!(
                "duplicate builtin module Rust variant {}",
                module.rust_variant
            ));
        }
        if let Some(required_feature) = &module.required_feature {
            validate_feature_token(required_feature, "builtin module required feature")?;
        }
    }

    let expected_profiles = [
        ("minimal", "Minimal"),
        ("client2d", "Client2d"),
        ("client3d", "Client3d"),
        ("editor", "Editor"),
        ("dev", "Dev"),
        ("server", "Server"),
    ];
    if document.profiles.len() != expected_profiles.len() {
        return Err(format!(
            "runtime profile preset source must declare every built-in profile exactly once; expected {}, found {}",
            expected_profiles.len(),
            document.profiles.len()
        ));
    }

    let mut profile_ids = HashSet::new();
    let mut profile_variants = HashSet::new();
    let mut descriptor_names = HashSet::new();
    for (row, (expected_id, expected_variant)) in document.profiles.iter().zip(expected_profiles) {
        if row.id != expected_id {
            return Err(format!(
                "runtime profile preset order drift: expected {expected_id}, found {}",
                row.id
            ));
        }
        if row.rust_variant != expected_variant {
            return Err(format!(
                "runtime profile variant drift for {}: expected {expected_variant}, found {}",
                row.id, row.rust_variant
            ));
        }
        validate_canonical_key(&row.id, "runtime profile id")?;
        validate_rust_variant(&row.rust_variant, "runtime profile")?;
        validate_canonical_key(&row.descriptor_name, "runtime profile descriptor name")?;
        validate_feature_token(&row.cargo_feature, "runtime profile Cargo feature")?;
        validate_unique_feature_tokens(
            &row.runtime_features,
            &format!("{} runtime feature", row.id),
        )?;
        validate_unique_feature_tokens(&row.app_features, &format!("{} app feature", row.id))?;
        if row.runtime_features.is_empty() {
            return Err(format!("{} has no runtime feature requirements", row.id));
        }
        if row.app_features.is_empty() {
            return Err(format!("{} has no app feature requirements", row.id));
        }
        if target_mode_variant(&row.target_mode).is_none() {
            return Err(format!(
                "{} has unsupported target mode {}",
                row.id, row.target_mode
            ));
        }
        if maturity_variant(&row.minimum_maturity).is_none() {
            return Err(format!(
                "{} has unsupported minimum maturity {}",
                row.id, row.minimum_maturity
            ));
        }
        if !profile_ids.insert(row.id.as_str()) {
            return Err(format!("duplicate runtime profile id {}", row.id));
        }
        if !profile_variants.insert(row.rust_variant.as_str()) {
            return Err(format!(
                "duplicate runtime profile Rust variant {}",
                row.rust_variant
            ));
        }
        if !descriptor_names.insert(row.descriptor_name.as_str()) {
            return Err(format!(
                "duplicate runtime profile descriptor name {}",
                row.descriptor_name
            ));
        }

        validate_profile_modules(row, &module_by_id)?;
        validate_profile_plugins(row)?;
        validate_profile_capabilities(row)?;
    }

    Ok(())
}

fn validate_profile_modules(
    row: &RuntimeFeaturePresetRow,
    module_by_id: &HashMap<&str, &BuiltinRuntimeModuleRow>,
) -> Result<(), String> {
    if row.builtin_modules.is_empty() {
        return Err(format!("{} has no builtin modules", row.id));
    }
    let mut selected_modules = HashSet::new();
    for module_id in &row.builtin_modules {
        let Some(module) = module_by_id.get(module_id.as_str()).copied() else {
            return Err(format!(
                "{} references unknown builtin module {module_id}",
                row.id
            ));
        };
        if !selected_modules.insert(module_id.as_str()) {
            return Err(format!(
                "{} declares duplicate builtin module {module_id}",
                row.id
            ));
        }
        if let Some(required_feature) = &module.required_feature {
            if !row.runtime_features.contains(required_feature) {
                return Err(format!(
                    "{} selects builtin module {module_id} without runtime feature {required_feature}",
                    row.id
                ));
            }
        }
    }
    Ok(())
}

fn validate_profile_plugins(row: &RuntimeFeaturePresetRow) -> Result<(), String> {
    let mut default_plugins = HashSet::new();
    for plugin in &row.default_plugins {
        validate_canonical_key(&plugin.id, &format!("{} default plugin id", row.id))?;
        if !default_plugins.insert(plugin.id.as_str()) {
            return Err(format!(
                "{} declares duplicate default plugin {}",
                row.id, plugin.id
            ));
        }
    }

    let mut optional_plugins = HashSet::new();
    for plugin_id in &row.optional_plugins {
        validate_canonical_key(plugin_id, &format!("{} optional plugin id", row.id))?;
        if !optional_plugins.insert(plugin_id.as_str()) {
            return Err(format!(
                "{} declares duplicate optional plugin {plugin_id}",
                row.id
            ));
        }
        if default_plugins.contains(plugin_id.as_str()) {
            return Err(format!(
                "{} declares plugin {plugin_id} as both default and optional",
                row.id
            ));
        }
    }
    Ok(())
}

fn validate_profile_capabilities(row: &RuntimeFeaturePresetRow) -> Result<(), String> {
    let mut capabilities = HashSet::new();
    for capability in &row.required_capabilities {
        validate_canonical_key(capability, &format!("{} required capability", row.id))?;
        if !capabilities.insert(capability.as_str()) {
            return Err(format!(
                "{} declares duplicate required capability {capability}",
                row.id
            ));
        }
    }
    Ok(())
}

fn validate_unique_feature_tokens(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for value in values {
        validate_feature_token(value, label)?;
        if !seen.insert(value.as_str()) {
            return Err(format!("duplicate {label} {value}"));
        }
    }
    Ok(())
}

fn validate_canonical_key(value: &str, label: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.trim() == value
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        });
    if valid {
        Ok(())
    } else {
        Err(format!("invalid canonical {label} {value:?}"))
    }
}

fn validate_feature_token(value: &str, label: &str) -> Result<(), String> {
    let weak_dependency_feature = value.split_once("?/").is_some_and(|(dependency, feature)| {
        !dependency.is_empty()
            && !feature.is_empty()
            && !dependency.contains(['?', '/', ':'])
            && !feature.contains(['?', '/', ':'])
    });
    let has_weak_dependency_marker = value.contains('?');
    let valid = !value.is_empty()
        && value.trim() == value
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b':' | b'/' | b'.' | b'?')
        })
        && (!has_weak_dependency_marker || weak_dependency_feature)
        && (!has_weak_dependency_marker || value.matches("?/").count() == 1);
    if valid {
        Ok(())
    } else {
        Err(format!("invalid {label} {value:?}"))
    }
}

fn validate_rust_variant(value: &str, label: &str) -> Result<(), String> {
    let valid = value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_uppercase())
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err(format!("invalid {label} Rust variant {value:?}"))
    }
}

fn render_runtime_profile_feature_presets(document: &RuntimeFeaturePresetDocument) -> String {
    let mut generated = String::from(
        "// @generated by zircon_runtime/build.rs from runtime-feature-presets.toml.\n\
         pub const RUNTIME_PROFILE_FEATURE_PRESETS: &[RuntimeProfileFeaturePreset] = &[\n",
    );
    for row in &document.profiles {
        writeln!(generated, "    RuntimeProfileFeaturePreset {{").unwrap();
        writeln!(
            generated,
            "        id: RuntimeProfileId::{},",
            row.rust_variant
        )
        .unwrap();
        writeln!(generated, "        name: {:?},", row.id).unwrap();
        writeln!(generated, "        cargo_feature: {:?},", row.cargo_feature).unwrap();
        render_string_slice(&mut generated, "runtime_features", &row.runtime_features);
        render_string_slice(&mut generated, "app_features", &row.app_features);
        generated.push_str("    },\n");
    }
    generated.push_str("];\n");
    generated
}

fn render_runtime_profile_assembly_presets(document: &RuntimeFeaturePresetDocument) -> String {
    let module_by_id = document
        .builtin_modules
        .iter()
        .map(|module| (module.id.as_str(), module))
        .collect::<HashMap<_, _>>();
    let mut generated = String::from(
        "// @generated by zircon_runtime/build.rs from runtime-feature-presets.toml.\n\
         const RUNTIME_PROFILE_ASSEMBLY_PRESETS: &[RuntimeProfileAssemblyPreset] = &[\n",
    );
    for row in &document.profiles {
        writeln!(generated, "    RuntimeProfileAssemblyPreset {{").unwrap();
        writeln!(
            generated,
            "        id: RuntimeProfileId::{},",
            row.rust_variant
        )
        .unwrap();
        writeln!(
            generated,
            "        descriptor_name: {:?},",
            row.descriptor_name
        )
        .unwrap();
        writeln!(
            generated,
            "        target_mode: RuntimeTargetMode::{},",
            target_mode_variant(&row.target_mode).expect("validated target mode")
        )
        .unwrap();
        generated.push_str("        builtin_modules: &[\n");
        for module_id in &row.builtin_modules {
            let module = module_by_id
                .get(module_id.as_str())
                .expect("validated builtin module reference");
            if let Some(required_feature) = &module.required_feature {
                writeln!(
                    generated,
                    "            #[cfg(feature = {required_feature:?})]"
                )
                .unwrap();
            }
            writeln!(
                generated,
                "            BuiltinRuntimeModuleId::{},",
                module.rust_variant
            )
            .unwrap();
        }
        generated.push_str("        ],\n        default_plugins: &[\n");
        for plugin in &row.default_plugins {
            writeln!(
                generated,
                "            RuntimeProfileAssemblyPluginPreset {{ id: {:?}, required: {} }},",
                plugin.id, plugin.required
            )
            .unwrap();
        }
        generated.push_str("        ],\n");
        render_string_slice(&mut generated, "optional_plugins", &row.optional_plugins);
        render_string_slice(
            &mut generated,
            "required_capabilities",
            &row.required_capabilities,
        );
        writeln!(
            generated,
            "        minimum_maturity: PluginMaturity::{},",
            maturity_variant(&row.minimum_maturity).expect("validated minimum maturity")
        )
        .unwrap();
        writeln!(
            generated,
            "        allow_externalized_required_plugins: {},",
            row.allow_externalized_required_plugins
        )
        .unwrap();
        generated.push_str("    },\n");
    }
    generated.push_str("];\n\n");
    generated.push_str(
        "fn generated_runtime_profile_assembly_preset_for(\n    id: RuntimeProfileId,\n) -> &'static RuntimeProfileAssemblyPreset {\n    match id {\n",
    );
    for (index, row) in document.profiles.iter().enumerate() {
        writeln!(
            generated,
            "        RuntimeProfileId::{} => &RUNTIME_PROFILE_ASSEMBLY_PRESETS[{index}],",
            row.rust_variant
        )
        .unwrap();
    }
    generated.push_str("    }\n}\n");
    generated
}

fn render_string_slice(generated: &mut String, field: &str, values: &[String]) {
    writeln!(generated, "        {field}: &[").unwrap();
    for value in values {
        writeln!(generated, "            {value:?},").unwrap();
    }
    generated.push_str("        ],\n");
}

fn target_mode_variant(value: &str) -> Option<&'static str> {
    match value {
        "client_runtime" => Some("ClientRuntime"),
        "server_runtime" => Some("ServerRuntime"),
        "editor_host" => Some("EditorHost"),
        _ => None,
    }
}

fn maturity_variant(value: &str) -> Option<&'static str> {
    match value {
        "core" => Some("Core"),
        "stable" => Some("Stable"),
        "beta" => Some("Beta"),
        "experimental" => Some("Experimental"),
        "externalized" => Some("Externalized"),
        "stub" => Some("Stub"),
        "deprecated" => Some("Deprecated"),
        _ => None,
    }
}

fn active_profile_dir(out_dir: &Path) -> Option<String> {
    let components = out_dir
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>();
    let build_index = components
        .iter()
        .rposition(|component| *component == "build")?;
    build_index
        .checked_sub(1)
        .map(|index| components[index].to_string())
}

#[cfg(test)]
#[path = "tests/unit/build.rs"]
mod tests;
