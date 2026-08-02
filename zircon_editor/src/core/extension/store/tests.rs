use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_extension::{
    DrawerDescriptor, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor, ViewDescriptor,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{
    FieldEditorContainer, FieldEditorDefinition, FieldEditorInstance, FieldEditorKind,
    InspectTarget, InspectTargetType, InspectorCustomization, InspectorCustomizationDescriptor,
    InspectorCustomizationSurface, InspectorField, InspectorLayoutBuilder,
};

use super::model::CONTRIBUTION_CHANGE_JOURNAL_CAPACITY;
use super::{
    CapabilitySet, ContributionBatch, ContributionChangeKind, ContributionError,
    ContributionSource, ContributionStore, PluginContributionId,
};

fn plugin_source(id: &str) -> ContributionSource {
    ContributionSource::Plugin(PluginContributionId::parse(id).unwrap())
}

fn operation(id: &str) -> EditorOperationPath {
    EditorOperationPath::parse(id).unwrap()
}

struct TicketCustomization;

impl InspectorCustomization for TicketCustomization {
    fn id(&self) -> &str {
        "plugin.sample.cloud_layer"
    }

    fn can_handle(&self, target: &InspectTargetType) -> bool {
        target.type_name() == "weather::CloudLayer"
    }

    fn build(&self, _target: &InspectTarget, layout: &mut InspectorLayoutBuilder) {
        layout.add_custom_row("plugin.sample.cloud_layer.row", "Cloud Layer");
    }
}

struct SurfaceCustomization {
    surface: InspectorCustomizationSurface,
}

impl InspectorCustomization for SurfaceCustomization {
    fn id(&self) -> &str {
        "plugin.sample.surface"
    }

    fn can_handle(&self, _target: &InspectTargetType) -> bool {
        false
    }

    fn build(&self, _target: &InspectTarget, _layout: &mut InspectorLayoutBuilder) {}

    fn surface(&self) -> Option<&InspectorCustomizationSurface> {
        Some(&self.surface)
    }
}

#[test]
fn plugin_contribution_requires_its_typed_namespace_without_publishing() {
    let mut batch = ContributionBatch::default();
    batch
        .register_view(ViewDescriptor::new("foreign.view", "Foreign", "Tests"))
        .unwrap();
    let mut store = ContributionStore::default();

    let error = store
        .contribute(plugin_source("sample"), batch)
        .unwrap_err();

    assert!(matches!(
        error,
        ContributionError::PluginNamespace {
            plugin_id,
            kind: "view",
            id,
        } if plugin_id.as_str() == "sample" && id == "foreign.view"
    ));
    assert_eq!(store.generation(), 0);
    assert_eq!(store.snapshot().views(&CapabilitySet::default()).count(), 0);
}

#[test]
fn field_editor_requires_plugin_namespace_and_unique_cross_ticket_type_key() {
    let mut foreign = ContributionBatch::default();
    foreign
        .register_field_editor(FieldEditorDefinition::new(
            "plugin.foreign.CloudCoverage",
            |_| FieldEditorInstance::new(FieldEditorKind::Color),
        ))
        .unwrap();
    let mut store = ContributionStore::default();

    let namespace_error = store
        .contribute(plugin_source("sample"), foreign)
        .unwrap_err();
    assert!(matches!(
        namespace_error,
        ContributionError::PluginNamespace {
            plugin_id,
            kind: "field editor",
            id,
        } if plugin_id.as_str() == "sample" && id == "plugin.foreign.CloudCoverage"
    ));
    assert_eq!(store.generation(), 0);

    let mut first = ContributionBatch::default();
    first
        .register_field_editor(FieldEditorDefinition::new(
            "plugin.sample.BrandColor",
            |_| FieldEditorInstance::new(FieldEditorKind::Color),
        ))
        .unwrap();
    store.contribute(plugin_source("sample"), first).unwrap();
    let generation = store.generation();

    let mut duplicate = ContributionBatch::default();
    duplicate
        .register_field_editor(FieldEditorDefinition::new(
            "plugin.sample.BrandColor",
            |_| FieldEditorInstance::new(FieldEditorKind::Auto),
        ))
        .unwrap();
    let collision_error = store
        .contribute(plugin_source("sample"), duplicate)
        .unwrap_err();
    assert!(matches!(
        collision_error,
        ContributionError::DuplicateContribution {
            kind: "field editor",
            id,
        } if id == "plugin.sample.BrandColor"
    ));
    assert_eq!(store.generation(), generation);
    assert_eq!(
        store
            .snapshot()
            .field_editors(&CapabilitySet::default())
            .count(),
        1
    );
}

#[test]
fn inspector_customization_is_ticket_owned_capability_filtered_and_revocable() {
    let mut batch = ContributionBatch::default().with_required_capabilities(["inspector.cloud"]);
    batch
        .register_inspector_customization(Arc::new(TicketCustomization))
        .unwrap();
    let mut store = ContributionStore::default();
    let ticket = store.contribute(plugin_source("sample"), batch).unwrap();
    let contributed = store.snapshot();

    assert_eq!(
        contributed
            .inspector_customizations(&CapabilitySet::default())
            .count(),
        0
    );
    assert_eq!(
        contributed
            .inspector_customizations(&CapabilitySet::from(["inspector.cloud"]))
            .map(|customization| customization.id())
            .collect::<Vec<_>>(),
        vec!["plugin.sample.cloud_layer"]
    );

    let report = store.revoke(ticket);
    assert_eq!(report.removed().inspector_customizations(), 1);
    assert_eq!(
        store
            .snapshot()
            .inspector_customizations(&CapabilitySet::from(["inspector.cloud"]))
            .count(),
        0
    );
    assert_eq!(
        contributed
            .inspector_customizations(&CapabilitySet::from(["inspector.cloud"]))
            .count(),
        1,
        "published reader must remain immutable after ticket revoke"
    );
}

#[test]
fn invalid_inspector_target_is_rejected_before_batch_publication() {
    let descriptor = InspectorCustomizationDescriptor::new(
        "invalid target type",
        "plugins://sample/editor/invalid_target.zui",
        "plugin.sample.InvalidTargetController",
    )
    .with_id("plugin.sample.invalid_target");
    let mut batch = ContributionBatch::default();

    let error = batch
        .register_inspector_customization(Arc::new(descriptor))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorExtensionRegistryError::View(message)
            if message.contains("invalid target type")
    ));
    assert_eq!(batch.inspector_customizations().count(), 0);
}

