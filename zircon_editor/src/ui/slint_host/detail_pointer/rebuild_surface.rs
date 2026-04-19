use zircon_ui::{
    dispatch::UiPointerDispatcher, event_ui::UiNodePath, event_ui::UiTreeId, UiAxis,
    UiContainerKind, UiFrame, UiInputPolicy, UiScrollState, UiScrollableBoxConfig,
    UiScrollbarVisibility, UiSurface, UiTreeNode,
};

use super::base_state::base_state;
use super::bridge_constants::{ROOT_NODE_ID, VIEWPORT_NODE_ID};
use super::scroll_surface_pointer_bridge::ScrollSurfacePointerBridge;
use super::viewport_frame::viewport_frame;

impl ScrollSurfacePointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new(self.tree_id));
        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                UiNodePath::new(format!("{}/root", self.path_prefix)),
            )
            .with_frame(UiFrame::new(
                0.0,
                0.0,
                self.layout.pane_size.width.max(0.0),
                self.layout.pane_size.height.max(0.0),
            ))
            .with_state_flags(base_state(false)),
        );

        let viewport = viewport_frame(&self.layout);
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    VIEWPORT_NODE_ID,
                    UiNodePath::new(format!("{}/viewport", self.path_prefix)),
                )
                .with_frame(viewport)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_clip_to_bounds(true)
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: self.state.scroll_offset,
                    viewport_extent: viewport.height.max(0.0),
                    content_extent: self.layout.content_extent.max(0.0),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("scroll surface root must exist");

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = UiPointerDispatcher::default();
    }
}
