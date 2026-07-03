use crate::builtin::RuntimeTargetMode;
use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use crate::{
    asset::AssetImporterDescriptor,
    core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor},
    plugin::CapabilityStatusManifest,
    plugin::ComponentTypeDescriptor,
    plugin::ExportPackagingStrategy,
    plugin::ExportTargetPlatform,
    plugin::PluginMaturity,
    plugin::UiComponentDescriptor,
};

use super::{
    PluginDependencyManifest, PluginDistributionManifest, PluginEventCatalogManifest,
    PluginFeatureBundleManifest, PluginInterfaceManifest, PluginModuleKind, PluginModuleManifest,
    PluginOptionManifest, PluginPackageKind, PluginPackageManifest,
    PluginShaderPermutationIdManifest,
};

impl PluginModuleManifest {
    pub fn runtime(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Runtime, &name),
            name,
            kind: PluginModuleKind::Runtime,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
        }
    }

    pub fn editor(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Editor, &name),
            name,
            kind: PluginModuleKind::Editor,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: vec![RuntimeTargetMode::EditorHost],
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
        }
    }

    pub fn native(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Native, &name),
            name,
            kind: PluginModuleKind::Native,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
        }
    }

    pub fn vm(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Vm, &name),
            name,
            kind: PluginModuleKind::Vm,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.init_level = init_level;
        self
    }

    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.module_dependencies.push(dependency);
        self
    }

    pub fn with_module_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = ModuleDependencySpec>,
    ) -> Self {
        self.module_dependencies = dependencies.into_iter().collect();
        self
    }

    pub fn module_descriptor(&self) -> ModuleDescriptor {
        let description = if self.description.is_empty() {
            default_module_description(self.kind, &self.name)
        } else {
            self.description.clone()
        };
        let mut descriptor =
            ModuleDescriptor::new(self.name.clone(), description).with_init_level(self.init_level);
        for dependency in self.module_dependencies.iter().cloned() {
            descriptor = descriptor.with_module_dependency(dependency);
        }
        descriptor
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_sets<I, S>(mut self, system_sets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.system_sets = system_sets.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_anchors<I, S>(mut self, system_anchors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.system_anchors = system_anchors.into_iter().map(Into::into).collect();
        self
    }
}

fn default_module_description(kind: PluginModuleKind, name: &str) -> String {
    let label = match kind {
        PluginModuleKind::Runtime => "Runtime",
        PluginModuleKind::Editor => "Editor",
        PluginModuleKind::Native => "Native",
        PluginModuleKind::Vm => "VM",
    };
    format!("{label} plugin module {name}")
}

impl PluginPackageManifest {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        let id = id.into();
        let package_name = default_package_coordinate_name(&id);
        Self {
            id: id.clone(),
            version: "0.1.0".to_string(),
            sdk_api_version: "0.1.0".to_string(),
            package_prefix: "com".to_string(),
            package_company: "zircon".to_string(),
            package_name,
            package_kind: PluginPackageKind::Standard,
            display_name: display_name.into(),
            category: "uncategorized".to_string(),
            description: String::new(),
            supported_targets: Vec::new(),
            supported_platforms: Vec::new(),
            capabilities: Vec::new(),
            capability_statuses: Vec::new(),
            maturity: PluginMaturity::default(),
            asset_roots: Vec::new(),
            content_roots: Vec::new(),
            modules: Vec::new(),
            dependencies: Vec::new(),
            provides_interfaces: Vec::new(),
            options: Vec::new(),
            event_catalogs: Vec::new(),
            components: Vec::new(),
            ui_components: Vec::new(),
            asset_importers: Vec::new(),
            optional_features: Vec::new(),
            feature_extensions: Vec::new(),
            geometry_sources: Vec::new(),
            shading_models: Vec::new(),
            shader_permutation: Default::default(),
            default_packaging: vec![
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
            ],
            distribution: None,
        }
    }

    pub fn with_package_identity(
        mut self,
        prefix: impl Into<String>,
        company: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        self.package_prefix = prefix.into();
        self.package_company = company.into();
        self.package_name = name.into();
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    pub fn with_sdk_api_version(mut self, sdk_api_version: impl Into<String>) -> Self {
        self.sdk_api_version = sdk_api_version.into();
        self
    }

    pub fn with_package_kind(mut self, package_kind: PluginPackageKind) -> Self {
        self.package_kind = package_kind;
        self
    }

    pub fn as_feature_extension(mut self) -> Self {
        self.package_kind = PluginPackageKind::FeatureExtension;
        self
    }

    pub fn with_supported_targets(
        mut self,
        targets: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.supported_targets = targets.into_iter().collect();
        self
    }

    pub fn with_supported_target(mut self, target: RuntimeTargetMode) -> Self {
        self.supported_targets.push(target);
        self
    }

    pub fn with_supported_platforms(
        mut self,
        platforms: impl IntoIterator<Item = ExportTargetPlatform>,
    ) -> Self {
        self.supported_platforms = platforms.into_iter().collect();
        self
    }

    pub fn with_supported_platform(mut self, platform: ExportTargetPlatform) -> Self {
        self.supported_platforms.push(platform);
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn with_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.maturity = maturity;
        self
    }

    pub fn with_capability_status(mut self, status: CapabilityStatusManifest) -> Self {
        self.capability_statuses.push(status);
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities
            .extend(capabilities.into_iter().map(Into::into));
        self
    }

    pub fn with_asset_root(mut self, asset_root: impl Into<String>) -> Self {
        self.asset_roots.push(asset_root.into());
        self
    }

    pub fn with_asset_roots<I, S>(mut self, asset_roots: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.asset_roots
            .extend(asset_roots.into_iter().map(Into::into));
        self
    }

    pub fn with_content_root(mut self, content_root: impl Into<String>) -> Self {
        self.content_roots.push(content_root.into());
        self
    }

    pub fn with_content_roots<I, S>(mut self, content_roots: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.content_roots
            .extend(content_roots.into_iter().map(Into::into));
        self
    }

    pub fn with_runtime_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.modules.push(PluginModuleManifest::runtime(
            format!("{}.runtime", self.id),
            crate_name,
        ));
        self
    }

    pub fn with_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_runtime_module(self, module: PluginModuleManifest) -> Self {
        self.with_module(module)
    }

    pub fn with_dependency(mut self, dependency: PluginDependencyManifest) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn with_provided_interface(mut self, interface: PluginInterfaceManifest) -> Self {
        self.provides_interfaces.push(interface);
        self
    }

    pub fn with_provided_interface_id(mut self, interface_id: impl Into<String>) -> Self {
        self.provides_interfaces
            .push(PluginInterfaceManifest::new(interface_id));
        self
    }

    pub fn with_option(mut self, option: PluginOptionManifest) -> Self {
        self.options.push(option);
        self
    }

    pub fn with_event_catalog(mut self, event_catalog: PluginEventCatalogManifest) -> Self {
        self.event_catalogs.push(event_catalog);
        self
    }

    pub fn with_component(mut self, descriptor: ComponentTypeDescriptor) -> Self {
        self.components.push(descriptor);
        self
    }

    pub fn with_ui_component(mut self, descriptor: UiComponentDescriptor) -> Self {
        self.ui_components.push(descriptor);
        self
    }

    pub fn with_asset_importer(mut self, descriptor: AssetImporterDescriptor) -> Self {
        self.asset_importers.push(descriptor);
        self
    }

    pub fn with_optional_feature(mut self, feature: PluginFeatureBundleManifest) -> Self {
        self.optional_features.push(feature);
        self
    }

    pub fn with_feature_extension(mut self, feature: PluginFeatureBundleManifest) -> Self {
        self.feature_extensions.push(feature);
        self
    }

    pub fn with_geometry_source_descriptor(mut self, descriptor: GeometrySourceDescriptor) -> Self {
        self.geometry_sources.push(descriptor);
        self
    }

    pub fn with_geometry_source_descriptors(
        mut self,
        descriptors: impl IntoIterator<Item = GeometrySourceDescriptor>,
    ) -> Self {
        self.geometry_sources.extend(descriptors);
        self
    }

    pub fn with_shading_model_descriptor(mut self, descriptor: ShadingModelDescriptor) -> Self {
        self.shading_models.push(descriptor);
        self
    }

    pub fn with_shading_model_descriptors(
        mut self,
        descriptors: impl IntoIterator<Item = ShadingModelDescriptor>,
    ) -> Self {
        self.shading_models.extend(descriptors);
        self
    }

    pub fn with_shader_geometry_source_id(mut self, token: impl Into<String>, id: u8) -> Self {
        self.shader_permutation
            .geometry_source_ids
            .push(PluginShaderPermutationIdManifest::new(token, id));
        self
    }

    pub fn with_shader_shading_model_id(mut self, token: impl Into<String>, id: u8) -> Self {
        self.shader_permutation
            .shading_model_ids
            .push(PluginShaderPermutationIdManifest::new(token, id));
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.default_packaging = packaging.into_iter().collect();
        self
    }

    pub fn with_distribution(mut self, distribution: PluginDistributionManifest) -> Self {
        self.distribution = Some(distribution);
        self
    }

    pub fn with_editor_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.modules.push(PluginModuleManifest::editor(
            format!("{}.editor", self.id),
            crate_name,
        ));
        self
    }

    pub fn with_editor_module(self, module: PluginModuleManifest) -> Self {
        self.with_module(module)
    }

    pub fn with_native_module(self, module: PluginModuleManifest) -> Self {
        self.with_module(module)
    }

    pub fn with_vm_module(self, module: PluginModuleManifest) -> Self {
        self.with_module(module)
    }
}

fn default_package_coordinate_name(package_id: &str) -> String {
    package_id.replace('.', "_")
}
