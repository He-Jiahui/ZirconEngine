//! Sound framework contracts for runtime clip loading, playback, and mixing.

mod error;
mod ids;
mod manager;
mod mix;
mod playback;
mod status;

pub use error::SoundError;
pub use ids::{SoundClipId, SoundPlaybackId};
pub use manager::SoundManager;
pub use mix::SoundMixBlock;
pub use playback::{SoundClipInfo, SoundPlaybackSettings};
pub use status::{SoundBackendState, SoundBackendStatus};
