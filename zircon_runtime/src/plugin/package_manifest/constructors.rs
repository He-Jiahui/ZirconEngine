use crate::{
    asset::AssetImporterDescriptor, plugin::ComponentTypeDescriptor,
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::UiComponentDescriptor,
    RuntimeTargetMode,
};

use super::{
    PluginDependencyManifest, PluginEventCatalogManifest, PluginFeatureBundleManifest,
    PluginModuleKind, PluginModuleManifest, PluginOptionManifest, PluginPackageKind,
    PluginPackageManifest,
};

impl PluginModuleManifest {
    pub fn runtime(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: PluginModuleKind::Runtime,
            crate_name: crate_name.into(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn editor(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: PluginModuleKind::Editor,
            crate_name: crate_name.into(),
            target_modes: vec![RuntimeTargetMode::EditorHost],
            capabilities: Vec::new(),
        }
    }

    pub fn native(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: PluginModuleKind::Native,
            crate_name: crate_name.into(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn vm(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: PluginModuleKind::Vm,
            crate_name: crate_name.into(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
        }
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
}

impl PluginPackageManifest {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: "0.1.0".to_string(),
            sdk_api_version: "0.1.0".to_string(),
            package_kind: PluginPackageKind::Standard,
            display_name: display_name.into(),
            category: "uncategorized".to_string(),
            description: String::new(),
            supported_targets: Vec::new(),
            supported_platforms: Vec::new(),
            capabilities: Vec::new(),
            asset_roots: Vec::new(),
            content_roots: Vec::new(),
            modules: Vec::new(),
            dependencies: Vec::new(),
            options: Vec::new(),
            event_catalogs: Vec::new(),
            components: Vec::new(),
            ui_components: Vec::new(),
            asset_importers: Vec::new(),
            optional_features: Vec::new(),
            feature_extensions: Vec::new(),
            default_packaging: vec![
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
            ],
        }
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

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.default_packaging = packaging.into_iter().collect();
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
