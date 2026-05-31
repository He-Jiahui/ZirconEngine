use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::v2::{UiV2AssetLoader, UiZuiAssetLoader};
use zircon_runtime_interface::ui::v2::{UiV2AssetKind, UI_V2_ASSET_SCHEMA_VERSION};

mod support;

mod class;
mod component;
mod component_root;
mod control;
mod event;
mod graph;
mod layout;
mod layout_axis;
mod metadata;
mod node_component;
mod node_metadata;
mod props;
mod slot;
mod slot_schema;
mod style;

use self::metadata::string_metadata_offender;
use self::support::{
    builtin_zui_asset_id_alias_for, collect_v2_ui_toml_files, collect_zui_files, editor_asset_root,
    is_component_directory_path, is_zui_component_import_asset_id, pascal_case_file_stem,
    production_widget_import_asset_ids, production_widget_import_zui_locators, resolve_res_locator,
    resource_locator_for_path, runtime_asset_root, split_import_fragment,
    split_widget_component_import, zui_component_import_path, BUILTIN_ZUI_ASSET_ID_ALIASES,
};

fn duplicate_entries<'a>(values: impl IntoIterator<Item = &'a String>) -> Vec<String> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() {
            *counts.entry(value).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter_map(|(value, count)| (count > 1).then(|| value.to_string()))
        .collect()
}

fn import_entry_metadata_offenders(
    path: &PathBuf,
    import_section: &str,
    imports: &[String],
) -> (usize, Vec<String>) {
    let mut offenders = Vec::new();

    for (import_index, import) in imports.iter().enumerate() {
        if let Some(invalid_import) = string_metadata_offender(import, "import entry") {
            offenders.push(format!(
                "{} {import_section} #{} declares {invalid_import}",
                path.display(),
                import_index + 1
            ));
        }
    }

    (imports.len(), offenders)
}

fn push_asset_header_metadata_offenders(
    path: &PathBuf,
    asset_id: &str,
    display_name: &str,
    offenders: &mut Vec<String>,
) {
    if let Some(invalid_asset_id) = string_metadata_offender(asset_id, "asset id") {
        offenders.push(format!("{} declares {invalid_asset_id}", path.display()));
    }
    if let Some(invalid_display_name) = string_metadata_offender(display_name, "asset display_name")
    {
        offenders.push(format!(
            "{} declares {invalid_display_name}",
            path.display()
        ));
    }
}

#[test]
fn production_zui_assets_live_in_component_directories_or_registered_alias_paths() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut component_directory_assets = 0usize;
    let mut alias_directory_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            if is_component_directory_path(asset_root, &path) {
                component_directory_assets += 1;
                continue;
            }

            let locator = resource_locator_for_path(asset_root, &path);
            if builtin_zui_asset_id_alias_for(&locator).is_some() {
                alias_directory_assets += 1;
                continue;
            }

            offenders.push(format!(
                "{} is outside a component directory and is not a registered builtin .zui alias path",
                path.display()
            ));
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        component_directory_assets > 0,
        "production .zui assets should primarily live under component directories"
    );
    assert!(
        alias_directory_assets <= BUILTIN_ZUI_ASSET_ID_ALIASES.len(),
        "registered alias paths are migration exceptions, not a second .zui directory convention"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component assets must live under component directories unless the file path is an explicit builtin alias exception: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_names_match_file_stems() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut prototype_named_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let expected_name = pascal_case_file_stem(&path);
            let expected_prototype_name = format!("{expected_name}Prototype");
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let component_name = document
                .components
                .keys()
                .next()
                .expect("UiZuiAssetLoader validates exactly one component");

            if component_name == &expected_prototype_name {
                prototype_named_assets += 1;
                continue;
            }
            if component_name != &expected_name {
                offenders.push(format!(
                    "{} declares component `{}` but expected `{}` or `{}`",
                    path.display(),
                    component_name,
                    expected_name,
                    expected_prototype_name
                ));
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        prototype_named_assets > 0,
        "production .zui assets should include explicit Prototype component names for lab/showcase prototypes"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component names must match the file stem PascalCase, with an optional Prototype suffix for authoring prototypes: {offenders:#?}"
    );
}

#[test]
fn production_v2_zui_widget_imports_resolve_to_named_components() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for import in &document.imports.widgets {
                let Some((asset_id, component_name)) = split_widget_component_import(import) else {
                    let import_asset_id = import
                        .split_once('#')
                        .map_or(import.as_str(), |(asset_id, _)| asset_id);
                    if is_zui_component_import_asset_id(import_asset_id.trim()) {
                        checked_imports += 1;
                        offenders.push(format!(
                            "{} imports `{}` without an explicit #ComponentName",
                            path.display(),
                            import
                        ));
                    }
                    continue;
                };
                if !is_zui_component_import_asset_id(asset_id) {
                    continue;
                }
                checked_imports += 1;

                let Some(component_path) = zui_component_import_path(asset_id, &asset_roots) else {
                    offenders.push(format!(
                        "{} imports `{}` but no production asset root resolves it",
                        path.display(),
                        import
                    ));
                    continue;
                };

                let component_source =
                    fs::read_to_string(&component_path).unwrap_or_else(|error| {
                        panic!("read component `{}`: {error}", component_path.display())
                    });
                let component_document = UiZuiAssetLoader::load_zui_str(&component_source)
                    .unwrap_or_else(|error| {
                        panic!("parse component `{}`: {error}", component_path.display())
                    });
                if !component_document.components.contains_key(component_name) {
                    offenders.push(format!(
                        "{} imports `{}` but `{}` declares {:?}",
                        path.display(),
                        import,
                        component_path.display(),
                        component_document.components.keys().collect::<Vec<_>>()
                    ));
                }
            }
        }
    }

    assert!(
        checked_imports > 0,
        "production v2 UI assets should import .zui component prototypes"
    );
    assert!(
        offenders.is_empty(),
        ".zui widget imports must name a real single-component asset: {offenders:#?}"
    );
}

