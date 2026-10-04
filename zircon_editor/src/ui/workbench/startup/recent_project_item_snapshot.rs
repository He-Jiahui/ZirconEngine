use crate::core::project::RecentProjectValidation;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// recent项目展示行；缓存校验和显示路径不代替打开时的真实root验证。
pub struct RecentProjectItemSnapshot {
    pub display_name: String,
    pub path: String,
    pub validation: RecentProjectValidation,
    pub last_opened_label: String,
    pub selected: bool,
}
