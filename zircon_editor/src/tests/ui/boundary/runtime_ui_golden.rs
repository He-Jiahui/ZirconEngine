use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::v2::{UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder};
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
    runtime_asset: &'static str,
    required_controls: &'static [&'static str],
    required_text: &'static [&'static str],
    minimum_buttons: usize,
    minimum_text_commands: usize,
    minimum_runtime_quads: usize,
}

#[test]
fn quest_log_runtime_v2_asset_preserves_runtime_semantic_golden() {
    let editor_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = editor_root
        .parent()
        .expect("zircon_editor lives directly under workspace root");
    let runtime_root = workspace_root.join("zircon_runtime");

    let runtime_surface = build_v2_surface(
        &runtime_root.join("assets/ui/runtime/fixtures/quest_log_dialog.v2.ui.toml"),
        "runtime.ui.quest_log_dialog",
    );

    let runtime_snapshot = semantic_snapshot(&runtime_surface);

    let expected_click_binding_ids =
        BTreeSet::from(["QuestLog/Close".to_string(), "QuestLog/Track".to_string()]);
    let expected_click_routes = BTreeSet::from([
        "RuntimeAction.CloseQuestLog".to_string(),
        "RuntimeAction.TrackQuest".to_string(),
    ]);
    assert_eq!(
        runtime_snapshot.component_counts.get("Button").copied(),
        Some(2),
        "runtime quest log v2 surface should expose two runtime action buttons"
    );
    assert!(
        runtime_snapshot.rendered_text.contains("Quest Log"),
        "runtime quest log v2 surface should render the dialog title"
    );
    assert!(
        runtime_snapshot.rendered_text.contains("Track")
            && runtime_snapshot.rendered_text.contains("Close"),
        "runtime quest log v2 surface should render matching action labels"
    );
    assert_eq!(
        runtime_snapshot.click_binding_ids, expected_click_binding_ids,
        "runtime quest log v2 surface should preserve shared Click binding ids"
    );
    assert_eq!(
        runtime_snapshot.click_routes, expected_click_routes,
        "runtime quest log v2 surface should preserve shared runtime Click routes"
    );
    assert!(
        runtime_snapshot.control_ids.contains("QuestLogActions"),
        "runtime quest log v2 surface should preserve the action-row semantic control id"
    );
}

#[test]
fn all_runtime_v2_fixtures_share_template_semantic_golden() {
    let editor_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = editor_root
        .parent()
        .expect("zircon_editor lives directly under workspace root");
    let runtime_root = workspace_root.join("zircon_runtime");

    for golden in runtime_ui_semantic_goldens() {
        let runtime_surface = build_v2_surface(
            &runtime_root.join(golden.runtime_asset),
            &format!("runtime.ui.{}", golden.name),
        );

        let runtime_snapshot = runtime_ui_semantic_snapshot(&runtime_surface);

        assert_runtime_ui_snapshot(golden, &runtime_snapshot);
    }
}

