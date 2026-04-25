use serde::{Deserialize, Serialize};

use crate::ui::workbench::autolayout::PaneConstraints;

use super::{
    ActivityWindowTemplateSpec, DockPolicy, PaneTemplateSpec, PreferredHost, ViewDescriptorId,
    ViewKind,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewDescriptor {
    pub descriptor_id: ViewDescriptorId,
    pub kind: ViewKind,
    pub default_title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub dock_policy: DockPolicy,
    pub preferred_drawer_slot: Option<crate::ui::workbench::layout::ActivityDrawerSlot>,
    pub preferred_host: PreferredHost,
    pub persistence_key_policy: String,
    #[serde(default)]
    pub default_constraints: PaneConstraints,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pane_template: Option<PaneTemplateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_window_template: Option<ActivityWindowTemplateSpec>,
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
            preferred_drawer_slot: None,
            preferred_host: PreferredHost::DocumentCenter,
            persistence_key_policy: descriptor_key,
            default_constraints: PaneConstraints::default(),
            pane_template: None,
            activity_window_template: None,
        }
    }
}
