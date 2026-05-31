use std::collections::BTreeSet;
use std::fs;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::support::{
    collect_zui_files, editor_asset_root, runtime_asset_root, split_widget_component_import,
};

fn registered_component_descriptor_ids() -> BTreeSet<String> {
    [
        UiComponentDescriptorRegistry::editor_showcase(),
        UiComponentDescriptorRegistry::material_editor_foundation(),
    ]
    .into_iter()
    .flat_map(|registry| {
        registry
            .descriptors()
            .map(|descriptor| descriptor.id.clone())
            .collect::<Vec<_>>()
    })
    .collect()
}

fn imported_widget_component_names(imports: &[String]) -> BTreeSet<String> {
    imports
        .iter()
        .filter_map(|import| {
            split_widget_component_import(import)
                .map(|(_asset_id, component_name)| component_name.trim().to_string())
        })
        .filter(|component_name| !component_name.is_empty())
        .collect()
}

#[test]
fn production_zui_node_component_names_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_node_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                checked_node_components += 1;
                let component = node.component.trim();
                if component.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` declares an empty component type",
                        path.display(),
                        node_id
                    ));
                    continue;
                }
                if component != node.component.as_str() {
                    offenders.push(format!(
                        "{} node `{}` declares whitespace-padded component type `{}`",
                        path.display(),
                        node_id,
                        node.component
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
        checked_node_components > 0,
        "production .zui assets should declare node component types"
    );
    assert!(
        offenders.is_empty(),
        "production .zui node component types must be non-empty and trimmed for stable prototype expansion and authoring diagnostics: {offenders:#?}"
    );
}

#[test]
fn production_zui_node_components_resolve_to_known_descriptors_or_imported_components() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let registered_component_ids = registered_component_descriptor_ids();
    let mut checked_assets = 0usize;
    let mut checked_node_components = 0usize;
    let mut checked_imported_component_names = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let local_component_names =
                document.components.keys().cloned().collect::<BTreeSet<_>>();
            let imported_component_names =
                imported_widget_component_names(&document.imports.widgets);
            checked_imported_component_names += imported_component_names.len();

            for (node_id, node) in &document.nodes {
                checked_node_components += 1;
                if node.component.trim().is_empty() {
                    continue;
                }
                if node.component.contains('#') {
                    offenders.push(format!(
                        "{} node `{}` uses inline component locator `{}`; .zui nodes must reference local/imported component names or registered descriptor ids",
                        path.display(),
                        node_id,
                        node.component
                    ));
                    continue;
                }
                if local_component_names.contains(&node.component)
                    || imported_component_names.contains(&node.component)
                    || registered_component_ids.contains(&node.component)
                {
                    continue;
                }
                offenders.push(format!(
                    "{} node `{}` references unresolved component `{}`",
                    path.display(),
                    node_id,
                    node.component
                ));
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_node_components > 0,
        "production .zui assets should declare node component types"
    );
    assert!(
        checked_imported_component_names == 0 || offenders.is_empty(),
        "production .zui assets with internal widget imports should resolve imported component names through imports.widgets fragments"
    );
    assert!(
        offenders.is_empty(),
        "production .zui node component types must resolve to a local component, an imported .zui component fragment, or a registered descriptor id: {offenders:#?}"
    );
}
