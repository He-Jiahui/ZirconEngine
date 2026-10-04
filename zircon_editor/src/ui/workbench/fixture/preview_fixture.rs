use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};

use super::PreviewEditorData;

#[derive(Clone, Debug)]
/// 工作台预览的跨领域样本；布局、实例和数据属于同一场景，不是持久化项目文档。
pub struct PreviewFixture {
    pub layout: WorkbenchLayout,
    pub descriptors: Vec<ViewDescriptor>,
    pub instances: Vec<ViewInstance>,
    pub editor: PreviewEditorData,
}
