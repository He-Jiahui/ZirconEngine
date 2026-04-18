use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationHostEvent {
    AddFrame { track_path: String, frame: u32 },
}
