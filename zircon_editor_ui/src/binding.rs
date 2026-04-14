use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_ui::{
    UiBindingCall, UiBindingParseError, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath,
};

pub type EditorUiEventKind = UiEventKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SelectionCommand {
    SelectSceneNode { node_id: u64 },
}

impl SelectionCommand {
    fn to_call(&self) -> UiBindingCall {
        match self {
            Self::SelectSceneNode { node_id } => {
                UiBindingCall::new("SelectionCommand.SelectSceneNode")
                    .with_argument(UiBindingValue::Unsigned(*node_id))
            }
        }
    }

    fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "SelectionCommand.SelectSceneNode" => Self::SelectSceneNode {
                node_id: call.argument(0).and_then(UiBindingValue::as_u32).ok_or(
                    EditorUiBindingError::InvalidPayload(
                        "SelectionCommand.SelectSceneNode expects unsigned node_id".to_string(),
                    ),
                )? as u64,
            },
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AssetCommand {
    OpenAsset { asset_path: String },
}

impl AssetCommand {
    fn to_call(&self) -> UiBindingCall {
        match self {
            Self::OpenAsset { asset_path } => UiBindingCall::new("AssetCommand.OpenAsset")
                .with_argument(UiBindingValue::string(asset_path)),
        }
    }

    fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "AssetCommand.OpenAsset" => Self::OpenAsset {
                asset_path: required_string_argument(&call, 0, "AssetCommand.OpenAsset")?,
            },
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InspectorFieldChange {
    pub field_id: String,
    pub value: UiBindingValue,
}

impl InspectorFieldChange {
    pub fn new(field_id: impl Into<String>, value: UiBindingValue) -> Self {
        Self {
            field_id: field_id.into(),
            value,
        }
    }

    fn as_binding_value(&self) -> UiBindingValue {
        UiBindingValue::Array(vec![
            UiBindingValue::string(self.field_id.clone()),
            self.value.clone(),
        ])
    }

