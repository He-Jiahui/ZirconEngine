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
    assert!(surface
        .render_extract
        .list
        .to_paint_elements()
        .iter()
        .all(|element| element.cache_generation.is_some()));
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
fn surface_dirty_text_edit_visual_metadata_stays_render_only() {
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
