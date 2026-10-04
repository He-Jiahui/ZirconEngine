#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 创建草稿的显示投影；预览路径和按钮提示不能代替执行时的位置与项目authority验证。
pub struct NewProjectFormSnapshot {
    pub project_name: String,
    pub location: String,
    pub project_path_preview: String,
    pub template_label: String,
    pub can_create: bool,
    pub can_open_existing: bool,
    pub validation_message: String,
}
