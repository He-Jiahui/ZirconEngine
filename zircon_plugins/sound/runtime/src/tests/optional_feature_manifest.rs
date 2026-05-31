#[test]
fn static_plugin_manifest_keeps_optional_feature_manifests_in_sync() {
    let mut static_features =
        optional_features_from_plugin_toml(include_str!("../../../plugin.toml"));
    let mut runtime_features = crate::package_manifest()
        .optional_features
        .iter()
        .map(optional_feature_signature)
        .collect::<Vec<_>>();
    static_features.sort_unstable_by_key(|feature| feature.id.clone());
    runtime_features.sort_unstable_by_key(|feature| feature.id.clone());

    assert_eq!(static_features, runtime_features);
}

#[derive(Debug, PartialEq, Eq)]
struct StaticOptionalFeatureManifest {
    id: String,
    display_name: String,
    owner_plugin_id: String,
    capabilities: Vec<String>,
    default_packaging: Vec<zircon_runtime::plugin::ExportPackagingStrategy>,
    enabled_by_default: bool,
    dependencies: Vec<(String, String, bool)>,
    modules: Vec<(
        String,
        zircon_runtime::plugin::PluginModuleKind,
        String,
        Vec<zircon_runtime::RuntimeTargetMode>,
        Vec<String>,
    )>,
}

#[derive(Default)]
struct PendingOptionalFeatureManifest {
    id: Option<String>,
    display_name: Option<String>,
    owner_plugin_id: Option<String>,
    capabilities: Vec<String>,
    default_packaging: Vec<zircon_runtime::plugin::ExportPackagingStrategy>,
    enabled_by_default: Option<bool>,
    dependencies: Vec<(String, String, bool)>,
    modules: Vec<(
        String,
        zircon_runtime::plugin::PluginModuleKind,
        String,
        Vec<zircon_runtime::RuntimeTargetMode>,
        Vec<String>,
    )>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OptionalFeatureSection {
    None,
    Feature,
    Dependency,
    Module,
}

fn optional_feature_signature(
    feature: &zircon_runtime::plugin::PluginFeatureBundleManifest,
) -> StaticOptionalFeatureManifest {
    let mut capabilities = feature.capabilities.clone();
    let mut dependencies = feature
        .dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.plugin_id.clone(),
                dependency.capability.clone(),
                dependency.primary,
            )
        })
        .collect::<Vec<_>>();
    let mut modules = feature
        .modules
        .iter()
        .map(|module| {
            (
                module.name.clone(),
                module.kind,
                module.crate_name.clone(),
                module.target_modes.clone(),
                module.capabilities.clone(),
            )
        })
        .collect::<Vec<_>>();
    capabilities.sort_unstable();
    dependencies.sort_unstable();
    modules.sort_unstable_by_key(|module| module.0.clone());

    StaticOptionalFeatureManifest {
        id: feature.id.clone(),
        display_name: feature.display_name.clone(),
        owner_plugin_id: feature.owner_plugin_id.clone(),
        capabilities,
        default_packaging: feature.default_packaging.clone(),
        enabled_by_default: feature.enabled_by_default,
        dependencies,
        modules,
    }
}

