use serde::{Deserialize, Serialize};

use crate::ui::template::{UiActionPolicyReport, UiInvalidationReport, UiLocalizationReport};

use super::{
    UiCompiledAssetDependencyManifest, UiCompiledAssetHeader, UiCompiledAssetPackageProfile,
};

// TODO: [CR-UITEMPLATE-0003] Editor 报告称保留 `SourceDocument` 等作者态区段，
// 但当前运行时产物仅含 `report` 和 `compiled`；明确列表是目标合同还是实际内容。
/// 编译包的验证结果，汇集依赖快照、动作策略、本地化与失效报告。
/// `retained_sections` 和 `stripped_sections` 由 profile 生成，并非产物字节的检查结果。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetPackageValidationReport {
    pub profile: UiCompiledAssetPackageProfile,
    pub header: UiCompiledAssetHeader,
    pub dependencies: UiCompiledAssetDependencyManifest,
    pub retained_sections: Vec<UiCompiledAssetPackageSection>,
    pub stripped_sections: Vec<UiCompiledAssetPackageSection>,
    #[serde(default)]
    pub binding_lifecycle_stage: UiBindingPackageLifecycleStage,
    pub invalidation_report: UiInvalidationReport,
    pub action_policy_report: UiActionPolicyReport,
    pub localization_report: UiLocalizationReport,
}

/// 跨编译、装载和执行阶段共用的报告词汇；当前包生成器只发布 `Compiled`。
/// 后续阶段应由完成相应工作的执行者更新，序列化该枚举本身不推进生命周期。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingPackageLifecycleStage {
    #[default]
    Declared,
    Compiled,
    Loaded,
    Bound,
    Executed,
    Applied,
}

impl UiBindingPackageLifecycleStage {
    pub const ALL: [Self; 6] = [
        Self::Declared,
        Self::Compiled,
        Self::Loaded,
        Self::Bound,
        Self::Executed,
        Self::Applied,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Compiled => "compiled",
            Self::Loaded => "loaded",
            Self::Bound => "bound",
            Self::Executed => "executed",
            Self::Applied => "applied",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCompiledAssetPackageSection {
    RuntimeTemplateTree,
    RuntimeStyleValues,
    RuntimeBindings,
    SourceDocument,
    AuthoringDiagnostics,
    MigrationReport,
}
