use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    navigation::{UiNavigationGroup, UiNavigationGroupId},
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeError},
    widget::UiWidgetBehavior,
};

pub trait UiRuntimeTreeFocusExt {
    fn first_focusable_in_route(&self, route: &[UiNodeId])
        -> Result<Option<UiNodeId>, UiTreeError>;
    fn first_focusable_in_route_iter(
        &self,
        route: impl IntoIterator<Item = UiNodeId>,
    ) -> Result<Option<UiNodeId>, UiTreeError>;
    fn first_focusable_in_subtree(&self, root: UiNodeId) -> Result<Option<UiNodeId>, UiTreeError>;
    fn focusable_nodes_in_navigation_order(&self) -> Result<Vec<UiNodeId>, UiTreeError>;
    fn active_modal_focus_root(&self, current: Option<UiNodeId>) -> Option<UiNodeId>;
    fn active_modal_navigation_group_id(
        &self,
        current: Option<UiNodeId>,
    ) -> Option<UiNavigationGroupId>;
    fn active_modal_focus_allows_target(
        &self,
        current: Option<UiNodeId>,
        requested: UiNodeId,
    ) -> bool;
    fn node_is_in_modal_navigation_group(
        &self,
        node_id: UiNodeId,
        group_id: &UiNavigationGroupId,
    ) -> bool;
    fn node_is_descendant_of(&self, root: UiNodeId, node_id: UiNodeId) -> bool;
}

impl UiRuntimeTreeFocusExt for UiTree {
    fn first_focusable_in_route(
        &self,
        route: &[UiNodeId],
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        self.first_focusable_in_route_iter(route.iter().copied())
    }

