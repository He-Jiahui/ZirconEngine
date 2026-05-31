use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::template::UiStyleScope;

use super::metadata::string_token_metadata_offender;
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

#[test]
fn production_zui_component_names_are_clean_authoring_tokens() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for component_name in document.components.keys() {
                checked_components += 1;
                if let Some(invalid_name) =
                    string_token_metadata_offender(component_name, "component name")
                {
                    offenders.push(format!("{} declares {invalid_name}", path.display()));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_components > 0,
        "production .zui assets should declare component contracts"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component names must be non-empty, trimmed, and whitespace-free authoring tokens: {offenders:#?}"
    );
}

#[test]
fn production_zui_components_with_default_classes_expose_style_anchors() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components_with_defaults = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                if component.default_classes.is_empty() {
                    continue;
                }
                checked_components_with_defaults += 1;

                if !component
                    .default_classes
                    .iter()
                    .any(|class| string_token_metadata_offender(class, "default class").is_none())
                {
                    offenders.push(format!(
                        "{} component `{}` declares default_classes without any usable class token",
                        path.display(),
                        component_name
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
        checked_components_with_defaults > 0,
        "production .zui assets should include components with default style anchors"
    );
    assert!(
        offenders.is_empty(),
        "production .zui components that declare default_classes must expose at least one usable style anchor: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_style_scopes_are_closed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                checked_components += 1;
                if component.style_scope != UiStyleScope::Closed {
                    offenders.push(format!(
                        "{} component `{}` opens style_scope before public style parts are governed",
                        path.display(),
                        component_name
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
        checked_components > 0,
        "production .zui assets should declare component prototypes"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component style scopes must remain closed until public style parts and style penetration contracts are governed: {offenders:#?}"
    );
}