#[test]
fn production_v2_widget_imports_use_zui_component_assets_only() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for import in &document.imports.widgets {
                checked_imports += 1;
                let (asset_id, fragment) = split_import_fragment(import);
                let asset_id = asset_id.trim();
                if !is_zui_component_import_asset_id(asset_id) {
                    offenders.push(format!(
                        "{} imports widget `{}` from a non-.zui component asset",
                        path.display(),
                        import
                    ));
                    continue;
                }
                if fragment.is_none_or(|fragment| fragment.trim().is_empty()) {
                    offenders.push(format!(
                        "{} imports widget `{}` without an explicit component fragment",
                        path.display(),
                        import
                    ));
                }
            }
        }
    }

    assert!(
        checked_imports > 0,
        "production v2 UI assets should import .zui widget prototypes"
    );
    assert!(
        offenders.is_empty(),
        "production v2 widget imports must point at .zui component assets or registered builtin .zui aliases: {offenders:#?}"
    );
}

#[test]
fn production_ui_import_entries_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (import_section, imports) in [
                ("imports.widgets", document.imports.widgets.as_slice()),
                ("imports.styles", document.imports.styles.as_slice()),
            ] {
                let (section_checked_imports, section_offenders) =
                    import_entry_metadata_offenders(&path, import_section, imports);
                checked_imports += section_checked_imports;
                offenders.extend(section_offenders);
            }
        }

        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (import_section, imports) in [
                ("imports.widgets", document.imports.widgets.as_slice()),
                ("imports.styles", document.imports.styles.as_slice()),
            ] {
                let (section_checked_imports, section_offenders) =
                    import_entry_metadata_offenders(&path, import_section, imports);
                checked_imports += section_checked_imports;
                offenders.extend(section_offenders);
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain UI v2 or .zui assets"
    );
    assert!(
        checked_imports > 0,
        "production UI assets should declare widget or style imports"
    );
    assert!(
        offenders.is_empty(),
        "production UI import entries must be non-empty and trimmed before dependency resolution: {offenders:#?}"
    );
}

