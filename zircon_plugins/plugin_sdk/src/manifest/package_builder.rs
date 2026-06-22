use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, ExportTargetPlatform, PluginMaturity, PluginModuleManifest,
    PluginPackageManifest,
};

use super::{default_export_packaging, default_supported_platforms, SDK_API_VERSION};

#[derive(Clone, Debug)]
pub struct PluginManifestBuilder {
    manifest: PluginPackageManifest,
}

impl PluginManifestBuilder {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            manifest: PluginPackageManifest::new(id, display_name)
                .with_sdk_api_version(SDK_API_VERSION)
                .with_supported_platforms(default_supported_platforms())
                .with_default_packaging(default_export_packaging()),
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.manifest = self.manifest.with_category(category);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.manifest.description = description.into();
        self
    }

    pub fn with_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.manifest = self.manifest.with_maturity(maturity);
        self
    }

    pub fn with_supported_targets(
        mut self,
        targets: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.manifest = self.manifest.with_supported_targets(targets);
        self
    }

    pub fn with_supported_platforms(
        mut self,
        platforms: impl IntoIterator<Item = ExportTargetPlatform>,
    ) -> Self {
        self.manifest = self.manifest.with_supported_platforms(platforms);
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.manifest = self.manifest.with_capability(capability);
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.manifest = self.manifest.with_capabilities(capabilities);
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.manifest = self.manifest.with_default_packaging(packaging);
        self
    }

    pub fn with_asset_root(mut self, asset_root: impl Into<String>) -> Self {
        self.manifest = self.manifest.with_asset_root(asset_root);
        self
    }

    pub fn with_content_root(mut self, content_root: impl Into<String>) -> Self {
        self.manifest = self.manifest.with_content_root(content_root);
        self
    }

    pub fn with_module(mut self, module: PluginModuleManifest) -> Self {
        self.manifest = self.manifest.with_module(module);
        self
    }

    pub fn build(self) -> PluginPackageManifest {
        self.manifest
    }
}
