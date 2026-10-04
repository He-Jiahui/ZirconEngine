use super::{AuthoredNodeIdentity, SourceIdentityIndex};
use crate::ui::template_runtime::RetainedUiHostNodeProjection;

fn retained_node(
    component: &str,
    control_id: &str,
    source_path: &str,
    source_node_id: &str,
) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: "host/generated/path".into(),
        surface_node_id: None,
        has_workbench_icon_tooltip: false,
        parent_id: None,
        component: component.into(),
        control_id: Some(control_id.into()),
        source_path: Some(source_path.into()),
        source_node_id: Some(source_node_id.into()),
        instance_path: Some(Vec::new()),
        parent_source_path: None,
        parent_source_node_id: None,
        parent_instance_path: None,
        frame: Default::default(),
        clip_frame: None,
        z_index: 0,
        attributes: Default::default(),
        style_overrides: Default::default(),
        style_tokens: Default::default(),
        bindings: Vec::new(),
    }
}

fn identity(source_path: &str, source_node_id: &str) -> AuthoredNodeIdentity {
    AuthoredNodeIdentity {
        source_path: source_path.into(),
        source_node_id: source_node_id.into(),
        control_id: "stable-control".into(),
        component: "Button".into(),
        instance_path: "[]".into(),
    }
}

#[test]
fn staged_resource_uri_joins_only_one_catalog_source_path() {
    let catalog_source = "zircon_editor/assets/ui/editor/windows/workbench_window.zui";
    let uri = "res://ui/editor/windows/workbench_window.zui";
    let mut index = SourceIdentityIndex::default();
    index.source_paths.insert(catalog_source.into());
    index
        .resource_uri_sources
        .entry(uri.into())
        .or_default()
        .insert(catalog_source.into());

    assert_eq!(
        index.unique_catalog_source_path(uri).as_deref(),
        Some(catalog_source)
    );

    index
        .resource_uri_sources
        .get_mut(uri)
        .unwrap()
        .insert("zircon_plugins/other/assets/ui/editor/windows/workbench_window.zui".into());
    assert_eq!(index.unique_catalog_source_path(uri), None);
}

#[test]
fn product_template_identity_uses_source_node_and_canonical_instance_path() {
    let mut index = SourceIdentityIndex::default();
    index.by_source_node.insert(
        ("one.zui".into(), "button".into()),
        identity("one.zui", "button"),
    );

    let resolved = index
        .resolve_template_row("one.zui", "button", "stable-control", "[]")
        .unwrap();
    assert_eq!(resolved.source_node_id, "button");
    assert_eq!(resolved.component, "Button");
    assert_eq!(resolved.instance_path, "[]");
    assert!(index
        .resolve_template_row("one.zui", "button", "stable-control", "[ ]")
        .is_err());
    assert!(index
        .resolve_template_row("one.zui", "button", "other-control", "[]")
        .is_err());
}

#[test]
fn authored_identity_uses_the_provenance_tuple_without_control_inference() {
    let mut index = SourceIdentityIndex::default();
    index.by_source_node.insert(
        ("one.zui".into(), "button".into()),
        identity("one.zui", "button"),
    );

    let resolved = index
        .resolve(&retained_node(
            "Button",
            "stable-control",
            "one.zui",
            "button",
        ))
        .unwrap();
    assert_eq!(resolved.source_path, "one.zui");
    assert_eq!(resolved.source_node_id, "button");

    index.by_source_node.insert(
        ("two.zui".into(), "button".into()),
        identity("two.zui", "button"),
    );
    let distinct = index
        .resolve(&retained_node(
            "Button",
            "stable-control",
            "two.zui",
            "button",
        ))
        .unwrap();
    assert_eq!(distinct.source_path, "two.zui");
}

#[test]
fn authored_identity_does_not_join_on_control_id_alone() {
    let mut index = SourceIdentityIndex::default();
    index.by_source_node.insert(
        ("one.zui".into(), "button".into()),
        identity("one.zui", "button"),
    );

    assert!(index
        .resolve(&retained_node(
            "IconButton",
            "stable-control",
            "one.zui",
            "button"
        ))
        .unwrap_err()
        .contains("differs in component"));
}

#[test]
fn explicit_workbench_selector_resolves_its_source_before_global_candidates() {
    let mut index = SourceIdentityIndex {
        selected: Some(super::super::state::WorkbenchStateSelector {
            source_path: "two.zui".into(),
            control_id: "stable-control".into(),
            source_node_id: Some("selected".into()),
            instance_path: None,
            scroll_target: None,
            text_overrides: Vec::new(),
        }),
        ..SourceIdentityIndex::default()
    };
    index.by_source_node.insert(
        ("two.zui".into(), "selected".into()),
        identity("two.zui", "selected"),
    );

    let resolved = index
        .resolve(&retained_node(
            "Button",
            "stable-control",
            "two.zui",
            "selected",
        ))
        .unwrap();
    assert_eq!(resolved.source_path, "two.zui");
    assert_eq!(resolved.source_node_id, "selected");
}
