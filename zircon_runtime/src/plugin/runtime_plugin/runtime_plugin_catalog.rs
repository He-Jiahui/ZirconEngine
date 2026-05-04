use std::collections::{HashMap, HashSet};

use crate::{
    plugin::PluginFeatureBundleManifest, plugin::PluginModuleKind, plugin::PluginPackageKind,
    plugin::PluginPackageManifest, plugin::ProjectPluginFeatureSelection,
    plugin::ProjectPluginManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError, plugin::RuntimePlugin,
    plugin::RuntimePluginFeature, RuntimeTargetMode,
};

use super::runtime_plugin_feature_registration_report::project_selection_from_feature_manifest;
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

#[derive(Clone, Debug, Default)]
pub struct RuntimePluginCatalog {
    registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
    diagnostics: Vec<String>,
}

impl RuntimePluginCatalog {
    pub fn from_plugins<'a>(plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>) -> Self {
        let mut catalog = Self::default();
        for plugin in plugins {
            catalog.register(plugin);
        }
        catalog
    }

    pub fn from_descriptors(
        descriptors: impl IntoIterator<Item = super::RuntimePluginDescriptor>,
    ) -> Self {
        let mut catalog = Self::default();
        for descriptor in descriptors {
            catalog.register(&descriptor);
        }
        catalog
    }

    pub fn from_registration_reports(
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        let mut catalog = Self::default();
        for registration in registrations {
            catalog
                .diagnostics
                .extend(registration.diagnostics.iter().cloned());
            catalog.registrations.push(registration);
        }
        for registration in feature_registrations {
            catalog
                .diagnostics
                .extend(registration.diagnostics.iter().cloned());
            catalog.feature_registrations.push(registration);
        }
        catalog
    }

    pub fn builtin() -> Self {
        Self::from_descriptors(super::RuntimePluginDescriptor::builtin_catalog())
    }

    pub fn register(&mut self, plugin: &dyn RuntimePlugin) {
        let report = RuntimePluginRegistrationReport::from_plugin(plugin);
        self.diagnostics.extend(report.diagnostics.iter().cloned());
        self.registrations.push(report);
    }

    pub fn register_feature(&mut self, feature: &dyn RuntimePluginFeature) {
        let report = RuntimePluginFeatureRegistrationReport::from_feature(feature);
        self.diagnostics.extend(report.diagnostics.iter().cloned());
        self.feature_registrations.push(report);
    }

    pub fn registrations(&self) -> &[RuntimePluginRegistrationReport] {
        &self.registrations
    }

    pub fn feature_registrations(&self) -> &[RuntimePluginFeatureRegistrationReport] {
        &self.feature_registrations
    }

    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        self.registrations
            .iter()
            .map(|registration| registration.package_manifest.clone())
            .collect()
    }

    pub fn project_manifest(&self) -> ProjectPluginManifest {
        self.complete_project_manifest(&ProjectPluginManifest {
            selections: self
                .registrations
                .iter()
                .map(|registration| registration.project_selection.clone())
                .collect(),
        })
    }

    pub fn complete_project_manifest(
        &self,
        manifest: &ProjectPluginManifest,
    ) -> ProjectPluginManifest {
        let mut completed = manifest.clone();
        for registration in &self.registrations {
            if completed
                .selections
                .iter()
                .any(|selection| selection.id == registration.project_selection.id)
            {
                continue;
            }
            let mut selection = registration.project_selection.clone();
            selection.enabled = false;
            completed.selections.push(selection);
        }
        for selection in &mut completed.selections {
            if let Some(catalog_selection) = self.project_selection_for_package(&selection.id) {
                if selection.runtime_crate.is_none() {
                    selection.runtime_crate = catalog_selection.runtime_crate.clone();
                }
                if selection.editor_crate.is_none() {
                    selection.editor_crate = catalog_selection.editor_crate.clone();
                }
                if selection.target_modes.is_empty() {
                    selection.target_modes = catalog_selection.target_modes.clone();
                }
            }
        }
        let feature_definitions = self.feature_definition_map();
        for selection in &mut completed.selections {
            let owner_id = selection.id.clone();
            for feature_key in &feature_definitions.definition_order {
                let Some(feature_definition) = feature_definitions.definitions.get(feature_key)
                else {
                    continue;
                };
                let feature = &feature_definition.manifest;
                if feature.owner_plugin_id != owner_id {
                    continue;
                }
                complete_owner_feature_selection(
                    selection,
                    feature,
                    feature_definition.external_provider_for_owner(),
                );
            }
        }
        completed
    }

    pub fn project_selection_for_package(
        &self,
        package_id: &str,
    ) -> Option<ProjectPluginSelection> {
        self.registrations
            .iter()
            .find(|registration| registration.package_manifest.id == package_id)
            .map(|registration| registration.project_selection.clone())
    }

    pub fn feature_dependency_report(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> RuntimePluginFeatureDependencyReport {
        let completed = self.complete_project_manifest(manifest);
        let feature_definitions = self.feature_definition_map();
        let plugin_selections = completed
            .selections
            .iter()
            .map(|selection| (selection.id.as_str(), selection))
            .collect::<HashMap<_, _>>();
        let enabled_plugins = completed
            .enabled_for_target(target)
            .map(|selection| selection.id.clone())
            .collect::<HashSet<_>>();
        let mut available_capabilities =
            self.base_capabilities_for_target(&enabled_plugins, target);
        let active_features = active_feature_selections(&completed);
        let mut report = RuntimePluginFeatureDependencyReport {
            diagnostics: feature_definitions.diagnostics.clone(),
            ..RuntimePluginFeatureDependencyReport::default()
        };

        let mut pending = Vec::new();
        for active in active_features {
            if let Some(feature_definition) = feature_definitions.definition_for_selection(&active)
            {
                pending.push(PendingFeatureSelection {
                    active,
                    definition_key: feature_definition.key.clone(),
                });
            } else {
                report.blocked_features.push(RuntimePluginFeatureBlock {
                    feature_id: active.feature.id.clone(),
                    owner_plugin_id: active.owner_plugin_id.clone(),
                    required: active.feature.required,
                    unknown_feature: true,
                    ..RuntimePluginFeatureBlock::default()
                });
            }
        }

        let mut made_progress = true;
        while made_progress && !pending.is_empty() {
            made_progress = false;
            let mut index = 0;
            while index < pending.len() {
                let active = &pending[index];
                let feature = feature_definitions
                    .definitions
                    .get(&active.definition_key)
                    .expect("unknown features removed before dependency resolution");
                let status = feature_status(
                    feature,
                    active.active.feature,
                    target,
                    &plugin_selections,
                    &enabled_plugins,
                    &available_capabilities,
                );
                if status.is_available() {
                    let active = pending.remove(index);
                    report
                        .available_features
                        .push(active.active.feature.id.clone());
                    extend_unique(
                        &mut available_capabilities,
                        feature_capabilities_for_target(&feature.manifest, target),
                    );
                    made_progress = true;
                } else if status.is_immediately_blocked() {
                    let active = pending.remove(index);
                    report
                        .blocked_features
                        .push(status.into_block(active.active.feature));
                } else {
                    index += 1;
                }
            }
        }

        let unresolved_feature_ids = pending
            .iter()
            .map(|active| active.definition_key.clone())
            .collect::<HashSet<_>>();
        for active in pending {
            let feature = feature_definitions
                .definitions
                .get(&active.definition_key)
                .expect("unknown features removed before dependency resolution");
            let mut status = feature_status(
                feature,
                active.active.feature,
                target,
                &plugin_selections,
                &enabled_plugins,
                &available_capabilities,
            );
            if status.is_waiting_for_feature_capability(
                &feature_definitions.definitions,
                &unresolved_feature_ids,
                target,
            ) {
                status.cycle = true;
            }
            report
                .blocked_features
                .push(status.into_block(active.active.feature));
        }

        report
    }

    pub fn runtime_extensions(&self) -> RuntimeExtensionCatalogReport {
        let mut registry = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        let mut fatal_diagnostics = Vec::new();
        for registration in &self.registrations {
            merge_runtime_extensions(
                registration,
                &mut registry,
                &mut diagnostics,
                &mut fatal_diagnostics,
            );
        }
        RuntimeExtensionCatalogReport {
            registry,
            diagnostics,
            fatal_diagnostics,
        }
    }

    pub fn runtime_extensions_for_project(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> RuntimeExtensionCatalogReport {
        let completed = self.complete_project_manifest(manifest);
        let enabled_plugins = completed
            .enabled_for_target(target)
            .map(|selection| selection.id.clone())
            .collect::<HashSet<_>>();
        let mut registry = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        let mut fatal_diagnostics = Vec::new();
        for registration in self
            .registrations
            .iter()
            .filter(|registration| enabled_plugins.contains(&registration.package_manifest.id))
        {
            merge_runtime_extensions(
                registration,
                &mut registry,
                &mut diagnostics,
                &mut fatal_diagnostics,
            );
        }
        let feature_report = self.feature_dependency_report(&completed, target);
        diagnostics.extend(feature_report.diagnostics.iter().cloned());
        fatal_diagnostics.extend(feature_report.diagnostics.iter().cloned());
        for blocked in &feature_report.blocked_features {
            let diagnostic = blocked.to_diagnostic();
            if blocked.required {
                fatal_diagnostics.push(diagnostic.clone());
            }
            diagnostics.push(diagnostic);
        }
        for feature_id in &feature_report.available_features {
            if let Some(registration) = self.feature_registrations.iter().find(|registration| {
                registration.manifest.id == *feature_id
                    && feature_registration_matches_project_selection(
                        registration,
                        &completed,
                        feature_id,
                    )
            }) {
                merge_feature_extensions(
                    registration,
                    &mut registry,
                    &mut diagnostics,
                    &mut fatal_diagnostics,
                );
            }
        }
        RuntimeExtensionCatalogReport {
            registry,
            diagnostics,
            fatal_diagnostics,
        }
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }

    fn feature_definition_map(&self) -> FeatureDefinitionMap {
        let mut definitions = HashMap::new();
        let mut diagnostics = Vec::new();
        let mut definition_order = Vec::new();
        let mut declared_feature_ids = HashSet::new();
        let mut registered_feature_ids = HashSet::new();
        for registration in &self.registrations {
            for feature_definition in package_feature_definitions(&registration.package_manifest) {
                let key = feature_definition.key.clone();
                declared_feature_ids.insert(key.clone());
                if definitions
                    .insert(key.clone(), feature_definition)
                    .is_some()
                {
                    diagnostics.push(format!(
                        "duplicate optional feature provider {} declared in plugin catalog",
                        key
                    ));
                } else {
                    definition_order.push(key);
                }
            }
        }
        for registration in &self.feature_registrations {
            let feature_definition = FeatureDefinition::from_runtime_registration(registration);
            let key = feature_definition.key.clone();
            if !registered_feature_ids.insert(key.clone()) {
                diagnostics.push(format!(
                    "duplicate optional feature id {} registered at runtime (provider {})",
                    registration.manifest.id, feature_definition.provider_package_id
                ));
                continue;
            }
            if declared_feature_ids.contains(&key) {
                if let Some(declared) = definitions.get(&key) {
                    if !feature_definition_registration_matches(
                        &declared.manifest,
                        &registration.manifest,
                    ) {
                        diagnostics.push(format!(
                            "optional feature id {} has conflicting package manifest and runtime registration",
                            registration.manifest.id
                        ));
                    }
                }
                continue;
            }
            if definitions
                .insert(key.clone(), feature_definition)
                .is_some()
            {
                diagnostics.push(format!(
                    "duplicate optional feature provider {} declared or registered in plugin catalog",
                    key
                ));
            } else {
                definition_order.push(key);
            }
        }
        FeatureDefinitionMap {
            definitions,
            diagnostics,
            definition_order,
        }
    }

    fn base_capabilities_for_target(
        &self,
        enabled_plugins: &HashSet<String>,
        target: RuntimeTargetMode,
    ) -> HashSet<String> {
        let mut capabilities = HashSet::new();
        for registration in &self.registrations {
            if !enabled_plugins.contains(&registration.package_manifest.id) {
                continue;
            }
            for module in &registration.package_manifest.modules {
                if module.target_modes.is_empty() || module.target_modes.contains(&target) {
                    extend_unique(&mut capabilities, module.capabilities.iter().cloned());
                }
            }
        }
        capabilities
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginFeatureDependencyReport {
    pub available_features: Vec<String>,
    pub blocked_features: Vec<RuntimePluginFeatureBlock>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginFeatureBlock {
    pub feature_id: String,
    pub owner_plugin_id: String,
    pub required: bool,
    pub missing_plugins: Vec<String>,
    pub missing_capabilities: Vec<String>,
    pub target_unsupported: bool,
    pub cycle: bool,
    pub invalid_owner_dependency: bool,
    pub unknown_feature: bool,
}

impl RuntimePluginFeatureBlock {
    pub fn to_diagnostic(&self) -> String {
        let severity = if self.required {
            "required feature"
        } else {
            "optional feature"
        };
        let mut details = Vec::new();
        if self.unknown_feature {
            details.push("feature is not declared by the plugin catalog".to_string());
        }
        if self.invalid_owner_dependency {
            details.push(
                "owner dependency is missing, not marked primary, or not the only primary dependency"
                    .to_string(),
            );
        }
        if self.target_unsupported {
            details.push("target mode is not supported".to_string());
        }
        if !self.missing_plugins.is_empty() {
            details.push(format!(
                "missing plugins: {}",
                self.missing_plugins.join(", ")
            ));
        }
        if !self.missing_capabilities.is_empty() {
            details.push(format!(
                "missing capabilities: {}",
                self.missing_capabilities.join(", ")
            ));
        }
        if self.cycle {
            details.push("feature capability dependencies form a cycle".to_string());
        }
        if details.is_empty() {
            details.push("dependency status is unresolved".to_string());
        }
        format!(
            "{severity} {} is blocked: {}",
            self.feature_id,
            details.join("; ")
        )
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeExtensionCatalogReport {
    pub registry: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}

impl RuntimeExtensionCatalogReport {
    pub fn is_success(&self) -> bool {
        self.fatal_diagnostics.is_empty()
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.fatal_diagnostics.is_empty()
    }
}

#[derive(Clone, Debug)]
struct FeatureDefinitionMap {
    definitions: HashMap<String, FeatureDefinition>,
    diagnostics: Vec<String>,
    definition_order: Vec<String>,
}

impl FeatureDefinitionMap {
    fn definition_for_selection(
        &self,
        active: &ActiveFeatureSelection<'_>,
    ) -> Option<&FeatureDefinition> {
        let requested_provider = active
            .feature
            .provider_package_id
            .as_deref()
            .unwrap_or(active.owner_plugin_id.as_str());
        let preferred_key = feature_definition_key(&active.feature.id, requested_provider);
        if let Some(definition) = self.definitions.get(&preferred_key) {
            return Some(definition);
        }
        if active.feature.provider_package_id.is_some() {
            return None;
        }
        self.definitions
            .values()
            .filter(|definition| definition.manifest.id == active.feature.id)
            .single()
    }
}

trait SingleDefinition<'a> {
    fn single(self) -> Option<&'a FeatureDefinition>;
}

impl<'a, I> SingleDefinition<'a> for I
where
    I: Iterator<Item = &'a FeatureDefinition>,
{
    fn single(mut self) -> Option<&'a FeatureDefinition> {
        let value = self.next()?;
        if self.next().is_some() {
            return None;
        }
        Some(value)
    }
}

#[derive(Clone, Debug)]
struct FeatureDefinition {
    key: String,
    manifest: PluginFeatureBundleManifest,
    provider_package_id: String,
}

impl FeatureDefinition {
    fn new(manifest: PluginFeatureBundleManifest, provider_package_id: String) -> Self {
        let key = feature_definition_key(&manifest.id, &provider_package_id);
        Self {
            key,
            manifest,
            provider_package_id,
        }
    }

    fn from_runtime_registration(registration: &RuntimePluginFeatureRegistrationReport) -> Self {
        Self::new(
            registration.manifest.clone(),
            registration.provider_package_id_or_owner().to_string(),
        )
    }

    fn external_provider_for_owner(&self) -> Option<&str> {
        (self.provider_package_id != self.manifest.owner_plugin_id)
            .then_some(self.provider_package_id.as_str())
    }
}

#[derive(Clone, Debug)]
struct ActiveFeatureSelection<'a> {
    owner_plugin_id: String,
    feature: &'a ProjectPluginFeatureSelection,
}

#[derive(Clone, Debug)]
struct PendingFeatureSelection<'a> {
    active: ActiveFeatureSelection<'a>,
    definition_key: String,
}

#[derive(Clone, Debug, Default)]
struct FeatureStatus {
    feature_id: String,
    owner_plugin_id: String,
    missing_plugins: Vec<String>,
    missing_capabilities: Vec<String>,
    target_unsupported: bool,
    cycle: bool,
    invalid_owner_dependency: bool,
}

impl FeatureStatus {
    fn is_available(&self) -> bool {
        self.missing_plugins.is_empty()
            && self.missing_capabilities.is_empty()
            && !self.target_unsupported
            && !self.cycle
            && !self.invalid_owner_dependency
    }

    fn is_immediately_blocked(&self) -> bool {
        !self.missing_plugins.is_empty() || self.target_unsupported || self.invalid_owner_dependency
    }

    fn is_waiting_for_feature_capability(
        &self,
        definitions: &HashMap<String, FeatureDefinition>,
        unresolved_feature_ids: &HashSet<String>,
        target: RuntimeTargetMode,
    ) -> bool {
        !self.missing_capabilities.is_empty()
            && self.missing_plugins.is_empty()
            && !self.target_unsupported
            && !self.invalid_owner_dependency
            && self.missing_capabilities.iter().all(|capability| {
                definitions.iter().any(|(key, candidate)| {
                    unresolved_feature_ids.contains(key)
                        && feature_declares_capability_for_target(candidate, capability, target)
                })
            })
    }

    fn into_block(self, selection: &ProjectPluginFeatureSelection) -> RuntimePluginFeatureBlock {
        RuntimePluginFeatureBlock {
            feature_id: self.feature_id,
            owner_plugin_id: self.owner_plugin_id,
            required: selection.required,
            missing_plugins: self.missing_plugins,
            missing_capabilities: self.missing_capabilities,
            target_unsupported: self.target_unsupported,
            cycle: self.cycle,
            invalid_owner_dependency: self.invalid_owner_dependency,
            unknown_feature: false,
        }
    }
}

fn feature_definition_registration_matches(
    declared: &PluginFeatureBundleManifest,
    registered: &PluginFeatureBundleManifest,
) -> bool {
    declared.id == registered.id
        && declared.owner_plugin_id == registered.owner_plugin_id
        && declared.dependencies == registered.dependencies
        && declared.modules == registered.modules
        && declared.capabilities == registered.capabilities
        && declared.default_packaging == registered.default_packaging
        && declared.enabled_by_default == registered.enabled_by_default
}

fn package_feature_definitions(package_manifest: &PluginPackageManifest) -> Vec<FeatureDefinition> {
    let mut definitions = Vec::new();
    for feature in &package_manifest.optional_features {
        let provider_package_id = if package_manifest.package_kind
            == PluginPackageKind::FeatureExtension
            || feature.owner_plugin_id != package_manifest.id
        {
            package_manifest.id.clone()
        } else {
            feature.owner_plugin_id.clone()
        };
        definitions.push(FeatureDefinition::new(feature.clone(), provider_package_id));
    }
    for feature in &package_manifest.feature_extensions {
        definitions.push(FeatureDefinition::new(
            feature.clone(),
            package_manifest.id.clone(),
        ));
    }
    definitions
}

fn feature_definition_key(feature_id: &str, provider_package_id: &str) -> String {
    format!("{feature_id}@{provider_package_id}")
}

fn feature_registration_matches_project_selection(
    registration: &RuntimePluginFeatureRegistrationReport,
    manifest: &ProjectPluginManifest,
    feature_id: &str,
) -> bool {
    let Some((owner_selection, feature_selection)) = feature_selection(manifest, feature_id) else {
        return false;
    };
    registration.provider_package_id_or_owner()
        == feature_selection.provider_package_id_or_owner(&owner_selection.id)
}

fn feature_selection<'a>(
    manifest: &'a ProjectPluginManifest,
    feature_id: &str,
) -> Option<(
    &'a ProjectPluginSelection,
    &'a ProjectPluginFeatureSelection,
)> {
    manifest.selections.iter().find_map(|selection| {
        selection
            .features
            .iter()
            .find(|feature| feature.id == feature_id)
            .map(|feature| (selection, feature))
    })
}

fn complete_owner_feature_selection(
    owner_selection: &mut ProjectPluginSelection,
    feature: &PluginFeatureBundleManifest,
    provider_package_id: Option<&str>,
) {
    let mut catalog_selection = project_selection_from_feature_manifest(feature);
    if let Some(provider_package_id) = provider_package_id {
        catalog_selection.provider_package_id = Some(provider_package_id.to_string());
    }
    if let Some(selection) = owner_selection
        .features
        .iter_mut()
        .find(|selection| selection.id == catalog_selection.id)
    {
        if selection.runtime_crate.is_none() {
            selection.runtime_crate = catalog_selection.runtime_crate;
        }
        if selection.editor_crate.is_none() {
            selection.editor_crate = catalog_selection.editor_crate;
        }
        if selection.target_modes.is_empty() {
            selection.target_modes = catalog_selection.target_modes;
        }
        if selection.provider_package_id.is_none() {
            selection.provider_package_id = catalog_selection.provider_package_id;
        }
        return;
    }
    owner_selection.features.push(catalog_selection);
}

fn active_feature_selections(manifest: &ProjectPluginManifest) -> Vec<ActiveFeatureSelection<'_>> {
    let mut active = Vec::new();
    for owner_selection in &manifest.selections {
        for feature in &owner_selection.features {
            if feature.enabled {
                active.push(ActiveFeatureSelection {
                    owner_plugin_id: owner_selection.id.clone(),
                    feature,
                });
            }
        }
    }
    active
}

