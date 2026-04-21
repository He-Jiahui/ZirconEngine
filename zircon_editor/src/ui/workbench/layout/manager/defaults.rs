use super::super::{LayoutManager, WorkbenchLayout};

impl LayoutManager {
    pub fn default_layout(&self) -> WorkbenchLayout {
        crate::ui::host::builtin_hybrid_layout()
    }
}
