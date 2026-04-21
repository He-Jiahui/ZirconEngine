use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance};

use super::PreviewEditorData;

#[derive(Clone, Debug)]
pub struct PreviewFixture {
    pub layout: WorkbenchLayout,
    pub descriptors: Vec<ViewDescriptor>,
    pub instances: Vec<ViewInstance>,
    pub editor: PreviewEditorData,
}
