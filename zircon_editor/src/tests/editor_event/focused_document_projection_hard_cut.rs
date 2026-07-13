#[test]
fn focused_document_projection_has_one_typed_owner() {
    let descriptor = include_str!("../../ui/workbench/view/view_descriptor.rs");
    let chrome = include_str!("../../ui/workbench/snapshot/data/editor_chrome_snapshot.rs");
    let projection = include_str!("../../ui/host/command_eval_projection.rs");
    let session = include_str!("../../ui/host/editor_session_state.rs");
    let workspace = include_str!("../../ui/workbench/project/project_editor_workspace.rs");
    let extension_root = include_str!("../../core/editor_extension.rs");
    let extension_view_descriptor = include_str!("../../core/editor_extension/view_descriptor.rs");
    let animation_editing = include_str!("../../ui/host/animation_editor_sessions/editing.rs");
    let animation_execution =
        include_str!("../../ui/host/editor_event_execution/animation_event.rs");

    assert!(descriptor.contains("pub document_kind: Option<DocumentKind>"));
    assert!(chrome.contains("pub focused_document_kind: Option<DocumentKind>"));
    assert!(projection
        .contains(".with_optional_focused_document_kind(chrome.focused_document_kind.clone())"));
    assert!(session.contains("pub(crate) focused_view: Option<ViewInstanceId>"));
    assert!(workspace.contains("pub focused_view: Option<ViewInstanceId>"));
    assert!(extension_root.contains("mod view_descriptor;"));
    assert!(extension_root.contains("pub use view_descriptor::ViewDescriptor;"));
    assert!(!extension_root.contains("pub struct ViewDescriptor"));
    assert!(extension_view_descriptor.contains("pub struct ViewDescriptor"));

    let retired_focus_field = concat!("active_center", "_tab");
    let retired_focus_wording = concat!("active center", " tab");
    let retired_animation_focus_wording = concat!("active ", "animation");
    let retired_animation_focus_function = concat!("active_animation", "_sequence_instance");
    assert!(!session.contains(retired_focus_field));
    assert!(!workspace.contains(retired_focus_field));
    assert!(!animation_editing.contains(retired_focus_wording));
    assert!(!animation_execution.contains(retired_focus_wording));
    assert!(!animation_editing.contains(retired_animation_focus_wording));
    assert!(!animation_execution.contains(retired_animation_focus_wording));
    assert!(!animation_editing.contains(retired_animation_focus_function));
    assert!(animation_editing.contains("focused_animation_sequence_instance"));
}
