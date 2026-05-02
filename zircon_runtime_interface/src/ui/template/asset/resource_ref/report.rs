use serde::{Deserialize, Serialize};

use super::{UiResourceDependency, UiResourceDiagnostic};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiResourceCollectionReport {
    #[serde(default)]
    pub dependencies: Vec<UiResourceDependency>,
    #[serde(default)]
    pub diagnostics: Vec<UiResourceDiagnostic>,
}
