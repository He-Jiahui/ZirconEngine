use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::UiPointerEventKind,
    tree::{UiInputPolicy, UiTreeNode},
};

pub(super) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const VIEWPORT_NODE_ID: UiNodeId = UiNodeId::new(2);

pub(super) struct AssetPointerSurfaceAuthority {
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    #[cfg(test)]
    generation: u64,
}

impl AssetPointerSurfaceAuthority {
    pub(super) fn new(
        tree_id: &str,
        root_path: &str,
        viewport_path: &str,
        root_frame: UiFrame,
        viewport_frame: UiFrame,
    ) -> Self {
        let mut surface = UiSurface::new(UiTreeId::new(tree_id));
        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new(root_path))
                .with_frame(root_frame)
                .with_state_flags(base_state(false)),
        );
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(VIEWPORT_NODE_ID, UiNodePath::new(viewport_path))
                    .with_frame(viewport_frame)
                    .with_z_index(10)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_clip_to_bounds(true)
                    .with_state_flags(base_state(true)),
            )
            .expect("asset pointer root must exist");
        surface.rebuild();

        let mut dispatcher = UiPointerDispatcher::default();
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        Self {
            surface,
            dispatcher,
            #[cfg(test)]
            generation: 1,
        }
    }

    pub(super) fn dispatch_event(&mut self, event: UiPointerEvent) -> Result<bool, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(dispatch.handled_by.or(dispatch.route.target) == Some(VIEWPORT_NODE_ID))
    }

    pub(super) fn patch_geometry(&mut self, root_frame: UiFrame, viewport_frame: UiFrame) {
        let root_changed = self
            .surface
            .tree
            .node(ROOT_NODE_ID)
            .map(|node| node.layout_cache.frame != root_frame)
            .unwrap_or(true);
        let viewport_changed = self
            .surface
            .tree
            .node(VIEWPORT_NODE_ID)
            .map(|node| node.layout_cache.frame != viewport_frame)
            .unwrap_or(true);
        if !root_changed && !viewport_changed {
            return;
        }
        if root_changed {
            self.surface
                .tree
                .node_mut(ROOT_NODE_ID)
                .expect("asset pointer root must exist")
                .layout_cache
                .frame = root_frame;
        }
        if viewport_changed {
            self.surface
                .tree
                .node_mut(VIEWPORT_NODE_ID)
                .expect("asset pointer viewport must exist")
                .layout_cache
                .frame = viewport_frame;
        }
        self.surface.rebuild();
    }

    #[cfg(test)]
    pub(super) fn node_count(&self) -> usize {
        self.surface.tree.nodes.len()
    }

    #[cfg(test)]
    pub(super) const fn generation(&self) -> u64 {
        self.generation
    }
}

pub(super) fn row_index_at_point(
    point_y: f32,
    first_row_y: f32,
    row_height: f32,
    row_gap: f32,
    row_count: usize,
) -> Option<usize> {
    if row_count == 0
        || !point_y.is_finite()
        || !first_row_y.is_finite()
        || !row_height.is_finite()
        || !row_gap.is_finite()
        || row_height <= 0.0
        || row_gap < 0.0
        || point_y < first_row_y
    {
        return None;
    }
    let stride = row_height + row_gap;
    if stride <= 0.0 || !stride.is_finite() {
        return None;
    }
    let row_index = ((point_y - first_row_y) / stride).floor() as usize;
    if row_index >= row_count {
        return None;
    }
    let row_y = first_row_y + row_index as f32 * stride;
    (point_y <= row_y + row_height).then_some(row_index)
}

fn register_handled_pointer_node(dispatcher: &mut UiPointerDispatcher, node_id: UiNodeId) {
    for kind in [
        UiPointerEventKind::Move,
        UiPointerEventKind::Down,
        UiPointerEventKind::Scroll,
    ] {
        dispatcher.register(node_id, kind, |_context| UiPointerDispatchEffect::handled());
    }
}

pub(super) fn base_state(interactive: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
