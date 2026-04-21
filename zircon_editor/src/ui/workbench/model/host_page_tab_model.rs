use crate::ui::workbench::layout::MainPageId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostPageTabModel {
    pub id: MainPageId,
    pub title: String,
    pub dirty: bool,
    pub closeable: bool,
}
