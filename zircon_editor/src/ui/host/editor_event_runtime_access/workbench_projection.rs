use crate::core::commands::CommandEvalCtx;
use crate::core::extension::CapabilitySet;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

impl EditorHostEventController {
    pub(crate) fn active_activity_window_template_document_is(&self, document_id: &str) -> bool {
        self.shell()
            .lock()
            .manager
            .active_activity_window_template_document_is(document_id)
    }

    pub(crate) fn floating_window_focus_target(
        &self,
        window_id: &crate::ui::workbench::layout::MainPageId,
    ) -> Option<crate::ui::workbench::view::ViewInstanceId> {
        self.shell()
            .lock()
            .manager
            .floating_window_focus_target(window_id)
    }

    pub(crate) fn floating_window_id_for_surface_key(
        &self,
        surface_key: &str,
    ) -> Option<MainPageId> {
        self.shell()
            .lock()
            .manager
            .floating_window_id_for_surface_key(surface_key)
    }

    pub(crate) fn active_drawer_toggle_state(
        &self,
        slot: ActivityDrawerSlot,
        instance_id: &ViewInstanceId,
    ) -> Result<(ActivityDrawerMode, bool, bool), String> {
        self.shell()
            .lock()
            .manager
            .active_drawer_toggle_state(slot, instance_id)
    }

    pub(crate) fn active_drawer_mode(
        &self,
        slot: ActivityDrawerSlot,
    ) -> Option<ActivityDrawerMode> {
        self.shell().lock().manager.active_drawer_mode(slot)
    }

    pub(crate) fn floating_window_exists(&self, window_id: &MainPageId) -> bool {
        self.shell()
            .lock()
            .manager
            .floating_window_exists(window_id)
    }

    pub(crate) fn floating_window_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Option<Vec<ViewInstanceId>> {
        self.shell()
            .lock()
            .manager
            .floating_window_instance_ids(window_id)
    }

    pub(crate) fn view_host_for_instance_key(&self, surface_key: &str) -> Option<ViewHost> {
        self.shell()
            .lock()
            .manager
            .view_host_for_instance_key(surface_key)
    }

    pub(crate) fn current_view_instance_ids(&self) -> Vec<ViewInstanceId> {
        self.shell().lock().manager.current_view_instance_ids()
    }

    pub(crate) fn view_instance_ids_for_descriptor_key(
        &self,
        descriptor_key: &str,
    ) -> Vec<ViewInstanceId> {
        self.shell()
            .lock()
            .manager
            .view_instance_ids_for_descriptor_key(descriptor_key)
    }

    pub(crate) fn editor_pane_instance_ids(
        &self,
        collect_ui_asset_panes: bool,
        collect_animation_panes: bool,
    ) -> (Vec<ViewInstanceId>, Vec<ViewInstanceId>) {
        self.shell()
            .lock()
            .manager
            .editor_pane_instance_ids(collect_ui_asset_panes, collect_animation_panes)
    }

    pub(crate) fn build_workbench_view_model(
        &self,
        chrome: &EditorChromeSnapshot,
        context: &CommandEvalCtx,
    ) -> WorkbenchViewModel {
        let (keymap, contributions, capabilities, focused_toolkit) = {
            let inner = self.shell().lock();
            let capabilities = inner
                .manager
                .capability_snapshot()
                .enabled_capabilities()
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            (
                inner.manager.keymap(),
                inner.contributions.snapshot(),
                capabilities,
                inner.manager.focused_document_toolkit(),
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
    }
}
