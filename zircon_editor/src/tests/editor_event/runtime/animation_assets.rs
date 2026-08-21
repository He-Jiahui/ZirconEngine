use super::*;
use crate::core::asset::AssetToolkitOpenRoute;
use crate::tests::editor_event::support::TestProjectAssets;
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationSequenceAsset, AnimationStateAsset, AnimationStateMachineAsset,
};

const SEQUENCE_LOCATOR: &str = "res://animation/hero.sequence.zranim";
const GRAPH_LOCATOR: &str = "res://animation/hero.graph.zranim";
const STATE_MACHINE_LOCATOR: &str = "res://animation/hero.state_machine.zranim";
const UI_ASSET_LOCATOR: &str = "res://ui/menu.zui";

fn write_ui_asset_open_route_asset(project: &TestProjectAssets) {
    fs::write(
        project.source_path(UI_ASSET_LOCATOR),
        r#"
[asset]
kind = "view"
id = "editor.tests.menu_ui_asset"
version = 2
display_name = "Menu UI Asset"

[root]
node = "root"

[nodes.root]
component = "Label"
props = { text = "Menu" }
"#,
    )
    .unwrap();
}

fn write_animation_open_route_assets(project: &TestProjectAssets) {
    fs::write(
        project.source_path(SEQUENCE_LOCATOR),
        AnimationSequenceAsset {
            name: Some("Hero Sequence".to_string()),
            duration_seconds: 1.0,
            frames_per_second: 30.0,
            bindings: Vec::new(),
        }
        .to_bytes()
        .unwrap(),
    )
    .unwrap();
    fs::write(
        project.source_path(GRAPH_LOCATOR),
        AnimationGraphAsset {
            name: Some("Hero Graph".to_string()),
            parameters: Vec::new(),
            nodes: Vec::new(),
        }
        .to_bytes()
        .unwrap(),
    )
    .unwrap();
    fs::write(
        project.source_path(STATE_MACHINE_LOCATOR),
        AnimationStateMachineAsset {
            name: Some("Hero State Machine".to_string()),
            entry_state: "Idle".to_string(),
            states: vec![AnimationStateAsset::graph_ref(
                "Idle",
                AssetReference::from_locator(AssetUri::parse(GRAPH_LOCATOR).unwrap()),
            )],
            transitions: Vec::new(),
            layers: Vec::new(),
        }
        .to_bytes()
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn animation_binding_without_active_sequence_editor_reports_ignored_status_line() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_binding");
    let binding = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "CreateTrackButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTrack {
            track_path: "Root/Hero:AnimationPlayer.weight".to_string(),
        }),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("animation binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Animation(EditorAnimationEvent::CreateTrack {
            track_path: AnimationTrackPath::parse("Root/Hero:AnimationPlayer.weight").unwrap(),
        })
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(record
        .effects
        .contains(&EditorEventEffect::ReflectionChanged));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Ignored animation command because focused view is not an animation sequence editor"
    );
}

#[test]
fn animation_graph_and_state_machine_bindings_without_open_editor_report_ignored_status_line() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_graph_binding");
    let binding = EditorUiBinding::new(
        "AnimationGraphEditorView",
        "CreateTransition",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTransition {
            state_machine_locator: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        }),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("graph/state-machine animation binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Animation(EditorAnimationEvent::CreateTransition {
            state_machine_locator: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        })
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(record
        .effects
        .contains(&EditorEventEffect::ReflectionChanged));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Ignored animation command because focused view is not an animation graph editor"
    );
}

#[test]
fn workbench_menu_open_ui_asset_opens_ui_asset_editor_for_shared_asset() {
    let _guard = env_lock().lock().unwrap();

    let mut runtime = EventRuntimeHarness::new("zircon_editor_event_menu_open_ui_asset");
    let catalog = runtime.open_project_with_assets(
        "zircon_editor_event_menu_open_ui_asset_project",
        write_ui_asset_open_route_asset,
    );
    assert!(catalog
        .assets
        .iter()
        .any(|asset| asset.locator == UI_ASSET_LOCATOR));

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: UI_ASSET_LOCATOR.to_string(),
            }),
        )
        .expect("menu open ui asset");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_locator: UI_ASSET_LOCATOR.to_string(),
        })
    );
    assert!(record.effects.contains(&EditorEventEffect::LayoutChanged));
    let ui_asset_view = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset"))
        .expect("indexed UI asset should open its toolkit view");
    let route: AssetToolkitOpenRoute =
        serde_json::from_value(ui_asset_view.serializable_payload).expect("typed UI asset route");
    assert_eq!(route.asset_locator().to_string(), UI_ASSET_LOCATOR);
    assert_eq!(route.open_operation().as_str(), "view.editor.ui_asset.open");
}

