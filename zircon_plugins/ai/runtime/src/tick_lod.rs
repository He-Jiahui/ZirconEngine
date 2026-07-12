#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AiBehaviorTickLod {
    #[default]
    Full,
    Half,
    Quarter,
}

impl AiBehaviorTickLod {
    pub fn from_distance(distance: f32) -> Self {
        if distance <= FULL_RATE_MAX_DISTANCE {
            Self::Full
        } else if distance <= HALF_RATE_MAX_DISTANCE {
            Self::Half
        } else {
            Self::Quarter
        }
    }

    pub fn should_tick(self, frame: u64, entity: u64) -> bool {
        let divisor = match self {
            Self::Full => 1,
            Self::Half => 2,
            Self::Quarter => 4,
        };
        frame.wrapping_add(entity) % divisor == 0
    }
}

const FULL_RATE_MAX_DISTANCE: f32 = 20.0;
const HALF_RATE_MAX_DISTANCE: f32 = 60.0;
