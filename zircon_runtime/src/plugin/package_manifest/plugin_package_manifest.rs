use serde::{Deserialize, Serialize};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::{
    asset::AssetImporterDescriptor,
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportTargetPlatform,
    core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor},
    plugin::CapabilityStatusManifest,
    plugin::PluginMaturity,
    plugin::UiComponentDescriptor,
};

use super::{
    PluginDependencyManifest, PluginDistributionManifest, PluginEventCatalogManifest,
    PluginFeatureBundleManifest, PluginInterfaceManifest, PluginInterfaceMethodManifest,
    PluginModuleManifest, PluginOptionManifest, PluginPackageKind, PluginPackageRole,
    PluginShaderPermutationManifest,
};

/// 插件包的 TOML/运行时共同清单；native registration、导出计划和安装服务都以此作为字段契约。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPackageManifest {
    pub id: String,
    pub version: String,
    #[serde(default = "default_sdk_api_version")]
    pub sdk_api_version: String,
    #[serde(default)]
    pub package_prefix: String,
    #[serde(default)]
    pub package_company: String,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub package_kind: PluginPackageKind,
    #[serde(default, skip_serializing_if = "PluginPackageRole::is_production")]
    pub package_role: PluginPackageRole,
    pub display_name: String,
    #[serde(default = "default_plugin_category")]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub supported_targets: Vec<RuntimeTargetMode>,
    #[serde(default)]
    pub supported_platforms: Vec<ExportTargetPlatform>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub capability_statuses: Vec<CapabilityStatusManifest>,
    #[serde(default)]
    pub maturity: PluginMaturity,
    #[serde(default)]
    pub asset_roots: Vec<String>,
    #[serde(default)]
    pub content_roots: Vec<String>,
    #[serde(default)]
    pub modules: Vec<PluginModuleManifest>,
    #[serde(default)]
    pub dependencies: Vec<PluginDependencyManifest>,
    #[serde(default)]
    pub provides_interfaces: Vec<PluginInterfaceManifest>,
    #[serde(default)]
    pub options: Vec<PluginOptionManifest>,
    #[serde(default)]
    pub event_catalogs: Vec<PluginEventCatalogManifest>,
    #[serde(default)]
    pub components: Vec<ComponentTypeDescriptor>,
    #[serde(default)]
    pub ui_components: Vec<UiComponentDescriptor>,
    #[serde(default)]
    pub asset_importers: Vec<AssetImporterDescriptor>,
    #[serde(default)]
    pub optional_features: Vec<PluginFeatureBundleManifest>,
    #[serde(default)]
    pub feature_extensions: Vec<PluginFeatureBundleManifest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub geometry_sources: Vec<GeometrySourceDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shading_models: Vec<ShadingModelDescriptor>,
    #[serde(
        default,
        skip_serializing_if = "PluginShaderPermutationManifest::is_empty"
    )]
    pub shader_permutation: PluginShaderPermutationManifest,
    #[serde(default)]
    /// 包级默认策略由 registration validation 检查，并由项目导出选择投影为 profile 策略。
    pub default_packaging: Vec<ExportPackagingStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution: Option<PluginDistributionManifest>,
}

fn default_plugin_category() -> String {
    "uncategorized".to_string()
}

fn default_sdk_api_version() -> String {
    "0.1.0".to_string()
}

impl PluginPackageManifest {
    /// 返回用于安装、native artifact receipt 和桥接身份比对的完整包坐标；坐标不完整时回退到逻辑 id。
    pub fn package_id(&self) -> String {
        if self.package_prefix.is_empty()
            || self.package_company.is_empty()
            || self.package_name.is_empty()
        {
            return self.id.clone();
        }
        let capacity =
            self.package_prefix.len() + self.package_company.len() + self.package_name.len() + 2;
        let mut package_id = String::with_capacity(capacity);
        package_id.push_str(&self.package_prefix);
        package_id.push('.');
        package_id.push_str(&self.package_company);
        package_id.push('.');
        package_id.push_str(&self.package_name);
        package_id
    }

    /// 为资产扫描提供至少一个根目录；清单未声明时使用包内相对的 `assets` 根。
    pub fn asset_roots_or_default(&self) -> Vec<String> {
        if self.asset_roots.is_empty() {
            return vec!["assets".to_string()];
        }
        self.asset_roots.clone()
    }

    pub fn bridge_interface(&self, interface_id: &str) -> Option<&PluginInterfaceManifest> {
        self.provides_interfaces
            .iter()
            .find(|interface| interface.id == interface_id)
    }

    /// 展平接口与方法的有序视图；桥接绑定以该顺序分配描述符槽位并拒绝多余绑定。
    pub fn bridge_methods(
        &self,
    ) -> impl Iterator<Item = (&PluginInterfaceManifest, &PluginInterfaceMethodManifest)> {
        self.provides_interfaces.iter().flat_map(|interface| {
            interface
                .methods
                .iter()
                .map(move |method| (interface, method))
        })
    }
}

#[cfg(test)]
#[path = "tests/plugin_package_manifest.rs"]
mod tests;