#[test]
fn dynamic_inspector_customization_with_invalid_controller_is_not_published() {
    let mut batch = ContributionBatch::default();
    let error = batch
        .register_inspector_customization(Arc::new(SurfaceCustomization {
            surface: InspectorCustomizationSurface::new(
                "plugins://sample/editor/cloud_layer.zui",
                "invalid controller",
            ),
        }))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorExtensionRegistryError::View(message)
            if message.contains("invalid controller")
    ));
    assert_eq!(batch.inspector_customizations().count(), 0);

    let mut store = ContributionStore::default();
    store.contribute(plugin_source("sample"), batch).unwrap();
    assert_eq!(
        store
            .snapshot()
            .inspector_customizations(&CapabilitySet::default())
            .count(),
        0
    );
}

#[test]
fn plugin_field_editor_is_ticket_owned_capability_filtered_and_revocable() {
    let mut batch = ContributionBatch::default().with_required_capabilities(["inspector.cloud"]);
    batch
        .register_field_editor(FieldEditorDefinition::new(
            "plugin.sample.BrandColor",
            |_| FieldEditorInstance::new(FieldEditorKind::Color),
        ))
        .unwrap();
    let mut store = ContributionStore::default();
    let ticket = store.contribute(plugin_source("sample"), batch).unwrap();
    let contributed = store.snapshot();
    let enabled = CapabilitySet::from(["inspector.cloud"]);

    assert_eq!(
        contributed.field_editors(&CapabilitySet::default()).count(),
        0
    );
    assert_eq!(
        contributed
            .field_editors(&enabled)
            .map(FieldEditorDefinition::type_name)
            .collect::<Vec<_>>(),
        vec!["plugin.sample.BrandColor"]
    );
    let active =
        FieldEditorContainer::with_contributions(contributed.field_editors(&enabled).cloned())
            .unwrap();
    let field = InspectorField::new(
        "plugin.sample.cloud_layer.coverage",
        "Coverage",
        "plugin.sample.BrandColor",
        "0.75",
        true,
    )
    .unwrap();
    assert_eq!(active.resolve(field).kind(), FieldEditorKind::Color);

    let report = store.revoke(ticket);
    assert_eq!(report.removed().field_editors(), 1);
    assert_eq!(store.snapshot().field_editors(&enabled).count(), 0);
    assert_eq!(
        contributed.field_editors(&enabled).count(),
        1,
        "published reader must retain its resolved contribution generation"
    );
    let fallback =
        FieldEditorContainer::with_contributions(store.snapshot().field_editors(&enabled).cloned())
            .unwrap();
    let field = InspectorField::new(
        "plugin.sample.cloud_layer.coverage",
        "Coverage",
        "plugin.sample.BrandColor",
        "0.75",
        true,
    )
    .unwrap();
    assert_eq!(fallback.resolve(field).kind(), FieldEditorKind::Auto);
}

