#[test]
fn canonical_case_hash_normalizes_object_key_order() {
    let a = serde_json::json!({"b": 2, "a": 1});
    let b = serde_json::json!({"a": 1, "b": 2});
    assert_eq!(
        super::catalog::canonical_hash(&a).unwrap(),
        super::catalog::canonical_hash(&b).unwrap()
    );
}

#[test]
fn explicit_review_state_resets_scalar_and_keyboard_state() {
    use zircon_runtime::ui::v2::{UiV2SurfaceBuilder, UiZuiAssetLoader};
    use zircon_runtime_interface::ui::event_ui::UiTreeId;
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "view"
id = "res://review/button.zui"
version = 2
[root]
node = "button"
[nodes.button]
component = "Button"
props = { text = "Action", button_interaction_state = "disabled", enter_pressed = true }
"#,
    )
    .unwrap();
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
    super::state::apply(&mut surface, "hover").unwrap();
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
    assert_eq!(attributes["hovered"].as_bool(), Some(true));
    assert_eq!(attributes["disabled"].as_bool(), Some(false));
}

#[test]
fn validation_accepts_fractional_dpi_and_rejects_invalid_physical_extents() {
    let value = serde_json::json!({"id":"x","sourcePath":"x.zui","host":"fixture","viewport":{"width":1,"height":1},"dpi":1.5,"locale":"en-US","state":"default","data":{}});
    let mut case: super::catalog::Case = serde_json::from_value(value).unwrap();
    let entry: super::catalog::Entry = serde_json::from_value(serde_json::json!({"sourcePath":"x.zui","sourceSha256":"","outputPath":"x","outputSha256":"","category":"x","name":"x"})).unwrap();
    assert!(super::catalog::validate_case(&case, &entry).is_ok());
    let metrics = super::catalog::window_metrics(&case).unwrap();
    assert_eq!(metrics.physical_size.width, 2);
    for dpi in [0.0, -1.0, f64::NAN, f64::INFINITY, 10000.0] {
        case.dpi = dpi;
        assert!(super::catalog::validate_case(&case, &entry).is_err());
    }
}

#[test]
fn canonical_numbers_match_javascript_case_inputs() {
    let value: serde_json::Value = serde_json::from_str(
        r#"{"dpi":1.0,"viewport":{"width":360,"height":520},"nested":[1.5,0.0,-0.0]}"#,
    )
    .unwrap();
    assert_eq!(
        super::catalog::canonical_json(&value).unwrap(),
        r#"{"dpi":1,"nested":[1.5,0,0],"viewport":{"height":520,"width":360}}"#
    );
}

#[test]
fn native_case_requires_fingerprinted_host_theme() {
    let case: super::catalog::Case = serde_json::from_value(serde_json::json!({
        "id":"themed", "sourcePath":"one.zui", "host":"component",
        "themeSourcePath":"zircon_editor/assets/ui/editor/theme/editor_tokens.zui",
        "viewport":{"width":360,"height":520}, "dpi":1,
        "locale":"en-US", "state":"default", "data":{}
    }))
    .unwrap();
    let mut entry: super::catalog::Entry = serde_json::from_value(serde_json::json!({
        "sourcePath":"one.zui", "sourceSha256":"", "outputPath":"one",
        "outputSha256":"", "category":"components", "name":"one"
    }))
    .unwrap();
    assert!(super::catalog::validate_case(&case, &entry)
        .unwrap_err()
        .contains("not a fingerprinted dependency"));
    entry
        .dependency_fingerprints
        .push(super::catalog::Dependency {
            source_path: case.theme_source_path.clone().unwrap(),
            sha256: "reviewed-theme".into(),
        });
    assert!(super::catalog::validate_case(&case, &entry).is_ok());
}

