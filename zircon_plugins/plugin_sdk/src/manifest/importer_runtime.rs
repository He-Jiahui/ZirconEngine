//! 为 importer 插件生成运行时模块、Native 分发模块与包分发清单。
//! 包清单组装以 RuntimePluginDescriptor 为底稿；运行时模块项由调用方显式纳入。
use zircon_runtime::asset::AssetImporterDescriptor;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};
use zircon_runtime::plugin::{
    PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest,
    RuntimePluginDescriptor,
};

use super::SDK_API_VERSION;

pub const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
pub const NATIVE_ABI_VERSION_V3: u32 = 3;

/// 返回 importer 运行实现面向客户端运行时与编辑器宿主的目标集合。
pub fn importer_runtime_supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

/// 返回 importer 原生分发支持的桌面平台集合。
pub fn importer_runtime_supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

#[derive(Clone, Debug)]
/// 保存 runtime/native 两侧模块身份及 importer 能力，供包清单投影复用。
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

    /// 用新迭代器整体替换能力列表；后续两个模块清单从此列表投影。
    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }

    /// 用新迭代器整体替换待写入包清单的 importer 描述符列表。
    pub fn with_asset_importers(
        mut self,
        importers: impl IntoIterator<Item = AssetImporterDescriptor>,
    ) -> Self {
        self.importers = importers.into_iter().collect();
        self
    }

    /// 单独生成 runtime crate 的模块清单；包组装方法不会自动追加此项。
    pub fn runtime_module_manifest(&self) -> PluginModuleManifest {
        PluginModuleManifest::runtime(
            self.runtime_module_name.clone(),
            self.runtime_crate_name.clone(),
        )
        .with_target_modes(importer_runtime_supported_targets())
        .with_capabilities(self.capabilities.iter().cloned())
    }

    /// 生成 Native 分发 crate 的模块清单及其目标和能力投影。
    pub fn dist_module_manifest(&self) -> PluginModuleManifest {
        PluginModuleManifest::native(self.dist_module_name.clone(), self.dist_crate_name.clone())
            .with_target_modes(importer_runtime_supported_targets())
            .with_capabilities(self.capabilities.iter().cloned())
    }

    /// 生成指向 Native ABI v3 描述符入口的默认 dist 分发信息。
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

    /// 从运行时 descriptor 的包清单开始，补 SDK 版本、Native dist 模块、分发信息与 importer 项。
    ///
    /// 本方法不调用 runtime_module_manifest；调用方需自行确认运行时模块已进入底稿。
    pub fn build_package_manifest(
        self,
        descriptor: &RuntimePluginDescriptor,
    ) -> PluginPackageManifest {
        let Self {
            dist_module_name,
            dist_crate_name,
            dist_runtime_entry,
            engine_compat,
            capabilities,
            importers,
            ..
        } = self;
        let distribution_dist_crate = dist_crate_name.clone();
        let dist_module = PluginModuleManifest::native(dist_module_name, dist_crate_name)
            .with_target_modes(importer_runtime_supported_targets())
            .with_capabilities(capabilities);
        let distribution = PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat,
            dist_crate: distribution_dist_crate,
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: dist_runtime_entry,
            ..PluginDistributionManifest::default()
        };
        let mut manifest = descriptor
            .package_manifest()
            .with_sdk_api_version(SDK_API_VERSION);
        if !manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic)
        {
            manifest
                .default_packaging
                .push(ExportPackagingStrategy::NativeDynamic);
        }
        manifest = manifest.with_native_module(dist_module);
        manifest = manifest.with_distribution(distribution);
        for importer in importers {
            manifest = manifest.with_asset_importer(importer);
        }
        manifest
    }

    #[cfg(test)]
    fn legacy_build_package_manifest(
        self,
        descriptor: &RuntimePluginDescriptor,
    ) -> PluginPackageManifest {
        let mut manifest = descriptor
            .package_manifest()
            .with_sdk_api_version(SDK_API_VERSION);
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

#[cfg(test)]
#[path = "tests/importer_runtime_performance_tests.rs"]
mod performance_tests;
