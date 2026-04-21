use crate::core::editor_event::{LayoutCommand, MainPageId, ViewHost, ViewInstanceId};
use crate::ui::binding::{DockCommand, EditorUiBinding, EditorUiBindingPayload};

use super::super::error::EditorBindingDispatchError;
use super::drawer_mode::parse_drawer_mode;
use super::drawer_slot::parse_drawer_slot;

pub fn dispatch_docking_binding(
    binding: &EditorUiBinding,
) -> Result<LayoutCommand, EditorBindingDispatchError> {
    let EditorUiBindingPayload::DockCommand(command) = binding.payload() else {
        return Err(EditorBindingDispatchError::UnsupportedPayload);
    };

    match command {
        DockCommand::FocusView { instance_id } => Ok(LayoutCommand::FocusView {
            instance_id: ViewInstanceId::new(instance_id),
        }),
        DockCommand::CloseView { instance_id } => Ok(LayoutCommand::CloseView {
            instance_id: ViewInstanceId::new(instance_id),
        }),
        DockCommand::AttachViewToDrawer { instance_id, slot } => Ok(LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new(instance_id),
            target: ViewHost::Drawer(parse_drawer_slot(slot)?),
            anchor: None,
        }),
        DockCommand::AttachViewToDocument {
            instance_id,
            page_id,
        } => Ok(LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new(instance_id),
            target: ViewHost::Document(MainPageId::new(page_id), Vec::new()),
            anchor: None,
        }),
        DockCommand::DetachViewToWindow {
            instance_id,
            window_id,
        } => Ok(LayoutCommand::DetachViewToWindow {
            instance_id: ViewInstanceId::new(instance_id),
            new_window: MainPageId::new(window_id),
        }),
        DockCommand::ActivateDrawerTab { slot, instance_id } => {
            Ok(LayoutCommand::ActivateDrawerTab {
                slot: parse_drawer_slot(slot)?,
                instance_id: ViewInstanceId::new(instance_id),
            })
        }
        DockCommand::ActivateMainPage { page_id } => Ok(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new(page_id),
        }),
        DockCommand::SetDrawerMode { slot, mode } => Ok(LayoutCommand::SetDrawerMode {
            slot: parse_drawer_slot(slot)?,
            mode: parse_drawer_mode(mode)?,
        }),
        DockCommand::SetDrawerExtent { slot, extent } => Ok(LayoutCommand::SetDrawerExtent {
            slot: parse_drawer_slot(slot)?,
            extent: *extent,
        }),
        DockCommand::SavePreset { name } => Ok(LayoutCommand::SavePreset { name: name.clone() }),
        DockCommand::LoadPreset { name } => Ok(LayoutCommand::LoadPreset { name: name.clone() }),
        DockCommand::ResetToDefault => Ok(LayoutCommand::ResetToDefault),
    }
}