fn runtime_ui_semantic_goldens() -> &'static [RuntimeUiSemanticGolden] {
    &[
        RuntimeUiSemanticGolden {
            name: "hud",
            runtime_asset: "assets/ui/runtime/fixtures/hud_overlay.v2.ui.toml",
            required_controls: &["HudRoot", "HealthPanel", "WeaponIcon", "AmmoPanel"],
            required_text: &["HP 87 / 100", "Reach the relay before sunrise", "24 / 120"],
            minimum_buttons: 0,
            minimum_text_commands: 3,
            minimum_runtime_quads: 2,
        },
        RuntimeUiSemanticGolden {
            name: "pause",
            runtime_asset: "assets/ui/runtime/fixtures/pause_menu.v2.ui.toml",
            required_controls: &[
                "PauseMenuRoot",
                "PauseDialog",
                "ResumeButton",
                "SettingsButton",
                "QuitButton",
            ],
            required_text: &["Paused", "Resume Mission", "Settings", "Return To Title"],
            minimum_buttons: 3,
            minimum_text_commands: 5,
            minimum_runtime_quads: 4,
        },
        RuntimeUiSemanticGolden {
            name: "settings",
            runtime_asset: "assets/ui/runtime/fixtures/settings_dialog.v2.ui.toml",
            required_controls: &[
                "SettingsRoot",
                "SettingsDialog",
                "SettingsNav",
                "ApplySettings",
                "CancelSettings",
            ],
            required_text: &[
                "Graphics\nAudio\nGameplay",
                "Graphics Quality    Epic",
                "Apply",
            ],
            minimum_buttons: 4,
            minimum_text_commands: 5,
            minimum_runtime_quads: 5,
        },
        RuntimeUiSemanticGolden {
            name: "inventory",
            runtime_asset: "assets/ui/runtime/fixtures/inventory_list.v2.ui.toml",
            required_controls: &[
                "InventoryRoot",
                "InventoryPanel",
                "InventoryHeader",
                "InventoryList",
                "InventoryRow00",
                "InventoryRow11",
            ],
            required_text: &[
                "Field Inventory",
                "01  Pulse Cells x24",
                "12  Shield Capacitor x5",
            ],
            minimum_buttons: 12,
            minimum_text_commands: 13,
            minimum_runtime_quads: 14,
        },
        RuntimeUiSemanticGolden {
            name: "quest_log",
            runtime_asset: "assets/ui/runtime/fixtures/quest_log_dialog.v2.ui.toml",
            required_controls: &[
                "QuestLogRoot",
                "QuestLogDialog",
                "QuestLogActions",
                "TrackQuestButton",
                "CloseQuestLogButton",
            ],
            required_text: &["Quest Log", "The First Light", "Track", "Close"],
            minimum_buttons: 2,
            minimum_text_commands: 6,
            minimum_runtime_quads: 2,
        },
    ]
}

fn assert_runtime_ui_snapshot(
    golden: &RuntimeUiSemanticGolden,
    snapshot: &RuntimeUiSemanticSnapshot,
) {
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
        "runtime {} v2 surface should compile a rooted template tree",
        golden.name
    );
    assert!(
        snapshot
            .component_counts
            .get("Button")
            .copied()
            .unwrap_or(0)
            >= golden.minimum_buttons,
        "runtime {} v2 surface should expose at least {} runtime controls as buttons",
        golden.name,
        golden.minimum_buttons
    );
    assert!(
        snapshot.text_payload_count >= golden.minimum_text_commands,
        "runtime {} v2 surface should produce shared text payloads",
        golden.name
    );
    assert!(
        snapshot
            .render_kind_counts
            .get("Quad")
            .copied()
            .unwrap_or(0)
            >= golden.minimum_runtime_quads,
        "runtime {} v2 surface should produce shared material/quad paint commands",
        golden.name
    );

    for control in golden.required_controls {
        assert!(
            snapshot.control_ids.contains(*control),
            "runtime {} v2 surface should preserve semantic control `{control}`",
            golden.name
        );
    }
    for text in golden.required_text {
        assert!(
            snapshot.rendered_text.contains(*text),
            "runtime {} v2 surface should render semantic text `{text}`",
            golden.name
        );
    }
}

fn build_v2_surface(path: &Path, tree_id: &str) -> UiSurface {
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let outcome = cache
        .load_store(std::iter::once(path))
        .unwrap_or_else(|error| panic!("{} loads as ui v2 asset: {error}", path.display()));
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(tree_id),
        outcome.root_document.as_ref(),
        outcome.compiled.as_ref(),
    )
    .unwrap_or_else(|error| panic!("{} builds as shared v2 surface: {error}", path.display()));
    surface
        .compute_layout(UiSize::new(1280.0, 720.0))
        .expect("runtime v2 semantic golden computes layout");
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
