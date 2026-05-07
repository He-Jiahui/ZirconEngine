use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::template::{UiAssetLoader, UiDocumentCompiler, UiTemplateSurfaceBuilder};
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::layout::UiSize;

#[derive(Debug, Default, PartialEq, Eq)]
struct RuntimeQuestLogSemanticSnapshot {
    component_counts: BTreeMap<String, usize>,
    control_ids: BTreeSet<String>,
    click_binding_ids: BTreeSet<String>,
    click_routes: BTreeSet<String>,
    rendered_text: BTreeSet<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RuntimeUiSemanticSnapshot {
    component_counts: BTreeMap<String, usize>,
    control_ids: BTreeSet<String>,
    click_binding_ids: BTreeSet<String>,
    click_routes: BTreeSet<String>,
    rendered_text: BTreeSet<String>,
    render_kind_counts: BTreeMap<String, usize>,
    text_payload_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct RuntimeUiSemanticGolden {
    name: &'static str,
    editor_asset: &'static str,
    runtime_asset: &'static str,
    required_editor_controls: &'static [&'static str],
    required_runtime_controls: &'static [&'static str],
    required_editor_text: &'static [&'static str],
    required_runtime_text: &'static [&'static str],
    minimum_editor_buttons: usize,
    minimum_runtime_buttons: usize,
    minimum_editor_text_commands: usize,
    minimum_runtime_text_commands: usize,
    minimum_runtime_quads: usize,
}

#[test]
fn quest_log_editor_and_runtime_assets_share_runtime_semantic_golden() {
    let editor_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = editor_root
        .parent()
        .expect("zircon_editor lives directly under workspace root");
    let runtime_root = workspace_root.join("zircon_runtime");

    let editor_surface = build_surface(
        &editor_root.join("assets/ui/runtime/quest_log_dialog.ui.toml"),
        Some(&editor_root.join("assets/ui/theme/editor_material.ui.toml")),
        "editor.runtime.quest_log_dialog",
    );
    let runtime_surface = build_surface(
        &runtime_root.join("assets/ui/runtime/fixtures/quest_log_dialog.ui.toml"),
        None,
        "runtime.ui.quest_log_dialog",
    );

    let editor_snapshot = semantic_snapshot(&editor_surface);
    let runtime_snapshot = semantic_snapshot(&runtime_surface);

    let expected_click_binding_ids =
        BTreeSet::from(["QuestLog/Close".to_string(), "QuestLog/Track".to_string()]);
    let expected_click_routes = BTreeSet::from([
        "RuntimeAction.CloseQuestLog".to_string(),
        "RuntimeAction.TrackQuest".to_string(),
    ]);
    for (label, snapshot) in [("editor", &editor_snapshot), ("runtime", &runtime_snapshot)] {
        assert_eq!(
            snapshot.component_counts.get("Button").copied(),
            Some(2),
            "{label} quest log surface should expose two runtime action buttons"
        );
        assert!(
            snapshot.rendered_text.contains("Quest Log"),
            "{label} quest log surface should render the dialog title"
        );
        assert!(
            snapshot.rendered_text.contains("Track") && snapshot.rendered_text.contains("Close"),
            "{label} quest log surface should render matching action labels"
        );
        assert_eq!(
            snapshot.click_binding_ids, expected_click_binding_ids,
            "{label} quest log surface should preserve shared Click binding ids"
        );
        assert_eq!(
            snapshot.click_routes, expected_click_routes,
            "{label} quest log surface should preserve shared runtime Click routes"
        );
    }

    assert!(
        editor_snapshot.control_ids.contains("QuestLogActions")
            && runtime_snapshot.control_ids.contains("QuestLogActions"),
        "editor and runtime quest log surfaces should share the action-row semantic control id"
    );
}