#[test]
fn native_component_host_loads_actual_editor_theme() {
    let repo = super::batch::repo_root();
    let theme = repo.join("zircon_editor/assets/ui/editor/theme/editor_tokens.zui");
    let source = repo.join("zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_checkbox.zui");
    let metrics = zircon_runtime_interface::ui::window::UiWindowMetrics::new(
        zircon_runtime_interface::ui::layout::UiSize::new(360.0, 520.0),
        zircon_runtime_interface::ui::window::UiWindowPixelSize::new(360, 520),
        1.0,
    );
    let preview = super::preview::build(
        &source,
        &[],
        "themed-checkbox",
        metrics,
        "default",
        Some(&theme),
    )
    .unwrap();
    assert!(preview
        .extract
        .list
        .commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Checkbox")));
    assert!(super::preview::build(
        &source,
        &[],
        "invalid-theme",
        metrics,
        "default",
        Some(&source)
    )
    .err()
    .unwrap()
    .contains("style or theme_tokens"));
}

#[test]
fn native_tab_slot_host_loads_fingerprinted_product_components() {
    let repo = super::batch::repo_root();
    let catalog: super::catalog::Catalog = serde_json::from_str(
        &std::fs::read_to_string(repo.join("docs/layout/catalog.json")).unwrap(),
    )
    .unwrap();
    let entry = catalog
        .entries
        .iter()
        .find(|entry| entry.source_path.ends_with("/workbench_tab_strip.zui"))
        .unwrap();
    let case: super::catalog::Case = serde_json::from_value(entry.cases[0].clone()).unwrap();
    let (_, _, mut dependencies) =
        super::catalog::verify_entry(&repo, &repo.join("docs/layout"), entry).unwrap();
    super::catalog::validate_case(&case, entry).unwrap();
    let host = super::catalog::verify_review_host(&repo.join("docs/layout"), entry, &case)
        .unwrap()
        .unwrap();
    dependencies.push(repo.join(&entry.source_path));
    let preview = super::preview::build(
        &host,
        &dependencies,
        "tab-slot-host",
        super::catalog::window_metrics(&case).unwrap(),
        "default",
        case.theme_source_path
            .as_ref()
            .map(|path| repo.join(path))
            .as_deref(),
    )
    .unwrap();
    for label in ["overview", "details", "stats"] {
        assert!(preview
            .extract
            .list
            .commands
            .iter()
            .any(|command| command.text.as_deref() == Some(label)));
    }
    assert!(!preview
        .extract
        .list
        .commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Default")));
}

#[test]
fn output_paths_distinguish_sources_and_reject_traversal() {
    let value = serde_json::json!({"id":"default","sourcePath":"one.zui","host":"component","viewport":{"width":360,"height":520},"dpi":1,"locale":"en-US","state":"default","data":{}});
    let case: super::catalog::Case = serde_json::from_value(value).unwrap();
    let mut entry:super::catalog::Entry=serde_json::from_value(serde_json::json!({"sourcePath":"one.zui","sourceSha256":"","outputPath":"one","outputSha256":"","category":"components","name":"one"})).unwrap();
    let first = super::catalog::case_directory(&entry, &case).unwrap();
    entry.name = "two".into();
    assert_ne!(
        first,
        super::catalog::case_directory(&entry, &case).unwrap()
    );
    entry.name = "../escape".into();
    assert!(super::catalog::case_directory(&entry, &case).is_err());
    for path in ["../x", "x/../y", "C:/x", "x\\y", "/absolute"] {
        assert!(super::catalog::relative_path(path).is_err());
    }
}

#[test]
fn native_component_mount_applies_declared_default_parameters() {
    use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2PrototypeStore, UiZuiAssetLoader};
    let source = r#"
[asset]
kind="component"
id="res://ui/caption.zui"
version=2
[components.Caption]
root="label"
[components.Caption.params.caption]
type="string"
default="Declared caption"
[nodes.label]
component="Text"
props={text="$param.caption"}
"#;
    let document = UiZuiAssetLoader::load_zui_str(source).unwrap();
    let preview = super::preview::mount_component(&document).unwrap();
    let mut store = UiV2PrototypeStore::new();
    store.insert(document);
    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&preview, &store).unwrap();
    assert!(compiled.arena.nodes.iter().any(|node| {
        node.props.get("text").and_then(toml::Value::as_str) == Some("Declared caption")
    }));
    assert!(compiled.arena.root.is_some());
}

