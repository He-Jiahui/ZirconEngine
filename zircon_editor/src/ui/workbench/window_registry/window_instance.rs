use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::{ActivityWindowHostMode, ActivityWindowId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::{DrawerDockPosition, MenuOverflowMode, WindowKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowInstance {
    pub window_id: ActivityWindowId,
    pub descriptor_id: ViewDescriptorId,
    pub kind: WindowKind,
    pub title: String,
    pub host_mode: ActivityWindowHostMode,
    pub drawer_views: BTreeMap<DrawerDockPosition, Vec<ViewInstanceId>>,
    pub selected_drawer: Option<ViewInstanceId>,
    pub menu_overflow_mode: MenuOverflowMode,
}

impl WindowInstance {
    pub fn new(
        window_id: ActivityWindowId,
        descriptor_id: ViewDescriptorId,
        kind: WindowKind,
        title: impl Into<String>,
        host_mode: ActivityWindowHostMode,
    ) -> Self {
        Self {
            window_id,
            descriptor_id,
            kind,
            title: title.into(),
            host_mode,
            drawer_views: BTreeMap::new(),
            selected_drawer: None,
            menu_overflow_mode: MenuOverflowMode::Auto,
        }
    }

    pub fn with_menu_overflow_mode(mut self, mode: MenuOverflowMode) -> Self {
        self.menu_overflow_mode = mode;
        self
    }

    pub fn drawer_capable(&self) -> bool {
        matches!(
            self.kind,
            WindowKind::DrawerCapable | WindowKind::DrawerWindow
        )
    }
}
