#[path = "command/group.rs"]
mod group;
#[path = "command/image.rs"]
mod image;
#[path = "command/kind.rs"]
mod kind;
#[path = "command/model.rs"]
mod model;
#[path = "command/quad.rs"]
mod quad;
#[path = "command/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use kind::HostPaintCommandKind;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::{
    HostPaintCommand, PaintNodeIdentity,
};
