#[test]
fn static_plugin_manifest_keeps_runtime_option_keys_in_sync() {
    let mut static_keys = option_keys_from_plugin_toml(include_str!("../../../plugin.toml"));
    let runtime_options = crate::sound_options();
    let mut runtime_keys = runtime_options
        .iter()
        .map(|option| option.key.clone())
        .collect::<Vec<_>>();
    static_keys.sort_unstable();
    runtime_keys.sort_unstable();

    assert_eq!(static_keys, runtime_keys);
}

#[test]
fn static_plugin_manifest_keeps_runtime_option_metadata_in_sync() {
    let mut static_options =
        option_manifests_from_plugin_toml(include_str!("../../../plugin.toml"))
            .into_iter()
            .map(option_manifest_tuple)
            .collect::<Vec<_>>();
    let mut runtime_options = crate::sound_options()
        .into_iter()
        .map(option_manifest_tuple)
        .collect::<Vec<_>>();
    static_options.sort_unstable_by_key(|option| option.0.clone());
    runtime_options.sort_unstable_by_key(|option| option.0.clone());

    assert_eq!(static_options, runtime_options);
}

#[test]
fn static_plugin_manifest_keeps_runtime_contribution_keys_in_sync() {
    let static_manifest = static_sound_contributions(include_str!("../../../plugin.toml"));
    let runtime_manifest = crate::package_manifest();
    let mut runtime_dependencies = runtime_manifest
        .dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.id.clone(),
                dependency.required,
                dependency.capability.clone(),
            )
        })
        .collect::<Vec<_>>();
    let mut runtime_event_catalogs = runtime_manifest
        .event_catalogs
        .iter()
        .map(|catalog| (catalog.namespace.clone(), catalog.version))
        .collect::<Vec<_>>();
    let mut runtime_modules = runtime_manifest
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
    let mut runtime_components = runtime_manifest
        .components
        .iter()
        .map(|component| component.type_id.clone())
        .collect::<Vec<_>>();
    let mut descriptor_components = crate::components::sound_component_descriptors()
        .into_iter()
        .map(|component| component.type_id)
        .collect::<Vec<_>>();

    runtime_dependencies.sort_unstable();
    runtime_event_catalogs.sort_unstable();
    runtime_modules.sort_unstable_by_key(|module| module.0.clone());
    runtime_components.sort_unstable();
    descriptor_components.sort_unstable();

    assert_eq!(static_manifest.dependencies, runtime_dependencies);
    assert_eq!(static_manifest.event_catalogs, runtime_event_catalogs);
    let sound_event_catalog = runtime_manifest
        .event_catalogs
        .iter()
        .find(|catalog| catalog.namespace == crate::SOUND_DYNAMIC_EVENT_NAMESPACE)
        .expect("sound dynamic event catalog");
    assert_eq!(
        sound_event_catalog
            .events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "sound.dynamic_events.impact",
            "sound.dynamic_events.marker",
            "sound.dynamic_events.ambient_stinger",
        ]
    );
    assert_eq!(static_manifest.modules, runtime_modules);
    assert_eq!(descriptor_components.len(), 3);
    assert_eq!(descriptor_components, runtime_components);
}

#[test]
fn runtime_descriptor_keeps_static_maturity_and_capability_status_in_sync() {
    let static_manifest = static_plugin_metadata(include_str!("../../../plugin.toml"));
    let descriptor = crate::runtime_plugin_descriptor();
    let runtime_manifest = crate::package_manifest();
    let catalog_descriptor = zircon_runtime::plugin::RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id == zircon_runtime::RuntimePluginId::Sound)
        .expect("built-in runtime catalog should include sound");

    assert_eq!(static_manifest.maturity, descriptor.maturity);
    assert_eq!(static_manifest.maturity, runtime_manifest.maturity);
    assert_eq!(static_manifest.maturity, catalog_descriptor.maturity);
    assert_eq!(
        static_manifest.capability_statuses,
        descriptor.capability_statuses
    );
    assert_eq!(
        static_manifest.capability_statuses,
        runtime_manifest.capability_statuses
    );
    assert_eq!(
        static_manifest.capability_statuses,
        catalog_descriptor.capability_statuses
    );
    assert!(runtime_manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.sound"
            && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
    }));
}

