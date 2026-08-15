use std::collections::HashMap;

use crate::ui::retained_host::{primitives::ModelRc, TemplatePaneNodeData};

#[derive(Debug, PartialEq, Eq)]
struct WorkbenchProjectionIdentity {
    document_id: String,
    row_by_control_id: HashMap<String, usize>,
}

pub(super) struct PreviousWorkbenchNodeIndex<'a> {
    nodes: &'a ModelRc<TemplatePaneNodeData>,
    row_by_control_id: &'a HashMap<String, usize>,
}

impl<'a> PreviousWorkbenchNodeIndex<'a> {
    pub(super) fn for_projection(
        nodes: &'a ModelRc<TemplatePaneNodeData>,
        document_id: &str,
    ) -> Option<Self> {
        let identity = nodes.metadata::<WorkbenchProjectionIdentity>()?;
        (identity.document_id.as_str() == document_id).then_some(Self {
            nodes,
            row_by_control_id: &identity.row_by_control_id,
        })
    }

    pub(super) fn get(&self, control_id: &str) -> Option<&'a TemplatePaneNodeData> {
        self.nodes.get(self.row(control_id)?)
    }

    pub(super) fn row(&self, control_id: &str) -> Option<usize> {
        self.row_by_control_id.get(control_id).copied()
    }
}

pub(super) fn model_with_projection_identity(
    nodes: Vec<TemplatePaneNodeData>,
    document_id: String,
) -> ModelRc<TemplatePaneNodeData> {
    let row_by_control_id = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| !node.control_id.is_empty())
        .map(|(row, node)| (node.control_id.to_string(), row))
        .collect();
    ModelRc::with_metadata(
        nodes,
        WorkbenchProjectionIdentity {
            document_id,
            row_by_control_id,
        },
    )
}
