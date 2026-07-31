use zircon_editor::core::asset::AssetTypeId;
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_plugin_ai_runtime::behavior_tree::{standard_node_catalog, BehaviorNodeCategory};
use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, AiBlackboardEntry,
    AiBlackboardValue, AiDecisionStatus, AiPerceptionDebugSnapshot, AiPerceptionSense,
    AiPerceptionSnapshot, AiPerceptionStimulus, BtNodeResultEvent,
};
use zircon_runtime::core::framework::render::{
    OverlayPickShape, SceneGizmoKind, SceneGizmoOverlayExtract,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

const AI_AUTHORING_CAPABILITY: &str = "editor.extension.ai_authoring";
const AI_BEHAVIOR_TREE_ASSET_TYPE: &str = "ai.behavior_tree";
const AI_BEHAVIOR_TREE_VIEW_ID: &str = "ai.behavior_tree";
const AI_BEHAVIOR_TREE_PALETTE_ID: &str = "ai.behavior_tree.palette";
const AI_DEBUG_CAPABILITY: &str = "editor.extension.ai_debug";
const AI_PERCEPTION_DEBUG_VIEW_ID: &str = "ai.perception.debug";
const AI_PERCEPTION_OVERLAY_MODE_ID: &str = "ai.perception.viewport.overlay";
const AI_PERCEPTION_OVERLAY_PROVIDER_ID: &str = "ai.perception.viewport.overlay.provider";

#[derive(Default)]
struct RecordingAiPerceptionGizmoSink {
    overlay: Option<SceneGizmoOverlayExtract>,
}

impl crate::AiPerceptionViewportGizmoSink for RecordingAiPerceptionGizmoSink {
    fn replace_ai_perception_overlay(&mut self, overlay: Option<SceneGizmoOverlayExtract>) {
        self.overlay = overlay;
    }
}

#[test]
fn behavior_tree_editor_registers_catalog_derived_palette_and_asset_toolkit() {
    let plugin = crate::editor_plugin();
    let mut registry = EditorExtensionRegistry::default();
    plugin
        .register_editor_extensions(&mut registry)
        .expect("AI behavior-tree editor registration");

    let asset_type = AssetTypeId::parse(AI_BEHAVIOR_TREE_ASSET_TYPE).expect("asset type id");
    let graph_editor = registry
        .graph_editors()
        .into_iter()
        .find(|editor| editor.asset_type() == &asset_type)
        .expect("AI behavior-tree graph editor");
    assert_eq!(graph_editor.view_id(), AI_BEHAVIOR_TREE_VIEW_ID);
    assert_eq!(graph_editor.display_name(), "Behavior Tree");
    assert_eq!(
        graph_editor.open_operation().as_str(),
        "ai.behavior_tree.open"
    );
    assert_eq!(
        graph_editor.validate_operation().as_str(),
        "ai.behavior_tree.validate"
    );
    assert_eq!(
        graph_editor
            .compile_operation()
            .expect("behavior-tree compile operation")
            .as_str(),
        "ai.behavior_tree.compile"
    );
    assert_eq!(
        graph_editor.required_capabilities(),
        &[AI_AUTHORING_CAPABILITY.to_owned()]
    );

    let palette = registry
        .graph_node_palettes()
        .into_iter()
        .find(|palette| palette.id() == AI_BEHAVIOR_TREE_PALETTE_ID)
        .expect("AI behavior-tree node palette");
    let catalog = standard_node_catalog().expect("standard runtime behavior-node catalog");
    assert_eq!(palette.nodes().len(), catalog.descriptors().len());
    for descriptor in catalog.descriptors() {
        let node = palette
            .nodes()
            .iter()
            .find(|node| node.id() == descriptor.id())
            .expect("every runtime standard node is present in the palette");
        assert_eq!(node.display_name(), descriptor.display_name());
        assert_eq!(node.category(), category_name(descriptor.category()));
    }

    let asset_contribution = registry
        .asset_type_contributions()
        .into_iter()
        .find(|contribution| contribution.asset_type() == &asset_type)
        .expect("AI behavior-tree asset toolkit contribution");
    assert_eq!(
        asset_contribution
            .toolkit()
            .expect("AI behavior-tree toolkit")
            .view_id(),
        AI_BEHAVIOR_TREE_VIEW_ID
    );

    for operation in [
        "ai.behavior_tree.open",
        "ai.behavior_tree.validate",
        "ai.behavior_tree.compile",
    ] {
        let operation = EditorOperationPath::parse(operation).expect("operation path");
        assert!(
            registry.commands().command(&operation).is_some(),
            "missing behavior-tree command {operation}"
        );
    }

    let overlay_mode = registry
        .viewport_tool_modes()
        .into_iter()
        .find(|mode| mode.id() == AI_PERCEPTION_OVERLAY_MODE_ID)
        .expect("AI perception viewport overlay mode");
    assert_eq!(overlay_mode.view_id(), AI_PERCEPTION_DEBUG_VIEW_ID);
    assert_eq!(
        overlay_mode.overlay_provider_id(),
        Some(AI_PERCEPTION_OVERLAY_PROVIDER_ID)
    );
    assert_eq!(
        overlay_mode.required_capabilities(),
        &[AI_DEBUG_CAPABILITY.to_owned()]
    );
    let operation = EditorOperationPath::parse("ai.perception.toggle_overlay")
        .expect("perception overlay operation path");
    assert!(
        registry.commands().command(&operation).is_some(),
        "missing perception overlay command"
    );
}

fn category_name(category: BehaviorNodeCategory) -> &'static str {
    match category {
        BehaviorNodeCategory::Composite => "Composite",
        BehaviorNodeCategory::Decorator => "Decorator",
        BehaviorNodeCategory::Service => "Service",
        BehaviorNodeCategory::Task => "Task",
    }
}

