use std::{collections::BTreeMap, path::Path};

use super::{EditorExtensionRegistry, EditorExtensionRegistryError, EditorUiTemplateDescriptor};

#[test]
fn plugin_template_roots_are_host_local_and_not_serialized() {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.example.panel",
            "plugins://plugin.example/ui/panel.zui",
        ))
        .expect("template descriptor should register");
    registry.bind_ui_template_root(Path::new("plugins/plugin.example"));

    let encoded = serde_json::to_string(&registry).expect("registry should serialize");
    assert!(!encoded.contains("plugins/plugin.example"));

    let decoded = serde_json::from_str::<EditorExtensionRegistry>(&encoded)
        .expect("registry should deserialize");
    assert_eq!(decoded.ui_templates().len(), 1);
    assert!(decoded.ui_templates()[0].plugin_root().is_none());
}

#[test]
fn template_replacement_preserves_the_bound_plugin_root_for_the_same_id() {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.example.panel",
            "plugins://plugin.example/ui/panel.zui",
        ))
        .expect("template descriptor should register");
    registry.bind_ui_template_root(Path::new("plugins/plugin.example"));

    registry
        .replace_ui_template_contributions(
            [EditorUiTemplateDescriptor::new(
                "plugin.example.panel",
                "plugins://plugin.example/ui/panel-reloaded.zui",
            )],
            BTreeMap::new(),
        )
        .expect("same-id replacement should keep the host-bound plugin root");

    assert_eq!(
        registry.ui_templates()[0].plugin_root(),
        Some(Path::new("plugins/plugin.example"))
    );
}

#[test]
fn template_replacement_binds_new_ids_to_the_host_plugin_root() {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.example.previous",
            "plugins://plugin.example/ui/previous.zui",
        ))
        .expect("template descriptor should register");
    registry.bind_ui_template_root(Path::new("plugins/plugin.example"));

    registry
        .replace_ui_template_contributions(
            [EditorUiTemplateDescriptor::new(
                "plugin.example.added",
                "plugins://plugin.example/ui/added.zui",
            )],
            BTreeMap::new(),
        )
        .expect("new template id should inherit the host-bound plugin root");

    assert_eq!(
        registry.ui_templates()[0].plugin_root(),
        Some(Path::new("plugins/plugin.example"))
    );
}

#[test]
fn template_contribution_replacement_keeps_the_previous_set_on_validation_error() {
    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.example.previous",
            "plugins://plugin.example/ui/previous.zui",
        ))
        .expect("previous template should register");

    let error = registry
        .replace_ui_template_contributions(
            [
                EditorUiTemplateDescriptor::new(
                    "plugin.example.replacement",
                    "plugins://plugin.example/ui/replacement.zui",
                ),
                EditorUiTemplateDescriptor::new(
                    "plugin.example.replacement",
                    "plugins://plugin.example/ui/replacement-duplicate.zui",
                ),
            ],
            BTreeMap::new(),
        )
        .expect_err("duplicate replacement ids must reject the whole candidate set");

    assert!(matches!(
        error,
        EditorExtensionRegistryError::DuplicateContribution {
            kind: "ui template",
            ..
        }
    ));
    assert_eq!(
        registry
            .ui_templates()
            .into_iter()
            .map(|template| (template.id(), template.ui_document()))
            .collect::<Vec<_>>(),
        vec![(
            "plugin.example.previous",
            "plugins://plugin.example/ui/previous.zui"
        )]
    );
}
