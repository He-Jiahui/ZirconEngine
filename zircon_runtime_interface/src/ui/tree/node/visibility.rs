use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiVisibility {
    #[default]
    Visible,
    Hidden,
    Collapsed,
    HitTestInvisible,
    SelfHitTestInvisible,
}

impl UiVisibility {
    pub const fn occupies_layout(self) -> bool {
        !matches!(self, Self::Collapsed)
    }

    pub const fn is_render_visible(self) -> bool {
        matches!(
            self,
            Self::Visible | Self::HitTestInvisible | Self::SelfHitTestInvisible
        )
    }

    pub const fn allows_self_hit_test(self) -> bool {
        matches!(self, Self::Visible)
    }

    pub const fn allows_child_hit_test(self) -> bool {
        matches!(self, Self::Visible | Self::SelfHitTestInvisible)
    }

    pub const fn effective(self, legacy_visible: bool) -> Self {
        if !legacy_visible && !matches!(self, Self::Collapsed) {
            Self::Hidden
        } else {
            self
        }
    }
}
