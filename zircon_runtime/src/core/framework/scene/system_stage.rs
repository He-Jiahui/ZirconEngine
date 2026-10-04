use serde::{Deserialize, Serialize};

/// Stable runtime stage contract shared by the lifecycle kernel, plugins, and scene scheduler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemStage {
    First,
    PreUpdate,
    FixedFirst,
    FixedUpdate,
    FixedPostUpdate,
    Update,
    PostUpdate,
    Last,
    RenderExtract,
}

impl SystemStage {
    pub const COUNT: usize = 9;
    // 此顺序同时规定 Native Host API 的 u32 阶段编号；重排会改变既有插件的阶段含义。
    pub const ORDER: [Self; Self::COUNT] = [
        Self::First,
        Self::PreUpdate,
        Self::FixedFirst,
        Self::FixedUpdate,
        Self::FixedPostUpdate,
        Self::Update,
        Self::PostUpdate,
        Self::Last,
        Self::RenderExtract,
    ];
    /// WorldDriver 在每个固定步内完整运行这三个阶段，提交或回滚以整步为边界。
    pub const FIXED_LOOP: [Self; 3] = [Self::FixedFirst, Self::FixedUpdate, Self::FixedPostUpdate];

    pub const fn rank(self) -> usize {
        match self {
            Self::First => 0,
            Self::PreUpdate => 1,
            Self::FixedFirst => 2,
            Self::FixedUpdate => 3,
            Self::FixedPostUpdate => 4,
            Self::Update => 5,
            Self::PostUpdate => 6,
            Self::Last => 7,
            Self::RenderExtract => 8,
        }
    }

    pub const fn is_fixed_loop(self) -> bool {
        matches!(
            self,
            Self::FixedFirst | Self::FixedUpdate | Self::FixedPostUpdate
        )
    }
}
