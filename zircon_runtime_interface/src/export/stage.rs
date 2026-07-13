use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExportStage {
    Validate,
    SourceTemplate,
    NativeDynamic,
    CompileHost,
    CookAssets,
    Pack,
    PlatformBundle,
    Report,
}

impl ExportStage {
    pub const ALL: [Self; 8] = [
        Self::Validate,
        Self::SourceTemplate,
        Self::NativeDynamic,
        Self::CompileHost,
        Self::CookAssets,
        Self::Pack,
        Self::PlatformBundle,
        Self::Report,
    ];

    pub const fn cli_id(self) -> &'static str {
        match self {
            Self::Validate => "validate",
            Self::SourceTemplate => "source_template",
            Self::NativeDynamic => "native_dynamic",
            Self::CompileHost => "compile_host",
            Self::CookAssets => "cook_assets",
            Self::Pack => "pack",
            Self::PlatformBundle => "platform_bundle",
            Self::Report => "report",
        }
    }

    pub const fn report_name(self) -> &'static str {
        match self {
            Self::Validate => "Validate",
            Self::SourceTemplate => "SourceTemplate",
            Self::NativeDynamic => "NativeDynamic",
            Self::CompileHost => "CompileHost",
            Self::CookAssets => "CookAssets",
            Self::Pack => "Pack",
            Self::PlatformBundle => "PlatformBundle",
            Self::Report => "Report",
        }
    }
}

impl FromStr for ExportStage {
    type Err = ParseExportStageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = normalize_stage_name(value);
        Self::ALL
            .into_iter()
            .find(|stage| normalized == normalize_stage_name(stage.report_name()))
            .ok_or_else(|| ParseExportStageError {
                value: value.to_owned(),
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseExportStageError {
    value: String,
}

impl fmt::Display for ParseExportStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown export stage `{}`", self.value)
    }
}

impl std::error::Error for ParseExportStageError {}

fn normalize_stage_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| *character != '_' && *character != '-')
        .flat_map(char::to_lowercase)
        .collect()
}
