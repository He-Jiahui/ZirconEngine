use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::core::editor_extension::{EditorExtensionRegistry, EditorUiTemplateDescriptor};

use super::super::runtime_host::{EditorUiHostRuntime, EditorUiHostRuntimeError};
use super::*;

#[test]
fn plugin_document_inputs_reject_invalid_owner_generation_uri_and_sources() {
    assert!(matches!(
        EditorPluginV2DocumentOwner::new(" ", 1),
        Err(EditorPluginV2DocumentSourceError::InvalidOwnerId { .. })
    ));
    assert!(matches!(
        EditorPluginV2DocumentOwner::new("plugin.example", 0),
        Err(EditorPluginV2DocumentSourceError::InvalidGeneration)
    ));
    assert!(matches!(
        EditorPluginV2DocumentSource::new("plugin.example.panel", "asset://panel.zui", ["a"]),
        Err(EditorPluginV2DocumentSourceError::InvalidPluginDocumentUri { .. })
    ));
    assert!(matches!(
        EditorPluginV2DocumentSource::new::<std::path::PathBuf, _>(
            "plugin.example.panel",
            "plugins://plugin.example/panel.zui",
            [],
        ),
        Err(EditorPluginV2DocumentSourceError::MissingSourceFiles { .. })
    ));
}

fn fixture_document_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ui/host/workbench_shell.zui")
}

fn source(document_id: &str) -> EditorPluginV2DocumentSource {
    EditorPluginV2DocumentSource::new(
        document_id,
        "plugins://test.plugin/panel.zui",
        [fixture_document_path()],
    )
    .expect("plugin document source should be valid")
}

fn registered_template(uri: &str) -> EditorUiTemplateDescriptor {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new("plugin.example.panel", uri))
        .expect("template should register");
    let fixture_root = fixture_document_path()
        .parent()
        .map(Path::to_path_buf)
        .expect("fixture document should have a containing directory");
    registry.bind_ui_template_root(&fixture_root);
    registry.ui_templates()[0].clone()
}

fn registered_template_for(
    owner_id: &str,
    template_id: &str,
    relative_path: &str,
) -> EditorUiTemplateDescriptor {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            template_id,
            format!("plugins://{owner_id}/{relative_path}"),
        ))
        .expect("template should register");
    let fixture_root = fixture_document_path()
        .parent()
        .map(Path::to_path_buf)
        .expect("fixture document should have a containing directory");
    registry.bind_ui_template_root(&fixture_root);
    registry.ui_templates()[0].clone()
}

#[test]
fn runtime_sync_materializes_an_owned_plugin_template_descriptor() {
    let runtime = EditorUiHostRuntime::default();
    let descriptor = registered_template("plugins://plugin.example/workbench_shell.zui");

    runtime
        .sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([(
            "plugin.example".to_string(),
            vec![descriptor],
        )]))
        .expect("runtime should materialize an owned descriptor");

    assert_eq!(
        runtime.plugin_v2_document_owner("plugin.example.panel"),
        Some(
            EditorPluginV2DocumentOwner::new("plugin.example", 1)
                .expect("generation should be valid")
        )
    );
}

#[test]
fn runtime_sync_builds_a_shared_surface_from_the_owned_plugin_document() {
    let runtime = EditorUiHostRuntime::default();
    let descriptor = registered_template("plugins://plugin.example/workbench_shell.zui");

    runtime
        .sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([(
            "plugin.example".to_string(),
            vec![descriptor],
        )]))
        .expect("runtime should materialize the plugin descriptor before building its surface");

    let surface = runtime
        .build_shared_surface("plugin.example.panel")
        .expect("a published plugin V2 document should build through the shared surface path");
    assert!(!surface.tree.nodes.is_empty());
}

#[test]
fn runtime_sync_removes_an_owner_document_when_its_descriptor_set_is_empty() {
    let runtime = EditorUiHostRuntime::default();
    let descriptor = registered_template("plugins://plugin.example/workbench_shell.zui");

    runtime
        .sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([(
            "plugin.example".to_string(),
            vec![descriptor],
        )]))
        .expect("runtime should materialize the plugin descriptor before its owner is removed");
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.example.panel")
            .is_some()
    );

    runtime
        .sync_plugin_v2_template_descriptor_sets(&BTreeMap::new())
        .expect("an empty enabled-owner set should retire the plugin document atomically");
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.example.panel")
            .is_none()
    );
}