struct StaticSoundPluginMetadata {
    maturity: zircon_runtime::plugin::PluginMaturity,
    capability_statuses: Vec<zircon_runtime::plugin::CapabilityStatusManifest>,
}

struct StaticSoundContributions {
    dependencies: Vec<(String, bool, Option<String>)>,
    event_catalogs: Vec<(String, u32)>,
    modules: Vec<(
        String,
        zircon_runtime::plugin::PluginModuleKind,
        String,
        Vec<zircon_runtime::RuntimeTargetMode>,
        Vec<String>,
    )>,
}

fn static_plugin_metadata(manifest: &str) -> StaticSoundPluginMetadata {
    StaticSoundPluginMetadata {
        maturity: maturity_from_plugin_toml(manifest),
        capability_statuses: capability_statuses_from_plugin_toml(manifest),
    }
}

fn static_sound_contributions(manifest: &str) -> StaticSoundContributions {
    let mut dependencies = dependencies_from_plugin_toml(manifest);
    let mut event_catalogs = event_catalogs_from_plugin_toml(manifest);
    let mut modules = modules_from_plugin_toml(manifest)
        .into_iter()
        .filter(|module| module.1 == zircon_runtime::plugin::PluginModuleKind::Runtime)
        .collect::<Vec<_>>();
    dependencies.sort_unstable();
    event_catalogs.sort_unstable();
    modules.sort_unstable_by_key(|module| module.0.clone());

    StaticSoundContributions {
        dependencies,
        event_catalogs,
        modules,
    }
}

fn maturity_from_plugin_toml(manifest: &str) -> zircon_runtime::plugin::PluginMaturity {
    for line in manifest.lines().map(str::trim) {
        let Some(value) = line
            .strip_prefix("maturity = \"")
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        return match value {
            "core" => zircon_runtime::plugin::PluginMaturity::Core,
            "stable" => zircon_runtime::plugin::PluginMaturity::Stable,
            "beta" => zircon_runtime::plugin::PluginMaturity::Beta,
            "experimental" => zircon_runtime::plugin::PluginMaturity::Experimental,
            "externalized" => zircon_runtime::plugin::PluginMaturity::Externalized,
            "stub" => zircon_runtime::plugin::PluginMaturity::Stub,
            "deprecated" => zircon_runtime::plugin::PluginMaturity::Deprecated,
            _ => panic!("unknown sound plugin maturity {value}"),
        };
    }
    panic!("sound plugin.toml should declare maturity")
}