#[test]
fn all_runtime_fixture_pairs_share_template_semantic_golden() {
    let editor_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = editor_root
        .parent()
        .expect("zircon_editor lives directly under workspace root");
    let runtime_root = workspace_root.join("zircon_runtime");

    for golden in runtime_ui_semantic_goldens() {
        let editor_surface = build_surface(
            &editor_root.join(golden.editor_asset),
            Some(&editor_root.join("assets/ui/theme/editor_material.ui.toml")),
            &format!("editor.runtime.{}", golden.name),
        );
        let runtime_surface = build_surface(
            &runtime_root.join(golden.runtime_asset),
            None,
            &format!("runtime.ui.{}", golden.name),
        );

        let editor_snapshot = runtime_ui_semantic_snapshot(&editor_surface);
        let runtime_snapshot = runtime_ui_semantic_snapshot(&runtime_surface);

        assert_runtime_ui_snapshot("editor", golden, &editor_snapshot);
        assert_runtime_ui_snapshot("runtime", golden, &runtime_snapshot);
    }
}

fn runtime_ui_semantic_goldens() -> &'static [RuntimeUiSemanticGolden] {
    &[
        RuntimeUiSemanticGolden {
            name: "hud",
            editor_asset: "assets/ui/runtime/runtime_hud.ui.toml",
            runtime_asset: "assets/ui/runtime/fixtures/hud_overlay.ui.toml",
            required_editor_controls: &["RuntimeHudRoot", "HudStatusRow", "InventoryAction"],
            required_runtime_controls: &["HudRoot", "HealthPanel", "WeaponIcon", "AmmoPanel"],
            required_editor_text: &["Mission: Relay Tower", "Ammo 42 / 90", "Inventory"],
            required_runtime_text: &["HP 87 / 100", "Reach the relay before sunrise", "24 / 120"],
            minimum_editor_buttons: 2,
            minimum_runtime_buttons: 0,
            minimum_editor_text_commands: 4,
            minimum_runtime_text_commands: 3,
            minimum_runtime_quads: 2,
        },
        RuntimeUiSemanticGolden {
            name: "pause",
            editor_asset: "assets/ui/runtime/pause_dialog.ui.toml",
            runtime_asset: "assets/ui/runtime/fixtures/pause_menu.ui.toml",
            required_editor_controls: &["PauseDialogRoot", "PauseDialogCard", "ResumeAction"],
            required_runtime_controls: &[
                "PauseMenuRoot",
                "PauseDialog",
                "ResumeButton",
                "SettingsButton",
                "QuitButton",
            ],
            required_editor_text: &["Paused", "Resume", "Quit To Menu"],
            required_runtime_text: &["Paused", "Resume Mission", "Settings", "Return To Title"],
            minimum_editor_buttons: 2,
            minimum_runtime_buttons: 3,
            minimum_editor_text_commands: 4,
            minimum_runtime_text_commands: 5,
            minimum_runtime_quads: 4,
        },
        RuntimeUiSemanticGolden {
            name: "settings",
            editor_asset: "assets/ui/runtime/settings_dialog.ui.toml",
            runtime_asset: "assets/ui/runtime/fixtures/settings_dialog.ui.toml",
            required_editor_controls: &[
                "SettingsDialogRoot",
                "SettingsDialogPanel",
                "ApplySettings",
                "CloseSettings",
            ],
            required_runtime_controls: &[
                "SettingsRoot",
                "SettingsDialog",
                "SettingsNav",
                "ApplySettings",
                "CancelSettings",
            ],
            required_editor_text: &["Settings", "Graphics Quality", "Master Volume", "Apply"],
            required_runtime_text: &[
                "Graphics\nAudio\nGameplay",
                "Graphics Quality    Epic",
                "Apply",
            ],
            minimum_editor_buttons: 2,
            minimum_runtime_buttons: 4,
            minimum_editor_text_commands: 6,
            minimum_runtime_text_commands: 5,
            minimum_runtime_quads: 5,
        },
        RuntimeUiSemanticGolden {
            name: "inventory",
            editor_asset: "assets/ui/runtime/inventory_dialog.ui.toml",
            runtime_asset: "assets/ui/runtime/fixtures/inventory_list.ui.toml",
            required_editor_controls: &[
                "InventoryDialogRoot",
                "InventoryDialogPanel",
                "InventoryActions",
                "EquipItem",
            ],
            required_runtime_controls: &[
                "InventoryRoot",
                "InventoryPanel",
                "InventoryHeader",
                "InventoryList",
                "InventoryRow00",
                "InventoryRow11",
            ],
            required_editor_text: &["Inventory", "Weapons\nArmor\nConsumables", "Equip", "Close"],
            required_runtime_text: &[
                "Field Inventory",
                "01  Pulse Cells x24",
                "12  Shield Capacitor x5",
            ],
            minimum_editor_buttons: 2,
            minimum_runtime_buttons: 12,
            minimum_editor_text_commands: 5,
            minimum_runtime_text_commands: 13,
            minimum_runtime_quads: 14,
        },
        RuntimeUiSemanticGolden {
            name: "quest_log",
            editor_asset: "assets/ui/runtime/quest_log_dialog.ui.toml",
            runtime_asset: "assets/ui/runtime/fixtures/quest_log_dialog.ui.toml",
            required_editor_controls: &[
                "QuestLogDialogRoot",
                "QuestLogPanel",
                "QuestLogActions",
                "TrackQuest",
                "CloseQuestLog",
            ],
            required_runtime_controls: &[
                "QuestLogRoot",
                "QuestLogDialog",
                "QuestLogActions",
                "TrackQuestButton",
                "CloseQuestLogButton",
            ],
            required_editor_text: &["Quest Log", "Track", "Close"],
            required_runtime_text: &["Quest Log", "The First Light", "Track", "Close"],
            minimum_editor_buttons: 2,
            minimum_runtime_buttons: 2,
            minimum_editor_text_commands: 5,
            minimum_runtime_text_commands: 6,
            minimum_runtime_quads: 2,
        },
    ]
}

