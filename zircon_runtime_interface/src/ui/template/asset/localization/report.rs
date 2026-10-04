use serde::{Deserialize, Serialize};

use super::{UiLocalizationDiagnostic, UiLocalizedTextRef, UiTextDirection};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 报告并列保存运行时文本依赖、编辑器提取候选和诊断，不把候选文本伪装成已解析依赖。
pub struct UiLocalizationReport {
    pub dependencies: Vec<UiLocalizationDependency>,
    pub extraction_candidates: Vec<UiLocalizationTextCandidate>,
    pub diagnostics: Vec<UiLocalizationDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 依赖项记录源路径、locale table/key 引用和方向，供编译诊断与编辑器定位未解析文本。
pub struct UiLocalizationDependency {
    pub path: String,
    pub reference: UiLocalizedTextRef,
    pub direction: UiTextDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 候选项是供编辑器提取的源文本及路径，不代表运行时查找过该文本。
pub struct UiLocalizationTextCandidate {
    pub path: String,
    pub text: String,
}