#[test]
fn production_ui_import_lists_do_not_repeat_dependencies() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            checked_imports += document.imports.widgets.len() + document.imports.styles.len();

            let duplicate_widgets = duplicate_entries(&document.imports.widgets);
            if !duplicate_widgets.is_empty() {
                offenders.push(format!(
                    "{} repeats widget imports {duplicate_widgets:?}",
                    path.display()
                ));
            }

            let duplicate_styles = duplicate_entries(&document.imports.styles);
            if !duplicate_styles.is_empty() {
                offenders.push(format!(
                    "{} repeats style imports {duplicate_styles:?}",
                    path.display()
                ));
            }
        }

        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            checked_imports += document.imports.widgets.len() + document.imports.styles.len();

            let duplicate_widgets = duplicate_entries(&document.imports.widgets);
            if !duplicate_widgets.is_empty() {
                offenders.push(format!(
                    "{} repeats widget imports {duplicate_widgets:?}",
                    path.display()
                ));
            }

            let duplicate_styles = duplicate_entries(&document.imports.styles);
            if !duplicate_styles.is_empty() {
                offenders.push(format!(
                    "{} repeats style imports {duplicate_styles:?}",
                    path.display()
                ));
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain UI v2 or .zui assets"
    );
    assert!(
        checked_imports > 0,
        "production UI assets should declare widget or style imports"
    );
    assert!(
        offenders.is_empty(),
        "production UI import lists must not repeat widget or style dependencies within the same asset: {offenders:#?}"
    );
}

#[test]
fn production_v2_ui_toml_assets_are_view_or_style_roots_only() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut view_assets = 0usize;
    let mut style_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            match document.asset.kind {
                UiV2AssetKind::View => view_assets += 1,
                UiV2AssetKind::Style => style_assets += 1,
                UiV2AssetKind::Component => offenders.push(format!(
                    "{} declares component kind; production component assets must use .zui",
                    path.display()
                )),
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .v2.ui.toml assets"
    );
    assert!(
        view_assets > 0,
        "production .v2.ui.toml roots should include view assets"
    );
    assert!(
        style_assets > 0,
        "production .v2.ui.toml roots should include style assets"
    );
    assert!(
        offenders.is_empty(),
        "production .v2.ui.toml files are reserved for view/style roots; component prototypes must use .zui: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_assets_are_reachable_from_v2_widget_imports() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let referenced_zui_locators = production_widget_import_zui_locators(&asset_roots);
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let expected_locator = resource_locator_for_path(asset_root, &path);
            if !referenced_zui_locators.contains(&expected_locator) {
                offenders.push(format!(
                    "{} is not reachable from any production .v2.ui.toml widget import",
                    path.display()
                ));
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        !referenced_zui_locators.is_empty(),
        "production .v2.ui.toml roots should reference .zui component assets"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component assets must remain reachable from direct res:// .zui widget imports or registered builtin aliases: {offenders:#?}"
    );
}

#[test]
fn production_v2_style_imports_resolve_to_style_assets() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for import in &document.imports.styles {
                checked_imports += 1;
                let (asset_id, fragment) = split_import_fragment(import);
                let asset_id = asset_id.trim();
                if !asset_id.starts_with("res://") {
                    offenders.push(format!(
                        "{} imports style `{}` without a res:// locator",
                        path.display(),
                        import
                    ));
                    continue;
                }
                if fragment.is_some() {
                    offenders.push(format!(
                        "{} imports style `{}` with an unsupported component fragment",
                        path.display(),
                        import
                    ));
                    continue;
                }

                let Some(style_path) = resolve_res_locator(asset_id, &asset_roots) else {
                    offenders.push(format!(
                        "{} imports style `{}` but no production asset root contains it",
                        path.display(),
                        import
                    ));
                    continue;
                };

                let style_source = fs::read_to_string(&style_path).unwrap_or_else(|error| {
                    panic!("read style `{}`: {error}", style_path.display())
                });
                let style_document =
                    UiV2AssetLoader::load_toml_str(&style_source).unwrap_or_else(|error| {
                        panic!("parse style `{}`: {error}", style_path.display())
                    });
                if style_document.asset.kind != UiV2AssetKind::Style {
                    offenders.push(format!(
                        "{} imports style `{}` but `{}` declares {:?}",
                        path.display(),
                        import,
                        style_path.display(),
                        style_document.asset.kind
                    ));
                }
            }
        }
    }

    assert!(
        checked_imports > 0,
        "production v2 UI assets should import shared style assets"
    );
    assert!(
        offenders.is_empty(),
        "production v2 style imports must resolve to style assets: {offenders:#?}"
    );
}

