use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::{class_list_offenders, class_name_prop_offenders};
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

#[test]
fn production_zui_class_lists_are_clean_and_unique() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_class_lists = 0usize;
    let mut checked_class_tokens = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                if !component.default_classes.is_empty() {
                    checked_class_lists += 1;
                    checked_class_tokens += component.default_classes.len();
                    let invalid_classes = class_list_offenders(&component.default_classes);
                    if !invalid_classes.is_empty() {
                        offenders.push(format!(
                            "{} component `{}` default_classes contain {invalid_classes:?}",
                            path.display(),
                            component_name
                        ));
                    }
                }
            }

            for (node_id, node) in &document.nodes {
                if !node.classes.is_empty() {
                    checked_class_lists += 1;
                    checked_class_tokens += node.classes.len();
                    let invalid_classes = class_list_offenders(&node.classes);
                    if !invalid_classes.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` classes contain {invalid_classes:?}",
                            path.display(),
                            node_id
                        ));
                    }
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_class_lists > 0 && checked_class_tokens > 0,
        "production .zui assets should declare component or node class tokens"
    );
    assert!(
        offenders.is_empty(),
        "production .zui class lists must not contain empty, whitespace-padded, whitespace-containing, or duplicate class tokens: {offenders:#?}"
    );
}

#[test]
fn production_zui_class_name_props_are_clean_style_anchors() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_prop_maps = 0usize;
    let mut checked_class_name_props = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                if node.props.is_empty() {
                    continue;
                }
                checked_prop_maps += 1;
                let (checked_class_names, invalid_class_names) =
                    class_name_prop_offenders(&node.props);
                checked_class_name_props += checked_class_names;
                if !invalid_class_names.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` props contain {invalid_class_names:?}",
                        path.display(),
                        node_id
                    ));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_prop_maps > 0 && checked_class_name_props > 0,
        "production .zui assets should declare Material/MUI-compatible className props"
    );
    assert!(
        offenders.is_empty(),
        "production .zui className, baseClassName, and slotProps.*.className props must contain clean selector-safe style anchors: {offenders:#?}"
    );
}
