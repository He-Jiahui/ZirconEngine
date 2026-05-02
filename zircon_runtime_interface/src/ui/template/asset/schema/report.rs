use serde::{Deserialize, Serialize};

use super::super::document::UiAssetDocument;
use super::policy::UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetMigrationOutcome {
    pub document: UiAssetDocument,
    pub report: UiAssetMigrationReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetMigrationReport {
    pub source_kind: UiAssetSchemaSourceKind,
    pub source_schema_version: Option<u32>,
    pub target_schema_version: u32,
    pub steps: Vec<UiAssetMigrationStep>,
    pub diagnostics: Vec<UiAssetSchemaDiagnostic>,
    pub can_edit: bool,
}

impl UiAssetMigrationReport {
    pub fn new(source_kind: UiAssetSchemaSourceKind, source_schema_version: Option<u32>) -> Self {
        Self {
            can_edit: source_kind != UiAssetSchemaSourceKind::FutureVersion,
            source_kind,
            source_schema_version,
            target_schema_version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            steps: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn push_step(&mut self, step: UiAssetMigrationStep) {
        self.steps.push(step);
    }

    pub fn push_diagnostic(&mut self, diagnostic: UiAssetSchemaDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAssetSchemaSourceKind {
    CurrentTree,
    OlderTree,
    FlatNodeTable,
    LegacyTemplateFixture,
    FutureVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAssetMigrationStep {
    CurrentTreeValidated,
    SourceVersionBumped { from: u32, to: u32 },
    FlatNodeTableMaterialized,
    LegacyTemplateConverted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetSchemaDiagnostic {
    pub severity: UiAssetSchemaDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAssetSchemaDiagnosticSeverity {
    Info,
    Warning,
    Error,
}
