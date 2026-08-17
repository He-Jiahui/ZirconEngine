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
fn asset_creation_menu_keeps_a_compiled_generation_across_scale_matrix() {
    const TEMPLATE_COUNT: usize = 10_000;
    const RESIZE_COUNT: usize = 1_000;
    const ACTION_LOOKUP_COUNT: usize = 1_000_000;

    let shell_size = UiSize::new(900.0, 620.0);
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let create = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let mut contribution = AssetTypeContribution::augment(asset_type.clone());
    for ordinal in 0..TEMPLATE_COUNT {
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
    assert_eq!(generation.entries().len(), TEMPLATE_COUNT);
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
        TEMPLATE_COUNT,
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
    for _ in 0..RESIZE_COUNT {
        bridge
            .recompute_layout_with_workbench_model(
                shell_size,
                &model,
                &WorkbenchChromeMetrics::default(),
            )
            .unwrap();
    }
    assert_eq!(bridge.asset_creation_menu_publish_count(), 1);

    let entry = &generation.entries()[TEMPLATE_COUNT / 2];
    let action_id = entry.action_id().to_owned();
    for _ in 0..ACTION_LOOKUP_COUNT {
        assert!(bridge.is_asset_creation_menu_action("WorkbenchToolbarMainMenu", &action_id));
    }
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
            EditorCommandDescriptor::operation(create.clone(), "Create UI Layout")
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

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchToolbarMainMenu",
        "menu.item.create_u_i_layout",
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
