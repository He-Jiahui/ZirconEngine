use zircon_runtime::ui::{layout::UiFrame, surface::UiSurface};

use super::control_ids::{
    BOTTOM_DRAWER_CONTENT_CONTROL_ID, BOTTOM_DRAWER_HEADER_CONTROL_ID,
    BOTTOM_DRAWER_SHELL_CONTROL_ID, LEFT_DRAWER_CONTENT_CONTROL_ID, LEFT_DRAWER_HEADER_CONTROL_ID,
    LEFT_DRAWER_SHELL_CONTROL_ID, RIGHT_DRAWER_CONTENT_CONTROL_ID, RIGHT_DRAWER_HEADER_CONTROL_ID,
    RIGHT_DRAWER_SHELL_CONTROL_ID,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinHostDrawerSourceFrames {
    pub left_drawer_shell_frame: Option<UiFrame>,
    pub left_drawer_header_frame: Option<UiFrame>,
    pub left_drawer_content_frame: Option<UiFrame>,
    pub right_drawer_shell_frame: Option<UiFrame>,
    pub right_drawer_header_frame: Option<UiFrame>,
    pub right_drawer_content_frame: Option<UiFrame>,
    pub bottom_drawer_shell_frame: Option<UiFrame>,
    pub bottom_drawer_header_frame: Option<UiFrame>,
    pub bottom_drawer_content_frame: Option<UiFrame>,
}

impl BuiltinHostDrawerSourceFrames {
    pub(crate) fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        match control_id {
            LEFT_DRAWER_SHELL_CONTROL_ID => self.left_drawer_shell_frame,
            LEFT_DRAWER_HEADER_CONTROL_ID => self.left_drawer_header_frame,
            LEFT_DRAWER_CONTENT_CONTROL_ID => self.left_drawer_content_frame,
            RIGHT_DRAWER_SHELL_CONTROL_ID => self.right_drawer_shell_frame,
            RIGHT_DRAWER_HEADER_CONTROL_ID => self.right_drawer_header_frame,
            RIGHT_DRAWER_CONTENT_CONTROL_ID => self.right_drawer_content_frame,
            BOTTOM_DRAWER_SHELL_CONTROL_ID => self.bottom_drawer_shell_frame,
            BOTTOM_DRAWER_HEADER_CONTROL_ID => self.bottom_drawer_header_frame,
            BOTTOM_DRAWER_CONTENT_CONTROL_ID => self.bottom_drawer_content_frame,
            _ => None,
        }
    }
}

pub(super) fn source_frames_from_surface(surface: &UiSurface) -> BuiltinHostDrawerSourceFrames {
    BuiltinHostDrawerSourceFrames {
        left_drawer_shell_frame: surface_control_frame(surface, LEFT_DRAWER_SHELL_CONTROL_ID),
        left_drawer_header_frame: surface_control_frame(surface, LEFT_DRAWER_HEADER_CONTROL_ID),
        left_drawer_content_frame: surface_control_frame(surface, LEFT_DRAWER_CONTENT_CONTROL_ID),
        right_drawer_shell_frame: surface_control_frame(surface, RIGHT_DRAWER_SHELL_CONTROL_ID),
        right_drawer_header_frame: surface_control_frame(surface, RIGHT_DRAWER_HEADER_CONTROL_ID),
        right_drawer_content_frame: surface_control_frame(surface, RIGHT_DRAWER_CONTENT_CONTROL_ID),
        bottom_drawer_shell_frame: surface_control_frame(surface, BOTTOM_DRAWER_SHELL_CONTROL_ID),
        bottom_drawer_header_frame: surface_control_frame(surface, BOTTOM_DRAWER_HEADER_CONTROL_ID),
        bottom_drawer_content_frame: surface_control_frame(
            surface,
            BOTTOM_DRAWER_CONTENT_CONTROL_ID,
        ),
    }
}

fn surface_control_frame(surface: &UiSurface, control_id: &str) -> Option<UiFrame> {
    let node_id = surface_control_node_id(surface, control_id)?;
    surface
        .tree
        .node(node_id)
        .map(|node| node.layout_cache.frame)
}

fn surface_control_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> Option<zircon_runtime::ui::event_ui::UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}