#[test]
fn collision_rejects_the_complete_batch_without_consuming_a_ticket() {
    let mut first = ContributionBatch::default();
    first
        .register_view(ViewDescriptor::new("builtin.console", "Console", "Builtin"))
        .unwrap();
    let mut store = ContributionStore::default();
    store
        .contribute(ContributionSource::Builtin, first)
        .unwrap();
    assert_eq!(store.len(), 1);
    let first_generation = store.generation();

    let mut collision = ContributionBatch::default();
    collision
        .register_view(ViewDescriptor::new("builtin.console", "Duplicate", "Tests"))
        .unwrap();
    collision
        .register_drawer(DrawerDescriptor::new(
            "builtin.new_drawer",
            "Must Not Publish",
        ))
        .unwrap();

    let error = store
        .contribute(ContributionSource::Builtin, collision)
        .unwrap_err();
    assert!(matches!(
        error,
        ContributionError::DuplicateContribution {
            kind: "view",
            id,
        } if id == "builtin.console"
    ));
    assert_eq!(store.generation(), first_generation);
    assert_eq!(store.len(), 1);
    assert_eq!(
        store.snapshot().drawers(&CapabilitySet::default()).count(),
        0
    );

    let mut next = ContributionBatch::default();
    next.register_drawer(DrawerDescriptor::new("builtin.next", "Next"))
        .unwrap();
    store.contribute(ContributionSource::Builtin, next).unwrap();
    assert_eq!(store.len(), 2);
}

#[test]
fn revoke_removes_every_family_while_old_generation_readers_remain_immutable() {
    let mut batch = ContributionBatch::default();
    batch
        .register_view(ViewDescriptor::new(
            "plugin.sample.view",
            "Sample",
            "Plugins",
        ))
        .unwrap();
    batch
        .register_drawer(DrawerDescriptor::new(
            "plugin.sample.drawer",
            "Sample Drawer",
        ))
        .unwrap();
    batch
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.sample.template",
            "plugins://sample/template.zui",
        ))
        .unwrap();
    batch
        .register_command(EditorCommandDescriptor::operation(
            operation("plugin.sample.command"),
            "Sample Command",
        ))
        .unwrap();
    batch
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Plugins/Sample",
            operation("plugin.sample.command"),
        ))
        .unwrap();

    let mut store = ContributionStore::default();
    let ticket = store.contribute(plugin_source("sample"), batch).unwrap();
    let old_reader = store.snapshot();
    let report = store.revoke(ticket);
    let current_reader = store.snapshot();
    let capabilities = CapabilitySet::default();

    assert!(report.revoked());
    assert_eq!(report.removed().views(), 1);
    assert_eq!(report.removed().drawers(), 1);
    assert_eq!(report.removed().menu_items(), 1);
    assert_eq!(report.removed().ui_templates(), 1);
    assert_eq!(report.removed().commands(), 1);
    assert_eq!(old_reader.views(&capabilities).count(), 1);
    assert_eq!(old_reader.menu_items(&capabilities).count(), 1);
    assert_eq!(old_reader.ui_templates(&capabilities).count(), 1);
    assert_eq!(old_reader.commands(&capabilities).count(), 1);
    assert_eq!(current_reader.views(&capabilities).count(), 0);
    assert_eq!(current_reader.menu_items(&capabilities).count(), 0);
    assert_eq!(current_reader.ui_templates(&capabilities).count(), 0);
    assert_eq!(current_reader.commands(&capabilities).count(), 0);
}

