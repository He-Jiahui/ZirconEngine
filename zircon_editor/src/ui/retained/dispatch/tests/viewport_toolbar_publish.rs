use super::*;
use crate::scene::viewport::SceneViewportChromeSettings;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};
fn captured(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
) -> (
    std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
    std::sync::Arc<serde_json::Value>,
) {
    bridge
        .prepared_viewport_toolbar_publication()
        .expect("actual prepared toolbar")
}
fn node<'a>(value: &'a serde_json::Value, control: &str) -> &'a serde_json::Value {
    value["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["controlId"] == control)
        .unwrap()
}
#[test]
fn prepared_toolbar_retains_hidden_original_nodes_without_touching_source_properties() {
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(4096.0, 900.0)).unwrap();
    let (frame, value) = captured(&bridge);
    assert_eq!(value["surfaceGeneration"].as_u64(), Some(frame.generation));
    assert_eq!(value["surfaceTreeId"], serde_json::json!(frame.tree_id));
    assert_eq!(value["groupNodeIds"].as_array().unwrap().len(), 11);
    assert_eq!(node(&value, MORE)["visible"], false);
    assert_eq!(node(&value, OVERFLOW_CONTROL)["visible"], false);
    let root = bridge.control_node_id(TOOLBAR).unwrap();
    assert!(!bridge
        .template_surface
        .surface
        .tree
        .node(root)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .contains_key("__prepared_viewport_toolbar_publication"));
    let ids = value["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["nodeId"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), value["nodes"].as_array().unwrap().len());
}
#[test]
fn prepared_toolbar_state_only_update_uses_actual_popup_caption_and_three_snap_bools() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    bridge.mark_host_projection_committed();
    let (_, old) = captured(&bridge);
    let mut chrome = SceneViewportChromeSettings::default();
    chrome.translate_snap_enabled = false;
    chrome.rotate_snap_enabled = true;
    chrome.scale_snap_enabled = false;
    bridge.sync_unreal_viewport_toolbar_state(&chrome).unwrap();
    bridge
        .apply_unreal_overflow_action(MORE, OPEN_ACTION)
        .unwrap();
    bridge
        .select_dropdown_option("SetScaleSnap", "0.25")
        .unwrap();
    let (frame, value) = captured(&bridge);
    assert_eq!(value["surfaceGeneration"].as_u64(), Some(frame.generation));
    assert_eq!(node(&value, OVERFLOW_CONTROL)["state"]["popupOpen"], true);
    assert_eq!(node(&value, "SetScaleSnap")["state"]["value"], "0.25");
    assert_eq!(
        node(&value, "SetScaleSnap")["state"]["valueText"].as_str(),
        bridge
            .control_string("SetScaleSnap", "value_text")
            .as_deref()
    );
    for (control, expected) in [
        ("ToggleTranslateSnap", false),
        ("ToggleRotateSnap", true),
        ("ToggleScaleSnap", false),
    ] {
        assert_eq!(node(&value, control)["state"]["checked"], expected);
    }
    assert_eq!(node(&old, OVERFLOW_CONTROL)["state"]["popupOpen"], false);
}
#[test]
fn prepared_toolbar_geometry_only_scroll_update_is_fresh_and_old_snapshot_stays_frozen() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    bridge
        .apply_unreal_overflow_action(MORE, OPEN_ACTION)
        .unwrap();
    let width = bridge
        .template_surface
        .control_frame(OVERFLOW_CONTROL)
        .unwrap()
        .width;
    bridge
        .set_fixed_control_extent(OVERFLOW_CONTROL, UiSize::new(width, 32.0))
        .unwrap();
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    bridge.mark_host_projection_committed();
    let (_, old) = captured(&bridge);
    let popup = bridge.control_node_id(OVERFLOW_CONTROL).unwrap();
    let frame = bridge
        .template_surface
        .control_frame(OVERFLOW_CONTROL)
        .unwrap();
    let before = bridge
        .template_surface
        .surface
        .tree
        .node(popup)
        .unwrap()
        .scroll_state
        .unwrap()
        .offset;
    bridge
        .route_pointer_event(
            UiPointerEvent::new(
                UiPointerEventKind::Scroll,
                UiPoint::new(frame.x + frame.width / 2.0, frame.y + frame.height / 2.0),
            )
            .with_scroll_delta(3.0),
        )
        .unwrap();
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    let (source, current) = captured(&bridge);
    assert_eq!(
        node(&current, OVERFLOW_CONTROL)["scroll"]["offset"].as_f64(),
        Some((before + 3.0) as f64)
    );
    assert_eq!(
        node(&old, OVERFLOW_CONTROL)["scroll"]["offset"].as_f64(),
        Some(before as f64)
    );
    assert_eq!(
        current["surfaceGeneration"].as_u64(),
        Some(source.generation)
    );
    bridge
        .template_surface
        .recompute_layout(bridge.runtime.as_ref(), UiSize::new(960.0, 640.0))
        .unwrap();
    bridge.refresh_unreal_viewport_toolbar_overflow().unwrap();
    let (source, resized) = captured(&bridge);
    assert_eq!(
        resized["surfaceGeneration"].as_u64(),
        Some(source.generation)
    );
    assert_ne!(resized["surfaceGeneration"], current["surfaceGeneration"]);
}
#[test]
fn prepared_toolbar_rejects_stale_raw_value_and_duplicate_surface_identity() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let raw = bridge
        .template_surface
        .host_projection
        .nodes
        .iter_mut()
        .find(|node| node.control_id.as_deref() == Some("ToggleTranslateSnap"))
        .unwrap();
    raw.properties.insert(
        "checked".into(),
        crate::ui::template_runtime::RetainedUiHostValue::String("stale".into()),
    );
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    assert!(bridge.prepared_viewport_toolbar_publication().is_none());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let duplicate = bridge.template_surface.host_projection.nodes[0].clone();
    bridge
        .template_surface
        .host_projection
        .nodes
        .push(duplicate);
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    assert!(bridge.prepared_viewport_toolbar_publication().is_none());
}
#[test]
fn prepared_toolbar_rejects_same_bounds_stale_derived_text_value_and_disabled_fields() {
    for field in ["text", "value_text", "disabled", "selected"] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
        let raw = bridge
            .template_surface
            .host_projection
            .nodes
            .iter_mut()
            .find(|node| node.control_id.as_deref() == Some("SetProjectionMode"))
            .unwrap();
        let bounds = raw.frame;
        match field {
            "text" => raw.text = Some("stale-caption".into()),
            "value_text" => raw.value_text = Some("stale-value".into()),
            "disabled" => raw.disabled = !raw.disabled,
            _ => {
                raw.properties.insert(
                    "selected".into(),
                    crate::ui::template_runtime::RetainedUiHostValue::Bool(true),
                );
            }
        }
        assert_eq!(raw.frame, bounds);
        bridge
            .template_surface
            .refresh_after_state_change(bridge.runtime.as_ref())
            .unwrap();
        assert!(
            bridge.prepared_viewport_toolbar_publication().is_none(),
            "{field}"
        );
    }
}
#[test]
fn prepared_toolbar_rejects_unmerged_hidden_topology_and_stale_cached_arc() {
    for change in ["add", "remove", "reparent"] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
        let popup = bridge.control_node_id(OVERFLOW_CONTROL).unwrap();
        let children = bridge
            .template_surface
            .surface
            .tree
            .node(popup)
            .unwrap()
            .children
            .clone();
        assert!(!children.is_empty());
        let group = children[0];
        let root = bridge.control_node_id(TOOLBAR).unwrap();
        match change {
            "add" => bridge
                .template_surface
                .surface
                .tree
                .nodes
                .get_mut(&popup)
                .unwrap()
                .children
                .push(group),
            "remove" => {
                bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .get_mut(&popup)
                    .unwrap()
                    .children
                    .pop();
            }
            _ => {
                bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .get_mut(&group)
                    .unwrap()
                    .parent = Some(root);
            }
        }
        // No semantic workset has been queued. A refreshed label alone cannot
        // turn the old raw topology into current source evidence.
        bridge
            .template_surface
            .refresh_after_state_change(bridge.runtime.as_ref())
            .unwrap();
        assert!(
            bridge.prepared_viewport_toolbar_publication().is_none(),
            "{change}"
        );
    }
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let (frame, value) = captured(&bridge);
    bridge
        .template_surface
        .prepared_viewport_toolbar_publication =
        Some((std::sync::Arc::new((*frame).clone()), value));
    assert!(
        bridge.prepared_viewport_toolbar_publication().is_none(),
        "cached frame must be the actual prepared source Arc"
    );
}
#[test]
fn prepared_toolbar_preserves_original_logical_desired_sizes_across_hidden_popup_and_open_scroll() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let (_, closed) = captured(&bridge);
    let contract = bridge.viewport_toolbar_overflow.contract.as_ref().unwrap();
    let sizes = bridge
        .viewport_toolbar_overflow
        .measured_group_sizes
        .clone();
    assert_eq!(sizes.len(), contract.groups.len());
    assert!(bridge.viewport_toolbar_overflow.inline_count < sizes.len());
    let expected = contract
        .groups
        .iter()
        .zip(&sizes)
        .map(|(group, size)| (group.control_id.clone(), *size))
        .collect::<Vec<_>>();
    for (control, size) in &expected {
        let desired = &node(&closed, control)["desiredSize"];
        assert_eq!(desired["coordinateSpace"], "logical-pixels");
        assert_eq!(desired["width"].as_f64(), Some(size.width as f64));
        assert_eq!(desired["height"].as_f64(), Some(size.height as f64));
        assert!(size.width > 0.0 && size.height > 0.0);
    }
    assert_eq!(node(&closed, OVERFLOW_CONTROL)["visible"], false);
    bridge
        .apply_unreal_overflow_action(MORE, OPEN_ACTION)
        .unwrap();
    bridge.refresh_unreal_viewport_toolbar_overflow().unwrap();
    let (_, opened) = captured(&bridge);
    for (control, size) in &expected {
        assert_eq!(
            node(&opened, control)["desiredSize"]["width"].as_f64(),
            Some(size.width as f64)
        );
    }
    assert_eq!(node(&opened, OVERFLOW_CONTROL)["visible"], true);
    assert_eq!(node(&closed, OVERFLOW_CONTROL)["visible"], false);
    // A changed caption/layout input cannot reuse the old measurement proof.
    bridge.viewport_toolbar_overflow.measured_key = None;
    assert!(bridge.prepared_viewport_toolbar_publication().is_none());
    bridge.refresh_unreal_viewport_toolbar_overflow().unwrap();
    assert!(bridge.prepared_viewport_toolbar_publication().is_some());
}

