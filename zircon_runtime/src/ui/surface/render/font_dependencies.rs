use std::collections::BTreeSet;

use zircon_runtime_interface::ui::tree::UiTree;

use super::resolve::resolve_style;
use crate::text::font::DEFAULT_UI_FONT_ASSET;

pub(in crate::ui::surface) fn text_font_asset_dependencies(tree: &UiTree) -> Vec<String> {
    let mut dependencies = BTreeSet::from([DEFAULT_UI_FONT_ASSET.to_string()]);
    for node in tree.nodes.values() {
        let style = resolve_style(node.template_metadata.as_ref());
        if let Some(font) = style.font.filter(|font| !font.trim().is_empty()) {
            dependencies.insert(font);
        }
    }
    dependencies.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use toml::Value;
    use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
    use zircon_runtime_interface::ui::tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode};

    use super::text_font_asset_dependencies;
    use crate::text::font::DEFAULT_UI_FONT_ASSET;

    #[test]
    fn retained_tree_dependencies_include_default_and_deduplicate_explicit_fonts() {
        let mut tree = UiTree::new(UiTreeId::new("runtime.ui.font-dependencies"));
        let mut first = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("first"));
        first.template_metadata = Some(metadata_with_font("res://fonts/project.font.toml"));
        let mut second = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("second"));
        second.template_metadata = Some(metadata_with_font("res://fonts/project.font.toml"));
        tree.insert_root(first);
        tree.insert_root(second);

        assert_eq!(
            text_font_asset_dependencies(&tree),
            vec![
                DEFAULT_UI_FONT_ASSET.to_string(),
                "res://fonts/project.font.toml".to_string(),
            ]
        );
    }

    fn metadata_with_font(font: &str) -> UiTemplateNodeMetadata {
        let mut metadata = UiTemplateNodeMetadata::default();
        metadata.style_overrides.insert(
            "font".to_string(),
            Value::Table(toml::map::Map::from_iter([(
                "asset".to_string(),
                Value::String(font.to_string()),
            )])),
        );
        metadata
    }
}
