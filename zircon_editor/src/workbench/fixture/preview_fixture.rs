use crate::{ViewDescriptor, ViewInstance, WorkbenchLayout};

use super::PreviewEditorData;

#[derive(Clone, Debug)]
pub struct PreviewFixture {
    pub layout: WorkbenchLayout,
    pub descriptors: Vec<ViewDescriptor>,
    pub instances: Vec<ViewInstance>,
    pub editor: PreviewEditorData,
}
