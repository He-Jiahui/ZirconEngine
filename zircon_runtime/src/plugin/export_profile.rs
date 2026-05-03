use serde::{Deserialize, Serialize};

use crate::RuntimeTargetMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportTargetPlatform {
    Windows,
    Linux,
    Macos,
    Android,
    Ios,
    WebGpu,
    Wasm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportPlatformHostKind {
    Desktop,
    MobileApp,
    Browser,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfile {
    pub name: String,
    pub target_mode: RuntimeTargetMode,
    pub target_platform: ExportTargetPlatform,
    #[serde(default = "default_export_strategies")]
    pub strategies: Vec<ExportPackagingStrategy>,
    #[serde(default)]
    pub output_name: String,
}

impl ExportProfile {
    pub fn new(
        name: impl Into<String>,
        target_mode: RuntimeTargetMode,
        target_platform: ExportTargetPlatform,
    ) -> Self {
        let name = name.into();
        Self {
            output_name: name.clone(),
            name,
            target_mode,
            target_platform,
            strategies: default_export_strategies(),
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
        )
    }
}

fn default_export_strategies() -> Vec<ExportPackagingStrategy> {
    vec![
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
    ]
}
