use zircon_runtime::ui::{
    event_ui::UiNodeId, event_ui::UiNodePath, event_ui::UiStateFlags, event_ui::UiTreeId,
    layout::UiFrame, surface::UiSurface, tree::UiTreeNode,
};

use super::super::constants::{VIEWPORT_SURFACE_NODE_ID, VIEWPORT_SURFACE_ROOT_ID};

pub(crate) struct SharedViewportPointerBridge {
    pub(super) surface: UiSurface,
    pub(super) viewport_node_id: UiNodeId,
}

impl SharedViewportPointerBridge {
    pub(crate) fn new(viewport_frame: UiFrame) -> Self {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.viewport"));
        surface.tree.insert_root(
            UiTreeNode::new(
                VIEWPORT_SURFACE_ROOT_ID,
                UiNodePath::new("editor.viewport.root"),
            )
            .with_frame(viewport_frame)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
        );
        surface
            .tree
            .insert_child(
                VIEWPORT_SURFACE_ROOT_ID,
                UiTreeNode::new(
                    VIEWPORT_SURFACE_NODE_ID,
                    UiNodePath::new("editor.viewport.surface"),
                )
                .with_frame(viewport_frame)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                }),
            )
            .expect("viewport bridge root must exist");
        surface.rebuild();

        Self {
            surface,
            viewport_node_id: VIEWPORT_SURFACE_NODE_ID,
        }
    }

    pub(crate) fn update_viewport_frame(&mut self, viewport_frame: UiFrame) {
        if let Some(root) = self.surface.tree.node_mut(VIEWPORT_SURFACE_ROOT_ID) {
            root.layout_cache.frame = viewport_frame;
            root.layout_cache.clip_frame = None;
        }
        if let Some(viewport) = self.surface.tree.node_mut(self.viewport_node_id) {
            viewport.layout_cache.frame = viewport_frame;
            viewport.layout_cache.clip_frame = None;
        }
        self.surface.rebuild();
    }
}