fn optional_features_from_plugin_toml(manifest: &str) -> Vec<StaticOptionalFeatureManifest> {
    let mut features = Vec::new();
    let mut current_feature: Option<PendingOptionalFeatureManifest> = None;
    let mut current_dependency_plugin_id = None;
    let mut current_dependency_capability = None;
    let mut current_dependency_primary = None;
    let mut current_module_name = None;
    let mut current_module_kind = None;
    let mut current_module_crate_name = None;
    let mut current_module_target_modes = Vec::new();
    let mut current_module_capabilities = Vec::new();
    let mut section = OptionalFeatureSection::None;

    for line in manifest.lines().map(str::trim) {
        match line {
            "[[optional_features]]" => {
                push_optional_feature_dependency(
                    &mut current_feature,
                    &mut current_dependency_plugin_id,
                    &mut current_dependency_capability,
                    &mut current_dependency_primary,
                );
                push_optional_feature_module(
                    &mut current_feature,
                    &mut current_module_name,
                    &mut current_module_kind,
                    &mut current_module_crate_name,
                    &mut current_module_target_modes,
                    &mut current_module_capabilities,
                );
                push_optional_feature(&mut features, &mut current_feature);
                current_feature = Some(PendingOptionalFeatureManifest::default());
                section = OptionalFeatureSection::Feature;
                continue;
            }
            "[[optional_features.dependencies]]" => {
                push_optional_feature_dependency(
                    &mut current_feature,
                    &mut current_dependency_plugin_id,
                    &mut current_dependency_capability,
                    &mut current_dependency_primary,
                );
                push_optional_feature_module(
                    &mut current_feature,
                    &mut current_module_name,
                    &mut current_module_kind,
                    &mut current_module_crate_name,
                    &mut current_module_target_modes,
                    &mut current_module_capabilities,
                );
                section = OptionalFeatureSection::Dependency;
                continue;
            }
            "[[optional_features.modules]]" => {
                push_optional_feature_dependency(
                    &mut current_feature,
                    &mut current_dependency_plugin_id,
                    &mut current_dependency_capability,
                    &mut current_dependency_primary,
                );
                push_optional_feature_module(
                    &mut current_feature,
                    &mut current_module_name,
                    &mut current_module_kind,
                    &mut current_module_crate_name,
                    &mut current_module_target_modes,
                    &mut current_module_capabilities,
                );
                section = OptionalFeatureSection::Module;
                continue;
            }
            _ if line.starts_with("[[") => {
                push_optional_feature_dependency(
                    &mut current_feature,
                    &mut current_dependency_plugin_id,
                    &mut current_dependency_capability,
                    &mut current_dependency_primary,
                );
                push_optional_feature_module(
                    &mut current_feature,
                    &mut current_module_name,
                    &mut current_module_kind,
                    &mut current_module_crate_name,
                    &mut current_module_target_modes,
                    &mut current_module_capabilities,
                );
                push_optional_feature(&mut features, &mut current_feature);
                section = OptionalFeatureSection::None;
                continue;
            }
            _ => {}
        }

        match section {
            OptionalFeatureSection::Feature => parse_optional_feature_line(
                line,
                current_feature
                    .as_mut()
                    .expect("optional feature table should have a current feature"),
            ),
            OptionalFeatureSection::Dependency => parse_optional_feature_dependency_line(
                line,
                &mut current_dependency_plugin_id,
                &mut current_dependency_capability,
                &mut current_dependency_primary,
            ),
            OptionalFeatureSection::Module => parse_optional_feature_module_line(
                line,
                &mut current_module_name,
                &mut current_module_kind,
                &mut current_module_crate_name,
                &mut current_module_target_modes,
                &mut current_module_capabilities,
            ),
            OptionalFeatureSection::None => {}
        }
    }

    push_optional_feature_dependency(
        &mut current_feature,
        &mut current_dependency_plugin_id,
        &mut current_dependency_capability,
        &mut current_dependency_primary,
    );
    push_optional_feature_module(
        &mut current_feature,
        &mut current_module_name,
        &mut current_module_kind,
        &mut current_module_crate_name,
        &mut current_module_target_modes,
        &mut current_module_capabilities,
    );
    push_optional_feature(&mut features, &mut current_feature);
    features
}

fn parse_optional_feature_line(line: &str, feature: &mut PendingOptionalFeatureManifest) {
    if let Some(value) = line
        .strip_prefix("id = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        feature.id = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("display_name = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        feature.display_name = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("owner_plugin_id = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        feature.owner_plugin_id = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("capabilities = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        feature.capabilities = string_array_values(value);
        return;
    }
    if let Some(value) = line
        .strip_prefix("default_packaging = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        feature.default_packaging = string_array_values(value)
            .into_iter()
            .map(packaging_strategy_from_plugin_toml)
            .collect();
        return;
    }
    if let Some(value) = line.strip_prefix("enabled_by_default = ") {
        feature.enabled_by_default = Some(bool_from_plugin_toml(value));
    }
}

fn parse_optional_feature_dependency_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    if let Some(value) = line
        .strip_prefix("plugin_id = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *plugin_id = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("capability = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *capability = Some(value.to_string());
        return;
    }
    if let Some(value) = line.strip_prefix("primary = ") {
        *primary = Some(bool_from_plugin_toml(value));
    }
}