#[test]
fn changed_since_is_ordered_and_snapshot_queries_filter_batch_capabilities() {
    let mut batch =
        ContributionBatch::default().with_required_capabilities(["editor.sample.enabled"]);
    batch
        .register_view(ViewDescriptor::new(
            "plugin.sample.gated",
            "Gated",
            "Plugins",
        ))
        .unwrap();
    let mut store = ContributionStore::default();
    let ticket = store.contribute(plugin_source("sample"), batch).unwrap();
    let contributed = store.snapshot();

    assert_eq!(contributed.views(&CapabilitySet::default()).count(), 0);
    assert_eq!(
        contributed
            .views(&CapabilitySet::from(["editor.sample.enabled"]))
            .count(),
        1
    );

    store.revoke(ticket);
    let delta = store.changed_since(0);
    assert_eq!(delta.from_generation(), 0);
    assert_eq!(delta.to_generation(), 2);
    assert_eq!(delta.changes().len(), 2);
    assert_eq!(delta.changes()[0].generation(), 1);
    assert_eq!(
        delta.changes()[0].kind(),
        ContributionChangeKind::Contributed
    );
    assert_eq!(delta.changes()[1].generation(), 2);
    assert_eq!(delta.changes()[1].kind(), ContributionChangeKind::Revoked);
}

#[test]
fn snapshot_retains_all_views_for_an_active_ticket_before_revoke() {
    let mut owned = ContributionBatch::default().with_required_capabilities(["editor.sample"]);
    owned
        .register_view(ViewDescriptor::new(
            "plugin.sample.first",
            "First",
            "Plugins",
        ))
        .unwrap();
    owned
        .register_view(ViewDescriptor::new(
            "plugin.sample.second",
            "Second",
            "Plugins",
        ))
        .unwrap();
    let mut sibling = ContributionBatch::default();
    sibling
        .register_view(ViewDescriptor::new(
            "plugin.sample.sibling",
            "Sibling",
            "Plugins",
        ))
        .unwrap();

    let mut store = ContributionStore::default();
    let owned_ticket = store.contribute(plugin_source("sample"), owned).unwrap();
    let sibling_ticket = store.contribute(plugin_source("sample"), sibling).unwrap();
    let published = store.snapshot();

    assert_eq!(
        published
            .views_for_ticket(owned_ticket)
            .map(|view| view.id())
            .collect::<Vec<_>>(),
        vec!["plugin.sample.first", "plugin.sample.second"],
        "host revoke must obtain every owned descriptor before publishing removal"
    );
    assert_eq!(
        published
            .views_for_ticket(sibling_ticket)
            .map(|view| view.id())
            .collect::<Vec<_>>(),
        vec!["plugin.sample.sibling"]
    );
    assert_eq!(
        store
            .batch_for_ticket(owned_ticket)
            .expect("active ticket must retain its source batch")
            .views()
            .map(|view| view.id())
            .collect::<Vec<_>>(),
        vec!["plugin.sample.first", "plugin.sample.second"]
    );
    assert_eq!(
        published.views(&CapabilitySet::default()).count(),
        1,
        "ticket ownership lookup must not hide capability-gated descriptors"
    );

    store.revoke(owned_ticket);
    assert!(store.batch_for_ticket(owned_ticket).is_none());
    assert_eq!(store.snapshot().views_for_ticket(owned_ticket).count(), 0);
    assert_eq!(published.views_for_ticket(owned_ticket).count(), 2);
}

