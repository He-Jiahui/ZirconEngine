use super::contract::{read_json, Catalog, ReviewCase};
use serde_json::Value;

#[test]
fn capture_summary_requires_no_failed_or_pending_cases_for_native_readiness() {
    let empty = super::CaptureSummary {
        captured: 0,
        failed: 0,
        pending: 0,
        report_path: std::path::PathBuf::from("report.json"),
    };
    assert!(!empty.is_ready());

    let pending = super::CaptureSummary {
        captured: 1,
        failed: 0,
        pending: 1,
        report_path: std::path::PathBuf::from("report.json"),
    };
    assert!(!pending.is_ready());

    let failed = super::CaptureSummary {
        captured: 1,
        failed: 1,
        pending: 0,
        report_path: std::path::PathBuf::from("report.json"),
    };
    assert!(!failed.is_ready());

    let ready = super::CaptureSummary {
        captured: 1,
        failed: 0,
        pending: 0,
        report_path: std::path::PathBuf::from("report.json"),
    };
    assert!(ready.is_ready());
}

fn repo_root() -> std::path::PathBuf {
    std::env::var_os("ZUI_LAYOUT_REPO_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
        .canonicalize()
        .unwrap()
}

#[test]
fn editor_case_validates_physical_dpi_and_unimplemented_inputs() {
    let mut case: ReviewCase = serde_json::from_value(serde_json::json!({
        "id": "default", "sourcePath": "one.zui", "host": "component",
        "viewport": {"width": 360, "height": 520}, "dpi": 1,
        "locale": "en-US", "state": "default", "data": {}
    }))
    .unwrap();
    for state in [
        "default", "hover", "pressed", "focused", "disabled", "selected",
    ] {
        case.state = state.into();
        assert!(case.validate("one.zui").is_ok());
    }
    case.state = "unsupported".into();
    assert!(case.validate("one.zui").is_err());
    case.state = "default".into();
    case.dpi = 1.5;
    assert!(case.validate("one.zui").is_ok());
    let physical = case.physical_viewport().unwrap();
    assert_eq!((physical.width, physical.height), (540, 780));
    case.dpi = 0.0;
    assert!(case.validate("one.zui").unwrap_err().contains("DPI"));
    case.dpi = 16.0;
    assert!(case
        .validate("one.zui")
        .unwrap_err()
        .contains("physical viewport"));
    case.dpi = 1.0;
    case.data = serde_json::json!({"rows": []});
    assert!(case.validate("one.zui").is_err());
}

#[test]
fn workbench_state_selector_is_restricted_to_editor_cases() {
    let mut case: ReviewCase = serde_json::from_value(serde_json::json!({
        "id": "focused-control", "sourcePath": "one.zui", "host": "editor",
        "viewport": {"width": 360, "height": 520}, "dpi": 1,
        "locale": "en-US", "state": "focused",
        "data": {"workbenchState": {"sourcePath": "one.zui", "controlId": "NameField"}}
    }))
    .unwrap();
    assert!(case.validate("one.zui").is_ok());

    case.host = "component".into();
    assert!(case.validate("one.zui").is_err());
}

#[test]
fn focus_return_uses_the_popup_invocation_path_for_repeated_anchors() {
    use zircon_runtime::ui::v2::{UiV2SurfaceBuilder, UiZuiAssetLoader};
    use zircon_runtime_interface::ui::event_ui::UiTreeId;
    use zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep;

    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "view"
id = "res://review/repeated-popup-anchor.zui"
version = 2
[root]
node = "root"
[nodes.root]
component = "Overlay"
children = [
  { node = "anchor_first" },
  { node = "anchor_second" },
  { node = "popup" },
]
[nodes.anchor_first]
component = "Button"
control_id = "Invoker"
props = { text = "First", focused = false, focus_visible = false }
[nodes.anchor_second]
component = "Button"
control_id = "Invoker"
props = { text = "Second", focused = false, focus_visible = false }
[nodes.popup]
component = "Dialog"
control_id = "Popup"
widget = { behavior = "popup", popup_anchor = { kind = "control", control_id = "Invoker" }, open_property = "popup_open" }
props = { open = true, popup_open = true }
"#,
    )
    .unwrap();
    let mut surface =
        UiV2SurfaceBuilder::build_surface(UiTreeId::new("focus-return-review"), &document).unwrap();

    for node in surface.tree.nodes.values_mut() {
        let Some(metadata) = node.template_metadata.as_mut() else {
            continue;
        };
        metadata.source_path = Some("main.zui".into());
        match metadata.control_id.as_deref() {
            Some("Popup") => {
                metadata.source_node_id = Some("popup".into());
                metadata.instance_path = Some(Vec::new());
            }
            Some("Invoker") => {
                let text = metadata
                    .attributes
                    .get("text")
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default();
                metadata.source_node_id = Some("anchor".into());
                metadata.instance_path = Some(if text == "Second" {
                    vec![UiTemplateNodeInstancePathStep {
                        source_path: "main.zui".into(),
                        source_node_id: "repeated-component".into(),
                    }]
                } else {
                    Vec::new()
                });
            }
            _ => {}
        }
    }

    let case: ReviewCase = serde_json::from_value(serde_json::json!({
        "id": "focus-return-repeated-anchor",
        "sourcePath": "main.zui",
        "host": "editor",
        "viewport": {"width": 360, "height": 520},
        "dpi": 1,
        "locale": "en-US",
        "state": "focus-return",
        "data": {
            "workbenchState": {
                "sourcePath": "main.zui",
                "controlId": "Popup",
                "sourceNodeId": "popup",
                "instancePath": "[]"
            }
        }
    }))
    .unwrap();
    case.validate("main.zui").unwrap();
    super::state::apply_case(&mut surface, &case).unwrap();

    let buttons = surface
        .tree
        .nodes
        .values()
        .filter_map(|node| node.template_metadata.as_ref())
        .filter(|metadata| metadata.control_id.as_deref() == Some("Invoker"))
        .map(|metadata| {
            let text = metadata
                .attributes
                .get("text")
                .and_then(toml::Value::as_str)
                .unwrap();
            let focused = metadata
                .attributes
                .get("focused")
                .and_then(toml::Value::as_bool);
            (text, focused)
        })
        .collect::<Vec<_>>();
    assert_eq!(buttons.len(), 2);
    assert!(buttons.contains(&("First", Some(true))));
    assert!(buttons.contains(&("Second", Some(false))));
}

