use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::binding::UiEventKind;

use super::metadata::{
    is_authoring_path_identifier, is_authoring_route_identifier, string_token_metadata_offender,
};
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

const GOVERNED_EVENT_KINDS: &[UiEventKind] = &[
    UiEventKind::Change,
    UiEventKind::Click,
    UiEventKind::DragBegin,
    UiEventKind::DragEnd,
    UiEventKind::DragUpdate,
    UiEventKind::Drop,
    UiEventKind::Hover,
    UiEventKind::Press,
    UiEventKind::Scroll,
    UiEventKind::Submit,
    UiEventKind::Toggle,
];

fn governed_event_kind_names() -> Vec<String> {
    GOVERNED_EVENT_KINDS
        .iter()
        .map(|event_kind| format!("{event_kind:?}"))
        .collect()
}

fn leading_path_segment(path: &str) -> &str {
    path.split('/').next().unwrap_or_default()
}

fn leading_route_segment(route: &str) -> &str {
    route.split('.').next().unwrap_or_default()
}

#[test]
fn production_zui_event_bindings_are_authorable_and_unique() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_bindings = 0usize;
    let mut observed_event_kinds = BTreeSet::new();
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let mut binding_ids = BTreeMap::<String, Vec<String>>::new();

            for (node_id, node) in &document.nodes {
                for (binding_index, binding) in node.events.iter().enumerate() {
                    checked_bindings += 1;
                    let binding_label =
                        format!("node `{node_id}` event binding #{}", binding_index + 1);
                    observed_event_kinds.insert(format!("{:?}", binding.event));
                    if !GOVERNED_EVENT_KINDS.contains(&binding.event) {
                        let governed_event_kinds = governed_event_kind_names();
                        offenders.push(format!(
                            "{} {binding_label} declares ungoverned event kind `{:?}`; governed production events are {governed_event_kinds:?}",
                            path.display(),
                            binding.event
                        ));
                    }

                    if let Some(invalid_binding_id) =
                        string_token_metadata_offender(&binding.id, "event binding id")
                    {
                        offenders.push(format!(
                            "{} {binding_label} declares {invalid_binding_id}",
                            path.display()
                        ));
                    } else if !is_authoring_path_identifier(&binding.id) {
                        offenders.push(format!(
                            "{} {binding_label} declares event binding id `{}` outside slash-delimited authoring path form",
                            path.display(),
                            binding.id
                        ));
                    } else {
                        binding_ids
                            .entry(binding.id.trim().to_string())
                            .or_default()
                            .push(binding_label.clone());
                    }

                    let has_clean_route = if let Some(route) = binding.route.as_deref() {
                        if let Some(invalid_route) =
                            string_token_metadata_offender(route, "event binding route")
                        {
                            offenders.push(format!(
                                "{} {binding_label} declares {invalid_route}",
                                path.display()
                            ));
                            false
                        } else if !is_authoring_route_identifier(route) {
                            offenders.push(format!(
                                "{} {binding_label} declares event binding route `{route}` outside dotted authoring route form",
                                path.display()
                            ));
                            false
                        } else {
                            true
                        }
                    } else {
                        false
                    };

                    if !has_clean_route && binding.action.is_none() {
                        offenders.push(format!(
                            "{} {binding_label} declares no route or action target",
                            path.display()
                        ));
                    }
                }
            }

            for (binding_id, binding_labels) in binding_ids {
                if binding_labels.len() > 1 {
                    offenders.push(format!(
                        "{} declares duplicate event binding id `{}` on {binding_labels:?}",
                        path.display(),
                        binding_id
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
        checked_bindings > 0,
        "production .zui assets should declare event bindings"
    );
    assert!(
        !observed_event_kinds.is_empty(),
        "production .zui event bindings should exercise governed event kinds"
    );
    assert!(
        offenders.is_empty(),
        "production .zui event bindings must use governed event kinds, have slash-delimited clean ids, dotted clean route metadata, unique ids inside an asset, and at least one dispatch target: {offenders:#?}"
    );
}

#[test]
fn production_zui_event_binding_ids_are_globally_unique() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_bindings = 0usize;
    let mut binding_ids = BTreeMap::<String, Vec<String>>::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                for (binding_index, binding) in node.events.iter().enumerate() {
                    checked_bindings += 1;
                    if string_token_metadata_offender(&binding.id, "event binding id").is_some()
                        || !is_authoring_path_identifier(&binding.id)
                    {
                        continue;
                    }

                    binding_ids
                        .entry(binding.id.trim().to_string())
                        .or_default()
                        .push(format!(
                            "{} node `{}` event binding #{}",
                            path.display(),
                            node_id,
                            binding_index + 1
                        ));
                }
            }
        }
    }

    let offenders = binding_ids
        .into_iter()
        .filter_map(|(binding_id, labels)| {
            (labels.len() > 1)
                .then(|| format!("duplicate global event binding id `{binding_id}` on {labels:?}"))
        })
        .collect::<Vec<_>>();

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_bindings > 0,
        "production .zui assets should declare event bindings"
    );
    assert!(
        offenders.is_empty(),
        "production .zui event binding ids must be globally unique across component assets so replay traces and diagnostics can identify one binding without asset-local disambiguation: {offenders:#?}"
    );
}

