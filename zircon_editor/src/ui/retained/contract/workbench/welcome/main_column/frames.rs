#[path = "../frames/fallback.rs"]
mod fallback;
#[path = "../frames/model.rs"]
mod model;
#[path = "../frames/resolve.rs"]
mod resolve;
#[path = "../frames/sequence.rs"]
mod sequence;

pub(super) use sequence::welcome_main_column_frames;
