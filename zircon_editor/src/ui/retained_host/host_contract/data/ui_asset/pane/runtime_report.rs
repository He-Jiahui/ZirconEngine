use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetRuntimeReportData {
    pub action_policy_items: ModelRc<SharedString>,
    pub capability_explanation_items: ModelRc<SharedString>,
    pub host_enforcement_items: ModelRc<SharedString>,
    pub unsafe_action_guidance_items: ModelRc<SharedString>,
    pub locale_preview: UiAssetStringSelectionData,
    pub locale_preview_selected_locale: SharedString,
    pub locale_dependency_items: ModelRc<SharedString>,
    pub locale_extraction_items: ModelRc<SharedString>,
    pub locale_diagnostic_items: ModelRc<SharedString>,
    pub resource_dependency_items: ModelRc<SharedString>,
    pub resource_diagnostic_items: ModelRc<SharedString>,
}
