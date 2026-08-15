use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    surface::{UiRenderCommandKind, UiRenderList, UiResolvedStyle},
    tree::{UiInputPolicy, UiVisibility},
};

use super::*;

fn extract(commands: Vec<UiRenderCommand>) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("ui.cache.multi-command"),
        list: UiRenderList { commands },
        raster_scale: 1.0,
    }
}

fn quad(node_id: u64, frame: UiFrame) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

mod damage;
mod geometry_patch;
mod update;