fn assert_runtime_ui_snapshot(
    side: &str,
    golden: &RuntimeUiSemanticGolden,
    snapshot: &RuntimeUiSemanticSnapshot,
) {
    let required_controls = match side {
        "editor" => golden.required_editor_controls,
        "runtime" => golden.required_runtime_controls,
        _ => unreachable!("only editor/runtime semantic golden sides are valid"),
    };
    let required_text = match side {
        "editor" => golden.required_editor_text,
        "runtime" => golden.required_runtime_text,
        _ => unreachable!("only editor/runtime semantic golden sides are valid"),
    };
    let minimum_buttons = match side {
        "editor" => golden.minimum_editor_buttons,
        "runtime" => golden.minimum_runtime_buttons,
        _ => unreachable!("only editor/runtime semantic golden sides are valid"),
    };
    let minimum_text_commands = match side {
        "editor" => golden.minimum_editor_text_commands,
        "runtime" => golden.minimum_runtime_text_commands,
        _ => unreachable!("only editor/runtime semantic golden sides are valid"),
    };
    let minimum_quads = match side {
        "editor" => 0,
        "runtime" => golden.minimum_runtime_quads,
        _ => unreachable!("only editor/runtime semantic golden sides are valid"),
    };

    assert!(
        snapshot
            .component_counts
            .get("Overlay")
            .copied()
            .unwrap_or(0)
            >= 1
            || snapshot
                .component_counts
                .get("VerticalBox")
                .copied()
                .unwrap_or(0)
                >= 1,
        "{side} {} surface should compile a rooted template tree",
        golden.name
    );
    assert!(
        snapshot
            .component_counts
            .get("Button")
            .copied()
            .unwrap_or(0)
            >= minimum_buttons,
        "{side} {} surface should expose at least {minimum_buttons} runtime controls as buttons",
        golden.name
    );
    assert!(
        snapshot.text_payload_count >= minimum_text_commands,
        "{side} {} surface should produce shared text payloads",
        golden.name
    );
    assert!(
        snapshot
            .render_kind_counts
            .get("Quad")
            .copied()
            .unwrap_or(0)
            >= minimum_quads,
        "{side} {} surface should produce shared material/quad paint commands",
        golden.name
    );

    for control in required_controls {
        assert!(
            snapshot.control_ids.contains(*control),
            "{side} {} surface should preserve semantic control `{control}`",
            golden.name
        );
    }
    for text in required_text {
        assert!(
            snapshot.rendered_text.contains(*text),
            "{side} {} surface should render semantic text `{text}`",
            golden.name
        );
    }
}