#[test]
fn runtime_sync_rejects_cross_owner_and_parent_path_template_descriptors() {
    let runtime = EditorUiHostRuntime::default();
    let cross_owner = registered_template("plugins://other.plugin/workbench_shell.zui");
    assert!(matches!(
        runtime.sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([(
            "plugin.example".to_string(),
            vec![cross_owner],
        )])),
        Err(EditorUiHostRuntimeError::PluginDocumentUri { .. })
    ));

    let parent_path = registered_template("plugins://plugin.example/ui/../workbench_shell.zui");
    assert!(matches!(
        runtime.sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([(
            "plugin.example".to_string(),
            vec![parent_path],
        )])),
        Err(EditorUiHostRuntimeError::PluginDocumentTemplatePath { .. })
    ));
}

#[test]
fn batch_template_sync_keeps_last_good_documents_when_any_candidate_fails() {
    let runtime = EditorUiHostRuntime::default();
    runtime
        .sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([(
            "plugin.one".to_string(),
            vec![registered_template_for(
                "plugin.one",
                "plugin.one.panel",
                "workbench_shell.zui",
            )],
        )]))
        .expect("initial owner document should load");
    let last_good_owner = runtime
        .plugin_v2_document_owner("plugin.one.panel")
        .expect("initial owner should remain published");

    assert!(matches!(
        runtime.sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([
            (
                "plugin.one".to_string(),
                vec![registered_template_for(
                    "plugin.one",
                    "plugin.one.panel",
                    "workbench_shell.zui",
                )],
            ),
            (
                "plugin.two".to_string(),
                vec![registered_template_for(
                    "plugin.two",
                    "plugin.two.panel",
                    "ui/../workbench_shell.zui",
                )],
            ),
        ])),
        Err(EditorUiHostRuntimeError::PluginDocumentTemplatePath { .. })
    ));
    assert_eq!(
        runtime.plugin_v2_document_owner("plugin.one.panel"),
        Some(last_good_owner)
    );
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.two.panel")
            .is_none()
    );
}

#[test]
fn batch_template_sync_rejects_cross_owner_document_ids_without_replacing_last_good() {
    let runtime = EditorUiHostRuntime::default();
    runtime
        .sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([(
            "plugin.last-good".to_string(),
            vec![registered_template_for(
                "plugin.last-good",
                "plugin.last-good.panel",
                "workbench_shell.zui",
            )],
        )]))
        .expect("initial owner document should load");
    let last_good_owner = runtime
        .plugin_v2_document_owner("plugin.last-good.panel")
        .expect("initial owner should remain published");

    assert!(matches!(
        runtime.sync_plugin_v2_template_descriptor_sets(&BTreeMap::from([
            (
                "plugin.one".to_string(),
                vec![registered_template_for(
                    "plugin.one",
                    "plugin.shared.panel",
                    "workbench_shell.zui",
                )],
            ),
            (
                "plugin.two".to_string(),
                vec![registered_template_for(
                    "plugin.two",
                    "plugin.shared.panel",
                    "workbench_shell.zui",
                )],
            ),
        ])),
        Err(EditorUiHostRuntimeError::PluginDocumentIdConflict {
            document_id,
            owner_id,
        }) if document_id == "plugin.shared.panel" && owner_id == "plugin.one"
    ));
    assert_eq!(
        runtime.plugin_v2_document_owner("plugin.last-good.panel"),
        Some(last_good_owner)
    );
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.shared.panel")
            .is_none()
    );
}

#[test]
fn replacement_preserves_a_same_id_route_for_the_new_generation() {
    let runtime = EditorUiHostRuntime::default();
    let first = EditorPluginV2DocumentOwner::new("test.plugin", 1)
        .expect("first plugin generation should be valid");
    let second = EditorPluginV2DocumentOwner::new("test.plugin", 2)
        .expect("second plugin generation should be valid");

    runtime
        .replace_plugin_v2_documents(first.clone(), [source("plugin.test.panel")])
        .expect("first plugin generation should load");
    let update = runtime
        .replace_plugin_v2_documents(second.clone(), [source("plugin.test.panel")])
        .expect("second plugin generation should replace the document");

    assert!(update.retired_document_ids().is_empty());
    assert_eq!(
        runtime.plugin_v2_document_owner("plugin.test.panel"),
        Some(second.clone())
    );
    assert!(runtime.unregister_plugin_v2_documents(&first).is_empty());
    assert_eq!(
        runtime.unregister_plugin_v2_documents(&second),
        vec!["plugin.test.panel".to_string()]
    );
}