#[test]
fn prepared_toolbar_mounted_model_layout_keeps_current_measurement_after_chrome_and_resize() {
    use crate::ui::workbench::{
        autolayout::WorkbenchChromeMetrics, fixture::default_preview_fixture,
        model::WorkbenchViewModel,
    };

    let chrome = default_preview_fixture().build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(960.0, 640.0)).unwrap();
    let (_, original) = captured(&bridge);
    let original_value = (*original).clone();
    for (width, height, scale) in [
        (1280.0, 800.0, 1.25),
        (900.0, 620.0, 1.25),
        (640.0, 520.0, 1.25),
        (1280.0, 800.0, 1.5),
        (1280.0, 800.0, 1.0),
    ] {
        bridge.prepare_chrome_state_for_layout(&chrome).unwrap();
        bridge
            .recompute_mounted_layout_with_workbench_model_at_scale(
                zircon_runtime_interface::ui::layout::UiFrame::new(
                    17.0,
                    23.0,
                    width * scale,
                    height * scale,
                ),
                scale,
                &model,
                &metrics,
            )
            .unwrap();
        let (frame, publication) = captured(&bridge);
        assert!(std::sync::Arc::ptr_eq(
            &frame,
            &bridge.surface().surface_frame()
        ));
        let row = node(&publication, "UnrealViewportToolbarRow");
        let left = row["bounds"]["x"].as_f64().unwrap();
        let right = left + row["bounds"]["width"].as_f64().unwrap();
        for child in publication["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|child| child["parentNodeId"] == row["nodeId"] && child["visible"] == true)
        {
            let x = child["bounds"]["x"].as_f64().unwrap();
            let extent = child["bounds"]["width"].as_f64().unwrap();
            assert!(
                x >= left - 0.001 && x + extent <= right + 0.001,
                "visible toolbar child escapes after {width}x{height} at {scale}: {child}"
            );
        }
        assert_eq!(
            *original, original_value,
            "old source snapshot stays immutable"
        );
    }
}

