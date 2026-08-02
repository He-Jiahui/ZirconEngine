use serde::{Deserialize, Serialize};

use crate::core::commands::DocumentKind;
use crate::core::extension::{DefaultWorkbenchPreset, WorkbenchSlot};
use crate::ui::workbench::autolayout::PaneConstraints;

use super::{ActivityWindowTemplateSpec, DockPolicy, PaneTemplateSpec, ViewDescriptorId, ViewKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewDescriptor {
    pub descriptor_id: ViewDescriptorId,
    pub kind: ViewKind,
    pub default_title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub dock_policy: DockPolicy,
    pub workbench_slot: WorkbenchSlot,
    pub default_presets: Vec<DefaultWorkbenchPreset>,
    pub persistence_key_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_kind: Option<DocumentKind>,
    #[serde(default)]
    pub default_constraints: PaneConstraints,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pane_template: Option<PaneTemplateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_window_template: Option<ActivityWindowTemplateSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
}

impl ViewDescriptor {
    pub fn new(
        descriptor_id: ViewDescriptorId,
        kind: ViewKind,
        default_title: impl Into<String>,
    ) -> Self {
        let descriptor_key = descriptor_id.0.clone();
        Self {
            descriptor_id,
            kind,
            default_title: default_title.into(),
            icon_key: descriptor_key.clone(),
            multi_instance: false,
            dock_policy: DockPolicy::DrawerOrDocument,
            workbench_slot: WorkbenchSlot::DocumentCenter,
            default_presets: vec![DefaultWorkbenchPreset::Authoring],
            persistence_key_policy: descriptor_key,
            document_kind: None,
            default_constraints: PaneConstraints::default(),
            pane_template: None,
            activity_window_template: None,
            required_capabilities: Vec::new(),
        }
    }
}