#[test]
fn style_assets_require_real_consumer_specimens() {
    let source = "[asset]\nkind='style'\nid='res://ui/theme.zui'\nversion=2\n";
    let document = zircon_runtime::ui::v2::UiZuiAssetLoader::load_zui_str(source).unwrap();
    assert!(super::preview::mount_component(&document)
        .unwrap_err()
        .contains("consumer specimen"));
}

#[test]
fn native_case_refuses_unapplied_data_and_interaction_state() {
    let entry:super::catalog::Entry=serde_json::from_value(serde_json::json!({"sourcePath":"one.zui","sourceSha256":"","outputPath":"one","outputSha256":"","category":"components","name":"one"})).unwrap();
    for (state, data) in [
        ("unsupported-interaction", serde_json::json!({})),
        ("default", serde_json::json!({"rows":[1]})),
    ] {
        let case:super::catalog::Case=serde_json::from_value(serde_json::json!({"id":"default","sourcePath":"one.zui","host":"component","viewport":{"width":360,"height":520},"dpi":1,"locale":"en-US","state":state,"data":data})).unwrap();
        assert!(super::catalog::validate_case(&case, &entry).is_err());
    }
    for state in [
        "open",
        "closed",
        "dragging",
        "drop-allowed",
        "drop-blocked",
        "empty",
        "running",
        "complete",
        "blocked",
        "success",
        "failure",
        "pending",
        "approved",
        "denied",
        "normal",
        "warning",
        "exceeded",
        "error",
        "long-en",
        "long-zh",
        "scroll-before",
        "scroll-after",
    ] {
        let case: super::catalog::Case = serde_json::from_value(serde_json::json!({"id":"state","sourcePath":"one.zui","host":"fixture","viewport":{"width":360,"height":520},"dpi":1,"locale":"en-US","state":state,"data":{}})).unwrap();
        assert!(
            super::catalog::validate_case(&case, &entry).is_ok(),
            "{state} should be a supported native review state"
        );
    }
}

#[test]
fn reactbits_chat_review_states_keep_long_text_and_transient_flags_design_only() {
    use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
    use zircon_runtime_interface::ui::event_ui::UiTreeId;

    const SOURCE: &str = include_str!("../fixtures/ui/reactbits_agent_native_components.zui");
    let document = UiZuiAssetLoader::load_zui_str(SOURCE).unwrap();
    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-chat-state-contract"),
        &document,
        &compiled,
    )
    .unwrap();

    let chat = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .is_some_and(|metadata| metadata.component == "AgentChat")
        })
        .map(|node| node.node_id)
        .expect("AgentChat node");
    let composer = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .is_some_and(|metadata| metadata.component == "ChatComposer")
        })
        .map(|node| node.node_id)
        .expect("ChatComposer node");

    super::state::apply(&mut surface, "long-zh").unwrap();
    let chat_attributes = &surface
        .tree
        .node(chat)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes;
    assert_eq!(
        chat_attributes
            .get("messages")
            .and_then(toml::Value::as_array)
            .and_then(|messages| messages.first())
            .and_then(toml::Value::as_str),
        Some("user|请在窄屏断点检查响应式外壳，并保持完整上下文可见。")
    );
    let composer_attributes = &surface
        .tree
        .node(composer)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes;
    assert!(composer_attributes
        .get("composer_text")
        .and_then(toml::Value::as_str)
        .is_some_and(|text| text.contains("请继续检查完整外壳")));

    super::state::apply(&mut surface, "empty").unwrap();
    assert_eq!(
        surface
            .tree
            .node(chat)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("messages")
            .and_then(toml::Value::as_array)
            .map(Vec::len),
        Some(0)
    );
    assert_eq!(
        surface
            .tree
            .node(composer)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("composer_text")
            .and_then(toml::Value::as_str),
        Some("")
    );

    super::state::apply(&mut surface, "error").unwrap();
    assert_eq!(
        surface
            .tree
            .node(chat)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("error")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}

