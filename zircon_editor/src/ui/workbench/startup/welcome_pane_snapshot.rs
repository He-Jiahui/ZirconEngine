use super::new_project_form_snapshot::NewProjectFormSnapshot;
use super::recent_project_item_snapshot::RecentProjectItemSnapshot;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 欢迎pane的只读输入；包含平台能力和本轮草稿提示，实际创建/打开由宿主执行。
pub struct WelcomePaneSnapshot {
    pub title: String,
    pub subtitle: String,
    pub status_message: String,
    pub browse_supported: bool,
    pub recent_projects: Vec<RecentProjectItemSnapshot>,
    pub form: NewProjectFormSnapshot,
}
