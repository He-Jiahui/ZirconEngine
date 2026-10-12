#[path = "../../../chip/style/palette/lookup.rs"]
mod lookup;
#[path = "../../../chip/style/palette/token.rs"]
mod token;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) use lookup::{
    chip_palette_main_from_host,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) use token::chip_color_token;
