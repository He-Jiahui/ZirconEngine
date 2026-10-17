use serde::{Deserialize, Serialize};

use super::{UiResourceDependency, UiResourceDiagnostic};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 资产遍历结果同时保留按路径归属的依赖和声明诊断；它不包含运行时资源句柄。
/// 两个列表拥有收集时的快照；结果不会随收集器或资源注册表自动更新。
pub struct UiResourceCollectionReport {
    #[serde(default)]
    /// 完整声明及其来源；排序和去重由收集器负责，此载荷不执行加载。
    pub dependencies: Vec<UiResourceDependency>,
    #[serde(default)]
    /// 静态声明诊断的拥有列表；缺失字段按 Serde 默认值恢复为空。
    pub diagnostics: Vec<UiResourceDiagnostic>,
}
