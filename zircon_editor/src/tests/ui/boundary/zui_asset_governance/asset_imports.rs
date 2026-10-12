use std::collections::BTreeSet;

use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::metadata::{duplicate_entries, import_entry_metadata_offenders};
use super::support::{
    collect_ui_root_document_files, collect_zui_document_files, collect_zui_files,
    collect_zui_view_style_files, editor_asset_root, is_ui_root_kind,
    is_zui_component_import_asset_id, load_zui_document, production_widget_import_zui_locators,
    resolve_res_locator, resource_locator_for_path, runtime_asset_root, split_import_fragment,
    split_widget_component_import, zui_component_import_path,
};
use crate::ui::workbench::FloatingWindow;

#[test]
fn production_ui_root_zui_widget_imports_resolve_to_named_components() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_ui_root_document_files(&asset_root.join("ui")) {
            let document = load_zui_document(&path);

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

                let component_document = load_zui_document(&component_path);
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
        "production UI root assets should import .zui component prototypes"
    );
    assert!(
        offenders.is_empty(),
        ".zui widget imports must name a real single-component asset: {offenders:#?}"
    );
}

#[test]
fn production_ui_root_widget_imports_use_zui_component_assets_only() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_ui_root_document_files(&asset_root.join("ui")) {
            let document = load_zui_document(&path);

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
        "production UI root assets should import .zui widget prototypes"
    );
    assert!(
        offenders.is_empty(),
        "production UI root widget imports must point at .zui component assets or registered builtin .zui aliases: {offenders:#?}"
    );
}

