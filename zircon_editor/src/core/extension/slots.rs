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
    const ORDERED: [Self; 4] = [Self::Authoring, Self::Review, Self::Focus, Self::Debug];
    const COUNT: usize = Self::ORDERED.len();

    /// Produces the canonical, deterministic preset declaration shared by every contribution.
    /// 描述符与工作台共享此规范顺序；重复预设只显示一次。
    pub fn normalize(presets: impl IntoIterator<Item = Self>) -> Vec<Self> {
        let mut present = [false; Self::COUNT];
        for preset in presets {
            present[preset.index()] = true;
        }
        let present_count = present.iter().filter(|&&is_present| is_present).count();
        let mut normalized = Vec::with_capacity(present_count);
        for (preset, is_present) in Self::ORDERED.into_iter().zip(present) {
            if is_present {
                normalized.push(preset);
            }
        }
        normalized
    }

    const fn index(self) -> usize {
        match self {
            Self::Authoring => 0,
            Self::Review => 1,
            Self::Focus => 2,
            Self::Debug => 3,
        }
    }
}

#[cfg(test)]
#[path = "tests/slots.rs"]
mod tests;

#[cfg(test)]
#[path = "slots/tests/finite_normalize_tests.rs"]
mod finite_normalize_tests;
