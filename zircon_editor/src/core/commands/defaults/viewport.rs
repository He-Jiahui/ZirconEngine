use super::*;
use crate::core::commands::DocumentKind;
use crate::core::editor_event::EditorViewportEvent;
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::TransformHandleKind;

pub(super) fn append_viewport_commands(commands: &mut Vec<EditorCommandDescriptor>) {
    for (id, chord, event) in [
        (
            "viewport.selection.frame",
            "F",
            EditorViewportEvent::FrameSelection,
        ),
        (
            "viewport.tool.select",
            "Q",
            EditorViewportEvent::ActivateSceneMode {
                mode: SceneModeActivation::Select,
            },
        ),
        (
            "viewport.tool.translate",
            "W",
            EditorViewportEvent::ActivateSceneMode {
                mode: SceneModeActivation::Transform(TransformHandleKind::Move),
            },
        ),
        (
            "viewport.tool.rotate",
            "E",
            EditorViewportEvent::ActivateSceneMode {
                mode: SceneModeActivation::Transform(TransformHandleKind::Rotate),
            },
        ),
        (
            "viewport.tool.scale",
            "R",
            EditorViewportEvent::ActivateSceneMode {
                mode: SceneModeActivation::Transform(TransformHandleKind::Scale),
            },
        ),
    ] {
        commands.push(command(
            id,
            EditorCommandCategory::View,
            "view",
            &["viewport"],
            EditorEvent::Viewport(EditorViewportEvent::ForFocusedScene {
                event: Box::new(event),
            }),
            Some(chord),
            WhenClause::All(vec![
                WhenClause::ProjectOpen,
                WhenClause::FocusedDocumentKind(DocumentKind::scene()),
            ]),
            ["viewport", "scene", "tool"],
        ));
    }
    commands.push(command(
        "view.content_drawer.toggle",
        EditorCommandCategory::View,
        "view",
        &[],
        EditorEvent::Transient(EditorEventTransient::ToggleContentDrawer),
        Some("Ctrl+Space"),
        WhenClause::ProjectOpen,
        ["content", "drawer", "assets"],
    ));
}