#[test]
fn pie_mirror_keeps_debug_snapshot_data_and_rejects_cross_session_or_stale_frames() {
    let mut mirror = crate::AiPieMirror::default();
    mirror.begin_session(12);
    let frame = AiBehaviorDebugFrame {
        report: AiAgentTickReport {
            world: WorldHandle::new(7),
            entity: 44,
            status: AiDecisionStatus::Running,
            active_node: Some("move_to".to_owned()),
            diagnostic: None,
        },
        behavior_tree: Some("patrol".to_owned()),
        blackboard: vec![AiBlackboardEntry::new(
            "target",
            AiBlackboardValue::Entity(99),
        )],
        perception: None,
        perception_debug: None,
    };
    let snapshot = AiBehaviorDebugSnapshot {
        world: WorldHandle::new(7),
        frames: vec![frame.clone()],
    };
    assert_eq!(
        mirror.apply_debug_snapshot(12, 2, snapshot.clone()),
        crate::AiPieMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_debug_snapshot(12, 1, snapshot.clone()),
        crate::AiPieMirrorApply::Stale
    );
    assert_eq!(
        mirror.apply_debug_snapshot(99, 3, snapshot),
        crate::AiPieMirrorApply::WrongSession
    );
    let malformed = AiBehaviorDebugSnapshot {
        world: WorldHandle::new(7),
        frames: vec![AiBehaviorDebugFrame {
            report: AiAgentTickReport {
                world: WorldHandle::new(8),
                entity: 44,
                status: AiDecisionStatus::Running,
                active_node: None,
                diagnostic: None,
            },
            behavior_tree: None,
            blackboard: Vec::new(),
            perception: None,
            perception_debug: None,
        }],
    };
    assert_eq!(
        mirror.apply_debug_snapshot(12, 3, malformed),
        crate::AiPieMirrorApply::InvalidSnapshotWorld
    );

    let agent = mirror.agent(44).expect("mirrored AI agent");
    assert_eq!(agent.report.active_node.as_deref(), Some("move_to"));
    assert_eq!(agent.blackboard.len(), 1);
    assert_eq!(mirror.sequence(), Some(2));
}

#[test]
fn pie_mirror_keeps_same_entity_ids_from_different_worlds_separate() {
    let mut mirror = crate::AiPieMirror::default();
    mirror.begin_session(12);
    let mut first = AiBehaviorDebugFrame {
        report: AiAgentTickReport {
            world: WorldHandle::new(7),
            entity: 44,
            status: AiDecisionStatus::Running,
            active_node: Some("first".to_owned()),
            diagnostic: None,
        },
        behavior_tree: None,
        blackboard: Vec::new(),
        perception: None,
        perception_debug: None,
    };
    assert_eq!(
        mirror.apply_debug_snapshot(
            12,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(7),
                frames: vec![first.clone()],
            },
        ),
        crate::AiPieMirrorApply::Applied
    );
    first.report.world = WorldHandle::new(8);
    first.report.active_node = Some("second".to_owned());
    assert_eq!(
        mirror.apply_debug_snapshot(
            12,
            2,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(8),
                frames: vec![first],
            },
        ),
        crate::AiPieMirrorApply::Applied
    );

    assert_eq!(
        mirror
            .agent_in_world(&WorldHandle::new(7), 44)
            .and_then(|frame| frame.report.active_node.as_deref()),
        Some("first")
    );
    assert_eq!(
        mirror
            .agent_in_world(&WorldHandle::new(8), 44)
            .and_then(|frame| frame.report.active_node.as_deref()),
        Some("second")
    );
    assert!(
        mirror.agent(44).is_none(),
        "world-independent lookup must not pick an arbitrary PIE world"
    );
    assert_eq!(
        mirror.agents_in_world(&WorldHandle::new(7)).count(),
        1,
        "viewport consumers must select one PIE world explicitly"
    );
}

