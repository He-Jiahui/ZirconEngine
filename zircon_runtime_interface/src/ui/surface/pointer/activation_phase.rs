use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerActivationPhase {
    #[default]
    None,
    PrimaryPress,
    PrimaryRelease,
    SecondaryPress,
    SecondaryRelease,
    MiddlePress,
    MiddleRelease,
    Hover,
    Scroll,
}