    fn from_binding_value(value: &UiBindingValue) -> Result<Self, EditorUiBindingError> {
        let UiBindingValue::Array(parts) = value else {
            return Err(EditorUiBindingError::InvalidPayload(
                "InspectorFieldBatch expects [field_id,value] pairs".to_string(),
            ));
        };
        if parts.len() != 2 {
            return Err(EditorUiBindingError::InvalidPayload(
                "InspectorFieldBatch expects pairs with 2 elements".to_string(),
            ));
        }
        let field_id = parts[0]
            .as_str()
            .ok_or(EditorUiBindingError::InvalidPayload(
                "InspectorFieldBatch field ids must be strings".to_string(),
            ))?
            .to_string();
        Ok(Self {
            field_id,
            value: parts[1].clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DockCommand {
    FocusView {
        instance_id: String,
    },
    CloseView {
        instance_id: String,
    },
    AttachViewToDrawer {
        instance_id: String,
        slot: String,
    },
    AttachViewToDocument {
        instance_id: String,
        page_id: String,
    },
    DetachViewToWindow {
        instance_id: String,
        window_id: String,
    },
    ActivateDrawerTab {
        slot: String,
        instance_id: String,
    },
    ActivateMainPage {
        page_id: String,
    },
    SetDrawerMode {
        slot: String,
        mode: String,
    },
    SetDrawerExtent {
        slot: String,
        extent: f32,
    },
    SavePreset {
        name: String,
    },
    LoadPreset {
        name: String,
    },
    ResetToDefault,
}

impl DockCommand {
    fn to_call(&self) -> UiBindingCall {
        match self {
            Self::FocusView { instance_id } => UiBindingCall::new("DockCommand.FocusView")
                .with_argument(UiBindingValue::string(instance_id)),
            Self::CloseView { instance_id } => UiBindingCall::new("DockCommand.CloseView")
                .with_argument(UiBindingValue::string(instance_id)),
            Self::AttachViewToDrawer { instance_id, slot } => {
                UiBindingCall::new("DockCommand.AttachViewToDrawer")
                    .with_argument(UiBindingValue::string(instance_id))
                    .with_argument(UiBindingValue::string(slot))
            }
            Self::AttachViewToDocument {
                instance_id,
                page_id,
            } => UiBindingCall::new("DockCommand.AttachViewToDocument")
                .with_argument(UiBindingValue::string(instance_id))
                .with_argument(UiBindingValue::string(page_id)),
            Self::DetachViewToWindow {
                instance_id,
                window_id,
            } => UiBindingCall::new("DockCommand.DetachViewToWindow")
                .with_argument(UiBindingValue::string(instance_id))
                .with_argument(UiBindingValue::string(window_id)),
            Self::ActivateDrawerTab { slot, instance_id } => {
                UiBindingCall::new("DockCommand.ActivateDrawerTab")
                    .with_argument(UiBindingValue::string(slot))
                    .with_argument(UiBindingValue::string(instance_id))
            }
            Self::ActivateMainPage { page_id } => {
                UiBindingCall::new("DockCommand.ActivateMainPage")
                    .with_argument(UiBindingValue::string(page_id))
            }
            Self::SetDrawerMode { slot, mode } => UiBindingCall::new("DockCommand.SetDrawerMode")
                .with_argument(UiBindingValue::string(slot))
                .with_argument(UiBindingValue::string(mode)),
            Self::SetDrawerExtent { slot, extent } => {
                UiBindingCall::new("DockCommand.SetDrawerExtent")
                    .with_argument(UiBindingValue::string(slot))
                    .with_argument(UiBindingValue::Float(*extent as f64))
            }
            Self::SavePreset { name } => UiBindingCall::new("DockCommand.SavePreset")
                .with_argument(UiBindingValue::string(name)),
            Self::LoadPreset { name } => UiBindingCall::new("DockCommand.LoadPreset")
                .with_argument(UiBindingValue::string(name)),
            Self::ResetToDefault => UiBindingCall::new("DockCommand.ResetToDefault"),
        }
    }

    fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "DockCommand.FocusView" => Self::FocusView {
                instance_id: required_string_argument(&call, 0, "DockCommand.FocusView")?,
            },
            "DockCommand.CloseView" => Self::CloseView {
                instance_id: required_string_argument(&call, 0, "DockCommand.CloseView")?,
            },
            "DockCommand.AttachViewToDrawer" => Self::AttachViewToDrawer {
                instance_id: required_string_argument(&call, 0, "DockCommand.AttachViewToDrawer")?,
                slot: required_string_argument(&call, 1, "DockCommand.AttachViewToDrawer")?,
            },
            "DockCommand.AttachViewToDocument" => Self::AttachViewToDocument {
                instance_id: required_string_argument(
                    &call,
                    0,
                    "DockCommand.AttachViewToDocument",
                )?,
                page_id: required_string_argument(
                    &call,
                    1,
                    "DockCommand.AttachViewToDocument",
                )?,
            },
            "DockCommand.DetachViewToWindow" => Self::DetachViewToWindow {
                instance_id: required_string_argument(&call, 0, "DockCommand.DetachViewToWindow")?,
                window_id: required_string_argument(&call, 1, "DockCommand.DetachViewToWindow")?,
            },
            "DockCommand.ActivateDrawerTab" => Self::ActivateDrawerTab {
                slot: required_string_argument(&call, 0, "DockCommand.ActivateDrawerTab")?,
                instance_id: required_string_argument(&call, 1, "DockCommand.ActivateDrawerTab")?,
            },
            "DockCommand.ActivateMainPage" => Self::ActivateMainPage {
                page_id: required_string_argument(&call, 0, "DockCommand.ActivateMainPage")?,
            },
            "DockCommand.SetDrawerMode" => Self::SetDrawerMode {
                slot: required_string_argument(&call, 0, "DockCommand.SetDrawerMode")?,
                mode: required_string_argument(&call, 1, "DockCommand.SetDrawerMode")?,
            },
            "DockCommand.SetDrawerExtent" => Self::SetDrawerExtent {
                slot: required_string_argument(&call, 0, "DockCommand.SetDrawerExtent")?,
                extent: required_f32_argument(&call, 1, "DockCommand.SetDrawerExtent")?,
            },
            "DockCommand.SavePreset" => Self::SavePreset {
                name: required_string_argument(&call, 0, "DockCommand.SavePreset")?,
            },
            "DockCommand.LoadPreset" => Self::LoadPreset {
                name: required_string_argument(&call, 0, "DockCommand.LoadPreset")?,
            },
            "DockCommand.ResetToDefault" => Self::ResetToDefault,
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewportCommand {
    PointerMoved { x: f32, y: f32 },
    LeftPressed { x: f32, y: f32 },
    LeftReleased,
    RightPressed { x: f32, y: f32 },
    RightReleased,
    MiddlePressed { x: f32, y: f32 },
    MiddleReleased,
    Scrolled { delta: f32 },
    Resized { width: u32, height: u32 },
}

impl ViewportCommand {
    fn to_call(&self) -> UiBindingCall {
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
                .with_argument(UiBindingValue::Unsigned(*width as u64))
                .with_argument(UiBindingValue::Unsigned(*height as u64)),
        }
    }

    fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
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
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorUiBindingPayload {
    PositionOfTrackAndFrame {
        track_path: String,
        frame: u32,
    },
    MenuAction {
        action_id: String,
    },
    InspectorFieldBatch {
        subject_path: String,
        changes: Vec<InspectorFieldChange>,
    },
    SelectionCommand(SelectionCommand),
    AssetCommand(AssetCommand),
    DockCommand(DockCommand),
    ViewportCommand(ViewportCommand),
    Custom(UiBindingCall),
}

impl EditorUiBindingPayload {
    pub fn position_of_track_and_frame(track_path: impl Into<String>, frame: u32) -> Self {
        Self::PositionOfTrackAndFrame {
            track_path: track_path.into(),
            frame,
        }
    }

    pub fn menu_action(action_id: impl Into<String>) -> Self {
        Self::MenuAction {
            action_id: action_id.into(),
        }
    }

    pub fn inspector_field_batch(
        subject_path: impl Into<String>,
        changes: impl Into<Vec<InspectorFieldChange>>,
    ) -> Self {
        Self::InspectorFieldBatch {
            subject_path: subject_path.into(),
            changes: changes.into(),
        }
    }

    pub fn dock_command(command: DockCommand) -> Self {
        Self::DockCommand(command)
    }

    pub fn selection_command(command: SelectionCommand) -> Self {
        Self::SelectionCommand(command)
    }

    pub fn asset_command(command: AssetCommand) -> Self {
        Self::AssetCommand(command)
    }

    pub fn viewport_command(command: ViewportCommand) -> Self {
        Self::ViewportCommand(command)
    }

    fn to_call(&self) -> UiBindingCall {
        match self {
            Self::PositionOfTrackAndFrame { track_path, frame } => {
                UiBindingCall::new("PositionOfTrackAndFrame")
                    .with_argument(UiBindingValue::string(track_path))
                    .with_argument(UiBindingValue::unsigned(*frame))
            }
            Self::MenuAction { action_id } => {
                UiBindingCall::new("MenuAction").with_argument(UiBindingValue::string(action_id))
            }
            Self::InspectorFieldBatch {
                subject_path,
                changes,
            } => UiBindingCall::new("InspectorFieldBatch")
                .with_argument(UiBindingValue::string(subject_path))
                .with_argument(UiBindingValue::array(
                    changes
                        .iter()
                        .map(InspectorFieldChange::as_binding_value)
                        .collect::<Vec<_>>(),
                )),
            Self::SelectionCommand(command) => command.to_call(),
            Self::AssetCommand(command) => command.to_call(),
            Self::DockCommand(command) => command.to_call(),
            Self::ViewportCommand(command) => command.to_call(),
            Self::Custom(call) => call.clone(),
        }
    }

    fn from_call(call: UiBindingCall) -> Result<Self, EditorUiBindingError> {
        if let Some(command) = SelectionCommand::from_call(call.clone())? {
            return Ok(Self::SelectionCommand(command));
        }
        if let Some(command) = AssetCommand::from_call(call.clone())? {
            return Ok(Self::AssetCommand(command));
        }
        if let Some(command) = DockCommand::from_call(call.clone())? {
            return Ok(Self::DockCommand(command));
        }
        if let Some(command) = ViewportCommand::from_call(call.clone())? {
            return Ok(Self::ViewportCommand(command));
        }
        match call.symbol.as_str() {
            "PositionOfTrackAndFrame" => Ok(Self::PositionOfTrackAndFrame {
                track_path: call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "PositionOfTrackAndFrame expects string track_path".to_string(),
                    ))?
                    .to_string(),
                frame: call.argument(1).and_then(UiBindingValue::as_u32).ok_or(
                    EditorUiBindingError::InvalidPayload(
                        "PositionOfTrackAndFrame expects u32 frame".to_string(),
                    ),
                )?,
            }),
            "MenuAction" => Ok(Self::MenuAction {
                action_id: call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "MenuAction expects string action_id".to_string(),
                    ))?
                    .to_string(),
            }),
            "InspectorFieldBatch" => {
                let subject_path = call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "InspectorFieldBatch expects string subject_path".to_string(),
                    ))?
                    .to_string();
                let changes = match call.argument(1) {
                    Some(UiBindingValue::Array(values)) => values
                        .iter()
                        .map(InspectorFieldChange::from_binding_value)
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => {
                        return Err(EditorUiBindingError::InvalidPayload(
                            "InspectorFieldBatch expects [field_id,value] pairs".to_string(),
                        ))
                    }
                };
                Ok(Self::InspectorFieldBatch {
                    subject_path,
                    changes,
                })
            }
            _ => Ok(Self::Custom(call)),
        }
    }
}

fn required_string_argument(
    call: &UiBindingCall,
    index: usize,
    symbol: &str,
) -> Result<String, EditorUiBindingError> {
    call.argument(index)
        .and_then(UiBindingValue::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            EditorUiBindingError::InvalidPayload(format!(
                "{symbol} expects string argument at index {index}"
            ))
        })
}