#[test]
fn owner_scoped_replacement_advances_generation_across_unload() {
    let runtime = EditorUiHostRuntime::default();
    let first = runtime
        .replace_plugin_v2_documents_for_owner("test.plugin", [source("plugin.test.panel")])
        .expect("first owner-scoped registration should load");
    assert_eq!(first.owner().generation(), 1);
    assert_eq!(
        runtime.unregister_plugin_v2_documents_for_owner("test.plugin"),
        vec!["plugin.test.panel".to_string()]
    );

    let second = runtime
        .replace_plugin_v2_documents_for_owner("test.plugin", [source("plugin.test.panel")])
        .expect("replacement after unload should advance generation");
    assert_eq!(second.owner().generation(), 2);
}

#[test]
fn owner_scoped_replacement_returns_invalid_owner_errors_without_publishing() {
    let runtime = EditorUiHostRuntime::default();

    assert!(matches!(
        runtime.replace_plugin_v2_documents_for_owner(" ", [source("plugin.test.panel")]),
        Err(EditorUiHostRuntimeError::PluginDocumentSource(
            EditorPluginV2DocumentSourceError::InvalidOwnerId { .. }
        ))
    ));
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.test.panel")
            .is_none()
    );
}

#[test]
fn stale_generation_cannot_replace_a_newer_plugin_document() {
    let runtime = EditorUiHostRuntime::default();
    let older = EditorPluginV2DocumentOwner::new("test.plugin", 1)
        .expect("older generation should be valid");
    let newer = EditorPluginV2DocumentOwner::new("test.plugin", 2)
        .expect("newer generation should be valid");
    runtime
        .replace_plugin_v2_documents(newer.clone(), [source("plugin.test.newer")])
        .expect("newer generation should load");

    assert!(matches!(
        runtime.replace_plugin_v2_documents(older, [source("plugin.test.older")]),
        Err(EditorUiHostRuntimeError::PluginDocumentGenerationStale { .. })
    ));
    assert_eq!(
        runtime.plugin_v2_document_owner("plugin.test.newer"),
        Some(newer)
    );
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.test.older")
            .is_none()
    );
}

#[test]
fn same_generation_cannot_replace_a_current_plugin_document() {
    let runtime = EditorUiHostRuntime::default();
    let owner = EditorPluginV2DocumentOwner::new("test.plugin", 2)
        .expect("plugin generation should be valid");
    runtime
        .replace_plugin_v2_documents(owner.clone(), [source("plugin.test.current")])
        .expect("initial document should load");

    assert!(matches!(
        runtime.replace_plugin_v2_documents(owner.clone(), [source("plugin.test.replaced")]),
        Err(EditorUiHostRuntimeError::PluginDocumentGenerationStale { .. })
    ));
    assert_eq!(
        runtime.plugin_v2_document_owner("plugin.test.current"),
        Some(owner)
    );
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.test.replaced")
            .is_none()
    );
}

#[test]
fn explicit_generation_cannot_replay_after_its_documents_are_unloaded() {
    let runtime = EditorUiHostRuntime::default();
    let owner = EditorPluginV2DocumentOwner::new("test.plugin", 2)
        .expect("plugin generation should be valid");
    runtime
        .replace_plugin_v2_documents(owner.clone(), [source("plugin.test.current")])
        .expect("initial document should load");
    assert_eq!(
        runtime.unregister_plugin_v2_documents(&owner),
        vec!["plugin.test.current".to_string()]
    );

    assert!(matches!(
        runtime.replace_plugin_v2_documents(owner, [source("plugin.test.replayed")]),
        Err(EditorUiHostRuntimeError::PluginDocumentGenerationStale { .. })
    ));
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.test.replayed")
            .is_none()
    );
}

#[test]
fn owner_scoped_replacement_advances_after_an_explicit_catalog_generation() {
    let runtime = EditorUiHostRuntime::default();
    let explicit_owner = EditorPluginV2DocumentOwner::new("test.plugin", 4)
        .expect("explicit plugin generation should be valid");
    runtime
        .replace_plugin_v2_documents(explicit_owner, [source("plugin.test.current")])
        .expect("explicit document should load");

    let update = runtime
        .replace_plugin_v2_documents_for_owner("test.plugin", [source("plugin.test.next")])
        .expect("owner-scoped replacement should advance from the catalog generation");
    assert_eq!(update.owner().generation(), 5);
    assert!(
        runtime
            .plugin_v2_document_owner("plugin.test.current")
            .is_none()
    );
    assert_eq!(
        runtime.plugin_v2_document_owner("plugin.test.next"),
        Some(update.owner().clone())
    );
}