fn toolbar_projection_source_node_mut<'a>(
    root: &'a mut crate::ui::template_runtime::RetainedUiNodeProjection,
    control: &str,
) -> Option<&'a mut crate::ui::template_runtime::RetainedUiNodeProjection> {
    if root.control_id.as_deref() == Some(control) {
        return Some(root);
    }
    for child in &mut root.children {
        if let Some(node) = toolbar_projection_source_node_mut(child, control) {
            return Some(node);
        }
    }
    None
}

#[test]
fn actual_surface_preserves_known_inactive_bool_and_unknown_absence_in_full_and_sparse_projection()
{
    use crate::ui::template_runtime::RetainedUiHostValue;

    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(4096.0, 900.0)).unwrap();
    let known = bridge.control_node_id("SetProjectionMode").unwrap();
    let unknown = bridge.control_node_id(TOOLBAR).unwrap();
    let known_metadata = bridge
        .template_surface
        .surface
        .tree
        .node(known)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(
        known_metadata.attributes.get("selected"),
        Some(&toml::Value::Boolean(false))
    );
    assert_eq!(
        known_metadata.attributes.get("popup_open"),
        Some(&toml::Value::Boolean(false))
    );
    let unknown_metadata = bridge
        .template_surface
        .surface
        .tree
        .node(unknown)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    for key in ["selected", "checked", "open", "enabled"] {
        assert!(!unknown_metadata.attributes.contains_key(key), "{key}");
    }
    let (_, publication) = captured(&bridge);
    assert_eq!(
        node(&publication, "SetProjectionMode")["state"]["selected"],
        false
    );
    assert_eq!(
        node(&publication, "SetProjectionMode")["state"]["popupOpen"],
        false
    );
    assert!(node(&publication, TOOLBAR)["state"]["selected"].is_null());
    assert!(node(&publication, TOOLBAR)["state"]["checked"].is_null());

    let mut projection = bridge
        .runtime
        .project_document(crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID)
        .unwrap();
    // Old authored defaults in a cached projection cannot create current Source
    // state. Ordinary authored metadata must still supplement both paths.
    let source = toolbar_projection_source_node_mut(&mut projection.root, TOOLBAR).unwrap();
    for key in ["selected", "checked", "open", "enabled"] {
        source
            .attributes
            .insert(key.to_string(), toml::Value::Boolean(false));
    }
    source.attributes.insert(
        "source_only_metadata".to_string(),
        toml::Value::String("kept".into()),
    );
    let full = bridge
        .runtime
        .build_retained_host_projection_with_surface(&projection, &bridge.template_surface.surface)
        .unwrap();
    let index = projection.surface_metadata_index();
    let sparse = bridge
        .runtime
        .build_retained_host_nodes_with_surface(
            &projection,
            &bridge.template_surface.surface,
            &std::collections::BTreeSet::from([known, unknown]),
            &index,
        )
        .unwrap();
    let sparse_known = &sparse.iter().find(|(id, _)| *id == known).unwrap().1;
    let sparse_unknown = &sparse.iter().find(|(id, _)| *id == unknown).unwrap().1;
    for raw in [
        full.node_by_control_id("SetProjectionMode").unwrap(),
        sparse_known,
    ] {
        assert_eq!(
            raw.properties.get("selected"),
            Some(&RetainedUiHostValue::Bool(false))
        );
        assert_eq!(
            raw.properties.get("popup_open"),
            Some(&RetainedUiHostValue::Bool(false))
        );
    }
    for raw in [full.node_by_control_id(TOOLBAR).unwrap(), sparse_unknown] {
        for key in ["selected", "checked", "open", "enabled"] {
            assert!(!raw.properties.contains_key(key), "{key}");
        }
        assert_eq!(
            raw.properties.get("source_only_metadata"),
            Some(&RetainedUiHostValue::String("kept".into()))
        );
    }
}

