use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemStage {
    First,
    PreUpdate,
    FixedUpdate,
    Update,
    PostUpdate,
    Last,
    RenderExtract,
}

impl SystemStage {
    pub const COUNT: usize = 7;
    pub const ORDER: [Self; Self::COUNT] = [
        Self::First,
        Self::PreUpdate,
        Self::FixedUpdate,
        Self::Update,
        Self::PostUpdate,
        Self::Last,
        Self::RenderExtract,
    ];

    pub const fn rank(self) -> usize {
        match self {
            Self::First => 0,
            Self::PreUpdate => 1,
            Self::FixedUpdate => 2,
            Self::Update => 3,
            Self::PostUpdate => 4,
            Self::Last => 5,
            Self::RenderExtract => 6,
        }
    }
}
