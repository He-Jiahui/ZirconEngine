use super::pane_action_model::PaneActionModel;

#[derive(Clone, Debug, PartialEq)]
/// 内容未就绪时的提示和恢复入口；业务判断来自chrome，动作仍由宿主命令链执行。
pub struct PaneEmptyStateModel {
    pub title: String,
    pub body: String,
    pub primary_action: Option<PaneActionModel>,
    pub secondary_action: Option<PaneActionModel>,
    pub secondary_hint: Option<String>,
}
