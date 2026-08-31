use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostDocumentTabPointerItem {
    pub instance_id: ViewInstanceId,
    pub closeable: bool,
}
