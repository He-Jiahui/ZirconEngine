use crate::core::commands::CommandEvalCtx;
use crate::core::extension::CapabilitySet;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

impl EditorHostEventController {
    pub(crate) fn active_activity_window_template_document_is(
        &self,
        document_id: &str,
    ) -> Result<bool, crate::ui::host::EditorError> {
        Ok({
            self.enter_editor_manager()?
                .active_activity_window_template_document_is(document_id)
        })
    }

    pub(crate) fn floating_window_focus_target(
        &self,
        window_id: &crate::ui::workbench::layout::MainPageId,
    ) -> Result<Option<crate::ui::workbench::view::ViewInstanceId>, crate::ui::host::EditorError>
    {
        Ok({
            self.enter_editor_manager()?
                .floating_window_focus_target(window_id)
        })
    }

    pub(crate) fn floating_window_id_for_surface_key(
        &self,
        surface_key: &str,
    ) -> Result<Option<MainPageId>, crate::ui::host::EditorError> {
        Ok({
            self.enter_editor_manager()?
                .floating_window_id_for_surface_key(surface_key)
        })
    }

    pub(crate) fn active_drawer_toggle_state(
        &self,
        slot: ActivityDrawerSlot,
        instance_id: &ViewInstanceId,
    ) -> Result<(ActivityDrawerMode, bool, bool), String> {
        self.enter_editor_manager()
            .map_err(|error| error.to_string())?
            .active_drawer_toggle_state(slot, instance_id)
    }

    pub(crate) fn active_drawer_mode(
        &self,
        slot: ActivityDrawerSlot,
    ) -> Result<Option<ActivityDrawerMode>, crate::ui::host::EditorError> {
        Ok({ self.enter_editor_manager()?.active_drawer_mode(slot) })
    }

    pub(crate) fn floating_window_exists(
        &self,
        window_id: &MainPageId,
    ) -> Result<bool, crate::ui::host::EditorError> {
        Ok({
            self.enter_editor_manager()?
                .floating_window_exists(window_id)
        })
    }

    pub(crate) fn floating_window_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Result<Option<Vec<ViewInstanceId>>, crate::ui::host::EditorError> {
        Ok({
            self.enter_editor_manager()?
                .floating_window_instance_ids(window_id)
        })
    }

    pub(crate) fn view_host_for_instance_key(
        &self,
        surface_key: &str,
    ) -> Result<Option<ViewHost>, crate::ui::host::EditorError> {
        Ok({
            self.enter_editor_manager()?
                .view_host_for_instance_key(surface_key)
        })
    }

    pub(crate) fn current_view_instance_ids(
        &self,
    ) -> Result<Vec<ViewInstanceId>, crate::ui::host::EditorError> {
        Ok({ self.enter_editor_manager()?.current_view_instance_ids() })
    }

    pub(crate) fn view_instance_ids_for_descriptor_key(
        &self,
        descriptor_key: &str,
    ) -> Result<Vec<ViewInstanceId>, crate::ui::host::EditorError> {
        Ok({
            self.enter_editor_manager()?
                .view_instance_ids_for_descriptor_key(descriptor_key)
        })
    }

    pub(crate) fn editor_pane_instance_ids(
        &self,
        collect_ui_asset_panes: bool,
        collect_animation_panes: bool,
    ) -> Result<(Vec<ViewInstanceId>, Vec<ViewInstanceId>), crate::ui::host::EditorError> {
        Ok({
            self.enter_editor_manager()?
                .editor_pane_instance_ids(collect_ui_asset_panes, collect_animation_panes)
        })
    }

    pub(crate) fn build_workbench_view_model(
        &self,
        chrome: &EditorChromeSnapshot,
        context: &CommandEvalCtx,
    ) -> Result<WorkbenchViewModel, crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        Ok({
            let (keymap, contributions, capabilities, focused_toolkit) = {
                let inner = self.shell().lock();
                let capabilities = manager
                    .capability_snapshot()
                    .enabled_capabilities()
                    .iter()
                    .cloned()
                    .collect::<CapabilitySet>();
                (
                    manager.keymap(),
                    inner.contributions.snapshot(),
                    capabilities,
                    manager.focused_document_toolkit(),
                )
            };
            let commands = self.commands().lock();
            let i18n = self.context().i18n();
            let locale = i18n.active_locale();
            WorkbenchViewModel::build_with_contributions_and_context(
                &commands,
                &keymap,
                i18n,
                &locale,
                chrome,
                &contributions,
                &capabilities,
                focused_toolkit.as_ref(),
                context,
            )
        })
    }
}
