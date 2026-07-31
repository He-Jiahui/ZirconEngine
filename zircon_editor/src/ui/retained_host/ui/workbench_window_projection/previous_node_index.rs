use std::collections::HashMap;

use crate::ui::retained_host::{primitives::ModelRc, TemplatePaneNodeData};

#[derive(Debug, PartialEq, Eq)]
struct WorkbenchProjectionIdentity {
    document_id: String,
}

pub(super) struct PreviousWorkbenchNodeIndex<'a> {
    by_control_id: HashMap<&'a str, &'a TemplatePaneNodeData>,
}

impl<'a> PreviousWorkbenchNodeIndex<'a> {
    pub(super) fn for_projection(
        nodes: &'a ModelRc<TemplatePaneNodeData>,
        document_id: &str,
    ) -> Option<Self> {
        let identity = nodes.metadata::<WorkbenchProjectionIdentity>()?;
        (identity.document_id.as_str() == document_id).then(|| Self {
            by_control_id: nodes
                .iter()
                .filter(|node| !node.control_id.is_empty())
                .map(|node| (node.control_id.as_str(), node))
                .collect(),
        })
    }

    pub(super) fn get(&self, control_id: &str) -> Option<&'a TemplatePaneNodeData> {
        self.by_control_id.get(control_id).copied()
    }
}

pub(super) fn model_with_projection_identity(
    nodes: Vec<TemplatePaneNodeData>,
    document_id: String,
) -> ModelRc<TemplatePaneNodeData> {
    ModelRc::with_metadata(nodes, WorkbenchProjectionIdentity { document_id })
}