#[test]
fn pie_mirror_replaces_each_world_snapshot_and_removes_missing_agents() {
    let mut mirror = crate::AiPieMirror::default();
    mirror.begin_session(12);
    let first = AiBehaviorDebugFrame {
        report: AiAgentTickReport {
            world: WorldHandle::new(7),
            entity: 44,
            status: AiDecisionStatus::Running,
            active_node: Some("patrol".to_owned()),
            diagnostic: None,
        },
        behavior_tree: Some("guard".to_owned()),
        blackboard: Vec::new(),
        perception: None,
        perception_debug: None,
    };
    let mut second = first.clone();
    second.report.entity = 45;
    second.report.active_node = Some("scan".to_owned());
    assert_eq!(
        mirror.apply_debug_snapshot(
            12,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(7),
                frames: vec![first, second.clone()],
            },
        ),
        crate::AiPieMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_debug_snapshot(
            12,
            2,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(7),
                frames: vec![second],
            },
        ),
        crate::AiPieMirrorApply::Applied
    );

    assert!(mirror.agent_in_world(&WorldHandle::new(7), 44).is_none());
    assert_eq!(
        mirror
            .agent_in_world(&WorldHandle::new(7), 45)
            .and_then(|frame| frame.report.active_node.as_deref()),
        Some("scan")
    );
}

#[test]
fn behavior_tree_layout_exposes_a_read_only_blackboard_monitor() {
    let layout = include_str!("../behavior_tree.zui");
    assert!(layout.contains("control_id = \"AiBehaviorTreeBlackboard\""));
    assert!(layout.contains("text = \"BLACKBOARD\""));
}

#[test]
fn node_result_mirror_is_manifest_projected_and_keeps_only_current_pie_events() {
    let registration = crate::plugin_registration();
    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    let manifest = registration
        .package_manifest
        .modules
        .iter()
        .find(|module| module.name == "ai.editor")
        .expect("AI editor package manifest");
    assert!(manifest.event_consumers.iter().any(|consumer| {
        consumer.consumer_id == "ai.editor.behavior_debug"
            && consumer.event_id == crate::AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID
            && consumer.payload_schema == crate::AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA
    }));
    assert!(manifest.event_consumers.iter().any(|consumer| {
        consumer.consumer_id == "ai.editor.bt_node_result"
            && consumer.event_id == crate::BT_NODE_RESULT_EVENT_ID
            && consumer.payload_schema == crate::BT_NODE_RESULT_PAYLOAD_SCHEMA
    }));
    assert!(manifest.event_consumers.iter().any(|consumer| {
        consumer.consumer_id == "ai.editor.bt_node_result_snapshot_prune"
            && consumer.event_id == crate::AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID
            && consumer.payload_schema == crate::AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA
    }));

    let event = BtNodeResultEvent {
        world: WorldHandle::new(7),
        entity: 44,
        node_id: "move_to".to_owned(),
        status: AiDecisionStatus::Running,
        diagnostic: None,
    };
    let mut mirror = crate::AiBtNodeResultMirror::default();
    mirror.begin_session(12);
    assert_eq!(
        mirror.apply_node_result(12, 1, event.clone()),
        crate::AiBtNodeResultMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_node_result(12, 1, event.clone()),
        crate::AiBtNodeResultMirrorApply::Stale
    );
    assert_eq!(
        mirror.apply_node_result(99, 2, event),
        crate::AiBtNodeResultMirrorApply::WrongSession
    );
    assert_eq!(
        mirror
            .node_result(&WorldHandle::new(7), 44, "move_to")
            .expect("mirrored node result")
            .status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        mirror.apply_debug_snapshot(
            12,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(7),
                frames: Vec::new(),
            },
        ),
        crate::AiBtNodeResultMirrorApply::Applied
    );
    assert!(
        mirror
            .node_result(&WorldHandle::new(7), 44, "move_to")
            .is_none(),
        "a full debug snapshot clears stale behavior-tree highlights"
    );
}

