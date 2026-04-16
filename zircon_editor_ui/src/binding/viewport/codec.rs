use zircon_scene::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use zircon_ui::{UiBindingCall, UiBindingValue};

use super::ViewportCommand;
use crate::binding::core::{
    required_bool_argument, required_f32_argument, required_string_argument, required_u32_argument,
    EditorUiBindingError,
};

impl ViewportCommand {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::PointerMoved { x, y } => UiBindingCall::new("ViewportCommand.PointerMoved")
                .with_argument(UiBindingValue::Float(*x as f64))
                .with_argument(UiBindingValue::Float(*y as f64)),
            Self::LeftPressed { x, y } => UiBindingCall::new("ViewportCommand.LeftPressed")
                .with_argument(UiBindingValue::Float(*x as f64))
                .with_argument(UiBindingValue::Float(*y as f64)),
            Self::LeftReleased => UiBindingCall::new("ViewportCommand.LeftReleased"),
            Self::RightPressed { x, y } => UiBindingCall::new("ViewportCommand.RightPressed")
                .with_argument(UiBindingValue::Float(*x as f64))
                .with_argument(UiBindingValue::Float(*y as f64)),
            Self::RightReleased => UiBindingCall::new("ViewportCommand.RightReleased"),
            Self::MiddlePressed { x, y } => UiBindingCall::new("ViewportCommand.MiddlePressed")
                .with_argument(UiBindingValue::Float(*x as f64))
                .with_argument(UiBindingValue::Float(*y as f64)),
            Self::MiddleReleased => UiBindingCall::new("ViewportCommand.MiddleReleased"),
            Self::Scrolled { delta } => UiBindingCall::new("ViewportCommand.Scrolled")
                .with_argument(UiBindingValue::Float(*delta as f64)),
            Self::Resized { width, height } => UiBindingCall::new("ViewportCommand.Resized")
                .with_argument(UiBindingValue::unsigned(*width))
                .with_argument(UiBindingValue::unsigned(*height)),
            Self::SetTool(tool) => UiBindingCall::new("ViewportCommand.SetTool")
                .with_argument(UiBindingValue::string(super::tool::symbol(*tool))),
            Self::SetTransformSpace(space) => {
                UiBindingCall::new("ViewportCommand.SetTransformSpace").with_argument(
                    UiBindingValue::string(super::transform_space::symbol(*space)),
                )
            }
            Self::SetProjectionMode(mode) => {
                UiBindingCall::new("ViewportCommand.SetProjectionMode").with_argument(
                    UiBindingValue::string(super::projection_mode::symbol(*mode)),
                )
            }
            Self::AlignView(orientation) => UiBindingCall::new("ViewportCommand.AlignView")
                .with_argument(UiBindingValue::string(super::view_orientation::symbol(
                    *orientation,
                ))),
            Self::SetDisplayMode(mode) => UiBindingCall::new("ViewportCommand.SetDisplayMode")
                .with_argument(UiBindingValue::string(super::display_mode::symbol(*mode))),
            Self::SetGridMode(mode) => UiBindingCall::new("ViewportCommand.SetGridMode")
                .with_argument(UiBindingValue::string(super::grid_mode::symbol(*mode))),
            Self::SetTranslateSnap(step) => UiBindingCall::new("ViewportCommand.SetTranslateSnap")
                .with_argument(UiBindingValue::Float(*step as f64)),
            Self::SetRotateSnapDegrees(step) => {
                UiBindingCall::new("ViewportCommand.SetRotateSnapDegrees")
                    .with_argument(UiBindingValue::Float(*step as f64))
            }
            Self::SetScaleSnap(step) => UiBindingCall::new("ViewportCommand.SetScaleSnap")
                .with_argument(UiBindingValue::Float(*step as f64)),
            Self::SetPreviewLighting(enabled) => {
                UiBindingCall::new("ViewportCommand.SetPreviewLighting")
                    .with_argument(UiBindingValue::Bool(*enabled))
            }
            Self::SetPreviewSkybox(enabled) => {
                UiBindingCall::new("ViewportCommand.SetPreviewSkybox")
                    .with_argument(UiBindingValue::Bool(*enabled))
            }
            Self::SetGizmosEnabled(enabled) => {
                UiBindingCall::new("ViewportCommand.SetGizmosEnabled")
                    .with_argument(UiBindingValue::Bool(*enabled))
            }
            Self::FrameSelection => UiBindingCall::new("ViewportCommand.FrameSelection"),
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "ViewportCommand.PointerMoved" => Self::PointerMoved {
                x: required_f32_argument(&call, 0, "ViewportCommand.PointerMoved")?,
                y: required_f32_argument(&call, 1, "ViewportCommand.PointerMoved")?,
            },
            "ViewportCommand.LeftPressed" => Self::LeftPressed {
                x: required_f32_argument(&call, 0, "ViewportCommand.LeftPressed")?,
                y: required_f32_argument(&call, 1, "ViewportCommand.LeftPressed")?,
            },
            "ViewportCommand.LeftReleased" => Self::LeftReleased,
            "ViewportCommand.RightPressed" => Self::RightPressed {
                x: required_f32_argument(&call, 0, "ViewportCommand.RightPressed")?,
                y: required_f32_argument(&call, 1, "ViewportCommand.RightPressed")?,
            },
            "ViewportCommand.RightReleased" => Self::RightReleased,
            "ViewportCommand.MiddlePressed" => Self::MiddlePressed {
                x: required_f32_argument(&call, 0, "ViewportCommand.MiddlePressed")?,
                y: required_f32_argument(&call, 1, "ViewportCommand.MiddlePressed")?,
            },
            "ViewportCommand.MiddleReleased" => Self::MiddleReleased,
            "ViewportCommand.Scrolled" => Self::Scrolled {
                delta: required_f32_argument(&call, 0, "ViewportCommand.Scrolled")?,
            },
            "ViewportCommand.Resized" => Self::Resized {
                width: required_u32_argument(&call, 0, "ViewportCommand.Resized")?,
                height: required_u32_argument(&call, 1, "ViewportCommand.Resized")?,
            },
            "ViewportCommand.SetTool" => Self::SetTool(parse_scene_viewport_tool(
                &required_string_argument(&call, 0, "ViewportCommand.SetTool")?,
            )?),
            "ViewportCommand.SetTransformSpace" => Self::SetTransformSpace(parse_transform_space(
                &required_string_argument(&call, 0, "ViewportCommand.SetTransformSpace")?,
            )?),
            "ViewportCommand.SetProjectionMode" => Self::SetProjectionMode(parse_projection_mode(
                &required_string_argument(&call, 0, "ViewportCommand.SetProjectionMode")?,
            )?),
            "ViewportCommand.AlignView" => Self::AlignView(parse_view_orientation(
                &required_string_argument(&call, 0, "ViewportCommand.AlignView")?,
            )?),
            "ViewportCommand.SetDisplayMode" => Self::SetDisplayMode(parse_display_mode(
                &required_string_argument(&call, 0, "ViewportCommand.SetDisplayMode")?,
            )?),
            "ViewportCommand.SetGridMode" => Self::SetGridMode(parse_grid_mode(
                &required_string_argument(&call, 0, "ViewportCommand.SetGridMode")?,
            )?),
            "ViewportCommand.SetTranslateSnap" => Self::SetTranslateSnap(required_f32_argument(
                &call,
                0,
                "ViewportCommand.SetTranslateSnap",
            )?),
            "ViewportCommand.SetRotateSnapDegrees" => Self::SetRotateSnapDegrees(
                required_f32_argument(&call, 0, "ViewportCommand.SetRotateSnapDegrees")?,
            ),
            "ViewportCommand.SetScaleSnap" => Self::SetScaleSnap(required_f32_argument(
                &call,
                0,
                "ViewportCommand.SetScaleSnap",
            )?),
            "ViewportCommand.SetPreviewLighting" => Self::SetPreviewLighting(
                required_bool_argument(&call, 0, "ViewportCommand.SetPreviewLighting")?,
            ),
            "ViewportCommand.SetPreviewSkybox" => Self::SetPreviewSkybox(required_bool_argument(
                &call,
                0,
                "ViewportCommand.SetPreviewSkybox",
            )?),
            "ViewportCommand.SetGizmosEnabled" => Self::SetGizmosEnabled(required_bool_argument(
                &call,
                0,
                "ViewportCommand.SetGizmosEnabled",
            )?),
            "ViewportCommand.FrameSelection" => Self::FrameSelection,
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}

