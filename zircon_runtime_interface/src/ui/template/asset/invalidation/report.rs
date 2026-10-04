use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{UiAssetChange, UiInvalidationDiagnostic, UiInvalidationImpact, UiInvalidationStage};

/// 失效图的分类结果；影响标记由阶段集合派生，避免调用方维护两份状态。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationReport {
    pub changes: Vec<UiAssetChange>,
    pub stages: BTreeSet<UiInvalidationStage>,
    pub impact: UiInvalidationImpact,
    pub diagnostics: Vec<UiInvalidationDiagnostic>,
}

impl UiInvalidationReport {
    /// 缓存命中没有新的输入差异，因此以空变更、阶段和诊断构造默认报告。
    pub fn cache_hit() -> Self {
        Self::default()
    }

    /// 以阶段集为准生成影响标记，并保留差异来源及诊断供调用方展示。
    pub fn from_stages(
        changes: Vec<UiAssetChange>,
        stages: BTreeSet<UiInvalidationStage>,
        diagnostics: Vec<UiInvalidationDiagnostic>,
    ) -> Self {
        let impact = UiInvalidationImpact::from_stages(&stages);
        Self {
            changes,
            stages,
            impact,
            diagnostics,
        }
    }
}
