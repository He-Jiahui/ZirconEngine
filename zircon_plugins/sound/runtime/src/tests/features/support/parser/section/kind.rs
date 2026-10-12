// 节类型只划分功能本体、依赖与模块，阻止同名字段进入错误的待提交记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum OptionalFeatureSection {
    None,
    Feature,
    Dependency,
    Module,
}

impl Default for OptionalFeatureSection {
    fn default() -> Self {
        Self::None
    }
}
