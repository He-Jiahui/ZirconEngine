use crate::ui::retained_host::primitives::ModelRc;

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct PerformanceTimelinePaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}