fn dependencies_from_plugin_toml(manifest: &str) -> Vec<(String, bool, Option<String>)> {
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
            current_required = Some(match value {
                "true" => true,
                "false" => false,
                _ => panic!("unknown sound dependency required value {value}"),
            });
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
    dependencies: &mut Vec<(String, bool, Option<String>)>,
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

fn event_catalogs_from_plugin_toml(manifest: &str) -> Vec<(String, u32)> {
    let mut catalogs = Vec::new();
    let mut current_namespace = None;
    let mut current_version = None;
    let mut inside_catalog = false;

    for line in manifest.lines().map(str::trim) {
        if line == "[[event_catalogs]]" {
            push_event_catalog(&mut catalogs, &mut current_namespace, &mut current_version);
            inside_catalog = true;
            continue;
        }
        if line.starts_with("[[") {
            push_event_catalog(&mut catalogs, &mut current_namespace, &mut current_version);
            inside_catalog = false;
        }
        if !inside_catalog {
            continue;
        }
        if let Some(value) = line
            .strip_prefix("namespace = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_namespace = Some(value.to_string());
            continue;
        }
        if let Some(value) = line.strip_prefix("version = ") {
            current_version = Some(
                value
                    .parse::<u32>()
                    .expect("sound event catalog version should be an integer"),
            );
        }
    }
    push_event_catalog(&mut catalogs, &mut current_namespace, &mut current_version);
    catalogs
}

fn modules_from_plugin_toml(
    manifest: &str,
) -> Vec<(
    String,
    zircon_runtime::plugin::PluginModuleKind,
    String,
    Vec<zircon_runtime::RuntimeTargetMode>,
    Vec<String>,
)> {
    let mut modules = Vec::new();
    let mut current_name = None;
    let mut current_kind = None;
    let mut current_crate_name = None;
    let mut current_target_modes = Vec::new();
    let mut current_capabilities = Vec::new();
    let mut inside_module = false;

    for line in manifest.lines().map(str::trim) {
        if line == "[[modules]]" {
            push_module(
                &mut modules,
                &mut current_name,
                &mut current_kind,
                &mut current_crate_name,
                &mut current_target_modes,
                &mut current_capabilities,
            );
            inside_module = true;
            continue;
        }
        if line.starts_with("[[") {
            push_module(
                &mut modules,
                &mut current_name,
                &mut current_kind,
                &mut current_crate_name,
                &mut current_target_modes,
                &mut current_capabilities,
            );
            inside_module = false;
        }
        if !inside_module {
            continue;
        }
        if let Some(value) = line
            .strip_prefix("name = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_name = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("kind = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_kind = Some(match value {
                "runtime" => zircon_runtime::plugin::PluginModuleKind::Runtime,
                "editor" => zircon_runtime::plugin::PluginModuleKind::Editor,
                "native" => zircon_runtime::plugin::PluginModuleKind::Native,
                "vm" => zircon_runtime::plugin::PluginModuleKind::Vm,
                _ => panic!("unknown sound module kind {value}"),
            });
            continue;
        }
        if let Some(value) = line
            .strip_prefix("crate_name = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_crate_name = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("target_modes = [")
            .and_then(|value| value.strip_suffix(']'))
        {
            current_target_modes = string_array_values(value)
                .into_iter()
                .map(|mode| match mode.as_str() {
                    "client_runtime" => zircon_runtime::RuntimeTargetMode::ClientRuntime,
                    "editor_host" => zircon_runtime::RuntimeTargetMode::EditorHost,
                    "server_runtime" => zircon_runtime::RuntimeTargetMode::ServerRuntime,
                    _ => panic!("unknown sound module target mode {mode}"),
                })
                .collect();
            continue;
        }
        if let Some(value) = line
            .strip_prefix("capabilities = [")
            .and_then(|value| value.strip_suffix(']'))
        {
            current_capabilities = string_array_values(value).into_iter().collect();
        }
    }
    push_module(
        &mut modules,
        &mut current_name,
        &mut current_kind,
        &mut current_crate_name,
        &mut current_target_modes,
        &mut current_capabilities,
    );
    modules
}

fn push_module(
    modules: &mut Vec<(
        String,
        zircon_runtime::plugin::PluginModuleKind,
        String,
        Vec<zircon_runtime::RuntimeTargetMode>,
        Vec<String>,
    )>,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    let Some(name) = name.take() else {
        return;
    };
    modules.push((
        name,
        kind.take().expect("sound module should declare kind"),
        crate_name
            .take()
            .expect("sound module should declare crate_name"),
        std::mem::take(target_modes),
        std::mem::take(capabilities),
    ));
}

fn push_event_catalog(
    catalogs: &mut Vec<(String, u32)>,
    namespace: &mut Option<String>,
    version: &mut Option<u32>,
) {
    let Some(namespace) = namespace.take() else {
        return;
    };
    catalogs.push((
        namespace,
        version
            .take()
            .expect("sound event catalog should declare version"),
    ));
}

fn string_array_values(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter_map(|entry| entry.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}

fn capability_statuses_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::CapabilityStatusManifest> {
    let mut statuses = Vec::new();
    let mut current_capability = None;
    let mut current_status = None;
    let mut current_bevy_references = Vec::new();
    let mut inside_status = false;

    for line in manifest.lines().map(str::trim) {
        if line == "[[capability_statuses]]" {
            push_capability_status(
                &mut statuses,
                &mut current_capability,
                &mut current_status,
                &mut current_bevy_references,
            );
            inside_status = true;
            continue;
        }
        if line.starts_with("[[") {
            push_capability_status(
                &mut statuses,
                &mut current_capability,
                &mut current_status,
                &mut current_bevy_references,
            );
            inside_status = false;
        }
        if !inside_status {
            continue;
        }
        if let Some(value) = line
            .strip_prefix("capability = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_capability = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("status = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_status = Some(match value {
                "complete" => zircon_runtime::plugin::CapabilityStatus::Complete,
                "partial" => zircon_runtime::plugin::CapabilityStatus::Partial,
                "stub" => zircon_runtime::plugin::CapabilityStatus::Stub,
                "externalized" => zircon_runtime::plugin::CapabilityStatus::Externalized,
                "unsupported" => zircon_runtime::plugin::CapabilityStatus::Unsupported,
                _ => panic!("unknown sound capability status {value}"),
            });
            continue;
        }
        if let Some(value) = line
            .strip_prefix("bevy_references = [")
            .and_then(|value| value.strip_suffix(']'))
        {
            current_bevy_references = string_array_values(value).into_iter().collect();
        }
    }
    push_capability_status(
        &mut statuses,
        &mut current_capability,
        &mut current_status,
        &mut current_bevy_references,
    );
    statuses
}

fn push_capability_status(
    statuses: &mut Vec<zircon_runtime::plugin::CapabilityStatusManifest>,
    capability: &mut Option<String>,
    status: &mut Option<zircon_runtime::plugin::CapabilityStatus>,
    bevy_references: &mut Vec<String>,
) {
    let Some(capability) = capability.take() else {
        return;
    };
    let mut manifest = zircon_runtime::plugin::CapabilityStatusManifest::new(
        capability,
        status
            .take()
            .expect("sound capability status should declare status"),
    );
    for reference in bevy_references.drain(..) {
        manifest = manifest.with_bevy_reference(reference);
    }
    statuses.push(manifest);
}

fn option_keys_from_plugin_toml(manifest: &str) -> Vec<String> {
    option_manifests_from_plugin_toml(manifest)
        .into_iter()
        .map(|option| option.key)
        .collect()
}

fn option_manifests_from_plugin_toml(
    manifest: &str,
) -> Vec<zircon_runtime::plugin::PluginOptionManifest> {
    let mut options = Vec::new();
    let mut key = None;
    let mut display_name = None;
    let mut value_type = None;
    let mut default_value = None;
    let mut required_capability = None;
    let mut inside_option = false;

    for line in manifest.lines().map(str::trim) {
        if line == "[[options]]" {
            push_option_manifest(
                &mut options,
                &mut key,
                &mut display_name,
                &mut value_type,
                &mut default_value,
                &mut required_capability,
            );
            inside_option = true;
            continue;
        }
        if line.starts_with("[[") {
            push_option_manifest(
                &mut options,
                &mut key,
                &mut display_name,
                &mut value_type,
                &mut default_value,
                &mut required_capability,
            );
            inside_option = false;
        }
        if !inside_option {
            continue;
        }
        if let Some(value) = line
            .strip_prefix("key = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            key = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("display_name = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            display_name = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("value_type = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            value_type = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("default_value = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            default_value = Some(value.to_string());
            continue;
        }
        if let Some(value) = line
            .strip_prefix("required_capability = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            required_capability = Some(value.to_string());
        }
    }
    push_option_manifest(
        &mut options,
        &mut key,
        &mut display_name,
        &mut value_type,
        &mut default_value,
        &mut required_capability,
    );
    options
}

fn push_option_manifest(
    options: &mut Vec<zircon_runtime::plugin::PluginOptionManifest>,
    key: &mut Option<String>,
    display_name: &mut Option<String>,
    value_type: &mut Option<String>,
    default_value: &mut Option<String>,
    required_capability: &mut Option<String>,
) {
    let Some(key) = key.take() else {
        return;
    };
    let mut option = zircon_runtime::plugin::PluginOptionManifest::new(
        key,
        display_name
            .take()
            .expect("sound option should declare display_name"),
        value_type
            .take()
            .expect("sound option should declare value_type"),
        default_value
            .take()
            .expect("sound option should declare default_value"),
    );
    if let Some(capability) = required_capability.take() {
        option = option.with_required_capability(capability);
    }
    options.push(option);
}

fn option_manifest_tuple(
    option: zircon_runtime::plugin::PluginOptionManifest,
) -> (String, String, String, String, Option<String>) {
    (
        option.key,
        option.display_name,
        option.value_type,
        option.default_value,
        option.required_capability,
    )
}
