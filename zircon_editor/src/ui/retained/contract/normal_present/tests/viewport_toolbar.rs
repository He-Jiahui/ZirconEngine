use super::*;
use crate::ui::retained_host::{
    callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    ui::to_host_contract_workbench_window_nodes,
};
use zircon_runtime_interface::ui::layout::UiSize;
fn actual_fixture() -> (HostWindowPresentationData, Vec<Option<Value>>) {
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let (frame, logical) = bridge.prepared_viewport_toolbar_publication().unwrap();
    // At unit scale and zero mount the captured logical and physical coordinates
    // are identical. Production remount/COW/cursor behavior has its own tests.
    let mut physical = (*logical).clone();
    physical["coordinateSpace"] = json!("physical-pixels");
    let root = physical["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["nodeId"] == physical["rootNodeId"])
        .unwrap();
    let sources = vec![Some(
        json!({"sourcePath":root["sourcePath"],"sourceNodeId":root["sourceNodeId"],"instancePath":root["instancePath"]}),
    )];
    let data = HostWindowPresentationData {
        workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(
            bridge.host_projection(),
        )),
        normal_viewport_toolbar: Some((Default::default(), frame, std::sync::Arc::new(physical))),
        ..Default::default()
    };
    (data, sources)
}
#[test]
fn immutable_toolbar_publication_requires_prepared_source_and_actual_submitted_owner() {
    assert!(
        viewport_toolbar_publication(&HostWindowPresentationData::default(), &[], None).is_none()
    );
    let (data, sources) = actual_fixture();
    assert!(viewport_toolbar_publication(&data, &sources, None).is_some());
    assert!(viewport_toolbar_publication(&data, &[], None).is_none());
}
#[test]
fn immutable_toolbar_publication_rejects_mismatched_frame_identity_and_source_owner() {
    let (data, sources) = actual_fixture();
    for key in ["surfaceGeneration", "surfaceTreeId", "rootNodeId"] {
        let mut wrong = data.clone();
        let (_, _, value) = wrong.normal_viewport_toolbar.as_mut().unwrap();
        let value = std::sync::Arc::make_mut(value);
        value[key] = if key == "surfaceGeneration" {
            json!(value[key].as_u64().unwrap() + 1)
        } else {
            json!("foreign")
        };
        assert!(
            viewport_toolbar_publication(&wrong, &sources, None).is_none(),
            "{key}"
        );
    }
    let mut wrong = data.clone();
    let (_, _, value) = wrong.normal_viewport_toolbar.as_mut().unwrap();
    let value = std::sync::Arc::make_mut(value);
    let root = value["rootNodeId"].as_str().unwrap().to_string();
    let root = value["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["nodeId"] == root)
        .unwrap();
    root["instancePath"] = json!("foreign");
    assert!(viewport_toolbar_publication(&wrong, &sources, None).is_none());
}
#[test]
fn immutable_toolbar_accepts_an_older_unchanged_arranged_owner_but_rejects_changed_topology() {
    let (mut data, sources) = actual_fixture();
    let (_, frame, value) = data.normal_viewport_toolbar.as_mut().unwrap();
    std::sync::Arc::make_mut(frame).generation += 1;
    std::sync::Arc::make_mut(value)["surfaceGeneration"] = json!(frame.generation);
    // Actual typed nodes retain their old Arc. The newer source frame is
    // admitted because each complete arranged owner and displayed state agrees.
    assert!(viewport_toolbar_publication(&data, &sources, None).is_some());
    let (_, frame, _) = data.normal_viewport_toolbar.as_mut().unwrap();
    let mut arranged = serde_json::to_value(frame.arranged_tree.as_ref()).unwrap();
    for node in arranged["nodes"].as_array_mut().unwrap() {
        node["enabled"] = json!(false);
    }
    std::sync::Arc::make_mut(frame).arranged_tree =
        std::sync::Arc::new(serde_json::from_value(arranged).unwrap());
    assert!(viewport_toolbar_publication(&data, &sources, None).is_none());
}
#[test]
fn immutable_toolbar_rejects_same_bounds_selection_caption_text_and_disabled_mismatch() {
    let (data, sources) = actual_fixture();
    for field in ["checked", "selected", "text", "valueText", "disabled"] {
        let mut wrong = data.clone();
        let (_, _, value) = wrong.normal_viewport_toolbar.as_mut().unwrap();
        let value = std::sync::Arc::make_mut(value);
        let target = if field == "valueText" {
            "SetProjectionMode"
        } else {
            "UnrealViewportToolbarMore"
        };
        let node = value["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["controlId"] == target)
            .unwrap();
        assert_eq!(node["visible"], true, "visible submitted owner: {target}");
        node["state"][field] = match field {
            "text" | "valueText" => json!("stale-caption"),
            _ => json!(!node["state"][field].as_bool().unwrap_or(false)),
        };
        assert!(
            viewport_toolbar_publication(&wrong, &sources, None).is_none(),
            "{field}"
        );
    }
}
