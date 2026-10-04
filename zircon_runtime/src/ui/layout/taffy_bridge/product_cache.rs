use std::collections::BTreeMap;

use taffy::prelude::{AvailableSpace, NodeId, Size as TaffySize, Style, TaffyTree};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiLayoutEngineTaffyTreeBuildStats},
};

use super::{compute::TaffyLayoutChildFrame, UiTaffyChildContractScope};

#[derive(Debug, Default)]
pub(crate) struct TaffyParentProductCache {
    products: BTreeMap<UiNodeId, TaffyParentProduct>,
    #[cfg(test)]
    last_updates: BTreeMap<UiNodeId, TaffyParentProductUpdate>,
}

#[derive(Debug)]
struct TaffyParentProduct {
    taffy: TaffyTree<()>,
    parent_node: NodeId,
    parent_style: Style,
    last_available_size: Option<(f32, f32)>,
    last_frame: Option<UiFrame>,
    last_inherited_clip: Option<UiFrame>,
    ordered_children_revision: u64,
    children: Vec<TaffyRetainedChild>,
    // Retain the child handle projection so topology updates do not allocate a
    // second temporary vector before calling `TaffyTree::set_children`.
    taffy_children: Vec<NodeId>,
    child_index_by_id: BTreeMap<UiNodeId, usize>,
}

// SAFETY: This private product owns its tree, whose node context is `()`.
// Styles enter only through the bridge's numeric layout projections: Taffy's
// raw-pointer length representation contains tagged values, never calc handles.
// Moving the product transfers all owned storage; mutation requires `&mut self`.
unsafe impl Send for TaffyParentProduct {}