fn feature_status(
    feature_definition: &FeatureDefinition,
    selection: &ProjectPluginFeatureSelection,
    target: RuntimeTargetMode,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &HashSet<String>,
) -> FeatureStatus {
    let feature = &feature_definition.manifest;
    let mut status = FeatureStatus {
        feature_id: feature.id.clone(),
        owner_plugin_id: feature.owner_plugin_id.clone(),
        ..FeatureStatus::default()
    };
    if !owner_dependency_is_valid(feature) {
        status.invalid_owner_dependency = true;
    }
    if !plugin_is_enabled_for_target(&feature.owner_plugin_id, plugin_selections, enabled_plugins) {
        push_unique(&mut status.missing_plugins, feature.owner_plugin_id.clone());
    }
    if feature_definition.provider_package_id != feature.owner_plugin_id
        && !plugin_is_enabled_for_target(
            &feature_definition.provider_package_id,
            plugin_selections,
            enabled_plugins,
        )
    {
        push_unique(
            &mut status.missing_plugins,
            feature_definition.provider_package_id.clone(),
        );
    }
    if !feature_manifest_supports_target(feature, target) || !selection.supports_target(target) {
        status.target_unsupported = true;
    }
    for dependency in &feature.dependencies {
        if !plugin_is_enabled_for_target(&dependency.plugin_id, plugin_selections, enabled_plugins)
        {
            push_unique(&mut status.missing_plugins, dependency.plugin_id.clone());
        }
        if !available_capabilities.contains(&dependency.capability) {
            push_unique(
                &mut status.missing_capabilities,
                dependency.capability.clone(),
            );
        }
    }
    status
}

