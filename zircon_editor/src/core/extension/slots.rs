use serde::{Deserialize, Serialize};

/// Stable workbench placement declared by built-in and plugin contributions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkbenchSlot {
    LeftTopDrawer,
    LeftBottomDrawer,
    RightTopDrawer,
    RightBottomDrawer,
    BottomDrawer,
    #[default]
    DocumentCenter,
    FloatingWindow,
    ExclusiveMainPage,
}

impl WorkbenchSlot {
    pub const fn is_drawer(self) -> bool {
        matches!(
            self,
            Self::LeftTopDrawer
                | Self::LeftBottomDrawer
                | Self::RightTopDrawer
                | Self::RightBottomDrawer
                | Self::BottomDrawer
        )
    }
}

/// Built-in workbench presets that a contribution may opt into by default.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum DefaultWorkbenchPreset {
    #[default]
    Authoring,
    Review,
    Focus,
    Debug,
}

impl DefaultWorkbenchPreset {
    /// Produces the canonical, deterministic preset declaration shared by every contribution.
    pub fn normalize(presets: impl IntoIterator<Item = Self>) -> Vec<Self> {
        let mut presets = presets.into_iter().collect::<Vec<_>>();
        presets.sort_unstable();
        presets.dedup();
        presets
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultWorkbenchPreset, WorkbenchSlot};

    #[test]
    fn drawer_classification_is_exhaustive_and_document_center_is_default() {
        assert_eq!(WorkbenchSlot::default(), WorkbenchSlot::DocumentCenter);
        for slot in [
            WorkbenchSlot::LeftTopDrawer,
            WorkbenchSlot::LeftBottomDrawer,
            WorkbenchSlot::RightTopDrawer,
            WorkbenchSlot::RightBottomDrawer,
            WorkbenchSlot::BottomDrawer,
        ] {
            assert!(slot.is_drawer());
        }
        for slot in [
            WorkbenchSlot::DocumentCenter,
            WorkbenchSlot::FloatingWindow,
            WorkbenchSlot::ExclusiveMainPage,
        ] {
            assert!(!slot.is_drawer());
        }
    }

    #[test]
    fn default_preset_names_are_finite_and_stably_ordered() {
        assert!(DefaultWorkbenchPreset::Authoring < DefaultWorkbenchPreset::Review);
        assert!(DefaultWorkbenchPreset::Review < DefaultWorkbenchPreset::Focus);
        assert!(DefaultWorkbenchPreset::Focus < DefaultWorkbenchPreset::Debug);
    }

    #[test]
    fn default_preset_declarations_are_normalized_by_the_finite_owner() {
        assert_eq!(
            DefaultWorkbenchPreset::normalize([
                DefaultWorkbenchPreset::Debug,
                DefaultWorkbenchPreset::Authoring,
                DefaultWorkbenchPreset::Focus,
                DefaultWorkbenchPreset::Debug,
            ]),
            vec![
                DefaultWorkbenchPreset::Authoring,
                DefaultWorkbenchPreset::Focus,
                DefaultWorkbenchPreset::Debug,
            ]
        );
    }
}
