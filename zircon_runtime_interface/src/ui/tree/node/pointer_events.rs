use serde::{Deserialize, Serialize};

/// Declares whether a UI node can become a pointer target and whether its descendants remain hit-testable.
/// 声明节点自身和后代能否进入指针命中路径；实际目标还受可见性与输入策略约束。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiPointerEvents {
    #[default]
    Auto,
    None,
    SelfNone,
    /// Remains hittable; ordinary consumption and default actions pass to ancestors.
    Pass,
}

impl UiPointerEvents {
    pub const fn allows_self_hit_test(self) -> bool {
        !matches!(self, Self::None | Self::SelfNone)
    }

    pub const fn allows_child_hit_test(self) -> bool {
        !matches!(self, Self::None)
    }

    pub const fn is_passthrough(self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// Declares the cursor requested by a node after it wins the pointer hit path.
/// 节点为指针命中声明的预期光标形状。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiCursor {
    #[default]
    Default,
    Pointer,
    Text,
    ResizeEw,
    ResizeNs,
    Grab,
    Grabbing,
}

impl UiCursor {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Pointer => "pointer",
            Self::Text => "text",
            Self::ResizeEw => "resize-ew",
            Self::ResizeNs => "resize-ns",
            Self::Grab => "grab",
            Self::Grabbing => "grabbing",
        }
    }
}