#[test]
fn production_zui_event_binding_routes_share_id_namespaces() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_routes = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                for (binding_index, binding) in node.events.iter().enumerate() {
                    let Some(route) = binding.route.as_deref() else {
                        continue;
                    };
                    checked_routes += 1;
                    if string_token_metadata_offender(&binding.id, "event binding id").is_some()
                        || !is_authoring_path_identifier(&binding.id)
                        || string_token_metadata_offender(route, "event binding route").is_some()
                        || !is_authoring_route_identifier(route)
                    {
                        continue;
                    }

                    let id_namespace = leading_path_segment(&binding.id);
                    let route_namespace = leading_route_segment(route);
                    if id_namespace != route_namespace {
                        offenders.push(format!(
                            "{} node `{}` event binding #{} uses id namespace `{}` but route namespace `{}` in id `{}` route `{}`",
                            path.display(),
                            node_id,
                            binding_index + 1,
                            id_namespace,
                            route_namespace,
                            binding.id,
                            route
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
        checked_routes > 0,
        "production .zui event bindings should declare legacy route metadata until action migration finishes"
    );
    assert!(
        offenders.is_empty(),
        "production .zui event binding ids and legacy routes must share their leading authoring namespace so route tables, replay traces, and future action targets group the same interaction consistently: {offenders:#?}"
    );
}

#[test]
fn production_zui_event_binding_routes_are_globally_unique() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_routes = 0usize;
    let mut routes = BTreeMap::<String, Vec<String>>::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                for (binding_index, binding) in node.events.iter().enumerate() {
                    let Some(route) = binding.route.as_deref() else {
                        continue;
                    };
                    checked_routes += 1;
                    if string_token_metadata_offender(route, "event binding route").is_some()
                        || !is_authoring_route_identifier(route)
                    {
                        continue;
                    }

                    routes
                        .entry(route.trim().to_string())
                        .or_default()
                        .push(format!(
                            "{} node `{}` event binding #{} id `{}`",
                            path.display(),
                            node_id,
                            binding_index + 1,
                            binding.id
                        ));
                }
            }
        }
    }

    let offenders = routes
        .into_iter()
        .filter_map(|(route, labels)| {
            (labels.len() > 1)
                .then(|| format!("duplicate global event binding route `{route}` on {labels:?}"))
        })
        .collect::<Vec<_>>();

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_routes > 0,
        "production .zui event bindings should declare legacy route metadata until action migration finishes"
    );
    assert!(
        offenders.is_empty(),
        "production .zui legacy event routes must be globally unique so route dispatch, diagnostics, and replay traces do not need asset-local disambiguation: {offenders:#?}"
    );
}