#[test]
fn actual_surface_true_to_false_state_is_published_without_replaying_cached_defaults() {
    use crate::ui::template_runtime::RetainedUiHostValue;

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(4096.0, 900.0)).unwrap();
    let id = bridge.control_node_id("SetProjectionMode").unwrap();
    let projection = bridge
        .runtime
        .project_document(crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID)
        .unwrap();
    let index = projection.surface_metadata_index();
    let (_, old) = captured(&bridge);
    for active in [true, false] {
        bridge.mutate_node_bool(id, "selected", active).unwrap();
        bridge.mutate_node_bool(id, "disabled", active).unwrap();
        assert!(bridge
            .template_surface
            .surface
            .pending_rebuild_node_ids()
            .contains(&id));
        bridge
            .template_surface
            .refresh_after_state_change(bridge.runtime.as_ref())
            .unwrap();
        let metadata = bridge
            .template_surface
            .surface
            .tree
            .node(id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap();
        for key in ["selected", "disabled"] {
            assert_eq!(
                metadata.attributes.get(key),
                Some(&toml::Value::Boolean(active)),
                "{key}"
            );
        }
        let full = bridge
            .runtime
            .build_retained_host_projection_with_surface(
                &projection,
                &bridge.template_surface.surface,
            )
            .unwrap();
        let sparse = bridge
            .runtime
            .build_retained_host_nodes_with_surface(
                &projection,
                &bridge.template_surface.surface,
                &std::collections::BTreeSet::from([id]),
                &index,
            )
            .unwrap();
        for raw in [
            bridge
                .template_surface
                .host_projection
                .node_by_control_id("SetProjectionMode")
                .unwrap(),
            full.node_by_control_id("SetProjectionMode").unwrap(),
            &sparse[0].1,
        ] {
            for key in ["selected", "disabled"] {
                assert_eq!(
                    raw.properties.get(key),
                    Some(&RetainedUiHostValue::Bool(active)),
                    "{key}"
                );
            }
        }
        let (frame, publication) = captured(&bridge);
        assert_eq!(
            publication["surfaceGeneration"].as_u64(),
            Some(frame.generation)
        );
        assert_eq!(
            node(&publication, "SetProjectionMode")["state"]["selected"],
            active
        );
        assert_eq!(
            node(&publication, "SetProjectionMode")["state"]["disabled"],
            active
        );
    }
    assert_eq!(node(&old, "SetProjectionMode")["state"]["selected"], false);
    assert_eq!(node(&old, "SetProjectionMode")["state"]["disabled"], false);
}

#[test]
fn prepared_toolbar_rejects_false_when_actual_source_state_is_unknown() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(4096.0, 900.0)).unwrap();
    let id = bridge.control_node_id(TOOLBAR).unwrap();
    assert!(!bridge
        .template_surface
        .surface
        .tree
        .node(id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .contains_key("selected"));
    let (_, before) = captured(&bridge);
    assert!(node(&before, TOOLBAR)["state"]["selected"].is_null());
    bridge
        .template_surface
        .host_projection
        .nodes
        .iter_mut()
        .find(|raw| raw.surface_node_id == Some(id))
        .unwrap()
        .properties
        .insert(
            "selected".into(),
            crate::ui::template_runtime::RetainedUiHostValue::Bool(false),
        );
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    assert!(bridge.prepared_viewport_toolbar_publication().is_none());
}
