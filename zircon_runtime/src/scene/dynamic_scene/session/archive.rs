use serde::{Deserialize, Serialize};

use super::slot::RuntimeSessionSlot;

pub const RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSessionArchive {
    pub format_version: u32,
    #[serde(default)]
    pub slots: Vec<RuntimeSessionSlot>,
}
