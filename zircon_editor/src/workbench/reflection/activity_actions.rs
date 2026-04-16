use zircon_ui::{UiActionDescriptor, UiEventKind, UiParameterDescriptor, UiValueType};

use crate::snapshot::{ViewContentKind, ViewTabSnapshot};

pub(super) fn activity_actions_for_tab(tab: &ViewTabSnapshot) -> Vec<UiActionDescriptor> {
    if tab.placeholder {
        return Vec::new();
    }

    let mut actions = vec![
        UiActionDescriptor::new("focus_view", UiEventKind::Click, "DockCommand.FocusView")
            .with_parameter(UiParameterDescriptor::new(
                "instance_id",
                UiValueType::String,
            )),
        UiActionDescriptor::new(
            "detach_to_window",
            UiEventKind::Click,
            "DockCommand.DetachViewToWindow",
        )
        .with_parameter(UiParameterDescriptor::new(
            "instance_id",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("window_id", UiValueType::String)),
    ];

    match tab.content_kind {
        ViewContentKind::Inspector => {
            actions.push(
                UiActionDescriptor::new("apply_batch", UiEventKind::Click, "InspectorFieldBatch")
                    .with_parameter(UiParameterDescriptor::new(
                        "subject_path",
                        UiValueType::String,
                    ))
                    .with_parameter(UiParameterDescriptor::new("changes", UiValueType::Array)),
            );
            actions.push(
                UiActionDescriptor::new(
                    "edit_field",
                    UiEventKind::Change,
                    "DraftCommand.SetInspectorField",
                )
                .with_parameter(UiParameterDescriptor::new(
                    "subject_path",
                    UiValueType::String,
                ))
                .with_parameter(UiParameterDescriptor::new("field_id", UiValueType::String))
                .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
            );
        }
        ViewContentKind::Assets => {
            actions.push(
                UiActionDescriptor::new(
                    "set_mesh_import_path",
                    UiEventKind::Change,
                    "DraftCommand.SetMeshImportPath",
                )
                .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
            );
            actions.push(UiActionDescriptor::new(
                "import_model",
                UiEventKind::Click,
                "AssetCommand.ImportModel",
            ));
        }
        ViewContentKind::Scene | ViewContentKind::Game => {
            actions.push(
                UiActionDescriptor::new(
                    "pointer_move",
                    UiEventKind::Hover,
                    "ViewportCommand.PointerMoved",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(
                UiActionDescriptor::new(
                    "left_press",
                    UiEventKind::Press,
                    "ViewportCommand.LeftPressed",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(UiActionDescriptor::new(
                "left_release",
                UiEventKind::Release,
                "ViewportCommand.LeftReleased",
            ));
            actions.push(
                UiActionDescriptor::new(
                    "right_press",
                    UiEventKind::Press,
                    "ViewportCommand.RightPressed",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(UiActionDescriptor::new(
                "right_release",
                UiEventKind::Release,
                "ViewportCommand.RightReleased",
            ));
            actions.push(
                UiActionDescriptor::new(
                    "middle_press",
                    UiEventKind::Press,
                    "ViewportCommand.MiddlePressed",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(UiActionDescriptor::new(
                "middle_release",
                UiEventKind::Release,
                "ViewportCommand.MiddleReleased",
            ));
            actions.push(
                UiActionDescriptor::new("scroll", UiEventKind::Scroll, "ViewportCommand.Scrolled")
                    .with_parameter(UiParameterDescriptor::new("delta", UiValueType::Float)),
            );
            actions.push(
                UiActionDescriptor::new("resize", UiEventKind::Resize, "ViewportCommand.Resized")
                    .with_parameter(UiParameterDescriptor::new("width", UiValueType::Unsigned))
                    .with_parameter(UiParameterDescriptor::new("height", UiValueType::Unsigned)),
            );
        }
        _ => {}
    }

    actions
}
