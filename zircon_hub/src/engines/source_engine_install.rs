use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEngineInstall {
    pub id: String,
    pub display_name: String,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    #[serde(default)]
    pub last_build_unix_ms: Option<u64>,
}

impl SourceEngineInstall {
    pub fn staged_engine_dir(&self) -> PathBuf {
        self.output_dir.join("ZirconEngine")
    }
}
