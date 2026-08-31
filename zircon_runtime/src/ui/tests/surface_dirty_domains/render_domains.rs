use super::*;

#[test]
fn surface_dirty_render_reuses_unchanged_commands_without_damage() {
    let mut surface = test_surface();
    let command_count = surface.render_extract.list.commands.len();

    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .dirty
        .render = true;

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert!(report.render_rebuilt);
    assert_eq!(report.render_command_reused_count, command_count);
    assert_eq!(report.render_command_rebuilt_count, 0);
    assert_eq!(report.render_damage_rect_count, 0);
    assert!(
        surface
            .render_extract
            .list
            .to_paint_elements()
            .iter()
            .all(|element| element.cache_generation.is_some())
    );
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_dirty_render_only_metadata_does_not_trigger_hit_or_input_rebuild() {
    let mut surface = test_surface();
    let metadata = UiTemplateNodeMetadata {
        component: "Button".to_string(),
        control_id: Some("DirtyDomainButton".to_string()),
        attributes: toml::from_str("material_tone = 'primary'").unwrap(),
        ..Default::default()
    };
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .template_metadata = Some(metadata);
    surface.clear_dirty_flags();

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "material_tone",
            UiValue::String("secondary".to_string()),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        mutation.invalidation.dirty,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert!(
        !surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        report,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: false,
            hit_grid: false,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);
}

#[test]
fn surface_rejects_direct_editable_text_derived_state_mutation() {
    let mut surface = test_surface();
    let metadata = UiTemplateNodeMetadata {
        component: "TextField".to_string(),
        control_id: Some("DirtyDomainTextField".to_string()),
        attributes: toml::from_str(
            r#"
value = "Runtime"
caret_offset = 3
selection_anchor = 3
selection_focus = 3
composition_text = ""
"#,
        )
        .unwrap(),
        ..Default::default()
    };
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .template_metadata = Some(metadata);
    surface.clear_dirty_flags();

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "caret_offset",
            UiValue::Int(4),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Rejected);
    assert!(
        mutation
            .message
            .as_deref()
            .is_some_and(|message| message.contains("editable text transaction"))
    );
    assert_eq!(mutation.invalidation.dirty, UiDirtyFlags::default());
    assert_eq!(mutation.binding.rejected_count, 1);
    assert_eq!(
        mutation.binding.updates[0].source.kind,
        UiBindingSourceKind::RuntimeState
    );
    assert_eq!(
        mutation.binding.updates[0].status,
        UiBindingUpdateStatus::Rejected
    );
    assert_eq!(
        surface
            .tree
            .node(button_id())
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get("caret_offset"))
            .and_then(toml::Value::as_integer),
        Some(3)
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
}

#[test]
fn surface_external_text_value_change_commits_complete_edit_state_once() {
    let mut surface = editable_text_property_surface(
        "TextField",
        r#"
value = "Runtime"
caret_offset = 7
caret_affinity = "upstream"
selection_anchor = 1
selection_focus = 6
composition_start = 1
composition_end = 6
composition_text = "untim"
composition_restore_text = "untim"
composition_clauses = [{ start_byte = 0, end_byte = 5, kind = "input" }]
"#,
    );
    let revision_before = editable_text_revision(&surface);

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "value",
            UiValue::String("Hi".to_string()),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        mutation.invalidation.dirty,
        UiDirtyFlags {
            layout: true,
            render: true,
            text: true,
            ..Default::default()
        }
    );
    assert_eq!(mutation.binding.applied_count, 20);
    assert!(mutation.binding.updates.iter().all(|update| {
        update.source.kind == UiBindingSourceKind::RuntimeState
            && update.status == UiBindingUpdateStatus::Applied
    }));
    assert_eq!(editable_text_revision(&surface), revision_before + 1);

    let attributes = editable_text_attributes(&surface);
    assert_eq!(attributes["value"].as_str(), Some("Hi"));
    assert_eq!(attributes["caret_offset"].as_integer(), Some(2));
    assert_eq!(attributes["caret_affinity"].as_str(), Some("downstream"));
    assert_eq!(attributes["selection_anchor"].as_integer(), Some(2));
    assert_eq!(attributes["selection_focus"].as_integer(), Some(2));
    assert_eq!(attributes["composition_start"].as_integer(), Some(2));
    assert_eq!(attributes["composition_end"].as_integer(), Some(2));
    assert_eq!(attributes["composition_text"].as_str(), Some(""));
    assert_eq!(attributes["composition_restore_text"].as_str(), Some(""));
    assert_eq!(
        attributes["composition_clauses"].as_array(),
        Some([].as_slice())
    );
}

