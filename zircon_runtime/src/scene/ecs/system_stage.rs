use serde::{Deserialize, Serialize};

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