    fn first_focusable_in_route_iter(
        &self,
        route: impl IntoIterator<Item = UiNodeId>,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        for node_id in route {
            let node = self
                .nodes
                .get(&node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if node.is_focus_candidate() {
                return Ok(Some(node_id));
            }
        }
        Ok(None)
    }

    fn first_focusable_in_subtree(&self, root: UiNodeId) -> Result<Option<UiNodeId>, UiTreeError> {
        first_focusable_in_subtree(self, root)
    }

    fn focusable_nodes_in_navigation_order(&self) -> Result<Vec<UiNodeId>, UiTreeError> {
        let mut focusable = Vec::new();
        for root_id in &self.roots {
            collect_focusable_nodes(self, *root_id, &mut focusable)?;
        }
        Ok(focusable)
    }

    fn active_modal_focus_root(&self, current: Option<UiNodeId>) -> Option<UiNodeId> {
        active_modal_scope_for(self, current).map(|scope| scope.root())
    }

    fn active_modal_navigation_group_id(
        &self,
        current: Option<UiNodeId>,
    ) -> Option<UiNavigationGroupId> {
        match active_modal_scope_for(self, current)? {
            ActiveModalScope::NavigationGroup { group_id, .. } => Some(group_id),
            ActiveModalScope::MuiOverlay(_) => None,
        }
    }

    fn active_modal_focus_allows_target(
        &self,
        current: Option<UiNodeId>,
        requested: UiNodeId,
    ) -> bool {
        active_modal_scope_for(self, current).is_none_or(|scope| match scope {
            ActiveModalScope::NavigationGroup { group_id, .. } => {
                modal_group_for(self, requested).as_ref() == Some(&group_id)
            }
            ActiveModalScope::MuiOverlay(root) => node_is_descendant_of(self, root, requested),
        })
    }

    fn node_is_in_modal_navigation_group(
        &self,
        node_id: UiNodeId,
        group_id: &UiNavigationGroupId,
    ) -> bool {
        modal_group_for(self, node_id).as_ref() == Some(group_id)
    }

    fn node_is_descendant_of(&self, root: UiNodeId, node_id: UiNodeId) -> bool {
        node_is_descendant_of(self, root, node_id)
    }
}

fn modal_group_for(tree: &UiTree, node_id: UiNodeId) -> Option<UiNavigationGroupId> {
    let mut current = Some(node_id);
    while let Some(node_id) = current {
        let node = tree.nodes.get(&node_id)?;
        if let Some(group) = node.navigation.group.as_ref().filter(|group| group.modal) {
            return Some(group.group_id.clone());
        }
        current = node.parent;
    }
    None
}

fn active_modal_scope_for(tree: &UiTree, current: Option<UiNodeId>) -> Option<ActiveModalScope> {
    let mut active_scope = None;
    if let Some(current) = current {
        if let Some((owner, group)) = active_modal_navigation_group_for_node(tree, current) {
            let root = group.root.unwrap_or(owner);
            retain_topmost_scope(
                &mut active_scope,
                modal_scope_rank(tree, root)?,
                ActiveModalScope::NavigationGroup {
                    group_id: group.group_id.clone(),
                    root,
                },
            );
        }
        if let Some(root) = active_mui_modal_root_for_node(tree, current) {
            retain_topmost_scope(
                &mut active_scope,
                modal_scope_rank(tree, root)?,
                ActiveModalScope::MuiOverlay(root),
            );
        }
    }
    if let Some(declared_scope) = topmost_active_declared_modal_scope(tree) {
        retain_topmost_scope(
            &mut active_scope,
            modal_scope_rank(tree, declared_scope.root())?,
            declared_scope,
        );
    }
    active_scope.map(|(_, scope)| scope)
}

fn active_mui_modal_root_for_node(tree: &UiTree, node_id: UiNodeId) -> Option<UiNodeId> {
    let mut current = Some(node_id);
    while let Some(node_id) = current {
        let node = tree.nodes.get(&node_id)?;
        if is_active_mui_modal_focus_scope(node) {
            return Some(node_id);
        }
        current = node.parent;
    }
    None
}

fn topmost_active_declared_modal_scope(tree: &UiTree) -> Option<ActiveModalScope> {
    let mut best = None;
    for node in tree.nodes.values() {
        if is_active_mui_modal_focus_scope(node) {
            retain_topmost_scope(
                &mut best,
                (node.z_index, node.paint_order, node.node_id),
                ActiveModalScope::MuiOverlay(node.node_id),
            );
        }
        let Some(group) = node.navigation.group.as_ref().filter(|group| group.modal) else {
            continue;
        };
        let root = group.root.unwrap_or(node.node_id);
        let Some(root_node) = tree.nodes.get(&root) else {
            continue;
        };
        let Some(metadata) = root_node.template_metadata.as_ref() else {
            continue;
        };
        if !metadata_declares_open(metadata)
            || !(bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
            || bool_attribute_any(metadata, &["disable_enforce_focus", "disableEnforceFocus"])
            || !root_node.state_flags.enabled
            || !root_node.is_render_visible()
        {
            continue;
        }
        retain_topmost_scope(
            &mut best,
            (root_node.z_index, root_node.paint_order, root),
            ActiveModalScope::NavigationGroup {
                group_id: group.group_id.clone(),
                root,
            },
        );
    }
    best.map(|(_, scope)| scope)
}

fn retain_topmost_scope(
    best: &mut Option<((i32, u64, UiNodeId), ActiveModalScope)>,
    rank: (i32, u64, UiNodeId),
    scope: ActiveModalScope,
) {
    if best.as_ref().is_none_or(|(best_rank, _)| rank > *best_rank) {
        *best = Some((rank, scope));
    }
}

fn modal_scope_rank(tree: &UiTree, root: UiNodeId) -> Option<(i32, u64, UiNodeId)> {
    let node = tree.nodes.get(&root)?;
    Some((node.z_index, node.paint_order, root))
}

#[derive(Clone)]
enum ActiveModalScope {
    NavigationGroup {
        group_id: UiNavigationGroupId,
        root: UiNodeId,
    },
    MuiOverlay(UiNodeId),
}

impl ActiveModalScope {
    const fn root(&self) -> UiNodeId {
        match self {
            Self::NavigationGroup { root, .. } | Self::MuiOverlay(root) => *root,
        }
    }
}

fn collect_focusable_nodes(
    tree: &UiTree,
    node_id: UiNodeId,
    focusable: &mut Vec<UiNodeId>,
) -> Result<(), UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    if node.is_focus_candidate() {
        focusable.push(node_id);
    }
    for child_id in &node.children {
        collect_focusable_nodes(tree, *child_id, focusable)?;
    }
    Ok(())
}

fn first_focusable_in_subtree(
    tree: &UiTree,
    root: UiNodeId,
) -> Result<Option<UiNodeId>, UiTreeError> {
    let node = tree
        .nodes
        .get(&root)
        .ok_or(UiTreeError::MissingNode(root))?;
    if node.is_focus_candidate() {
        return Ok(Some(root));
    }
    for child_id in &node.children {
        if let Some(target) = first_focusable_in_subtree(tree, *child_id)? {
            return Ok(Some(target));
        }
    }
    Ok(None)
}

fn node_is_descendant_of(tree: &UiTree, root: UiNodeId, node_id: UiNodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(current_id) = current {
        if current_id == root {
            return true;
        }
        current = tree.nodes.get(&current_id).and_then(|node| node.parent);
    }
    false
}

fn is_active_mui_modal_focus_scope(node: &zircon_runtime_interface::ui::tree::UiTreeNode) -> bool {
    let Some(metadata) = node.template_metadata.as_ref() else {
        return false;
    };
    is_mui_modal_focus_metadata(metadata)
        && node.state_flags.enabled
        && node.is_render_visible()
        && (bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
        && !bool_attribute_any(metadata, &["disable_enforce_focus", "disableEnforceFocus"])
}

fn active_modal_navigation_group_for_node(
    tree: &UiTree,
    node_id: UiNodeId,
) -> Option<(UiNodeId, &UiNavigationGroup)> {
    let mut current = Some(node_id);
    let (owner, group) = loop {
        let owner = current?;
        let node = tree.nodes.get(&owner)?;
        if let Some(group) = node.navigation.group.as_ref().filter(|group| group.modal) {
            break (owner, group);
        }
        current = node.parent;
    };
    let root = group.root.unwrap_or(owner);
    let root_node = tree.nodes.get(&root)?;
    if !root_node.state_flags.enabled || !root_node.is_render_visible() {
        return None;
    }
    let Some(metadata) = root_node.template_metadata.as_ref() else {
        return Some((owner, group));
    };
    if bool_attribute_any(metadata, &["disable_enforce_focus", "disableEnforceFocus"])
        || metadata_declares_open(metadata)
            && !(bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
    {
        return None;
    }
    Some((owner, group))
}

fn metadata_declares_open(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.attributes.contains_key("open") || metadata.attributes.contains_key("popup_open")
}

fn is_mui_modal_focus_metadata(metadata: &UiTemplateNodeMetadata) -> bool {
    is_mui_modal_focus_component(metadata.component.as_str())
        || metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Popup
}

fn is_mui_modal_focus_component(component: &str) -> bool {
    matches!(
        component,
        "Dialog" | "ConfirmDialog" | "Modal" | "Popover" | "Menu"
    )
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn bool_attribute_any(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> bool {
    keys.iter().any(|key| bool_attribute(metadata, key))
}
