use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use crate::ui::binding::{
    EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, ViewportCommand,
};

#[test]
fn viewport_command_binding_roundtrips_with_resize_event_kind() {
    let binding = EditorUiBinding::new(
        "SceneView",
        "ViewportSurface",
        EditorUiEventKind::Resize,
        EditorUiBindingPayload::viewport_command(ViewportCommand::Resized {
            width: 1024,
            height: 768,
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"SceneView/ViewportSurface:onResize(ViewportCommand.Resized(1024,768))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn viewport_toolbar_command_bindings_roundtrip_through_native_binding() {
    let bindings = [
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(
                    SceneViewportTool::Rotate,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetTool("Rotate"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTransformSpace(
                    TransformSpace::Global,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetTransformSpace("Global"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetProjectionMode(
                    ProjectionMode::Orthographic,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetProjectionMode("Orthographic"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::AlignView(
                    ViewOrientation::NegZ,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.AlignView("NegZ"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetDisplayMode(
                    DisplayMode::WireOverlay,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetDisplayMode("WireOverlay"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGridMode(
                    GridMode::VisibleAndSnap,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetGridMode("VisibleAndSnap"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTranslateSnap(2.0)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetTranslateSnap(2.0))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetRotateSnapDegrees(
                    30.0,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetRotateSnapDegrees(30.0))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetScaleSnap(0.25)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetScaleSnap(0.25))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewLighting(
                    false,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetPreviewLighting(false))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewSkybox(false)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetPreviewSkybox(false))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGizmosEnabled(false)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetGizmosEnabled(false))"#,
        ),
    ];

    for (binding, expected) in bindings {
        assert_eq!(binding.native_binding(), expected);
        assert_eq!(
            EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
            binding
        );
    }
}

#[test]
fn viewport_toolbar_command_binding_roundtrips_with_typed_settings_payload() {
    let binding = EditorUiBinding::new(
        "SceneView",
        "ViewportToolbar",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetDisplayMode(
            DisplayMode::WireOverlay,
        )),
    );

    assert_eq!(
        binding.native_binding(),
        r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetDisplayMode("WireOverlay"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn viewport_toolbar_command_roundtrips_for_projection_alignment_and_snaps() {
    let commands = [
        ViewportCommand::SetTool(SceneViewportTool::Scale),
        ViewportCommand::SetTransformSpace(TransformSpace::Global),
        ViewportCommand::SetProjectionMode(ProjectionMode::Orthographic),
        ViewportCommand::AlignView(ViewOrientation::PosY),
        ViewportCommand::SetGridMode(GridMode::VisibleAndSnap),
        ViewportCommand::SetTranslateSnap(2.5),
        ViewportCommand::SetRotateSnapDegrees(30.0),
        ViewportCommand::SetScaleSnap(0.25),
        ViewportCommand::SetPreviewLighting(false),
        ViewportCommand::SetPreviewSkybox(false),
        ViewportCommand::SetGizmosEnabled(false),
        ViewportCommand::FrameSelection,
    ];

    for command in commands {
        let binding = EditorUiBinding::new(
            "SceneView",
            "ViewportToolbar",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::viewport_command(command),
        );
        assert_eq!(
            EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
            binding
        );
    }
}
