use std::path::Path;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};
use zircon_runtime::plugin::native::NativePluginLoadReport;

use super::super::editor_error::EditorError;
use crate::core::document::DocumentLifecycleAuthority;
use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic,
    SharedEditorMessageBus, TOPIC_DOCUMENT,
};

use super::{
    project_close_terminal_root, project_diagnostics_configuration_message,
    publish_committed_project_close, publish_document_messages,
    required_native_editor_plugin_failure,
};

#[test]
fn optional_native_editor_failure_is_projected_to_project_status() {
    let source = include_str!("../editor_manager_project.rs");
    let apply = source
        .split_once("pub(super) fn apply_project_plugin_manifest(")
        .and_then(|(_, tail)| tail.split_once("pub(super) fn publish_document_messages("))
        .map(|(body, _)| body)
        .expect("project plugin application owner");
    assert!(apply.contains("native_plugin_status_report_from_load_report"));
    assert!(apply.contains("publish_project_plugin_status(status_report)"));
    assert!(
        apply.find(".apply_project_manifest(&completed.plugins)")
            < apply.find("self.native_plugin_status_report_from_load_report"),
        "status must observe the committed editor plugin catalog"
    );
}

#[test]
fn required_native_editor_plugin_cannot_enter_ready_without_admitted_package() {
    let report = NativePluginLoadReport::default();
    let required = ProjectPluginSelection::runtime_plugin("untrusted.editor", true, true)
        .with_packaging(ExportPackagingStrategy::NativeDynamic)
        .with_target_modes([RuntimeTargetMode::EditorHost]);
    let selected = ProjectPluginManifest {
        selections: vec![required.clone()],
    };
    let reason = required_native_editor_plugin_failure(&selected, report.projection(), &[])
        .expect("missing required native editor entry must prevent project Ready");
    assert!(reason.contains("untrusted.editor"));

    for ignored in [
        ProjectPluginSelection {
            required: false,
            ..required.clone()
        },
        ProjectPluginSelection {
            enabled: false,
            ..required.clone()
        },
        required
            .clone()
            .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        required.with_packaging(ExportPackagingStrategy::LibraryEmbed),
    ] {
        let optional = ProjectPluginManifest {
            selections: vec![ignored],
        };
        assert!(
            required_native_editor_plugin_failure(&optional, report.projection(), &[]).is_none()
        );
    }
}

#[cfg(windows)]
#[test]
fn project_diagnostics_configuration_message_hides_windows_verbatim_operation_roots() {
    assert_eq!(
        project_diagnostics_configuration_message(
            Path::new(r"\\?\C:\projects\forest"),
            "access denied"
        ),
        r"editor diagnostics cannot be configured for `C:\projects\forest`: access denied"
    );
}

#[test]
fn document_events_are_published_to_the_canonical_topic_in_lifecycle_order() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_DOCUMENT).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let document = DocumentId::new(42);

    publish_document_messages(
        &bus,
        [
            DocumentMessage::Opened { doc: document },
            DocumentMessage::Saved { doc: document },
            DocumentMessage::Closed { doc: document },
        ],
    );

    let delivered = bus.drain_deliveries(subscriber);
    assert_eq!(delivered.len(), 3);
    assert_eq!(
        delivered
            .iter()
            .map(|delivery| (delivery.topic().as_str(), delivery.message().clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                TOPIC_DOCUMENT,
                EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Opened {
                    doc: document,
                })),
            ),
            (
                TOPIC_DOCUMENT,
                EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Saved {
                    doc: document,
                })),
            ),
            (
                TOPIC_DOCUMENT,
                EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Closed {
                    doc: document,
                })),
            ),
        ]
    );
}

#[test]
fn committed_project_close_publishes_one_closed_document_message_and_never_fabricates_one() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_DOCUMENT).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let lifecycle = DocumentLifecycleAuthority::default();
    let root = Path::new("C:/projects/close-producer");
    let document = match lifecycle.activate(root).as_slice() {
        [DocumentMessage::Opened { doc }] => *doc,
        actual => panic!("expected opened document, got {actual:?}"),
    };

    publish_committed_project_close(&bus, &lifecycle, None);
    assert!(bus.drain_deliveries(subscriber).is_empty());

    publish_committed_project_close(&bus, &lifecycle, Some(root));
    assert_eq!(
        bus.drain_deliveries(subscriber)
            .into_iter()
            .map(|delivery| delivery.message().clone())
            .collect::<Vec<_>>(),
        vec![EditorMessage::new(EditorMessagePayload::Document(
            DocumentMessage::Closed { doc: document },
        ))]
    );

    publish_committed_project_close(&bus, &lifecycle, Some(root));
    assert!(bus.drain_deliveries(subscriber).is_empty());
}

#[test]
fn committed_project_close_closes_the_active_scene_document_for_a_project_session() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_DOCUMENT).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let lifecycle = DocumentLifecycleAuthority::default();
    let root = Path::new("C:/projects/close-active-scene");
    let session = lifecycle.begin_project_session(root).session;
    let scene = lifecycle
        .activate_scene(session, root, "res://scenes/main.scene.toml")
        .unwrap();

    publish_committed_project_close(&bus, &lifecycle, Some(root));

    assert_eq!(
        bus.drain_deliveries(subscriber)
            .into_iter()
            .map(|delivery| delivery.message().clone())
            .collect::<Vec<_>>(),
        vec![EditorMessage::new(EditorMessagePayload::Document(
            DocumentMessage::Closed {
                doc: scene.document
            }
        ))]
    );
}

#[test]
fn project_close_consumes_a_capability_and_quiesces_plugins_before_runtime() {
    let source = include_str!("../editor_manager_project.rs");
    let close_start = source
        .find("pub(crate) fn commit_project_close")
        .expect("project close entry point");
    let close_end = source[close_start..]
        .find("pub(crate) fn save_active_scene")
        .map(|offset| close_start + offset)
        .expect("project close boundary");
    let close = &source[close_start..close_end];
    let plugin_close = close
        .find("clear_project_registration_reports()")
        .expect("plugin teardown");
    let runtime_close = close
        .find(".close_project(operation.project_root())")
        .expect("runtime project teardown");

    assert!(close.contains("operation: &ProjectCloseOperation"));
    assert!(plugin_close < runtime_close);
    assert!(close.contains("require_project_close_recovery"));
    assert!(!close.contains("release_project_close_guard"));

    let finalize = source
        .find("pub(crate) fn finalize_project_close")
        .expect("final close owner");
    assert!(source[finalize..].contains("self.release_project_close_guard(operation)?"));
}

#[test]
fn project_close_retry_uses_the_retained_guard_root_after_host_close_has_committed() {
    let retained_root = Path::new("C:/projects/retained-close");

    assert_eq!(
        project_close_terminal_root(None, Some(retained_root)),
        Some(retained_root)
    );
}

#[test]
fn active_scene_save_routing_uses_lifecycle_identity_without_a_manifest_default_fallback() {
    let source = include_str!("../editor_manager_project.rs");
    let save_start = source
        .find("pub(crate) fn save_active_scene(")
        .expect("active-scene save entry point");
    let save_end = source[save_start..]
        .find("/// Publishes the manifest-selected startup scene")
        .map(|offset| save_start + offset)
        .expect("startup-scene boundary after active-scene save entry point");
    let save = &source[save_start..save_end];

    assert!(save.contains(".active_scene_identity(&project_root)"));
    assert!(save.contains(".save_active_scene(&project_root, &scene_uri, world)?"));
    assert!(save.contains(".save_scene_identity_if_active(&active_scene)"));
    assert!(!save.contains("manifest().default_scene"));
}
