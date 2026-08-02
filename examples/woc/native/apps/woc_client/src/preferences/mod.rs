mod settings;
mod storage;

pub use settings::*;
pub(crate) use storage::{read_preference_text, submit_preference_text, PreferenceRead};
