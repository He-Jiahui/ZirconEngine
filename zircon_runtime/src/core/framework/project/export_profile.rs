use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

use crate::core::framework::platform::RuntimeTargetMode;

use super::RuntimeProfileId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportTargetPlatform {
    #[serde(alias = "windows-x86_64")]
    Windows,
    #[serde(alias = "linux-x86_64")]
    Linux,
    #[serde(alias = "macos-aarch64")]
    Macos,
    Android,
    Ios,
    WebGpu,
    Wasm,
    Headless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportPlatformHostKind {
    Desktop,
    MobileApp,
    Browser,
    Headless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportPlatformResourceStrategy {
    FilesystemBundle,
    MobileAssetBundle,
    BrowserFetch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportPlatformPluginStrategy {
    NativeDynamicAllowed,
    StaticSourceOrVmOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPlatformPolicy {
    pub target_platform: ExportTargetPlatform,
    pub host_kind: ExportPlatformHostKind,
    pub resource_strategy: ExportPlatformResourceStrategy,
    pub plugin_strategy: ExportPlatformPluginStrategy,
    pub supports_native_dynamic: bool,
}

impl Default for ExportPlatformPolicy {
    fn default() -> Self {
        ExportTargetPlatform::Windows.policy()
    }
}

impl ExportTargetPlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::WebGpu => "web_gpu",
            Self::Wasm => "wasm",
            Self::Headless => "headless",
        }
    }

    pub fn is_desktop(self) -> bool {
        matches!(self, Self::Windows | Self::Linux | Self::Macos)
    }

    pub fn supports_native_dynamic(self) -> bool {
        self.policy().supports_native_dynamic
    }

    pub fn policy(self) -> ExportPlatformPolicy {
        let (host_kind, resource_strategy, plugin_strategy) = match self {
            Self::Windows | Self::Linux | Self::Macos => (
                ExportPlatformHostKind::Desktop,
                ExportPlatformResourceStrategy::FilesystemBundle,
                ExportPlatformPluginStrategy::NativeDynamicAllowed,
            ),
            Self::Android | Self::Ios => (
                ExportPlatformHostKind::MobileApp,
                ExportPlatformResourceStrategy::MobileAssetBundle,
                ExportPlatformPluginStrategy::StaticSourceOrVmOnly,
            ),
            Self::WebGpu | Self::Wasm => (
                ExportPlatformHostKind::Browser,
                ExportPlatformResourceStrategy::BrowserFetch,
                ExportPlatformPluginStrategy::StaticSourceOrVmOnly,
            ),
            Self::Headless => (
                ExportPlatformHostKind::Headless,
                ExportPlatformResourceStrategy::FilesystemBundle,
                ExportPlatformPluginStrategy::NativeDynamicAllowed,
            ),
        };
        ExportPlatformPolicy {
            target_platform: self,
            host_kind,
            resource_strategy,
            plugin_strategy,
            supports_native_dynamic: matches!(
                plugin_strategy,
                ExportPlatformPluginStrategy::NativeDynamicAllowed
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportPackagingStrategy {
    SourceTemplate,
    LibraryEmbed,
    NativeDynamic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportBuildMode {
    #[default]
    Debug,
    Release,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfile {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_export_target_mode")]
    pub target_mode: RuntimeTargetMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_profile_id: Option<RuntimeProfileId>,
    #[serde(alias = "platform")]
    pub target_platform: ExportTargetPlatform,
    #[serde(
        default = "default_export_strategies",
        alias = "path",
        deserialize_with = "deserialize_export_strategies"
    )]
    pub strategies: Vec<ExportPackagingStrategy>,
    #[serde(
        default,
        rename = "mode",
        alias = "build_mode",
        skip_serializing_if = "is_default_export_build_mode"
    )]
    pub build_mode: ExportBuildMode,
    #[serde(default)]
    pub output_name: String,
    #[serde(
        default,
        rename = "plugins",
        alias = "selected_plugins",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub selected_plugins: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub features: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_filter: Option<String>,
}

impl ExportProfile {
    pub fn new(
        name: impl Into<String>,
        target_mode: RuntimeTargetMode,
        target_platform: ExportTargetPlatform,
        runtime_profile_id: RuntimeProfileId,
    ) -> Self {
        let name = name.into();
        Self {
            output_name: name.clone(),
            name,
            target_mode,
            runtime_profile_id: Some(runtime_profile_id),
            target_platform,
            strategies: default_export_strategies(),
            build_mode: ExportBuildMode::Debug,
            selected_plugins: Vec::new(),
            features: BTreeMap::new(),
            asset_filter: None,
        }
    }

    pub fn with_strategy(mut self, strategy: ExportPackagingStrategy) -> Self {
        self.strategies.retain(|existing| existing != &strategy);
        self.strategies.push(strategy);
        self
    }

    pub fn with_strategies(
        mut self,
        strategies: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.strategies = strategies.into_iter().collect();
        self
    }

    pub fn with_build_mode(mut self, build_mode: ExportBuildMode) -> Self {
        self.build_mode = build_mode;
        self
    }

    pub fn with_selected_plugins(mut self, plugins: impl IntoIterator<Item = String>) -> Self {
        self.selected_plugins = plugins.into_iter().collect();
        self
    }

    pub fn with_feature_selection(
        mut self,
        owner_plugin_id: impl Into<String>,
        feature_ids: impl IntoIterator<Item = String>,
    ) -> Self {
        self.features
            .insert(owner_plugin_id.into(), feature_ids.into_iter().collect());
        self
    }

    pub fn with_asset_filter(mut self, asset_filter: impl Into<String>) -> Self {
        self.asset_filter = Some(asset_filter.into());
        self
    }

    pub fn uses_strategy(&self, strategy: ExportPackagingStrategy) -> bool {
        self.strategies.contains(&strategy)
    }
}

impl Default for ExportProfile {
    fn default() -> Self {
        Self::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        )
    }
}

fn default_export_strategies() -> Vec<ExportPackagingStrategy> {
    vec![
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
    ]
}

fn default_export_target_mode() -> RuntimeTargetMode {
    RuntimeTargetMode::ClientRuntime
}

fn is_default_export_build_mode(build_mode: &ExportBuildMode) -> bool {
    *build_mode == ExportBuildMode::Debug
}

fn deserialize_export_strategies<'de, D>(
    deserializer: D,
) -> Result<Vec<ExportPackagingStrategy>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrategyInput {
        Single(ExportPackagingStrategy),
        Multiple(Vec<ExportPackagingStrategy>),
    }

    Ok(match StrategyInput::deserialize(deserializer)? {
        StrategyInput::Single(strategy) => vec![strategy],
        StrategyInput::Multiple(strategies) => strategies,
    })
}
