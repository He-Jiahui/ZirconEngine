use super::super::super::construction;
use super::super::super::*;

impl RuntimeSessionArchive {
    /// 读取有版本的会话文本并完成槽位元数据规范化与场景验证；不接受超出档案字节上限的输入。
    pub fn from_versioned_json(json: &str) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_versioned_json(json)
    }

    pub fn to_versioned_json_pretty(&self) -> Result<String, RuntimeSessionArchiveError> {
        construction::to_versioned_json_pretty(self)
    }
}
