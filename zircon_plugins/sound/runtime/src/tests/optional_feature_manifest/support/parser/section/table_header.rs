// 非可选功能表会退出本次范围；静态扫描器只识别固定清单使用的表头语法。
use super::kind::OptionalFeatureSection;

impl OptionalFeatureSection {
    pub(in super::super) fn from_table_header(line: &str) -> Option<Self> {
        match line {
            "[[optional_features]]" => Some(Self::Feature),
            "[[optional_features.dependencies]]" => Some(Self::Dependency),
            "[[optional_features.modules]]" => Some(Self::Module),
            _ if line.starts_with("[[") => Some(Self::None),
            _ if line.starts_with('[') => Some(Self::None),
            _ => None,
        }
    }
}
