use crate::core::commands::CommandEvalCtx;
use crate::core::extension::CapabilitySet;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

impl EditorHostEventController {
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
