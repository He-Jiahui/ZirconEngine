use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

use super::builtin_layout::ensure_builtin_shell_instances;
use super::editor_error::EditorError;
use super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(super) fn apply_layout_command(&self, cmd: LayoutCommand) -> Result<bool, EditorError> {
        match cmd {
            LayoutCommand::SavePreset { name } => {
                self.save_preset(&name)?;
                return Ok(false);
            }
            LayoutCommand::LoadPreset { name } => {
                return self.load_preset(&name);
            }
            LayoutCommand::ResetToDefault => {
                let mut session = self.session.lock().unwrap();
                let mut registry = self.view_registry.lock().unwrap();
                ensure_builtin_shell_instances(&mut registry, &mut session)?;
                self.animation_editor_sessions.lock().unwrap().clear();
                self.ui_asset_sessions.lock().unwrap().clear();
                session.layout = self.layout_manager.default_layout();
                self.recompute_session_metadata(&mut session);
                return Ok(true);
            }
            _ => {}
        }

        let mut session = self.session.lock().unwrap();
        let diff = self
            .layout_manager
            .apply(&mut session.layout, cmd)
            .map_err(EditorError::Layout)?;
        self.recompute_session_metadata(&mut session);
        Ok(diff.changed)
    }

    pub(super) fn open_view(
        &self,
        descriptor_id: ViewDescriptorId,
        target_host: Option<ViewHost>,
    ) -> Result<ViewInstanceId, EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        let instance = registry
            .open_descriptor(descriptor_id)
            .map_err(EditorError::Registry)?;
        drop(registry);

        let target = target_host.unwrap_or_else(|| instance.host.clone());
        self.attach_instance(instance, target)
    }

    pub(super) fn close_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        if self.non_closeable_instance(instance_id) {
            return Ok(false);
        }
        let changed = self.apply_layout_command(LayoutCommand::CloseView {
            instance_id: instance_id.clone(),
        })?;
        if changed {
            self.session
                .lock()
                .unwrap()
                .open_view_instances
                .remove(instance_id);
            self.animation_editor_sessions
                .lock()
                .unwrap()
                .remove(instance_id);
            self.ui_asset_sessions.lock().unwrap().remove(instance_id);
            self.view_registry
                .lock()
                .unwrap()
                .remove_instance(instance_id);
        }
        Ok(changed)
    }

    pub(super) fn focus_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.apply_layout_command(LayoutCommand::FocusView {
            instance_id: instance_id.clone(),
        })
    }

    pub(super) fn detach_view_to_window(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        let window_id = MainPageId::new(format!("window:{}", instance_id.0));
        let changed = self.apply_layout_command(LayoutCommand::DetachViewToWindow {
            instance_id: instance_id.clone(),
            new_window: window_id.clone(),
        })?;
        if changed {
            self.window_host_manager
                .lock()
                .unwrap()
                .open_native_window(window_id, None);
        }
        Ok(changed)
    }

    pub(super) fn attach_view_to_target(
        &self,
        instance_id: &ViewInstanceId,
        drop_target: ViewHost,
    ) -> Result<bool, EditorError> {
        let previous_floating_window = self
            .session
            .lock()
            .unwrap()
            .open_view_instances
            .get(instance_id)
            .and_then(|instance| match &instance.host {
                ViewHost::FloatingWindow(window_id, _) => Some(window_id.clone()),
                ViewHost::Drawer(_) | ViewHost::Document(_, _) | ViewHost::ExclusivePage(_) => None,
            });
        let changed = self.apply_layout_command(LayoutCommand::AttachView {
            instance_id: instance_id.clone(),
            target: drop_target.clone(),
            anchor: None,
        })?;
        if changed {
            if let Some(window_id) = previous_floating_window {
                let window_still_exists = self
                    .current_layout()
                    .floating_windows
                    .iter()
                    .any(|window| window.window_id == window_id);
                if !window_still_exists {
                    self.window_host_manager
                        .lock()
                        .unwrap()
                        .reattach_window(&window_id, &drop_target);
                }
            }
        }
        Ok(changed)
    }

    pub(super) fn attach_instance(
        &self,
        instance: ViewInstance,
        target: ViewHost,
    ) -> Result<ViewInstanceId, EditorError> {
        {
            let mut session = self.session.lock().unwrap();
            session
                .open_view_instances
                .insert(instance.instance_id.clone(), instance.clone());
        }
        self.apply_layout_command(LayoutCommand::AttachView {
            instance_id: instance.instance_id.clone(),
            target,
            anchor: None,
        })?;
        Ok(instance.instance_id)
    }

    pub(super) fn non_closeable_instance(&self, instance_id: &ViewInstanceId) -> bool {
        self.session
            .lock()
            .unwrap()
            .open_view_instances
            .get(instance_id)
            .is_some_and(|instance| {
                matches!(
                    instance.descriptor_id.0.as_str(),
                    "editor.scene" | "editor.game"
                )
            })
    }
}
