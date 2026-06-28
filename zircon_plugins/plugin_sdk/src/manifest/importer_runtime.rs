use zircon_runtime::asset::AssetImporterDescriptor;
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, ExportTargetPlatform, PluginDistributionManifest,
    PluginModuleManifest, PluginPackageManifest, RuntimePluginDescriptor,
};

pub const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
pub const NATIVE_ABI_VERSION_V3: u32 = 3;

pub fn importer_runtime_supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn importer_runtime_supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

#[derive(Clone, Debug)]
pub struct ImporterRuntimeManifestBuilder {
    runtime_module_name: String,
    runtime_crate_name: String,
    dist_module_name: String,
    dist_crate_name: String,
    dist_runtime_entry: String,
    engine_compat: String,
    capabilities: Vec<String>,
    importers: Vec<AssetImporterDescriptor>,
}

impl ImporterRuntimeManifestBuilder {
    pub fn new(
        runtime_module_name: impl Into<String>,
        runtime_crate_name: impl Into<String>,
        dist_module_name: impl Into<String>,
        dist_crate_name: impl Into<String>,
        dist_runtime_entry: impl Into<String>,
    ) -> Self {
        Self {
            runtime_module_name: runtime_module_name.into(),
            runtime_crate_name: runtime_crate_name.into(),
            dist_module_name: dist_module_name.into(),
            dist_crate_name: dist_crate_name.into(),
            dist_runtime_entry: dist_runtime_entry.into(),
            engine_compat: ">=0.1, <0.2".to_string(),
            capabilities: Vec::new(),
            importers: Vec::new(),
        }
    }

    pub fn with_engine_compat(mut self, engine_compat: impl Into<String>) -> Self {
        self.engine_compat = engine_compat.into();
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

    pub fn with_asset_importers(
        mut self,
        importers: impl IntoIterator<Item = AssetImporterDescriptor>,
    ) -> Self {
        self.importers = importers.into_iter().collect();
        self
    }

    pub fn runtime_module_manifest(&self) -> PluginModuleManifest {
        PluginModuleManifest::runtime(
            self.runtime_module_name.clone(),
            self.runtime_crate_name.clone(),
        )
        .with_target_modes(importer_runtime_supported_targets())
        .with_capabilities(self.capabilities.iter().cloned())
    }

    pub fn dist_module_manifest(&self) -> PluginModuleManifest {
        PluginModuleManifest::native(self.dist_module_name.clone(), self.dist_crate_name.clone())
            .with_target_modes(importer_runtime_supported_targets())
            .with_capabilities(self.capabilities.iter().cloned())
    }

    pub fn distribution_manifest(&self) -> PluginDistributionManifest {
        PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: self.engine_compat.clone(),
            dist_crate: self.dist_crate_name.clone(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: self.dist_runtime_entry.clone(),
            ..PluginDistributionManifest::default()
        }
    }

    pub fn build_package_manifest(
        self,
        descriptor: &RuntimePluginDescriptor,
    ) -> PluginPackageManifest {
        let mut manifest = descriptor.package_manifest();
        if !manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic)
        {
            manifest
                .default_packaging
                .push(ExportPackagingStrategy::NativeDynamic);
        }
        manifest = manifest.with_native_module(self.dist_module_manifest());
        manifest = manifest.with_distribution(self.distribution_manifest());
        for importer in self.importers {
            manifest = manifest.with_asset_importer(importer);
        }
        manifest
    }
}
