use super::support::*;

#[test]
fn asset_creation_menu_reuses_generation_and_publishes_collision_safe_o1_actions() {
    let shell_size = UiSize::new(900.0, 620.0);
    let create_layout = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let create_widget = EditorOperationPath::parse("ui_asset.widget.create").unwrap();
    let layout_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let widget_type = AssetTypeId::from_resource_kind(ResourceKind::UiWidget);
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    registry
        .apply_contribution(
            "test.ui.creation",
            AssetTypeContribution::augment(layout_type.clone()).with_creation_template(
                AssetCreationTemplateDescriptor::new(
                    "ui_asset.layout",
                    "UI|\nAsset",
                    create_layout,
                ),
            ),
        )
        .unwrap();
    registry
        .apply_contribution(
            "test.ui.creation",
            AssetTypeContribution::augment(widget_type.clone()).with_creation_template(
                AssetCreationTemplateDescriptor::new("ui_asset.widget", "UI Asset", create_widget),
            ),
        )
        .unwrap();

    let generation = registry.creation_menu_generation();
    assert!(Arc::ptr_eq(
        &generation,
        &registry.creation_menu_generation()
    ));
    assert_eq!(generation.entries().len(), 2);
    assert_ne!(
        generation.entries()[0].action_id(),
        generation.entries()[1].action_id()
    );
    assert_ne!(
        generation.entries()[0].raw_item(),
        generation.entries()[1].raw_item()
    );
    for entry in generation.entries() {
        assert_eq!(generation.action(entry.action_id()), Some(entry));
        assert_eq!(entry.raw_item().matches('|').count(), 1);
    }

    let mut chrome = default_preview_fixture().build_chrome();
    chrome.asset_browser.creation_menu = Arc::clone(&generation);
    chrome.asset_browser.selected_folder_id = Some("res://ui".to_string());
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("componentized workbench template should project");
    for _ in 0..100 {
        bridge
            .recompute_layout_with_workbench_model(
                shell_size,
                &model,
                &WorkbenchChromeMetrics::default(),
            )
            .unwrap();
    }
    assert_eq!(bridge.asset_creation_menu_publish_count(), 1);
    assert!(
        bridge
            .control_frame("WorkbenchToolbarMainMenu")
            .expect("main menu should expose its content-measured frame")
            .width
            > 190.0,
        "the authored width must grow for Command Palette, its shortcut, and its icon"
    );

    for entry in generation.entries() {
        let request = bridge
            .asset_creation_menu_request(
                &chrome.asset_browser,
                "WorkbenchToolbarMainMenu",
                entry.action_id(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(request.asset_type(), entry.asset_type());
        assert_eq!(request.template_id(), entry.template_id());
    }
}

#[test]
fn main_menu_shortcuts_follow_effective_keymap_without_rebuilding_stable_generations() {
    let shell_size = UiSize::new(900.0, 620.0);
    let chrome = default_preview_fixture().build_chrome();
    let registry = EditorCommandRegistry::default_workbench();
    let context = CommandEvalCtx::interactive().with_project_open(true);
    let contributions = ContributionSnapshot::default();
    let capabilities = CapabilitySet::default();
    let i18n = crate::core::i18n::EditorI18nService::default();
    let locale = i18n.active_locale();
    let overridden = EditorKeymap::default_workbench().with_overrides(&EditorKeymapOverrides::new(
        BTreeMap::from([
            (
                EditorOperationPath::parse("file.project.open").unwrap(),
                Some("Alt+O".parse::<EditorKeyChord>().unwrap()),
            ),
            (
                EditorOperationPath::parse("file.project.save").unwrap(),
                None,
            ),
        ]),
    ));
    let overridden_model = WorkbenchViewModel::build_with_contributions_and_context(
        &registry,
        &overridden,
        &i18n,
        &locale,
        &chrome,
        &contributions,
        &capabilities,
        None,
        &context,
    );
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("componentized workbench template should project");

    for _ in 0..10 {
        bridge
            .recompute_layout_with_workbench_model(
                shell_size,
                &overridden_model,
                &WorkbenchChromeMetrics::default(),
            )
            .unwrap();
    }
    let overridden_items = control_string_array(&bridge, "WorkbenchToolbarMainMenu", "menu_items");
    assert!(overridden_items
        .contains(&"Open Project|action=menu.item.open_project,icon=folder|Alt+O".to_string()));
    assert!(overridden_items
        .contains(&"Save Project|action=menu.item.save_project,icon=save".to_string()));
    assert_eq!(bridge.asset_creation_menu_publish_count(), 1);

    let default_model = WorkbenchViewModel::build_with_contributions_and_context(
        &registry,
        &EditorKeymap::default_workbench(),
        &i18n,
        &locale,
        &chrome,
        &contributions,
        &capabilities,
        None,
        &context,
    );
    bridge
        .recompute_layout_with_workbench_model(
            shell_size,
            &default_model,
            &WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let default_items = control_string_array(&bridge, "WorkbenchToolbarMainMenu", "menu_items");
    assert!(default_items
        .contains(&"Open Project|action=menu.item.open_project,icon=folder".to_string()));
    assert!(default_items
        .contains(&"Open Scene...|action=menu.item.open_scene,icon=folder|Ctrl+O".to_string()));
    assert!(default_items
        .contains(&"Save Project|action=menu.item.save_project,icon=save|Ctrl+S".to_string()));
    assert_eq!(bridge.asset_creation_menu_publish_count(), 2);
}

#[test]
fn asset_creation_menu_keeps_a_compiled_generation_across_scale_matrix() {
    for (template_count, resize_count, action_dispatch_count) in
        [(1, 1, 1), (100, 100, 100), (10_000, 1_000, 1_000_000)]
    {
        assert_asset_creation_menu_scale(template_count, resize_count, action_dispatch_count);
    }
}

fn assert_asset_creation_menu_scale(
    template_count: usize,
    resize_count: usize,
    action_dispatch_count: usize,
) {
    let shell_size = UiSize::new(900.0, 620.0);
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let create = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let mut contribution = AssetTypeContribution::augment(asset_type.clone());
    for ordinal in 0..template_count {
        contribution = contribution.with_creation_template(AssetCreationTemplateDescriptor::new(
            display_colliding_template_id(ordinal),
            "Scale Asset",
            create.clone(),
        ));
    }

    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    registry
        .apply_contribution("test.asset.creation.scale", contribution)
        .unwrap();
    let generation = registry.creation_menu_generation();
    assert_eq!(generation.entries().len(), template_count);
    assert_eq!(
        generation
            .entries()
            .iter()
            .map(|entry| {
                entry
                    .raw_item()
                    .split_once('|')
                    .expect("compiled menu item should separate its visible label and action")
                    .0
            })
            .collect::<BTreeSet<_>>()
            .len(),
        template_count,
        "display-equivalent template identifiers must still receive unique menu labels"
    );
    assert!(Arc::ptr_eq(
        &generation,
        &registry.creation_menu_generation()
    ));

    let mut chrome = default_preview_fixture().build_chrome();
    chrome.asset_browser.creation_menu = Arc::clone(&generation);
    chrome.asset_browser.selected_folder_id = Some("res://scale".to_string());
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("componentized workbench template should project");
    let menu_node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("WorkbenchToolbarMainMenu")
        })
        .expect("main menu should retain its compiled control node")
        .node_id;
    let resize_sizes = [
        UiSize::new(1_024.0, 768.0),
        UiSize::new(640.0, 480.0),
        shell_size,
    ];
    for ordinal in 0..resize_count {
        let resized_shell_size = resize_sizes[ordinal % resize_sizes.len()];
        bridge
            .recompute_layout_with_workbench_model(
                resized_shell_size,
                &model,
                &WorkbenchChromeMetrics::default(),
            )
            .unwrap();
        if ordinal == 0 {
            bridge
                .dispatch_control_state("WorkbenchToolbarMenu", UiEventKind::Click)
                .expect("main menu should open for the resize contract")
                .expect("main menu should expose a click binding");
        }
        let menu_frame = bridge
            .control_frame("WorkbenchToolbarMainMenu")
            .expect("compiled menu should retain its control frame after resizing");
        assert!(menu_frame.width > 0.0 && menu_frame.width <= resized_shell_size.width);
        assert!(menu_frame.height > 0.0 && menu_frame.height <= resized_shell_size.height);
        let toolbar_bottom = bridge
            .control_frame("WorkbenchWindowTopToolbarRegion")
            .expect("toolbar should retain the menu anchor")
            .bottom();
        let expected_height =
            crate::ui::retained_host::menu_popup_contract::menu_popup_content_height(
                generation.entries().len() + 8,
            )
            .min((resized_shell_size.height - toolbar_bottom).max(1.0))
            .max(1.0);
        let menu_node = bridge
            .surface()
            .tree
            .node(menu_node_id)
            .expect("resize should preserve the compiled menu node");
        let width = menu_node.constraints.width;
        assert!(width.preferred > 0.0 && width.preferred <= resized_shell_size.width);
        assert_eq!(width.min, width.preferred);
        assert_eq!(width.max, width.preferred);
        let height = menu_node.constraints.height;
        assert_near("compiled menu height", height.preferred, expected_height);
        assert_eq!(height.min, height.preferred);
        assert_eq!(height.max, height.preferred);
    }
    assert_eq!(bridge.asset_creation_menu_publish_count(), 1);

    let entry = &generation.entries()[template_count / 2];
    let action_id = entry.action_id().to_owned();
    for _ in 0..action_dispatch_count {
        assert!(bridge.is_asset_creation_menu_action("WorkbenchToolbarMainMenu", &action_id));
        let request = bridge
            .asset_creation_menu_request(
                &chrome.asset_browser,
                "WorkbenchToolbarMainMenu",
                &action_id,
            )
            .unwrap()
            .unwrap();
        assert_eq!(request.asset_type(), entry.asset_type());
        assert_eq!(request.template_id(), entry.template_id());
        assert_eq!(request.target_folder(), "res://scale");
    }
    assert_eq!(bridge.asset_creation_menu_publish_count(), 1);
}

fn display_colliding_template_id(mut ordinal: usize) -> String {
    let mut id = String::from("test.scale");
    loop {
        id.push(char::from_u32((ordinal % 31 + 1) as u32).unwrap());
        ordinal /= 31;
        if ordinal == 0 {
            return id;
        }
    }
}

#[test]
fn enabled_asset_type_materialization_commits_one_generation_batch() {
    let source = include_str!("../../../../../ui/host/editor_extension_registration.rs");
    let implementation = source
        .split("pub(crate) fn materialize_enabled_asset_types(")
        .nth(1)
        .unwrap()
        .split("pub(crate) fn enabled_asset_types_for_shell(")
        .next()
        .unwrap();

    assert!(implementation.contains("asset_types.apply_contributions("));
    assert!(!implementation.contains("asset_types.apply_contribution("));
}

#[test]
fn workbench_main_menu_asset_creation_invokes_registered_operation() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_main_menu_asset_create");
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let create = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(create.clone())
                .with_event(EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser)),
        )
        .unwrap();
    extension
        .register_asset_type_contribution(
            AssetTypeContribution::augment(asset_type.clone()).with_creation_template(
                AssetCreationTemplateDescriptor::new(
                    "ui_asset.layout",
                    "UI Layout",
                    create.clone(),
                ),
            ),
        )
        .unwrap();
    harness
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();

    let shell_size = UiSize::new(900.0, 620.0);
    let chrome = harness.runtime.chrome_snapshot();
    let action_id = chrome
        .asset_browser
        .creation_menu
        .entries()
        .iter()
        .find(|entry| entry.template_id() == "ui_asset.layout")
        .expect("registered template should publish a compiled menu action")
        .action_id()
        .to_owned();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("componentized workbench template should project");
    bridge
        .recompute_layout_with_workbench_model(
            shell_size,
            &model,
            &WorkbenchChromeMetrics::default(),
        )
        .expect("registered asset creation template should project");
    bridge
        .dispatch_control_state("WorkbenchToolbarMenu", UiEventKind::Click)
        .expect("main menu should dispatch")
        .expect("main menu should expose a click binding");
    assert!(bridge.is_asset_creation_menu_action("WorkbenchToolbarMainMenu", &action_id));

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchToolbarMainMenu",
        &action_id,
    )
    .expect("asset creation menu item should be handled")
    .expect("asset creation menu item should dispatch");

    let record = harness.runtime.journal().records().last().unwrap().clone();
    assert_eq!(record.operation_id.as_deref(), Some(create.as_str()));
    assert_eq!(
        record
            .operation_arguments
            .as_ref()
            .and_then(|arguments| arguments.get("asset_type"))
            .and_then(serde_json::Value::as_str),
        Some(asset_type.as_str())
    );
    assert_eq!(
        record
            .operation_arguments
            .as_ref()
            .and_then(|arguments| arguments.get("template_id"))
            .and_then(serde_json::Value::as_str),
        Some("ui_asset.layout")
    );
    assert_eq!(
        record
            .operation_arguments
            .as_ref()
            .and_then(|arguments| arguments.get("target_folder"))
            .and_then(serde_json::Value::as_str),
        Some("res://")
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}
