use serde::{Deserialize, Serialize};
use zircon_runtime::core::framework::animation::AnimationTrackPath;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationHostEvent {
    AddFrame {
        track_path: AnimationTrackPath,
        frame: u32,
    },
}