fn parse_optional_feature_module_line(
    line: &str,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    if let Some(value) = line
        .strip_prefix("name = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *name = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("kind = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *kind = Some(plugin_module_kind_from_plugin_toml(value));
        return;
    }
    if let Some(value) = line
        .strip_prefix("crate_name = \"")
        .and_then(|value| value.strip_suffix('"'))
    {
        *crate_name = Some(value.to_string());
        return;
    }
    if let Some(value) = line
        .strip_prefix("target_modes = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        *target_modes = string_array_values(value)
            .into_iter()
            .map(runtime_target_mode_from_plugin_toml)
            .collect();
        return;
    }
    if let Some(value) = line
        .strip_prefix("capabilities = [")
        .and_then(|value| value.strip_suffix(']'))
    {
        *capabilities = string_array_values(value);
    }
}

fn push_optional_feature_dependency(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    let Some(plugin_id) = plugin_id.take() else {
        return;
    };
    feature
        .as_mut()
        .expect("optional feature dependency should have a parent feature")
        .dependencies
        .push((
            plugin_id,
            capability
                .take()
                .expect("optional feature dependency should declare capability"),
            primary
                .take()
                .expect("optional feature dependency should declare primary"),
        ));
}

fn push_optional_feature_module(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    let Some(name) = name.take() else {
        return;
    };
    feature
        .as_mut()
        .expect("optional feature module should have a parent feature")
        .modules
        .push((
            name,
            kind.take()
                .expect("optional feature module should declare kind"),
            crate_name
                .take()
                .expect("optional feature module should declare crate_name"),
            std::mem::take(target_modes),
            std::mem::take(capabilities),
        ));
}

fn push_optional_feature(
    features: &mut Vec<StaticOptionalFeatureManifest>,
    feature: &mut Option<PendingOptionalFeatureManifest>,
) {
    let Some(mut feature) = feature.take() else {
        return;
    };
    feature.capabilities.sort_unstable();
    feature.dependencies.sort_unstable();
    feature
        .modules
        .sort_unstable_by_key(|module| module.0.clone());
    features.push(StaticOptionalFeatureManifest {
        id: feature.id.expect("optional feature should declare id"),
        display_name: feature
            .display_name
            .expect("optional feature should declare display name"),
        owner_plugin_id: feature
            .owner_plugin_id
            .expect("optional feature should declare owner plugin id"),
        capabilities: feature.capabilities,
        default_packaging: feature.default_packaging,
        enabled_by_default: feature.enabled_by_default.unwrap_or(false),
        dependencies: feature.dependencies,
        modules: feature.modules,
    });
}

fn string_array_values(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter_map(|entry| entry.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}

fn bool_from_plugin_toml(value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("unknown sound boolean value {value}"),
    }
}

fn packaging_strategy_from_plugin_toml(
    value: String,
) -> zircon_runtime::plugin::ExportPackagingStrategy {
    match value.as_str() {
        "source_template" => zircon_runtime::plugin::ExportPackagingStrategy::SourceTemplate,
        "library_embed" => zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
        "native_dynamic" => zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
        _ => panic!("unknown sound packaging strategy {value}"),
    }
}

fn plugin_module_kind_from_plugin_toml(value: &str) -> zircon_runtime::plugin::PluginModuleKind {
    match value {
        "runtime" => zircon_runtime::plugin::PluginModuleKind::Runtime,
        "editor" => zircon_runtime::plugin::PluginModuleKind::Editor,
        "native" => zircon_runtime::plugin::PluginModuleKind::Native,
        "vm" => zircon_runtime::plugin::PluginModuleKind::Vm,
        _ => panic!("unknown sound module kind {value}"),
    }
}

fn runtime_target_mode_from_plugin_toml(value: String) -> zircon_runtime::RuntimeTargetMode {
    match value.as_str() {
        "client_runtime" => zircon_runtime::RuntimeTargetMode::ClientRuntime,
        "editor_host" => zircon_runtime::RuntimeTargetMode::EditorHost,
        "server_runtime" => zircon_runtime::RuntimeTargetMode::ServerRuntime,
        _ => panic!("unknown sound module target mode {value}"),
    }
}
