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
                let mut session = self.lock_session();
                let mut registry = self.lock_view_registry();
                let snapshot = self.lock_capability_snapshot().clone();
                ensure_builtin_shell_instances(&mut registry, &mut session, &snapshot)?;
                self.lock_animation_editor_sessions().clear();
                self.lock_ui_asset_sessions().clear();
                session.layout = self.layout_manager.default_layout();
                self.recompute_session_metadata(&mut session);
                return Ok(true);
            }
            _ => {}
        }

        let mut session = self.lock_session();
        let detached_window_title = match &cmd {
            LayoutCommand::DetachViewToWindow {
                instance_id,
                new_window,
            } => session
                .open_view_instances
                .get(instance_id)
                .map(|instance| (new_window.clone(), instance.title.clone())),
            _ => None,
        };
        let diff = self
            .layout_manager
            .apply(&mut session.layout, cmd)
            .map_err(EditorError::Layout)?;
        session
            .layout
            .sync_legacy_drawers_from_active_activity_window();
        if let Some((window_id, title)) = detached_window_title {
            if let Some(window) = session
                .layout
                .floating_windows
                .iter_mut()
                .find(|window| window.window_id == window_id)
            {
                window.title = title;
            }
        }
        self.recompute_session_metadata(&mut session);
        Ok(diff.changed)
    }

    pub(super) fn open_view(
        &self,
        descriptor_id: ViewDescriptorId,
        target_host: Option<ViewHost>,
    ) -> Result<ViewInstanceId, EditorError> {
        let mut registry = self.lock_view_registry();
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
            self.lock_session().open_view_instances.remove(instance_id);
            self.lock_animation_editor_sessions().remove(instance_id);
            self.lock_ui_asset_sessions().remove(instance_id);
            self.lock_view_registry().remove_instance(instance_id);
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
            self.lock_window_host_manager()
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
            .lock_session()
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
                    self.lock_window_host_manager()
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
            let mut session = self.lock_session();
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
        self.lock_session()
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
