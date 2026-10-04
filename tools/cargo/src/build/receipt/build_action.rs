//! 可复核构建动作的稳定输入。
//! BuildSet 去重与规范摘要共用 package、二进制目标和特征集合；特征顺序由规范键统一。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildAction {
    pub package: String,
    pub bin: Option<String>,
    pub features: Vec<String>,
}
