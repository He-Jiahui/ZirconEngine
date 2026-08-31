use crate::core::commands::DocumentKind;
use crate::core::extension::{DefaultWorkbenchPreset, WorkbenchSlot};
use crate::ui::workbench::autolayout::PaneConstraints;

use super::{ActivityWindowTemplateSpec, DockPolicy, PaneTemplateSpec, ViewDescriptor};

const ORDERED_DEFAULT_PRESETS: [DefaultWorkbenchPreset; 4] = [
    DefaultWorkbenchPreset::Authoring,
    DefaultWorkbenchPreset::Review,
    DefaultWorkbenchPreset::Focus,
    DefaultWorkbenchPreset::Debug,
];

impl ViewDescriptor {
    pub fn with_document_kind(mut self, document_kind: DocumentKind) -> Self {
        self.document_kind = Some(document_kind);
        self
    }

    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_dock_policy(mut self, dock_policy: DockPolicy) -> Self {
        self.dock_policy = dock_policy;
        self
    }

    pub fn with_workbench_slot(mut self, workbench_slot: WorkbenchSlot) -> Self {
        self.workbench_slot = workbench_slot;
        self
    }

    pub fn with_default_presets(
        mut self,
        presets: impl IntoIterator<Item = DefaultWorkbenchPreset>,
    ) -> Self {
        self.default_presets = normalize_default_presets(presets);
        self
    }

    pub fn with_icon_key(mut self, icon_key: impl Into<String>) -> Self {
        self.icon_key = icon_key.into();
        self
    }

    pub fn with_default_constraints(mut self, constraints: PaneConstraints) -> Self {
        self.default_constraints = constraints;
        self
    }

    pub fn with_pane_template(mut self, pane_template: PaneTemplateSpec) -> Self {
        self.pane_template = Some(pane_template);
        self
    }

    pub fn with_activity_window_template(
        mut self,
        activity_window_template: ActivityWindowTemplateSpec,
    ) -> Self {
        self.activity_window_template = Some(activity_window_template);
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }
}

fn normalize_default_presets(
    presets: impl IntoIterator<Item = DefaultWorkbenchPreset>,
) -> Vec<DefaultWorkbenchPreset> {
    let mut present = [false; ORDERED_DEFAULT_PRESETS.len()];
    for preset in presets {
        let index = match preset {
            DefaultWorkbenchPreset::Authoring => 0,
            DefaultWorkbenchPreset::Review => 1,
            DefaultWorkbenchPreset::Focus => 2,
            DefaultWorkbenchPreset::Debug => 3,
        };
        present[index] = true;
    }

    let mut normalized = Vec::with_capacity(present.iter().filter(|&&value| value).count());
    for (preset, is_present) in ORDERED_DEFAULT_PRESETS.into_iter().zip(present) {
        if is_present {
            normalized.push(preset);
        }
    }
    normalized
}

#[cfg(test)]
#[path = "view_descriptor_builder/finite_preset_tests.rs"]
mod finite_preset_tests;