#[test]
fn zh_editor_case_requires_a_valid_workbench_text_override() {
    let mut case: ReviewCase = serde_json::from_value(serde_json::json!({
        "id": "long-zh", "sourcePath": "one.zui", "host": "editor",
        "viewport": {"width": 640, "height": 520}, "dpi": 1,
        "locale": "zh-CN", "state": "default",
        "data": {"workbenchState": {"sourcePath": "one.zui", "controlId": "InspectorTitle"}}
    }))
    .unwrap();
    assert!(case.validate("one.zui").is_err());

    case.data = serde_json::json!({
        "workbenchState": {
            "sourcePath": "one.zui", "controlId": "InspectorTitle",
            "textOverrides": [{
                "sourcePath": "one.zui", "sourceNodeId": "inspector.title",
                "instancePath": "[]",
                "controlId": "InspectorTitle", "property": "text",
                "value": "检查器中的长文本内容，用于验证窄屏换行和溢出行为"
            }]
        }
    });
    assert!(case.validate("one.zui").is_ok());

    case.data["workbenchState"]["textOverrides"][0]["value"] = Value::String("\n".into());
    assert!(case.validate("one.zui").is_err());
}

#[test]
fn product_workbench_locale_adapter_requires_a_matching_editor_snapshot() {
    let mut case: ReviewCase = serde_json::from_value(serde_json::json!({
        "id": "product-long-zh", "sourcePath": "one.zui", "host": "editor",
        "viewport": {"width": 640, "height": 520}, "dpi": 1,
        "locale": "zh-CN", "state": "default",
        "data": {"workbenchPresentation": {"activeLocale": "zh-CN"}}
    }))
    .unwrap();
    // This guard checks the locale adapter. The product renderer separately validates the
    // complete snapshot, fingerprints and live Editor state before it can capture evidence.
    assert!(case.validate("one.zui").is_ok());
    case.data["workbenchPresentation"]["activeLocale"] = Value::String("en".into());
    assert!(case.validate("one.zui").is_err());
    case.locale = "en-US".into();
    assert!(case.validate("one.zui").is_ok());
    case.host = "component".into();
    assert!(case.validate("one.zui").is_err());
    case.host = "editor".into();
    case.data["workbenchPresentation"] = serde_json::json!({});
    assert!(case.validate("one.zui").is_err());
}

