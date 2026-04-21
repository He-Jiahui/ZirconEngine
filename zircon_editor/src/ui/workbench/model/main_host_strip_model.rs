#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainHostStripModel {
    Workbench,
    ExclusiveWindow {
        instance_id: crate::ui::workbench::view::ViewInstanceId,
    },
}
