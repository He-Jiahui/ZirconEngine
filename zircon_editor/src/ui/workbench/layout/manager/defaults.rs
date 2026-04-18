use super::super::{LayoutManager, WorkbenchLayout};

impl LayoutManager {
    pub fn default_layout(&self) -> WorkbenchLayout {
        crate::core::host::manager::builtin_hybrid_layout()
    }
}
