use serde::{Deserialize, Serialize};

use super::{UiResourceDependency, UiResourceDiagnostic};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 资产遍历结果同时保留按路径归属的依赖和声明诊断；它不包含运行时资源句柄。
pub struct UiResourceCollectionReport {
    #[serde(default)]
    pub dependencies: Vec<UiResourceDependency>,
    #[serde(default)]
    pub diagnostics: Vec<UiResourceDiagnostic>,
}
