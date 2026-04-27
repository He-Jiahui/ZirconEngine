use serde::{Deserialize, Serialize};

use crate::RuntimeTargetMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportTargetPlatform {
    Windows,
    Linux,
    Macos,
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