fn pause_menu_surface(
    dpi: f64,
    state: &str,
) -> (
    zircon_runtime::ui::surface::UiSurface,
    zircon_runtime_interface::ui::surface::UiRenderExtract,
) {
    let case: super::catalog::Case = serde_json::from_value(serde_json::json!({
        "id":"pause", "sourcePath":"zircon_runtime/assets/ui/runtime/fixtures/pause_menu.zui",
        "host":"fixture", "viewport":{"width":900,"height":620}, "dpi":dpi,
        "locale":"en-US", "state":state, "data":{}
    }))
    .unwrap();
    let preview = super::preview::build(
        &super::batch::repo_root().join(&case.source_path),
        &[],
        "native-pause-test",
        super::catalog::window_metrics(&case).unwrap(),
        state,
        None,
    )
    .unwrap();
    (preview.surface, preview.extract)
}

#[test]
fn native_default_fixture_uses_file_loader_compiler_and_logical_layout() {
    let (surface, extract) = pause_menu_surface(1.0, "default");
    assert_eq!(extract, surface.render_extract);
    let button = surface.unique_control_node_id("ResumeButton").unwrap();
    assert!(surface.arranged_node(button).is_some());
    assert!(extract
        .list
        .commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Resume Mission")));
    assert_eq!(
        surface.window_state.metrics.unwrap().physical_size.width,
        900
    );
}

#[test]
fn native_fractional_dpi_preserves_logical_layout_and_rebuilds_physical_text() {
    let (normal, normal_extract) = pause_menu_surface(1.0, "default");
    let (scaled, physical) = pause_menu_surface(1.5, "default");
    assert_eq!(normal.arranged_tree, scaled.arranged_tree);
    assert_eq!(
        scaled.window_state.metrics.unwrap().physical_size.width,
        1350
    );
    assert_eq!(
        scaled.window_state.metrics.unwrap().physical_size.height,
        930
    );
    assert_eq!(scaled.render_extract.raster_scale, 1.5);
    assert_eq!(physical.raster_scale, 1.0);
    assert_eq!(
        normal_extract.list.commands.len(),
        physical.list.commands.len()
    );
    let mut text_count = 0;
    for (logical, projected) in normal_extract
        .list
        .commands
        .iter()
        .zip(&physical.list.commands)
    {
        assert_eq!(projected.frame.x, logical.frame.x * 1.5);
        assert_eq!(projected.frame.y, logical.frame.y * 1.5);
        assert_eq!(projected.frame.width, logical.frame.width * 1.5);
        assert_eq!(projected.frame.height, logical.frame.height * 1.5);
        assert_eq!(projected.style.font_size, logical.style.font_size * 1.5);
        if let Some(layout) = &projected.text_layout {
            text_count += 1;
            assert!(
                layout.rich_text_artifact.is_some(),
                "physical text must retain actual shaped glyphs"
            );
            assert_eq!(layout.font_size, projected.style.font_size);
        }
    }
    assert!(text_count > 0);
}

#[test]
fn native_default_preserves_initial_flags_and_explicit_state_replaces_them() {
    use zircon_runtime::ui::surface::UiPropertyMutationRequest;
    use zircon_runtime_interface::ui::component::UiValue;
    let (mut surface, _) = pause_menu_surface(1.0, "default");
    let button = surface.unique_control_node_id("ResumeButton").unwrap();
    surface
        .mutate_property(UiPropertyMutationRequest::new(
            button,
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap();
    surface
        .mutate_property(UiPropertyMutationRequest::new(
            button,
            "disabled",
            UiValue::Bool(true),
        ))
        .unwrap();
    surface
        .mutate_property(UiPropertyMutationRequest::new(
            button,
            "checked",
            UiValue::Bool(true),
        ))
        .unwrap();
    let initial = surface.component_state(button).cloned();
    super::state::apply(&mut surface, "default").unwrap();
    assert_eq!(surface.component_state(button), initial.as_ref());
    assert!(!surface.tree.nodes.get(&button).unwrap().state_flags.enabled);
    for state in ["hover", "pressed", "focused", "disabled", "selected"] {
        super::state::apply(&mut surface, state).unwrap();
        assert!(surface.tree.nodes.get(&button).unwrap().state_flags.enabled);
        let flags = &surface.component_state(button).unwrap().flags;
        assert_eq!(flags.hovered, state == "hover");
        assert_eq!(flags.pressed, state == "pressed");
        assert_eq!(flags.focused, state == "focused");
        assert_eq!(flags.focus_visible, state == "focused");
        assert_eq!(flags.disabled, state == "disabled");
        assert_eq!(flags.selected, state == "selected");
        assert!(!flags.checked);
    }
    // Dynamic states are accepted by the catalog contract; applying one to a
    // surface without its matching native painter remains an explicit error.
    assert!(super::state::apply(&mut surface, "dragging").is_err());
}

#[test]
fn runtime_batch_delegates_product_surfaces_to_retained_host() {
    let editor: super::catalog::Entry = serde_json::from_value(serde_json::json!({
        "sourcePath":"zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui",
        "sourceSha256":"", "outputPath":"editor/button", "outputSha256":"",
        "category":"editor-ui", "name":"button", "cases":[]
    })).unwrap();
    let editor_case: super::catalog::Case = serde_json::from_value(serde_json::json!({
        "id":"default", "sourcePath":editor.source_path, "host":"component",
        "viewport":{"width":360,"height":520}, "dpi":1, "locale":"en-US",
        "state":"default", "data":{}
    }))
    .unwrap();
    assert!(super::catalog::requires_editor_renderer(
        &editor,
        &editor_case
    ));

    let fixture: super::catalog::Entry = serde_json::from_value(serde_json::json!({
        "sourcePath":"zircon_runtime/tests/fixtures/ui/reactbits_agent_workspace.zui",
        "sourceSha256":"", "outputPath":"runtime/workspace", "outputSha256":"",
        "category":"runtime-fixtures", "name":"workspace", "cases":[]
    }))
    .unwrap();
    let fixture_case: super::catalog::Case = serde_json::from_value(serde_json::json!({
        "id":"default", "sourcePath":fixture.source_path, "host":"fixture",
        "viewport":{"width":1280,"height":800}, "dpi":1, "locale":"en-US",
        "state":"default", "data":{}
    }))
    .unwrap();
    assert!(!super::catalog::requires_editor_renderer(
        &fixture,
        &fixture_case
    ));
}

#[test]
fn dependency_proof_reads_files_and_rejects_stale_bytes() {
    use super::catalog::{canonical_hash, sha256, verify_entry, Entry};
    let directory = std::env::temp_dir().join(format!(
        "zui-native-provenance-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    std::fs::write(directory.join("source.zui"), b"source").unwrap();
    std::fs::write(directory.join("prepared.zui"), b"prepared").unwrap();
    std::fs::write(directory.join("import.zui"), b"original").unwrap();
    let dependencies =
        serde_json::json!([{"sourcePath":"import.zui","sha256":sha256(b"original")}]);
    let entry: Entry = serde_json::from_value(serde_json::json!({
        "sourcePath":"source.zui","sourceSha256":sha256(b"source"),
        "outputPath":"prepared.zui","outputSha256":sha256(b"prepared"),
        "category":"fixtures","name":"proof","dependencyFingerprints":dependencies,
        "dependencySha256":canonical_hash(&dependencies).unwrap()
    }))
    .unwrap();
    assert!(verify_entry(&directory, &directory, &entry).is_ok());
    std::fs::write(directory.join("import.zui"), b"changed").unwrap();
    assert!(verify_entry(&directory, &directory, &entry)
        .unwrap_err()
        .contains("dependency SHA256 differs"));
}
