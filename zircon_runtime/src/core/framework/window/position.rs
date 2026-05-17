use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowPosition {
    #[default]
    Automatic,
    Centered,
    At {
        x: i32,
        y: i32,
    },
}