#[test]
fn production_ui_import_entries_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_ui_root_document_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let document = load_zui_document(&path);

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
            let document = load_zui_document(&path);

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
        for path in collect_ui_root_document_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let document = load_zui_document(&path);
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
            let document = load_zui_document(&path);
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
fn production_view_style_roots_are_zui_documents() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut zui_root_assets = 0usize;
    let mut view_assets = 0usize;
    let mut style_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_view_style_files(&asset_root.join("ui")) {
            zui_root_assets += 1;
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if !file_name.ends_with(".zui") {
                offenders.push(format!(
                    "{} was collected as a .zui view/style root without the .zui suffix",
                    path.display()
                ));
            }
            let document = load_zui_document(&path);
            match document.asset.kind {
                UiV2AssetKind::View => view_assets += 1,
                UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => style_assets += 1,
                UiV2AssetKind::Component => {
                    offenders.push(format!(
                        "{} was collected as a .zui view/style root but declares component kind",
                        path.display()
                    ));
                }
            }
            if !is_ui_root_kind(document.asset.kind) {
                offenders.push(format!(
                    "{} was collected as a .zui view/style root but declares {:?}",
                    path.display(),
                    document.asset.kind
                ));
            }
        }
    }

    assert!(
        zui_root_assets > 0,
        "production UI roots should exist as .zui documents"
    );
    assert!(
        view_assets > 0,
        "production .zui root documents should include view assets"
    );
    assert!(
        style_assets > 0,
        "production .zui root documents should include style assets"
    );
    assert!(
        offenders.is_empty(),
        "view/style roots must be recognized as .zui documents: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_assets_are_reachable_from_ui_root_widget_imports() {
    let _environment = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut referenced_zui_locators = production_widget_import_zui_locators(&asset_roots);
    referenced_zui_locators.extend(workbench_design_contract_zui_locators());
    referenced_zui_locators.extend(verified_live_component_zui_locators(&asset_roots));
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let expected_locator = resource_locator_for_path(asset_root, &path);
            if !referenced_zui_locators.contains(&expected_locator) {
                offenders.push(format!(
                    "{} is not reachable from any production UI root widget import, Workbench design contract, or verified live consumer",
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
        "production UI roots should reference .zui component assets"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component assets must remain reachable from direct or transitive res:// .zui widget imports, registered builtin aliases, or typed Workbench design contracts: {offenders:#?}"
    );
}

fn verified_live_component_zui_locators(asset_roots: &[std::path::PathBuf]) -> BTreeSet<String> {
    use std::collections::BTreeMap;

    use zircon_runtime::ui::v2::{UiV2PrototypeStoreFileCache, UiV2SourceFileReceipt};

    use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use crate::ui::retained_host::UiHostWindow;

    // The caller holds the environment lock. Each existing driver returns only
    // receipts from the owner whose live paint, input, and retirement it checked.
    let consumer_sources = [
        BuiltinWorkbenchWindowTemplateSurfaceBridge::exercise_core_module_sources_for_test(),
        BuiltinWorkbenchWindowTemplateSurfaceBridge::exercise_particle_library_sources_for_test(),
        UiHostWindow::exercise_module_tools_sources_for_test(),
        UiHostWindow::exercise_builtin_ui_asset_preview_consumers_for_test(),
        UiHostWindow::exercise_generated_bottom_sources_for_test(),
    ];

    let mut physical_documents = BTreeMap::new();
    for asset_root in asset_roots {
        for path in collect_zui_document_files(&asset_root.join("ui")) {
            let canonical = path
                .canonicalize()
                .expect("production source document must have a physical path");
            let locator = resource_locator_for_path(asset_root, &path);
            assert!(
                physical_documents
                    .insert(canonical, (path, locator))
                    .is_none(),
                "production documents must not alias the same physical file"
            );
        }
    }

    let mut fresh_sources = UiV2PrototypeStoreFileCache::new();
    assert!(fresh_sources.persistent_store().is_none());
    let mut verified_sources = BTreeMap::<std::path::PathBuf, UiV2SourceFileReceipt>::new();
    let mut locators = BTreeSet::new();
    for receipts in consumer_sources {
        assert!(
            !receipts.is_empty(),
            "a live component consumer must return its actual source closure"
        );
        for receipt in receipts {
            let canonical = receipt
                .physical_path
                .canonicalize()
                .expect("a live source receipt must identify an existing physical file");
            assert_eq!(
                receipt.physical_path, canonical,
                "a live receipt must retain its canonical physical source path"
            );
            let (production_path, locator) = physical_documents
                .get(&canonical)
                .expect("live coverage requires a production source document");
            let document = load_zui_document(production_path);
            assert_eq!(
                receipt.asset_id, document.asset.id,
                "the live receipt must identify the actual production document"
            );
            if let Some(previous) = verified_sources.get(&canonical) {
                assert_eq!(
                    previous, &receipt,
                    "consumers of the same physical source must agree on its identity and bytes"
                );
                continue;
            }

            // Revalidation checks only returned live receipts. Dependencies loaded
            // here receive no credit unless a real consumer also returned them.
            let reloaded = fresh_sources
                .load_store([canonical.clone()])
                .expect("a returned live physical source must still load");
            assert!(
                reloaded.unresolved_imports.is_empty(),
                "a returned source must retain a complete named import closure"
            );
            assert_eq!(reloaded.root_asset_id, receipt.asset_id);
            let actual = reloaded
                .source_receipts
                .iter()
                .find(|source| source.physical_path == canonical)
                .expect("fresh physical loading must reproduce the returned source receipt");
            assert_eq!(
                actual, &receipt,
                "live source provenance must match the current raw source bytes"
            );
            assert_eq!(reloaded.root_document.asset.kind, document.asset.kind);
            if document.asset.kind == UiV2AssetKind::Component {
                if let Some(resource_uri) = receipt.resource_uri.as_deref() {
                    assert_eq!(resource_uri, locator.as_str());
                }
                locators.insert(locator.clone());
            }
            verified_sources.insert(canonical, receipt);
        }
    }
    assert!(
        !locators.is_empty(),
        "live production consumers must exercise actual component sources"
    );
    locators
}

fn workbench_design_contract_zui_locators() -> BTreeSet<String> {
    [
        "res://ui/editor/unreal/level_editor_window.zui".to_string(),
        FloatingWindow::command_palette().content_asset,
        FloatingWindow::preferences().content_asset,
    ]
    .into_iter()
    .collect()
}

#[test]
fn production_v2_style_imports_resolve_to_style_assets() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_ui_root_document_files(&asset_root.join("ui")) {
            let document = load_zui_document(&path);

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

                let style_document = load_zui_document(&style_path);
                if !is_style_import_kind(style_document.asset.kind) {
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
        "production UI root assets should import shared style assets"
    );
    assert!(
        offenders.is_empty(),
        "production UI root style imports must resolve to style assets: {offenders:#?}"
    );
}

#[test]
fn production_zui_internal_imports_follow_component_and_style_boundaries() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_imports = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_document_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let document = load_zui_document(&path);

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
                let component_document = load_zui_document(&component_path);
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
                let style_document = load_zui_document(&style_path);
                if !is_style_import_kind(style_document.asset.kind) {
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
        "production asset roots should contain .zui documents"
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
        for path in collect_zui_document_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let current_locator = resource_locator_for_path(asset_root, &path);
            let document = load_zui_document(&path);

            for import in &document.imports.widgets {
                checked_imports += 1;
                let (asset_id, _fragment) = split_import_fragment(import);
                let asset_id = asset_id.trim();
                if asset_id == current_locator || asset_id == document.asset.id.as_str() {
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
        "production asset roots should contain .zui documents"
    );
    assert!(
        checked_imports == 0 || offenders.is_empty(),
        "production .zui widget imports must not self-reference the document being expanded: {offenders:#?}"
    );
}

fn is_style_import_kind(kind: UiV2AssetKind) -> bool {
    matches!(kind, UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens)
}
