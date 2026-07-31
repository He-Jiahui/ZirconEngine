use std::collections::BTreeSet;

use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetDependencyImpact {
    pub(crate) generation: u64,
    pub(crate) changed_asset_ids: BTreeSet<String>,
    pub(crate) direct_instances: BTreeSet<ViewInstanceId>,
    pub(crate) import_instances: BTreeSet<ViewInstanceId>,
}

impl UiAssetDependencyImpact {
    pub(crate) fn is_empty(&self) -> bool {
        self.direct_instances.is_empty() && self.import_instances.is_empty()
    }
}