#[test]
fn changed_since_requires_a_snapshot_rebuild_after_the_retained_journal_expires() {
    let mut store = ContributionStore::default();
    for _ in 0..=CONTRIBUTION_CHANGE_JOURNAL_CAPACITY {
        store
            .contribute(ContributionSource::Builtin, ContributionBatch::default())
            .unwrap();
    }

    let expired = store.changed_since(0);
    assert!(expired.is_reset());
    assert!(expired.changes().is_empty());
    assert_eq!(expired.to_generation(), store.generation());

    let retained_from = store
        .generation()
        .saturating_sub(CONTRIBUTION_CHANGE_JOURNAL_CAPACITY as u64);
    let retained = store.changed_since(retained_from);
    assert!(!retained.is_reset());
    assert_eq!(
        retained.changes().len(),
        CONTRIBUTION_CHANGE_JOURNAL_CAPACITY
    );
    assert_eq!(
        retained.changes().first().map(|change| change.generation()),
        Some(retained_from + 1)
    );

    let current = store.changed_since(store.generation());
    assert!(!current.is_reset());
    assert!(current.changes().is_empty());

    let future = store.changed_since(store.generation() + 1);
    assert!(future.is_reset());
    assert!(future.changes().is_empty());
    assert_eq!(future.to_generation(), store.generation());
}

#[test]
fn template_replacement_is_atomic_and_preserves_ticket_and_old_readers() {
    let mut original = ContributionBatch::default();
    original
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.sample.template",
            "plugins://sample/original.zui",
        ))
        .unwrap();
    let mut sibling = ContributionBatch::default();
    sibling
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.sample.sibling",
            "plugins://sample/sibling.zui",
        ))
        .unwrap();

    let mut store = ContributionStore::default();
    let ticket = store.contribute(plugin_source("sample"), original).unwrap();
    store.contribute(plugin_source("sample"), sibling).unwrap();
    let generation_before_replacement = store.generation();
    let old_reader = store.snapshot();

    let collision = store
        .replace_ui_template_contributions(
            ticket,
            [EditorUiTemplateDescriptor::new(
                "plugin.sample.sibling",
                "plugins://sample/collision.zui",
            )],
            BTreeMap::new(),
        )
        .unwrap_err();
    assert!(matches!(
        collision,
        ContributionError::DuplicateContribution {
            kind: "ui template",
            id,
        } if id == "plugin.sample.sibling"
    ));
    assert_eq!(store.generation(), generation_before_replacement);

    let invalid_id = store
        .replace_ui_template_contributions(
            ticket,
            [EditorUiTemplateDescriptor::new(
                " ",
                "plugins://sample/invalid.zui",
            )],
            BTreeMap::new(),
        )
        .unwrap_err();
    assert!(matches!(
        invalid_id,
        ContributionError::Batch(EditorExtensionRegistryError::InvalidContributionId {
            kind: "ui template",
            ..
        })
    ));
    assert_eq!(store.generation(), generation_before_replacement);

    store
        .replace_ui_template_contributions(
            ticket,
            [EditorUiTemplateDescriptor::new(
                "plugin.sample.template",
                "plugins://sample/reloaded.zui",
            )],
            BTreeMap::new(),
        )
        .unwrap();

    let capabilities = CapabilitySet::default();
    assert_eq!(
        old_reader
            .ui_templates(&capabilities)
            .find(|descriptor| descriptor.id() == "plugin.sample.template")
            .map(EditorUiTemplateDescriptor::ui_document),
        Some("plugins://sample/original.zui")
    );
    let current_reader = store.snapshot();
    assert_eq!(
        current_reader
            .ui_templates(&capabilities)
            .find(|descriptor| descriptor.id() == "plugin.sample.template")
            .map(EditorUiTemplateDescriptor::ui_document),
        Some("plugins://sample/reloaded.zui")
    );
    let delta = store.changed_since(generation_before_replacement);
    assert_eq!(delta.changes().len(), 1);
    assert_eq!(delta.changes()[0].ticket(), ticket);
    assert_eq!(delta.changes()[0].kind(), ContributionChangeKind::Replaced);

    let report = store.revoke(ticket);
    assert!(report.revoked());
    assert_eq!(report.removed().ui_templates(), 1);
    assert!(
        store
            .snapshot()
            .ui_templates(&capabilities)
            .all(|descriptor| descriptor.id() != "plugin.sample.template")
    );
}