#[test]
fn asset_open_event_rejects_canonical_unindexed_ui_asset_locator() {
    let _guard = env_lock().lock().unwrap();

    let mut runtime = EventRuntimeHarness::new("zircon_editor_event_unindexed_ui_asset_open");
    let catalog = runtime.open_project_with_assets(
        "zircon_editor_event_unindexed_ui_asset_open_project",
        |_| {},
    );
    let asset_locator = "res://ui/unindexed.zui";
    assert!(catalog
        .assets
        .iter()
        .all(|asset| asset.locator != asset_locator));

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.to_string(),
            }),
        )
        .expect("unindexed canonical locator should be rejected by the asset index");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_locator: asset_locator.to_string(),
        })
    );
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(!runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset")));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        format!("Asset type is not indexed for {asset_locator}")
    );
}

#[test]
fn asset_open_event_routes_animation_assets_to_animation_editor_views() {
    let _guard = env_lock().lock().unwrap();

    let mut runtime = EventRuntimeHarness::new("zircon_editor_event_animation_asset_open");
    runtime.register_animation_asset_toolkits();
    let catalog = runtime.open_project_with_assets(
        "zircon_editor_event_animation_asset_open_project",
        write_animation_open_route_assets,
    );
    for locator in [SEQUENCE_LOCATOR, GRAPH_LOCATOR, STATE_MACHINE_LOCATOR] {
        assert!(catalog.assets.iter().any(|asset| asset.locator == locator));
    }

    let sequence_record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: SEQUENCE_LOCATOR.to_string(),
            }),
        )
        .expect("open animation sequence asset");
    assert!(sequence_record
        .effects
        .contains(&EditorEventEffect::LayoutChanged));

    let instances = runtime.runtime.current_view_instances();
    let sequence_view = instances
        .iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("animation sequence view should open");
    let sequence_route: AssetToolkitOpenRoute =
        serde_json::from_value(sequence_view.serializable_payload.clone()).unwrap();
    assert_eq!(sequence_route.asset_locator().to_string(), SEQUENCE_LOCATOR);
    assert_eq!(
        sequence_route.open_operation().as_str(),
        "timeline_sequence.authoring.open"
    );

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: GRAPH_LOCATOR.to_string(),
            }),
        )
        .expect("open animation graph asset");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: STATE_MACHINE_LOCATOR.to_string(),
            }),
        )
        .expect("open animation state machine asset");

    let graph_views = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .filter(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph")
        })
        .collect::<Vec<_>>();
    assert_eq!(graph_views.len(), 2);
    let graph_routes = graph_views
        .iter()
        .map(|instance| {
            serde_json::from_value::<AssetToolkitOpenRoute>(instance.serializable_payload.clone())
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert!(graph_routes.iter().any(|route| {
        route.asset_locator().to_string() == GRAPH_LOCATOR
            && route.open_operation().as_str() == "animation_graph.authoring.open_graph"
    }));
    assert!(graph_routes.iter().any(|route| {
        route.asset_locator().to_string() == STATE_MACHINE_LOCATOR
            && route.open_operation().as_str() == "animation_graph.authoring.open_state_machine"
    }));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        format!("Opened asset toolkit for {STATE_MACHINE_LOCATOR}")
    );
}

#[test]
fn asset_kind_filter_event_accepts_physics_and_animation_asset_kinds() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_asset_kind_filters");
    for (kind, expected) in [
        ("PhysicsMaterial", ResourceKind::PhysicsMaterial),
        ("AnimationSequence", ResourceKind::AnimationSequence),
        ("AnimationGraph", ResourceKind::AnimationGraph),
        ("AnimationStateMachine", ResourceKind::AnimationStateMachine),
    ] {
        let record = runtime
            .runtime
            .dispatch_event(
                EditorEventSource::Headless,
                EditorEvent::Asset(EditorAssetEvent::SetKindFilter {
                    kind: Some(kind.to_string()),
                }),
            )
            .expect("asset kind filter event");

        assert_eq!(
            runtime.runtime.editor_snapshot().asset_activity.kind_filter,
            Some(expected)
        );
        assert_eq!(
            runtime.runtime.editor_snapshot().asset_browser.kind_filter,
            Some(expected)
        );
        assert!(record
            .effects
            .contains(&EditorEventEffect::PresentationChanged));
    }
}