#[test]
fn production_zui_internal_imports_follow_component_and_style_boundaries() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for import in &document.imports.widgets {
                checked_imports += 1;
                let (asset_id, fragment) = split_import_fragment(import);
                let asset_id = asset_id.trim();
                if !is_zui_component_import_asset_id(asset_id) {
                    offenders.push(format!(
                        "{} imports widget `{}` from a non-.zui component asset",
                        path.display(),
                        import
                    ));
                    continue;
                }
                if fragment.is_none_or(|fragment| fragment.trim().is_empty()) {
                    offenders.push(format!(
                        "{} imports widget `{}` without an explicit component fragment",
                        path.display(),
                        import
                    ));
                    continue;
                }
                let Some(component_path) = zui_component_import_path(asset_id, &asset_roots) else {
                    offenders.push(format!(
                        "{} imports widget `{}` but no production asset root resolves it",
                        path.display(),
                        import
                    ));
                    continue;
                };
                let component_name = fragment.expect("component fragment validated above").trim();
                let component_source =
                    fs::read_to_string(&component_path).unwrap_or_else(|error| {
                        panic!("read component `{}`: {error}", component_path.display())
                    });
                let component_document = UiZuiAssetLoader::load_zui_str(&component_source)
                    .unwrap_or_else(|error| {
                        panic!("parse component `{}`: {error}", component_path.display())
                    });
                if !component_document.components.contains_key(component_name) {
                    offenders.push(format!(
                        "{} imports `{}` but `{}` declares {:?}",
                        path.display(),
                        import,
                        component_path.display(),
                        component_document.components.keys().collect::<Vec<_>>()
                    ));
                }
            }

            for import in &document.imports.styles {
                checked_imports += 1;
                let (asset_id, fragment) = split_import_fragment(import);
                let asset_id = asset_id.trim();
                if !asset_id.starts_with("res://") {
                    offenders.push(format!(
                        "{} imports style `{}` without a res:// locator",
                        path.display(),
                        import
                    ));
                    continue;
                }
                if fragment.is_some() {
                    offenders.push(format!(
                        "{} imports style `{}` with an unsupported component fragment",
                        path.display(),
                        import
                    ));
                    continue;
                }
                let Some(style_path) = resolve_res_locator(asset_id, &asset_roots) else {
                    offenders.push(format!(
                        "{} imports style `{}` but no production asset root contains it",
                        path.display(),
                        import
                    ));
                    continue;
                };
                let style_source = fs::read_to_string(&style_path).unwrap_or_else(|error| {
                    panic!("read style `{}`: {error}", style_path.display())
                });
                let style_document =
                    UiV2AssetLoader::load_toml_str(&style_source).unwrap_or_else(|error| {
                        panic!("parse style `{}`: {error}", style_path.display())
                    });
                if style_document.asset.kind != UiV2AssetKind::Style {
                    offenders.push(format!(
                        "{} imports style `{}` but `{}` declares {:?}",
                        path.display(),
                        import,
                        style_path.display(),
                        style_document.asset.kind
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
        checked_imports == 0 || offenders.is_empty(),
        "production .zui internal imports must follow the same component/style asset boundaries as v2 roots: {offenders:#?}"
    );
}

#[test]
fn production_zui_widget_imports_do_not_self_reference() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let current_locator = resource_locator_for_path(asset_root, &path);
            let current_alias = builtin_zui_asset_id_alias_for(&current_locator);
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for import in &document.imports.widgets {
                checked_imports += 1;
                let (asset_id, _fragment) = split_import_fragment(import);
                let asset_id = asset_id.trim();
                if asset_id == current_locator
                    || Some(asset_id) == current_alias
                    || asset_id == document.asset.id.as_str()
                {
                    offenders.push(format!(
                        "{} imports itself as widget `{}`",
                        path.display(),
                        import
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
        checked_imports == 0 || offenders.is_empty(),
        "production .zui widget imports must not self-reference the component asset being expanded: {offenders:#?}"
    );
}

#[test]
fn production_zui_asset_ids_match_res_locator_or_registered_builtin_alias() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let referenced_widget_asset_ids = production_widget_import_asset_ids(&asset_roots);
    let mut checked_asset_ids = 0usize;
    let mut locator_asset_ids = 0usize;
    let mut observed_alias_locators = BTreeSet::new();
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_asset_ids += 1;
            let expected_locator = resource_locator_for_path(asset_root, &path);
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let actual_asset_id = document.asset.id.as_str();

            if actual_asset_id == expected_locator {
                locator_asset_ids += 1;
                continue;
            }

            if builtin_zui_asset_id_alias_for(&expected_locator) == Some(actual_asset_id) {
                observed_alias_locators.insert(expected_locator.clone());
                if !referenced_widget_asset_ids.contains(actual_asset_id) {
                    offenders.push(format!(
                        "{} declares builtin alias `{}` but no production .v2.ui.toml widget import references it",
                        path.display(),
                        actual_asset_id
                    ));
                }
                continue;
            }

            offenders.push(format!(
                "{} declares asset.id `{}` but expected `{}` or an explicitly registered builtin alias",
                path.display(),
                actual_asset_id,
                expected_locator
            ));
        }
    }

    for (locator, asset_id) in BUILTIN_ZUI_ASSET_ID_ALIASES {
        if !observed_alias_locators.contains(*locator) {
            offenders.push(format!(
                "builtin .zui alias `{asset_id}` for `{locator}` is registered but no matching production .zui asset was found"
            ));
        }
        if !referenced_widget_asset_ids.contains(*asset_id) {
            offenders.push(format!(
                "builtin .zui alias `{asset_id}` for `{locator}` is registered but no production .v2.ui.toml widget import references it"
            ));
        }
    }

    assert!(
        checked_asset_ids > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        locator_asset_ids > 0,
        "production .zui assets should primarily use res:// asset ids"
    );
    assert!(
        offenders.is_empty(),
        ".zui asset ids must match their res:// locator unless an explicit builtin alias is registered and referenced: {offenders:#?}"
    );
}

#[test]
fn production_ui_asset_ids_are_unique_across_v2_roots_and_zui_components() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut asset_ids = BTreeMap::<String, Vec<PathBuf>>::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            asset_ids
                .entry(document.asset.id.clone())
                .or_default()
                .push(path);
        }

        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            asset_ids
                .entry(document.asset.id.clone())
                .or_default()
                .push(path);
        }
    }

    let offenders = asset_ids
        .into_iter()
        .filter(|(_asset_id, paths)| paths.len() > 1)
        .map(|(asset_id, paths)| {
            let paths = paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>();
            format!("{asset_id} -> {paths:#?}")
        })
        .collect::<Vec<_>>();

    assert!(
        checked_assets > 0,
        "production asset roots should contain UI v2 or .zui assets"
    );
    assert!(
        offenders.is_empty(),
        "production UI asset ids must be globally unique across .v2.ui.toml view/style roots and .zui component assets: {offenders:#?}"
    );
}