#[test]
fn workbench_text_override_identity_distinguishes_repeated_instances() {
    let mut case: ReviewCase = serde_json::from_value(serde_json::json!({
        "id": "long-zh-repeated-instance",
        "sourcePath": "one.zui",
        "host": "editor",
        "viewport": {"width": 640, "height": 520},
        "dpi": 1,
        "locale": "zh-CN",
        "state": "default",
        "data": {
            "workbenchState": {
                "sourcePath": "one.zui",
                "controlId": "InspectorTitle",
                "sourceNodeId": "inspector.title",
                "instancePath": "[]",
                "textOverrides": [
                    {
                        "sourcePath": "one.zui",
                        "sourceNodeId": "inspector.title",
                        "controlId": "InspectorTitle",
                        "instancePath": "[]",
                        "property": "text",
                        "value": "属性检查器标题"
                    },
                    {
                        "sourcePath": "one.zui",
                        "sourceNodeId": "inspector.title",
                        "controlId": "InspectorTitle",
                        "instancePath": "[{\"sourcePath\":\"workbench_window.zui\",\"sourceNodeId\":\"right-dock\"}]",
                        "property": "text",
                        "value": "属性检查器标题"
                    }
                ]
            }
        }
    }))
    .unwrap();

    case.validate("one.zui").unwrap();
    let selector = super::state::workbench_state_selector(&case.data)
        .unwrap()
        .unwrap();
    assert_eq!(selector.text_overrides.len(), 2);
    assert_ne!(
        selector.text_overrides[0].instance_path,
        selector.text_overrides[1].instance_path
    );

    case.data["workbenchState"]["textOverrides"][1]["instancePath"] = Value::String("[]".into());
    assert!(case.validate("one.zui").is_err());
}

#[test]
fn editor_combines_open_interaction_with_explicit_scroll_offset() {
    let case: ReviewCase = serde_json::from_value(serde_json::json!({
        "id": "open-scroll-after-640x520-dpi1", "sourcePath": "one.zui", "host": "editor",
        "viewport": {"width": 640, "height": 520}, "dpi": 1,
        "locale": "en-US", "state": "open", "scrollPosition": "end", "data": {}
    }))
    .unwrap();
    case.validate("one.zui").unwrap();
    assert_eq!(case.state, "open");
    assert_eq!(case.scroll_review_state(), "scroll-after");
    let invalid: Result<ReviewCase, _> = serde_json::from_value(serde_json::json!({
        "id": "bad-offset", "sourcePath": "one.zui", "host": "editor",
        "viewport": {"width": 640, "height": 520}, "dpi": 1,
        "locale": "en-US", "state": "open", "scrollPosition": "middle", "data": {}
    }));
    assert!(invalid.is_err());
}