fn parse_scene_viewport_tool(symbol: &str) -> Result<SceneViewportTool, EditorUiBindingError> {
    super::tool::parse_symbol(symbol)
        .ok_or_else(|| invalid_enum_argument("ViewportCommand.SetTool", symbol))
}

fn parse_transform_space(symbol: &str) -> Result<TransformSpace, EditorUiBindingError> {
    super::transform_space::parse_symbol(symbol)
        .ok_or_else(|| invalid_enum_argument("ViewportCommand.SetTransformSpace", symbol))
}

fn parse_projection_mode(symbol: &str) -> Result<ProjectionMode, EditorUiBindingError> {
    super::projection_mode::parse_symbol(symbol)
        .ok_or_else(|| invalid_enum_argument("ViewportCommand.SetProjectionMode", symbol))
}

fn parse_view_orientation(symbol: &str) -> Result<ViewOrientation, EditorUiBindingError> {
    super::view_orientation::parse_symbol(symbol)
        .ok_or_else(|| invalid_enum_argument("ViewportCommand.AlignView", symbol))
}

fn parse_display_mode(symbol: &str) -> Result<DisplayMode, EditorUiBindingError> {
    super::display_mode::parse_symbol(symbol)
        .ok_or_else(|| invalid_enum_argument("ViewportCommand.SetDisplayMode", symbol))
}

fn parse_grid_mode(symbol: &str) -> Result<GridMode, EditorUiBindingError> {
    super::grid_mode::parse_symbol(symbol)
        .ok_or_else(|| invalid_enum_argument("ViewportCommand.SetGridMode", symbol))
}

fn invalid_enum_argument(symbol: &str, value: &str) -> EditorUiBindingError {
    EditorUiBindingError::InvalidPayload(format!(
        "{symbol} received unsupported variant \"{value}\""
    ))
}