#[test]
fn production_ui_asset_headers_are_authorable_and_current() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            if document.asset.version != UI_V2_ASSET_SCHEMA_VERSION {
                offenders.push(format!(
                    "{} declares schema version {}, expected {}",
                    path.display(),
                    document.asset.version,
                    UI_V2_ASSET_SCHEMA_VERSION
                ));
            }
            push_asset_header_metadata_offenders(
                &path,
                &document.asset.id,
                &document.asset.display_name,
                &mut offenders,
            );
        }

        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            if document.asset.version != UI_V2_ASSET_SCHEMA_VERSION {
                offenders.push(format!(
                    "{} declares schema version {}, expected {}",
                    path.display(),
                    document.asset.version,
                    UI_V2_ASSET_SCHEMA_VERSION
                ));
            }
            push_asset_header_metadata_offenders(
                &path,
                &document.asset.id,
                &document.asset.display_name,
                &mut offenders,
            );
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain UI v2 or .zui assets"
    );
    assert!(
        offenders.is_empty(),
        "production UI asset headers must use the current schema version and non-empty, trimmed author-facing asset id/display_name fields: {offenders:#?}"
    );
}

#[test]
fn builtin_template_registry_does_not_register_zui_component_assets() {
    let offenders = crate::ui::template_runtime::builtin::builtin_template_documents()
        .into_iter()
        .filter_map(|(document_id, path)| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|file_name| file_name.ends_with(".zui"))
                .then(|| format!("{document_id} -> {}", path.display()))
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        ".zui files are component prototypes imported by v2 view/style documents, not directly registered builtin template documents: {offenders:#?}"
    );
}
