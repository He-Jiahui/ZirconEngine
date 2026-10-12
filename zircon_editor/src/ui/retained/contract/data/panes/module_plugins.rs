use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct ModulePluginsPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub diagnostics: SharedString,
}
