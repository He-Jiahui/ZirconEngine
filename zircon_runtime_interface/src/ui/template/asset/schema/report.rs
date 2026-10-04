use serde::{Deserialize, Serialize};

use super::super::document::UiAssetDocument;
use super::policy::UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 迁移输出把新文档和过程报告成对返回，调用方据 can_edit 与诊断决定是否允许作者继续修改。
pub struct UiAssetMigrationOutcome {
    pub document: UiAssetDocument,
    pub report: UiAssetMigrationReport,
}

/// 源资产迁移的步骤、诊断和编辑许可，供调用方判定升级结果。
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
    /// 未来版本默认不可编辑；实际执行的步骤由迁移器后续追加。
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
/// 来源种类区分树文档、旧版本、扁平节点表和未来版本，决定迁移路径及编辑许可。
pub enum UiAssetSchemaSourceKind {
    CurrentTree,
    OlderTree,
    FlatNodeTable,
    FutureVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAssetMigrationStep {
    CurrentTreeValidated,
    SourceVersionBumped { from: u32, to: u32 },
    FlatNodeTableMaterialized,
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