#[test]
fn surface_external_number_field_value_preserves_numeric_storage() {
    let mut surface = editable_text_property_surface(
        "NumberField",
        r#"
value = 12.5
caret_offset = 4
caret_affinity = "upstream"
selection_anchor = 0
selection_focus = 4
composition_start = 0
composition_end = 4
composition_text = "12.5"
composition_restore_text = "12.5"
composition_clauses = [{ start_byte = 0, end_byte = 4, kind = "input" }]
"#,
    );

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "value",
            UiValue::Float(7.0),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Accepted);
    let attributes = editable_text_attributes(&surface);
    assert_eq!(attributes["value"].as_float(), Some(7.0));
    assert_eq!(attributes["caret_offset"].as_integer(), Some(1));
    assert_eq!(attributes["selection_anchor"].as_integer(), Some(1));
    assert_eq!(attributes["selection_focus"].as_integer(), Some(1));
    assert_eq!(attributes["composition_text"].as_str(), Some(""));
}

#[test]
fn surface_external_unchanged_text_value_preserves_edit_state() {
    let mut surface = editable_text_property_surface(
        "TextField",
        r#"
value = "Runtime"
caret_offset = 3
caret_affinity = "upstream"
selection_anchor = 1
selection_focus = 3
composition_start = 1
composition_end = 3
composition_text = "un"
composition_restore_text = "un"
composition_clauses = [{ start_byte = 0, end_byte = 2, kind = "input" }]
"#,
    );
    let revision_before = editable_text_revision(&surface);

    let mutation = surface
        .mutate_property(UiPropertyMutationRequest::new(
            button_id(),
            "value",
            UiValue::String("Runtime".to_string()),
        ))
        .unwrap();

    assert_eq!(mutation.status, UiPropertyMutationStatus::Unchanged);
    assert_eq!(mutation.binding.unchanged_count, 1);
    assert_eq!(editable_text_revision(&surface), revision_before);
    let attributes = editable_text_attributes(&surface);
    assert_eq!(attributes["caret_offset"].as_integer(), Some(3));
    assert_eq!(attributes["caret_affinity"].as_str(), Some("upstream"));
    assert_eq!(attributes["selection_anchor"].as_integer(), Some(1));
    assert_eq!(attributes["selection_focus"].as_integer(), Some(3));
    assert_eq!(attributes["composition_text"].as_str(), Some("un"));
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
}

fn editable_text_property_surface(component: &str, attributes: &str) -> UiSurface {
    let mut surface = test_surface();
    surface
        .tree
        .node_mut(button_id())
        .expect("editable text node should exist")
        .template_metadata = Some(UiTemplateNodeMetadata {
        component: component.to_string(),
        control_id: Some("DirtyDomainEditableText".to_string()),
        attributes: toml::from_str(attributes).expect("editable text attributes should parse"),
        ..Default::default()
    });
    surface.clear_dirty_flags();
    surface
}

fn editable_text_attributes(
    surface: &UiSurface,
) -> &std::collections::BTreeMap<String, toml::Value> {
    &surface
        .tree
        .node(button_id())
        .and_then(|node| node.template_metadata.as_ref())
        .expect("editable text metadata should exist")
        .attributes
}

fn editable_text_revision(surface: &UiSurface) -> u64 {
    surface
        .tree
        .node(button_id())
        .and_then(|node| node.layout_cache.retained_text_layout_revision())
        .expect("editable text layout revision should remain reusable")
}

#[test]
fn surface_dirty_render_only_dispatch_effect_does_not_trigger_hit_or_input_rebuild() {
    let mut surface = test_surface();
    surface.clear_dirty_flags();

    let result = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::DirtyRedraw {
            target: button_id(),
            dirty: UiDirtyFlags {
                render: true,
                ..Default::default()
            },
            reason: UiRedrawRequestReason::Style,
        }),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert!(
        !surface
            .tree
            .node(button_id())
            .expect("button node should exist")
            .state_flags
            .dirty
    );

    let report = surface.rebuild_dirty(root_size()).unwrap();

    assert_report_phases(
        &surface,
        report,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        },
        ExpectedPhases {
            layout: false,
            arranged: false,
            hit_grid: false,
            render: true,
        },
    );
    assert_dirty_cleared(&surface);
}
