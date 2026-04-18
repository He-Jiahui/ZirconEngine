use crate::layout::{LayoutCommand, MainPageId};
use crate::view::{ViewDescriptorId, ViewHost, ViewInstanceId};
use crate::ui::workbench::startup::{WELCOME_DESCRIPTOR_ID, WELCOME_INSTANCE_ID, WELCOME_PAGE_ID};

use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;
use super::welcome_view::{welcome_view_descriptor, welcome_view_instance};

impl EditorManager {
    pub(crate) fn show_welcome_page(&self) -> Result<(), EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        if registry
            .descriptor(&ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID))
            .is_none()
        {
            registry
                .register_view(welcome_view_descriptor())
                .map_err(EditorError::Registry)?;
        }
        let instance = if let Some(existing) = registry
            .instance(&ViewInstanceId::new(WELCOME_INSTANCE_ID))
            .cloned()
        {
            existing
        } else {
            registry
                .restore_instance(welcome_view_instance())
                .map_err(EditorError::Registry)?
        };
        drop(registry);

        self.attach_instance(
            instance,
            ViewHost::ExclusivePage(MainPageId::new(WELCOME_PAGE_ID)),
        )?;
        self.apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new(WELCOME_PAGE_ID),
        })?;
        Ok(())
    }

    pub(crate) fn dismiss_welcome_page(&self) -> Result<(), EditorError> {
        let _ = self.close_view(&ViewInstanceId::new(WELCOME_INSTANCE_ID));
        let _ = self.apply_layout_command(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::workbench(),
        })?;
        Ok(())
    }
}