#[test]
fn editor_fractional_dpi_uses_logical_layout_and_physical_retained_paint() {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-zui-dpi-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&root).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(root.clone());
    std::fs::create_dir_all(root.join("docs/layout")).unwrap();
    std::fs::write(
        root.join("source.zui"),
        r#"
[asset]
kind = "view"
id = "res://review/dpi.zui"
version = 2
[root]
node = "button"
[nodes.button]
component = "Button"
control_id = "DpiButton"
props = { text = "Physical evidence" }
"#,
    )
    .unwrap();
    let entry: super::contract::Entry = serde_json::from_value(serde_json::json!({
        "sourcePath": "source.zui", "sourceSha256": "unused",
        "outputPath": "source.zui", "outputSha256": "unused",
        "category": "fixtures", "name": "dpi", "cases": [],
        "dependencySha256": "unused", "dependencyFingerprints": []
    }))
    .unwrap();
    let raw_case = serde_json::json!({
        "id": "dpi-1.5", "sourcePath": "source.zui", "host": "editor",
        "viewport": {"width": 100, "height": 80}, "dpi": 1.5,
        "locale": "en-US", "state": "default", "data": {}
    });
    let case: ReviewCase = serde_json::from_value(raw_case.clone()).unwrap();

    let result = super::capture::render(&root, &entry, &raw_case, &case, &root).unwrap();
    let screenshot = root.join("editor-dpi-1.5.png");
    assert_eq!(image::image_dimensions(screenshot).unwrap(), (150, 120));
    let geometry = read_json(&root.join("editor-dpi-1.5.geometry.json")).unwrap();
    assert_eq!(geometry["coordinateSpace"], "logical");
    assert_eq!(geometry["case"], raw_case);
    assert!(geometry["layout"]["semanticNodes"].is_array());
    assert_eq!(geometry["nativeReadiness"]["status"], "pending");
    assert_eq!(geometry["windowMetrics"]["logicalSize"]["width"], 100);
    assert_eq!(geometry["windowMetrics"]["physicalSize"]["width"], 150);
    assert!(result["semanticSha256"]
        .as_str()
        .is_some_and(|hash| hash.len() == 64));
    assert!(root.join("editor-dpi-1.5.semantic.json").is_file());
    let text = read_json(&root.join("editor-dpi-1.5.text.json")).unwrap();
    assert_eq!(text["case"], raw_case);
    assert_eq!(text["coordinateSpace"], "logical");
    assert!(root.join("editor-dpi-1.5.text-painter.json").is_file());
}

