use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::event_ui::{UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::tree::UiTreeNode;

use super::base_state::base_state;
use super::constants::{LEFT_STRIP_NODE_ID, RIGHT_STRIP_NODE_ID, ROOT_NODE_ID};
use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::insert_strip::insert_strip;
use super::root_frame::root_frame;

impl HostActivityRailPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.activity_rail.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.activity_rail.root"))
                .with_frame(root_frame(&self.layout))
                .with_state_flags(base_state(false)),
        );

        insert_strip(
            &mut surface,
            &mut dispatcher,
            &mut targets,
            ROOT_NODE_ID,
            LEFT_STRIP_NODE_ID,
            "editor.activity_rail.left",
            self.layout.left_strip_frame,
            &self.layout.left_tabs,
            super::host_activity_rail_pointer_side::HostActivityRailPointerSide::Left,
        );
        insert_strip(
            &mut surface,
            &mut dispatcher,
            &mut targets,
            ROOT_NODE_ID,
            RIGHT_STRIP_NODE_ID,
            "editor.activity_rail.right",
            self.layout.right_strip_frame,
            &self.layout.right_tabs,
            super::host_activity_rail_pointer_side::HostActivityRailPointerSide::Right,
        );
        surface.rebuild();

        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
