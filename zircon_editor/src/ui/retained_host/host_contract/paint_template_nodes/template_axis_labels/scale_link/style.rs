use super::super::super::super::data::TemplatePaneNodeData;
use super::super::style::scale_link_color;

pub(super) fn scale_link_asset_tint(node: &TemplatePaneNodeData) -> [u8; 4] {
    scale_link_color(node)
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
