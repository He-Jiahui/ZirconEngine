// Avatar 入口认领角色节点；预览图、文字和回退图标走同一内容槽，预览图与表面边框按同一圆角框配合。
mod commands;
mod geometry;
mod glyph;
mod identity;
mod image;
mod mask;
mod style;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_avatar_primitive_commands;
