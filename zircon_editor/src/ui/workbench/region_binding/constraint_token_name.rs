use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// 作者资产中的尺寸令牌名；包装不验证存在性，映射布局DTO时按当前主题解析并诊断。
pub struct WorkbenchConstraintTokenName(String);

impl WorkbenchConstraintTokenName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
