#[derive(Clone, Debug, PartialEq, Eq)]
/// 活动页宿主模式；独占页携具体实例身份，普通页由文档树提供内容。
pub enum MainHostStripModel {
    Workbench,
    ExclusiveWindow {
        instance_id: crate::ui::workbench::view::ViewInstanceId,
    },
}
