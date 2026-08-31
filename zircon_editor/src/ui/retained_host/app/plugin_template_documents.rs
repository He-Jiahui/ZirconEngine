use crate::ui::template_runtime::EditorUiHostRuntimeError;

use super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(super) fn sync_plugin_template_documents_if_changed(
        &mut self,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let (generation, enabled_capabilities) = self.runtime.extension_projection_revision();
        if self.plugin_template_generation == generation
            && self.plugin_template_capabilities == enabled_capabilities
        {
            return Ok(());
        }

        let (generation, enabled_capabilities, templates_by_owner) =
            self.runtime.enabled_plugin_template_descriptors();
        self.builtin_template_runtime
            .sync_plugin_v2_template_descriptor_sets(&templates_by_owner)?;

        // A consumer advances its accepted revision only after materialization succeeds.
        self.plugin_template_generation = generation;
        self.plugin_template_capabilities = enabled_capabilities;
        self.mark_presentation_dirty();
        Ok(())
    }
}