fn owner_dependency_is_valid(feature: &PluginFeatureBundleManifest) -> bool {
    let primary_dependencies = feature
        .dependencies
        .iter()
        .filter(|dependency| dependency.primary)
        .collect::<Vec<_>>();
    primary_dependencies.len() == 1 && primary_dependencies[0].plugin_id == feature.owner_plugin_id
}

fn plugin_is_enabled_for_target(
    plugin_id: &str,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
) -> bool {
    plugin_selections.contains_key(plugin_id) && enabled_plugins.contains(plugin_id)
}

fn feature_manifest_supports_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> bool {
    let runtime_modules = feature
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
        .collect::<Vec<_>>();
    if runtime_modules.is_empty() {
        return true;
    }
    runtime_modules
        .iter()
        .any(|module| module.target_modes.is_empty() || module.target_modes.contains(&target))
}

fn feature_capabilities_for_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> impl Iterator<Item = String> + '_ {
    feature.capabilities.iter().cloned().chain(
        feature
            .modules
            .iter()
            .filter(move |module| {
                module.target_modes.is_empty() || module.target_modes.contains(&target)
            })
            .flat_map(|module| module.capabilities.iter().cloned()),
    )
}

fn feature_declares_capability_for_target(
    feature_definition: &FeatureDefinition,
    capability: &str,
    target: RuntimeTargetMode,
) -> bool {
    let feature = &feature_definition.manifest;
    feature
        .capabilities
        .iter()
        .any(|provided| provided == capability)
        || feature
            .modules
            .iter()
            .filter(move |module| {
                module.target_modes.is_empty() || module.target_modes.contains(&target)
            })
            .any(|module| {
                module
                    .capabilities
                    .iter()
                    .any(|provided| provided == capability)
            })
}

