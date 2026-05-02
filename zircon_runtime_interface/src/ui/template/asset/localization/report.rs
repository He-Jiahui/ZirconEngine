use serde::{Deserialize, Serialize};

use super::{UiLocalizationDiagnostic, UiLocalizedTextRef, UiTextDirection};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiLocalizationReport {
    pub dependencies: Vec<UiLocalizationDependency>,
    pub extraction_candidates: Vec<UiLocalizationTextCandidate>,
    pub diagnostics: Vec<UiLocalizationDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiLocalizationDependency {
    pub path: String,
    pub reference: UiLocalizedTextRef,
    pub direction: UiTextDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiLocalizationTextCandidate {
    pub path: String,
    pub text: String,
}
