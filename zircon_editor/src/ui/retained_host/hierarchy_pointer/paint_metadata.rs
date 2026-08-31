const HIERARCHY_VIEWPORT_CONTROL_IDS: [&str; 2] = ["HierarchyListPanel", "HierarchyTreeSlotAnchor"];

#[derive(Clone, Debug, Default)]
pub(crate) struct HierarchyPaintMetadata {
    viewport_node_rows: Vec<usize>,
}

impl HierarchyPaintMetadata {
    pub(crate) fn viewport_node_rows(&self) -> &[usize] {
        &self.viewport_node_rows
    }
}

pub(crate) fn hierarchy_paint_metadata<'a, I>(control_ids: I) -> HierarchyPaintMetadata
where
    I: Iterator<Item = &'a str>,
{
    let mut published = [false; HIERARCHY_VIEWPORT_CONTROL_IDS.len()];
    HierarchyPaintMetadata {
        viewport_node_rows: control_ids
            .enumerate()
            .filter_map(|(row, control_id)| {
                let identity = HIERARCHY_VIEWPORT_CONTROL_IDS
                    .iter()
                    .position(|candidate| *candidate == control_id)?;
                (!std::mem::replace(&mut published[identity], true)).then_some(row)
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_candidates_preserve_source_order_and_ignore_unrelated_nodes() {
        let metadata = hierarchy_paint_metadata(
            [
                "HierarchyHeaderPanel",
                "HierarchyTreeSlotAnchor",
                "HierarchyListPanel",
                "SelectRoot",
            ]
            .into_iter(),
        );

        assert_eq!(metadata.viewport_node_rows(), &[1, 2]);
    }

    #[test]
    fn duplicate_anchor_identities_publish_only_the_first_source_row() {
        let metadata = hierarchy_paint_metadata(
            [
                "HierarchyListPanel",
                "HierarchyListPanel",
                "HierarchyTreeSlotAnchor",
                "HierarchyTreeSlotAnchor",
            ]
            .into_iter(),
        );

        assert_eq!(metadata.viewport_node_rows(), &[0, 2]);
    }
}
