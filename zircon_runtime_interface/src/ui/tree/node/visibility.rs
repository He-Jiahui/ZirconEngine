use serde::{Deserialize, Serialize};

/// 同时定义布局占位、绘制与命中的可见性语义，Runtime 对三个阶段使用不同判定。
/// `Hidden` 保留占位，`Collapsed` 移出布局；两种 HitTestInvisible 仍绘制，分别屏蔽整支或仅自身命中。
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

    /// 合并旧版 `visible` 位；旧位为 false 时仍保留显式 `Collapsed` 的布局语义。
    pub const fn effective(self, legacy_visible: bool) -> Self {
        if !legacy_visible && !matches!(self, Self::Collapsed) {
            Self::Hidden
        } else {
            self
        }
    }
}