#[test]
fn editor_explicit_state_replaces_authored_button_inputs() {
    use zircon_runtime::ui::v2::{UiV2SurfaceBuilder, UiZuiAssetLoader};
    use zircon_runtime_interface::ui::event_ui::UiTreeId;
    let document = UiZuiAssetLoader::load_zui_str(r#"
[asset]
kind = "view"
id = "res://review/button.zui"
version = 2
[root]
node = "button"
[nodes.button]
component = "Button"
control_id = "WorkbenchButton"
props = { text = "Action", button_interaction_state = "disabled", enter_pressed = true, disabled = true }
"#).unwrap();
    let mut surface =
        UiV2SurfaceBuilder::build_surface(UiTreeId::new("state-review"), &document).unwrap();
    let node_id = surface.tree.roots[0];
    super::state::apply(&mut surface, "default").unwrap();
    assert_eq!(
        surface
            .tree
            .node(node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes["button_interaction_state"]
            .as_str(),
        Some("disabled")
    );
    for state in ["hover", "pressed", "focused", "disabled", "selected"] {
        super::state::apply(&mut surface, state).unwrap();
        let attributes = &surface
            .tree
            .node(node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes;
        assert_eq!(
            attributes["button_interaction_state"].as_str(),
            Some("normal")
        );
        assert_eq!(attributes["enter_pressed"].as_bool(), Some(false));
        assert_eq!(attributes["disabled"].as_bool(), Some(state == "disabled"));
        assert_eq!(attributes["hovered"].as_bool(), Some(state == "hover"));
        assert_eq!(
            attributes["focus_visible"].as_bool(),
            Some(state == "focused")
        );
        assert_eq!(attributes["selected"].as_bool(), Some(state == "selected"));
        assert_eq!(attributes["pressed"].as_bool(), Some(state == "pressed"));
        assert_eq!(
            surface.tree.node(node_id).unwrap().state_flags.pressed,
            state == "pressed"
        );
    }
}

#[test]
fn editor_tab_slot_host_resolves_fingerprinted_product_dependencies() {
    let repo = repo_root();
    let output = repo.join("docs/layout");
    let catalog: Catalog =
        serde_json::from_value(read_json(&output.join("catalog.json")).unwrap()).unwrap();
    let entry = catalog
        .entries
        .iter()
        .find(|entry| entry.source_path.ends_with("/workbench_tab_strip.zui"))
        .unwrap();
    let case: ReviewCase = serde_json::from_value(entry.cases[0].clone()).unwrap();
    entry.verify(&repo, &output).unwrap();
    case.validate(&entry.source_path).unwrap();
    assert!(super::contract::verify_review_host(&output, entry, &case)
        .unwrap()
        .is_some());
    let mut runtime = crate::ui::template_runtime::EditorUiHostRuntime::default();
    super::source::register(&mut runtime, &repo, entry, &case, &output).unwrap();
    let projection = runtime.project_document("layout-review").unwrap();
    let mut surface = runtime.build_shared_surface("layout-review").unwrap();
    surface
        .compute_layout(zircon_runtime_interface::ui::layout::UiSize::new(
            360.0, 520.0,
        ))
        .unwrap();
    let model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();
    for text in ["overview", "details", "stats"] {
        assert!(model.nodes.iter().any(|node| node
            .attributes
            .get("text")
            .and_then(toml::Value::as_str)
            == Some(text)));
    }
    let mut altered: ReviewCase = serde_json::from_value(entry.cases[0].clone()).unwrap();
    altered.review_host.as_mut().unwrap().sha256 = "0".repeat(64);
    assert!(
        super::contract::verify_review_host(&output, entry, &altered)
            .unwrap_err()
            .contains("SHA256")
    );
    altered.host = "editor".into();
    assert!(
        super::contract::verify_review_host(&output, entry, &altered)
            .unwrap_err()
            .contains("must belong")
    );
}

#[test]
fn editor_tab_strip_review_targets_one_tab_without_losing_authored_selection() {
    let repo = repo_root();
    let output = repo.join("docs/layout");
    let catalog: Catalog =
        serde_json::from_value(read_json(&output.join("catalog.json")).unwrap()).unwrap();
    let entry = catalog
        .entries
        .iter()
        .find(|entry| entry.source_path.ends_with("/workbench_tab_strip.zui"))
        .unwrap();
    let case: ReviewCase = serde_json::from_value(entry.cases[0].clone()).unwrap();
    let mut runtime = crate::ui::template_runtime::EditorUiHostRuntime::default();
    super::source::register(&mut runtime, &repo, entry, &case, &output).unwrap();

    for (state, expected_selection) in [("hover", "overview"), ("selected", "details")] {
        let mut surface = runtime.build_shared_surface("layout-review").unwrap();
        super::state::apply(&mut surface, state).unwrap();
        let mut tabs = surface
            .tree
            .nodes
            .iter()
            .filter_map(|(node_id, node)| {
                let metadata = node.template_metadata.as_ref()?;
                let component_state = surface.component_state(*node_id);
                (metadata.component == "Tab").then(|| {
                    (
                        metadata
                            .attributes
                            .get("text")
                            .and_then(toml::Value::as_str)
                            .unwrap(),
                        component_state.map(|state| state.flags.selected),
                        component_state.map(|state| state.flags.hovered),
                        metadata
                            .attributes
                            .get("selected")
                            .and_then(toml::Value::as_bool),
                        metadata
                            .attributes
                            .get("hovered")
                            .and_then(toml::Value::as_bool),
                    )
                })
            })
            .collect::<Vec<_>>();
        tabs.sort_by_key(|(text, _, _, _, _)| *text);
        assert_eq!(tabs.len(), 3, "review host must mount all three slot tabs");
        for (text, selected, hovered, selected_attribute, hovered_attribute) in tabs {
            assert_eq!(
                selected,
                Some(text == expected_selection),
                "{state}: {text}"
            );
            assert_eq!(
                hovered,
                Some(state == "hover" && text == "details"),
                "{state}: {text}"
            );
            assert_eq!(
                selected_attribute == Some(true),
                selected == Some(true),
                "{state}: {text} metadata selected must expose only active canonical state"
            );
            assert_eq!(
                hovered_attribute == Some(true),
                hovered == Some(true),
                "{state}: {text} metadata hovered must expose only active canonical state"
            );
        }
        let group = surface
            .tree
            .nodes
            .values()
            .filter_map(|node| node.template_metadata.as_ref())
            .find(|metadata| metadata.component == "Tabs")
            .expect("review host must mount a Tabs group");
        assert_eq!(
            group.attributes.get("value").and_then(toml::Value::as_str),
            Some(expected_selection),
        );
        assert_eq!(
            group
                .attributes
                .get("selected_index")
                .and_then(toml::Value::as_float),
            Some(if state == "selected" { 1.0 } else { 0.0 }),
        );
    }
}
