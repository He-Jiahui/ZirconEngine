// Avatar 入口认领角色节点；预览图、文字和回退图标走同一内容槽，预览图与表面边框按同一圆角框配合。
#[path = "avatar/commands.rs"]
mod commands;
#[path = "avatar/geometry.rs"]
mod geometry;
#[path = "avatar/glyph.rs"]
mod glyph;
#[path = "avatar/identity.rs"]
mod identity;
#[path = "avatar/image.rs"]
mod image;
#[path = "avatar/mask.rs"]
mod mask;
#[path = "avatar/style.rs"]
mod style;
#[path = "avatar/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_avatar_primitive_commands;