fn merge_runtime_extensions(
    registration: &RuntimePluginRegistrationReport,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for diagnostic in &registration.diagnostics {
        push_fatal_diagnostic(
            diagnostics,
            fatal_diagnostics,
            format!(
                "runtime plugin {} diagnostic: {diagnostic}",
                registration.package_manifest.id
            ),
        );
    }
    let plugin_id = registration.package_manifest.id.clone();
    for manager in registration.extensions.managers() {
        if let Err(error) = registry.register_manager(plugin_id.clone(), manager.clone()) {
            push_fatal_diagnostic(diagnostics, fatal_diagnostics, error.to_string());
        }
    }
    merge_extension_registry_contributions(
        &registration.extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}

fn merge_feature_extensions(
    registration: &RuntimePluginFeatureRegistrationReport,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for diagnostic in &registration.diagnostics {
        push_fatal_diagnostic(
            diagnostics,
            fatal_diagnostics,
            format!(
                "runtime plugin feature {} diagnostic: {diagnostic}",
                registration.manifest.id
            ),
        );
    }
    for manager in registration.extensions.managers() {
        if let Err(error) =
            registry.register_manager(registration.manifest.id.clone(), manager.clone())
        {
            push_fatal_diagnostic(diagnostics, fatal_diagnostics, error.to_string());
        }
    }
    merge_extension_registry_contributions(
        &registration.extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}

fn merge_extension_registry_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for module in extensions.modules() {
        push_runtime_extension_result(
            registry.register_module(module.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for render_feature in extensions.render_features() {
        push_runtime_extension_result(
            registry.register_render_feature(render_feature.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for executor in extensions.render_pass_executors() {
        push_runtime_extension_result(
            registry.register_render_pass_executor(executor.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for collector in extensions.runtime_prepare_collectors() {
        push_runtime_extension_result(
            registry.register_runtime_prepare_collector(collector.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for provider in extensions.virtual_geometry_runtime_providers() {
        push_runtime_extension_result(
            registry.register_virtual_geometry_runtime_provider(provider.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for provider in extensions.hybrid_gi_runtime_providers() {
        push_runtime_extension_result(
            registry.register_hybrid_gi_runtime_provider(provider.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for component in extensions.components() {
        push_runtime_extension_result(
            registry.register_component(component.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for ui_component in extensions.ui_components() {
        push_runtime_extension_result(
            registry.register_ui_component(ui_component.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for option in extensions.plugin_options() {
        push_runtime_extension_result(
            registry.register_plugin_option(option.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for event_catalog in extensions.plugin_event_catalogs() {
        push_runtime_extension_result(
            registry.register_plugin_event_catalog(event_catalog.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for importer in extensions.asset_importers().importers() {
        push_runtime_extension_result(
            registry.register_asset_importer_arc(importer),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for hook in extensions.scene_hooks() {
        push_runtime_extension_result(
            registry.register_scene_hook(hook.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
}

fn push_runtime_extension_result(
    result: Result<(), RuntimeExtensionRegistryError>,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    if let Err(error) = result {
        push_fatal_diagnostic(diagnostics, fatal_diagnostics, error.to_string());
    }
}

fn push_fatal_diagnostic(
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
    diagnostic: String,
) {
    diagnostics.push(diagnostic.clone());
    fatal_diagnostics.push(diagnostic);
}

fn extend_unique(collection: &mut HashSet<String>, values: impl IntoIterator<Item = String>) {
    for value in values {
        collection.insert(value);
    }
}

fn push_unique(collection: &mut Vec<String>, value: String) {
    if !collection.contains(&value) {
        collection.push(value);
    }
}