fn build_surface(path: &Path, style_import: Option<&Path>, tree_id: &str) -> UiSurface {
    let document = UiAssetLoader::load_toml_file(path)
        .unwrap_or_else(|error| panic!("{} loads as ui asset: {error}", path.display()));
    let mut compiler = UiDocumentCompiler::default();
    if let Some(style_import) = style_import {
        let style_document = UiAssetLoader::load_toml_file(style_import).unwrap_or_else(|error| {
            panic!("{} loads as style asset: {error}", style_import.display())
        });
        compiler
            .register_style_import("res://ui/theme/editor_material.ui.toml", style_document)
            .expect("editor material style import is accepted by compiler");
    }
    let compiled = compiler
        .compile(&document)
        .unwrap_or_else(|error| panic!("{} compiles as ui asset: {error}", path.display()));
    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(tree_id),
        &compiled,
    )
    .unwrap_or_else(|error| panic!("{} builds as shared surface: {error}", path.display()));
    surface
        .compute_layout(UiSize::new(1280.0, 720.0))
        .expect("quest log semantic golden computes layout");
    surface
}

fn semantic_snapshot(surface: &UiSurface) -> RuntimeQuestLogSemanticSnapshot {
    let mut snapshot = RuntimeQuestLogSemanticSnapshot::default();
    for node in surface.tree.nodes.values() {
        let Some(metadata) = node.template_metadata.as_ref() else {
            continue;
        };
        *snapshot
            .component_counts
            .entry(metadata.component.clone())
            .or_default() += 1;
        if let Some(control_id) = metadata.control_id.as_ref() {
            snapshot.control_ids.insert(control_id.clone());
        }
        for binding in metadata
            .bindings
            .iter()
            .filter(|binding| binding.event == UiEventKind::Click)
        {
            snapshot.click_binding_ids.insert(binding.id.clone());
            if let Some(route) = binding.route.as_ref() {
                snapshot.click_routes.insert(route.clone());
            }
        }
    }

    for command in &surface.render_extract.list.commands {
        if let Some(text) = command.text.as_deref().filter(|text| !text.is_empty()) {
            snapshot.rendered_text.insert(text.to_string());
        }
    }
    snapshot
}

fn runtime_ui_semantic_snapshot(surface: &UiSurface) -> RuntimeUiSemanticSnapshot {
    let mut snapshot = RuntimeUiSemanticSnapshot::default();
    for node in surface.tree.nodes.values() {
        let Some(metadata) = node.template_metadata.as_ref() else {
            continue;
        };
        *snapshot
            .component_counts
            .entry(metadata.component.clone())
            .or_default() += 1;
        if let Some(control_id) = metadata.control_id.as_ref() {
            snapshot.control_ids.insert(control_id.clone());
        }
        for binding in metadata
            .bindings
            .iter()
            .filter(|binding| binding.event == UiEventKind::Click)
        {
            snapshot.click_binding_ids.insert(binding.id.clone());
            if let Some(route) = binding.route.as_ref() {
                snapshot.click_routes.insert(route.clone());
            }
        }
    }

    for command in &surface.render_extract.list.commands {
        *snapshot
            .render_kind_counts
            .entry(format!("{:?}", command.kind))
            .or_default() += 1;
        if let Some(text) = command.text.as_deref().filter(|text| !text.is_empty()) {
            snapshot.text_payload_count += 1;
            snapshot.rendered_text.insert(text.to_string());
        }
    }
    snapshot
}
