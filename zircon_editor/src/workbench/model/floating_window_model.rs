use crate::ShellFrame;
use crate::layout::MainPageId;
use crate::view::ViewInstanceId;

use super::document_tab_model::DocumentTabModel;

#[derive(Clone, Debug, PartialEq)]
pub struct FloatingWindowModel {
    pub window_id: MainPageId,
    pub title: String,
    pub requested_frame: ShellFrame,
    pub focused_view: Option<ViewInstanceId>,
    pub tabs: Vec<DocumentTabModel>,
}