fn required_u32_argument(
    call: &UiBindingCall,
    index: usize,
    symbol: &str,
) -> Result<u32, EditorUiBindingError> {
    call.argument(index)
        .and_then(UiBindingValue::as_u32)
        .ok_or_else(|| {
            EditorUiBindingError::InvalidPayload(format!(
                "{symbol} expects unsigned argument at index {index}"
            ))
        })
}

fn required_f32_argument(
    call: &UiBindingCall,
    index: usize,
    symbol: &str,
) -> Result<f32, EditorUiBindingError> {
    let value = call.argument(index).ok_or_else(|| {
        EditorUiBindingError::InvalidPayload(format!(
            "{symbol} expects numeric argument at index {index}"
        ))
    })?;
    match value {
        UiBindingValue::Float(value) => Ok(*value as f32),
        UiBindingValue::Unsigned(value) => Ok(*value as f32),
        UiBindingValue::Signed(value) => Ok(*value as f32),
        _ => Err(EditorUiBindingError::InvalidPayload(format!(
            "{symbol} expects numeric argument at index {index}"
        ))),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorUiBinding {
    path: UiEventPath,
    payload: EditorUiBindingPayload,
}

impl EditorUiBinding {
    pub fn new(
        view_id: impl Into<String>,
        control_id: impl Into<String>,
        event_kind: EditorUiEventKind,
        payload: EditorUiBindingPayload,
    ) -> Self {
        Self {
            path: UiEventPath::new(view_id, control_id, event_kind),
            payload,
        }
    }

    pub fn path(&self) -> &UiEventPath {
        &self.path
    }

    pub fn payload(&self) -> &EditorUiBindingPayload {
        &self.payload
    }

    pub fn as_ui_binding(&self) -> UiEventBinding {
        UiEventBinding::new(self.path.clone(), self.payload.to_call())
    }

    pub fn from_ui_binding(binding: UiEventBinding) -> Result<Self, EditorUiBindingError> {
        let payload = binding
            .action
            .ok_or_else(|| EditorUiBindingError::InvalidPayload("missing binding action".into()))
            .and_then(EditorUiBindingPayload::from_call)?;
        Ok(Self {
            path: binding.path,
            payload,
        })
    }

    pub fn with_arguments(
        &self,
        arguments: Vec<UiBindingValue>,
    ) -> Result<Self, EditorUiBindingError> {
        let mut binding = self.as_ui_binding();
        let action = binding
            .action
            .as_mut()
            .ok_or_else(|| EditorUiBindingError::InvalidPayload("missing binding action".into()))?;
        action.arguments = arguments;
        Self::from_ui_binding(binding)
    }

    pub fn native_binding(&self) -> String {
        self.as_ui_binding().native_binding()
    }

    pub fn parse_native_binding(input: &str) -> Result<Self, EditorUiBindingError> {
        Self::from_ui_binding(UiEventBinding::parse_native_binding(input)?)
    }
}

#[derive(Debug, Error)]
pub enum EditorUiBindingError {
    #[error(transparent)]
    Parse(#[from] UiBindingParseError),
    #[error("invalid editor ui payload: {0}")]
    InvalidPayload(String),
}

type Handler<T> = Box<dyn Fn(&EditorUiBinding) -> T + Send + Sync + 'static>;

pub struct EditorUiRouter<T> {
    exact_routes: BTreeMap<UiEventPath, Vec<Handler<T>>>,
}

impl<T> Default for EditorUiRouter<T> {
    fn default() -> Self {
        Self {
            exact_routes: BTreeMap::new(),
        }
    }
}

impl<T> EditorUiRouter<T> {
    pub fn register_exact<F>(&mut self, path: UiEventPath, handler: F)
    where
        F: Fn(&EditorUiBinding) -> T + Send + Sync + 'static,
    {
        self.exact_routes
            .entry(path)
            .or_default()
            .push(Box::new(handler));
    }

    pub fn dispatch(&self, binding: &EditorUiBinding) -> Vec<T> {
        self.exact_routes
            .get(binding.path())
            .into_iter()
            .flat_map(|handlers| handlers.iter())
            .map(|handler| handler(binding))
            .collect()
    }
}
