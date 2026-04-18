use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use crate::{LevelSystem, WorldHandle};

#[derive(Debug, Default)]
pub struct DefaultLevelManager {
    pub(super) next_handle: AtomicU64,
    pub(super) levels: Mutex<HashMap<WorldHandle, LevelSystem>>,
}
