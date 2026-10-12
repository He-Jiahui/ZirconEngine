//! 汇总插件包清单的字段模型、构造器和公开重导出；运行时注册、导出计划与安装服务共享这些类型。

#[path = "builtin_catalog.rs"]
mod builtin_catalog;
#[path = "constructors.rs"]
mod constructors;
#[path = "plugin_dependency_manifest.rs"]
mod plugin_dependency_manifest;
#[path = "plugin_distribution_manifest.rs"]
mod plugin_distribution_manifest;
#[path = "plugin_event_manifest.rs"]
mod plugin_event_manifest;
#[path = "plugin_feature_bundle_manifest.rs"]
mod plugin_feature_bundle_manifest;
#[path = "plugin_feature_dependency.rs"]
mod plugin_feature_dependency;
#[path = "plugin_interface_manifest.rs"]
mod plugin_interface_manifest;
#[path = "plugin_module_kind.rs"]
mod plugin_module_kind;
#[path = "plugin_module_manifest.rs"]
mod plugin_module_manifest;
#[path = "plugin_option_manifest.rs"]
mod plugin_option_manifest;
#[path = "plugin_package_kind.rs"]
mod plugin_package_kind;
#[path = "plugin_package_manifest.rs"]
mod plugin_package_manifest;
#[path = "plugin_package_role.rs"]
mod plugin_package_role;
#[path = "plugin_shader_permutation_manifest.rs"]
mod plugin_shader_permutation_manifest;

pub use plugin_dependency_manifest::PluginDependencyManifest;
pub use plugin_distribution_manifest::PluginDistributionManifest;
pub use plugin_event_manifest::{PluginEventCatalogManifest, PluginEventManifest};
pub use plugin_feature_bundle_manifest::PluginFeatureBundleManifest;
pub use plugin_feature_dependency::PluginFeatureDependency;
pub use plugin_interface_manifest::{PluginInterfaceManifest, PluginInterfaceMethodManifest};
pub use plugin_module_kind::PluginModuleKind;
pub use plugin_module_manifest::{PluginEventConsumerManifest, PluginModuleManifest};
pub use plugin_option_manifest::PluginOptionManifest;
pub use plugin_package_kind::PluginPackageKind;
pub use plugin_package_manifest::PluginPackageManifest;
pub use plugin_package_role::PluginPackageRole;
pub use plugin_shader_permutation_manifest::{
    PluginShaderModuleManifest, PluginShaderModuleSource, PluginShaderPermutationIdManifest,
    PluginShaderPermutationManifest, ShaderModuleSourceBinding,
};