#[test]
fn declared_runtime_mirrors_are_directly_accessible() {
    let plugin = crate::editor_plugin();
    let _: std::sync::Arc<std::sync::Mutex<crate::AiPieMirror>> = plugin.pie_mirror();
    let _: std::sync::Arc<std::sync::Mutex<crate::AiBtNodeResultMirror>> =
        plugin.node_result_mirror();
}

#[test]
fn perception_overlay_draws_runtime_fov_ranges_and_stimuli_from_the_pie_mirror() {
    let mut mirror = crate::AiPieMirror::default();
    mirror.begin_session(9);
    let stimulus_position = Vec3::new(4.0, 0.0, 7.0);
    let frame = AiBehaviorDebugFrame {
        report: AiAgentTickReport {
            world: WorldHandle::new(3),
            entity: 27,
            status: AiDecisionStatus::Running,
            active_node: Some("scan".to_owned()),
            diagnostic: None,
        },
        behavior_tree: Some("guard".to_owned()),
        blackboard: Vec::new(),
        perception: Some(AiPerceptionSnapshot {
            agent: 27,
            stimuli: vec![AiPerceptionStimulus {
                source: 41,
                sense: AiPerceptionSense::Hearing,
                position: stimulus_position,
                strength: 1.0,
                age_seconds: 0.0,
            }],
        }),
        perception_debug: Some(AiPerceptionDebugSnapshot {
            position: Vec3::new(1.0, 0.0, 2.0),
            forward: Vec3::Z,
            sight_fov_degrees: 90.0,
            sight_range: 6.0,
            hearing_radius: 4.0,
        }),
    };
    assert_eq!(
        mirror.apply_debug_snapshot(
            9,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(3),
                frames: vec![frame],
            },
        ),
        crate::AiPieMirrorApply::Applied
    );

    let overlay = crate::build_ai_perception_overlay(101, &WorldHandle::new(3), &mirror);
    assert_eq!(overlay.kind, SceneGizmoKind::AiPerception);
    assert!(
        overlay.lines.len() >= 35,
        "FOV, hearing radius, and stimulus link"
    );
    assert!(overlay.pick_shapes.iter().any(|shape| {
        matches!(shape, OverlayPickShape::Sphere { center, .. } if *center == stimulus_position)
    }));
}

#[test]
fn perception_overlay_controller_publishes_only_the_selected_pie_world() {
    let mut mirror = crate::AiPieMirror::default();
    mirror.begin_session(9);
    let first = AiBehaviorDebugFrame {
        report: AiAgentTickReport {
            world: WorldHandle::new(3),
            entity: 27,
            status: AiDecisionStatus::Running,
            active_node: Some("scan".to_owned()),
            diagnostic: None,
        },
        behavior_tree: Some("guard".to_owned()),
        blackboard: Vec::new(),
        perception: None,
        perception_debug: Some(AiPerceptionDebugSnapshot {
            position: Vec3::new(1.0, 0.0, 2.0),
            forward: Vec3::Z,
            sight_fov_degrees: 90.0,
            sight_range: 6.0,
            hearing_radius: 4.0,
        }),
    };
    let mut second = first.clone();
    second.report.world = WorldHandle::new(4);
    second.report.entity = 28;
    second
        .perception_debug
        .as_mut()
        .expect("debug data")
        .position = Vec3::new(10.0, 0.0, 20.0);
    assert_eq!(
        mirror.apply_debug_snapshot(
            9,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(3),
                frames: vec![first],
            },
        ),
        crate::AiPieMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_debug_snapshot(
            9,
            2,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(4),
                frames: vec![second],
            },
        ),
        crate::AiPieMirrorApply::Applied
    );

    let mut controller =
        crate::AiPerceptionOverlayController::new(RecordingAiPerceptionGizmoSink::default());
    controller.set_enabled(true);
    assert!(controller.publish(101, &WorldHandle::new(3), &mirror));
    let overlay = controller
        .sink()
        .overlay
        .as_ref()
        .expect("selected-world overlay was published");
    assert_eq!(overlay.pick_shapes.len(), 2, "one agent sphere and circle");
    assert!(overlay.pick_shapes.iter().any(
        |shape| matches!(shape, OverlayPickShape::Sphere { center, .. } if *center == Vec3::new(1.0, 0.0, 2.0))
    ));
    assert!(!overlay.pick_shapes.iter().any(
        |shape| matches!(shape, OverlayPickShape::Sphere { center, .. } if *center == Vec3::new(10.0, 0.0, 20.0))
    ));
}