#[derive(Debug)]
struct TaffyRetainedChild {
    node_id: UiNodeId,
    taffy_node: NodeId,
    style: Style,
    relative_frame: UiFrame,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TaffyParentProductUpdate {
    pub tree_build: Option<UiLayoutEngineTaffyTreeBuildStats>,
    pub exact_contract_delta: bool,
    pub child_contract_visit_count: usize,
    pub child_style_update_count: usize,
    pub compute_reused: bool,
    pub child_layout_read_count: usize,
    pub published_child_frame_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TaffyParentProductError {
    pub kind: TaffyParentProductErrorKind,
    pub tree_build: UiLayoutEngineTaffyTreeBuildStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TaffyParentProductErrorKind {
    TreeBuild,
    Compute,
}

impl TaffyParentProductCache {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn compute_child_frames(
        &mut self,
        parent_id: UiNodeId,
        parent_style: Style,
        frame: UiFrame,
        child_node_ids: &[UiNodeId],
        child_styles: &[Style],
        output: &mut Vec<TaffyLayoutChildFrame>,
        contract_scope: UiTaffyChildContractScope,
        ordered_children_revision: u64,
        inherited_clip: Option<UiFrame>,
    ) -> Result<TaffyParentProductUpdate, TaffyParentProductError> {
        output.clear();
        if child_node_ids.len() != child_styles.len() {
            return Err(product_error(TaffyParentProductErrorKind::TreeBuild, 0));
        }

        #[cfg(test)]
        self.last_updates.remove(&parent_id);

        let (mut product, tree_build, details) = match self.products.remove(&parent_id) {
            Some(mut product) => match contract_scope {
                UiTaffyChildContractScope::Exact => {
                    if product.ordered_children_revision != ordered_children_revision {
                        return Err(product_error(TaffyParentProductErrorKind::TreeBuild, 0));
                    }
                    let details =
                        product.update_exact(parent_style, child_node_ids, child_styles)?;
                    (product, None, details)
                }
                UiTaffyChildContractScope::Full => {
                    let topology_changed = product.ordered_children_revision
                        != ordered_children_revision
                        || product.topology_changed(child_node_ids);
                    let details = product.update_full(
                        parent_style,
                        ordered_children_revision,
                        child_node_ids,
                        child_styles,
                    )?;
                    let tree_build =
                        topology_changed.then(|| tree_build_stats(details.created_node_count));
                    (product, tree_build, details)
                }
            },
            None => {
                if matches!(contract_scope, UiTaffyChildContractScope::Exact) {
                    return Err(product_error(TaffyParentProductErrorKind::TreeBuild, 0));
                }
                let product = TaffyParentProduct::new(
                    &parent_style,
                    ordered_children_revision,
                    child_node_ids,
                    child_styles,
                )?;
                let details = TaffyParentProductUpdateDetails {
                    created_node_count: child_node_ids.len().saturating_add(1),
                    child_contract_visit_count: child_node_ids.len(),
                    child_style_update_count: 0,
                };
                (
                    product,
                    Some(tree_build_stats(details.created_node_count)),
                    details,
                )
            }
        };

        let available_size = (frame.width.max(0.0), frame.height.max(0.0));
        let compute_reused = product.last_available_size == Some(available_size)
            && !product.taffy.dirty(product.parent_node).map_err(|_| {
                product_error(
                    TaffyParentProductErrorKind::Compute,
                    details.created_node_count,
                )
            })?;
        let child_layout_read_count = if compute_reused {
            0
        } else {
            product
                .taffy
                .compute_layout(
                    product.parent_node,
                    TaffySize {
                        width: AvailableSpace::Definite(available_size.0),
                        height: AvailableSpace::Definite(available_size.1),
                    },
                )
                .map_err(|_| {
                    product_error(
                        TaffyParentProductErrorKind::Compute,
                        details.created_node_count,
                    )
                })?;
            for child in &mut product.children {
                let layout = product.taffy.layout(child.taffy_node).map_err(|_| {
                    product_error(
                        TaffyParentProductErrorKind::Compute,
                        details.created_node_count,
                    )
                })?;
                child.relative_frame = UiFrame::new(
                    layout.location.x,
                    layout.location.y,
                    layout.size.width.max(0.0),
                    layout.size.height.max(0.0),
                );
            }
            product.children.len()
        };

        let publish_exact = compute_reused
            && matches!(contract_scope, UiTaffyChildContractScope::Exact)
            && product.last_frame == Some(frame)
            && product.last_inherited_clip == inherited_clip;
        if publish_exact {
            for node_id in child_node_ids.iter().copied() {
                let Some(index) = product.child_index_by_id.get(&node_id).copied() else {
                    return Err(product_error(
                        TaffyParentProductErrorKind::TreeBuild,
                        details.created_node_count,
                    ));
                };
                output.push(absolute_child_frame(&product.children[index], frame));
            }
        } else {
            output.extend(
                product
                    .children
                    .iter()
                    .map(|child| absolute_child_frame(child, frame)),
            );
        }
        product.last_available_size = Some(available_size);
        product.last_frame = Some(frame);
        product.last_inherited_clip = inherited_clip;

        self.products.insert(parent_id, product);
        let update = TaffyParentProductUpdate {
            tree_build,
            exact_contract_delta: matches!(contract_scope, UiTaffyChildContractScope::Exact),
            child_contract_visit_count: details.child_contract_visit_count,
            child_style_update_count: details.child_style_update_count,
            compute_reused,
            child_layout_read_count,
            published_child_frame_count: output.len(),
        };
        crate::profile_counter!(
            "runtime",
            "ui.layout.taffy.child_contract_visit_count",
            update.child_contract_visit_count,
        );
        crate::profile_counter!(
            "runtime",
            "ui.layout.taffy.child_style_update_count",
            update.child_style_update_count,
        );
        if update.exact_contract_delta {
            crate::profile_counter!(
                "runtime",
                "ui.layout.taffy.exact_contract_delta_count",
                1_u8,
            );
        }
        if update.compute_reused {
            crate::profile_counter!("runtime", "ui.layout.taffy.compute_reuse_count", 1_u8,);
        } else {
            crate::profile_counter!("runtime", "ui.layout.taffy.compute_count", 1_u8,);
        }
        crate::profile_counter!(
            "runtime",
            "ui.layout.taffy.child_layout_read_count",
            update.child_layout_read_count,
        );
        crate::profile_counter!(
            "runtime",
            "ui.layout.taffy.published_child_frame_count",
            update.published_child_frame_count,
        );
        #[cfg(test)]
        self.last_updates.insert(parent_id, update);
        Ok(update)
    }

    pub(crate) fn discard(&mut self, parent_id: UiNodeId) {
        self.products.remove(&parent_id);
        #[cfg(test)]
        self.last_updates.remove(&parent_id);
    }

    pub(crate) fn matches_order_revision(&self, parent_id: UiNodeId, revision: u64) -> bool {
        self.products
            .get(&parent_id)
            .is_some_and(|product| product.ordered_children_revision == revision)
    }

    pub(crate) fn active_children_contain_all(
        &self,
        parent_id: UiNodeId,
        node_ids: &[UiNodeId],
    ) -> bool {
        self.products.get(&parent_id).is_some_and(|product| {
            node_ids
                .iter()
                .all(|node_id| product.child_index_by_id.contains_key(node_id))
        })
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(UiNodeId) -> bool) {
        self.products.retain(|parent_id, _| keep(*parent_id));
        #[cfg(test)]
        {
            let products = &self.products;
            self.last_updates
                .retain(|parent_id, _| products.contains_key(parent_id));
        }
    }

    #[cfg(test)]
    pub(crate) fn product_counts(&self) -> (usize, usize) {
        (
            self.products.len(),
            self.products
                .values()
                .map(|product| product.children.len().saturating_add(1))
                .sum(),
        )
    }

    #[cfg(test)]
    pub(crate) fn last_update(&self, parent_id: UiNodeId) -> Option<TaffyParentProductUpdate> {
        self.last_updates.get(&parent_id).copied()
    }
}

impl TaffyParentProduct {
    fn new(
        parent_style: &Style,
        ordered_children_revision: u64,
        child_node_ids: &[UiNodeId],
        child_styles: &[Style],
    ) -> Result<Self, TaffyParentProductError> {
        let mut taffy = TaffyTree::new();
        taffy.disable_rounding();
        let mut children = Vec::with_capacity(child_node_ids.len());
        let mut taffy_children = Vec::with_capacity(child_node_ids.len());
        for (node_id, style) in child_node_ids.iter().copied().zip(child_styles) {
            let taffy_node = taffy.new_leaf(style.clone()).map_err(|_| {
                product_error(TaffyParentProductErrorKind::TreeBuild, children.len())
            })?;
            taffy_children.push(taffy_node);
            children.push(TaffyRetainedChild {
                node_id,
                taffy_node,
                style: style.clone(),
                relative_frame: UiFrame::default(),
            });
        }
        let parent_node = taffy
            .new_with_children(parent_style.clone(), &taffy_children)
            .map_err(|_| product_error(TaffyParentProductErrorKind::TreeBuild, children.len()))?;
        Ok(Self {
            taffy,
            parent_node,
            parent_style: parent_style.clone(),
            last_available_size: None,
            last_frame: None,
            last_inherited_clip: None,
            ordered_children_revision,
            child_index_by_id: index_children(&children),
            children,
            taffy_children,
        })
    }

    fn update_exact(
        &mut self,
        parent_style: Style,
        child_node_ids: &[UiNodeId],
        child_styles: &[Style],
    ) -> Result<TaffyParentProductUpdateDetails, TaffyParentProductError> {
        self.update_parent_style(parent_style)?;
        let mut child_style_update_count = 0usize;
        for (node_id, style) in child_node_ids.iter().copied().zip(child_styles) {
            let Some(index) = self.child_index_by_id.get(&node_id).copied() else {
                return Err(product_error(TaffyParentProductErrorKind::TreeBuild, 0));
            };
            let child = &mut self.children[index];
            if child.style != *style {
                self.taffy
                    .set_style(child.taffy_node, style.clone())
                    .map_err(|_| product_error(TaffyParentProductErrorKind::TreeBuild, 0))?;
                child.style = style.clone();
                child_style_update_count = child_style_update_count.saturating_add(1);
            }
        }
        Ok(TaffyParentProductUpdateDetails {
            created_node_count: 0,
            child_contract_visit_count: child_node_ids.len(),
            child_style_update_count,
        })
    }

    fn update_full(
        &mut self,
        parent_style: Style,
        ordered_children_revision: u64,
        child_node_ids: &[UiNodeId],
        child_styles: &[Style],
    ) -> Result<TaffyParentProductUpdateDetails, TaffyParentProductError> {
        self.update_parent_style(parent_style)?;
        if !self.topology_changed(child_node_ids)
            && self.ordered_children_revision == ordered_children_revision
        {
            let mut child_style_update_count = 0usize;
            for (child, style) in self.children.iter_mut().zip(child_styles) {
                if child.style != *style {
                    self.taffy
                        .set_style(child.taffy_node, style.clone())
                        .map_err(|_| product_error(TaffyParentProductErrorKind::TreeBuild, 0))?;
                    child.style = style.clone();
                    child_style_update_count = child_style_update_count.saturating_add(1);
                }
            }
            return Ok(TaffyParentProductUpdateDetails {
                created_node_count: 0,
                child_contract_visit_count: child_node_ids.len(),
                child_style_update_count,
            });
        }

        let previous_children = std::mem::take(&mut self.children);
        let mut previous_by_id = previous_children
            .into_iter()
            .map(|child| (child.node_id, child))
            .collect::<BTreeMap<_, _>>();
        let mut next_children = Vec::with_capacity(child_node_ids.len());
        let mut created_node_count = 0usize;
        let mut child_style_update_count = 0usize;
        for (node_id, style) in child_node_ids.iter().copied().zip(child_styles) {
            let child = if let Some(mut child) = previous_by_id.remove(&node_id) {
                if child.style != *style {
                    self.taffy
                        .set_style(child.taffy_node, style.clone())
                        .map_err(|_| {
                            product_error(
                                TaffyParentProductErrorKind::TreeBuild,
                                created_node_count,
                            )
                        })?;
                    child.style = style.clone();
                    child_style_update_count = child_style_update_count.saturating_add(1);
                }
                child
            } else {
                let taffy_node = self.taffy.new_leaf(style.clone()).map_err(|_| {
                    product_error(TaffyParentProductErrorKind::TreeBuild, created_node_count)
                })?;
                created_node_count = created_node_count.saturating_add(1);
                TaffyRetainedChild {
                    node_id,
                    taffy_node,
                    style: style.clone(),
                    relative_frame: UiFrame::default(),
                }
            };
            next_children.push(child);
        }

        self.taffy_children.clear();
        self.taffy_children
            .extend(next_children.iter().map(|child| child.taffy_node));
        self.taffy
            .set_children(self.parent_node, &self.taffy_children)
            .map_err(|_| {
                product_error(TaffyParentProductErrorKind::TreeBuild, created_node_count)
            })?;
        for retired in previous_by_id.into_values() {
            self.taffy.remove(retired.taffy_node).map_err(|_| {
                product_error(TaffyParentProductErrorKind::TreeBuild, created_node_count)
            })?;
        }
        self.children = next_children;
        self.child_index_by_id = index_children(&self.children);
        self.ordered_children_revision = ordered_children_revision;
        Ok(TaffyParentProductUpdateDetails {
            created_node_count,
            child_contract_visit_count: child_node_ids.len(),
            child_style_update_count,
        })
    }

    fn update_parent_style(&mut self, parent_style: Style) -> Result<(), TaffyParentProductError> {
        if self.parent_style != parent_style {
            self.taffy
                .set_style(self.parent_node, parent_style.clone())
                .map_err(|_| product_error(TaffyParentProductErrorKind::TreeBuild, 0))?;
            self.parent_style = parent_style;
        }
        Ok(())
    }

    fn topology_changed(&self, child_node_ids: &[UiNodeId]) -> bool {
        self.children.len() != child_node_ids.len()
            || self
                .children
                .iter()
                .zip(child_node_ids)
                .any(|(child, node_id)| child.node_id != *node_id)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TaffyParentProductUpdateDetails {
    created_node_count: usize,
    child_contract_visit_count: usize,
    child_style_update_count: usize,
}

fn index_children(children: &[TaffyRetainedChild]) -> BTreeMap<UiNodeId, usize> {
    children
        .iter()
        .enumerate()
        .map(|(index, child)| (child.node_id, index))
        .collect()
}

fn absolute_child_frame(
    child: &TaffyRetainedChild,
    parent_frame: UiFrame,
) -> TaffyLayoutChildFrame {
    TaffyLayoutChildFrame {
        node_id: child.node_id,
        frame: UiFrame::new(
            parent_frame.x + child.relative_frame.x,
            parent_frame.y + child.relative_frame.y,
            child.relative_frame.width,
            child.relative_frame.height,
        ),
    }
}

fn product_error(
    kind: TaffyParentProductErrorKind,
    created_node_count: usize,
) -> TaffyParentProductError {
    TaffyParentProductError {
        kind,
        tree_build: tree_build_stats(created_node_count),
    }
}

fn tree_build_stats(node_count: usize) -> UiLayoutEngineTaffyTreeBuildStats {
    UiLayoutEngineTaffyTreeBuildStats::new(u64::try_from(node_count).unwrap_or(u64::MAX))
}

#[cfg(test)]
#[path = "tests/product_cache.rs"]
mod tests;
