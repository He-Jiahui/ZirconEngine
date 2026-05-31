use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const BUILD_HISTORY_LIMIT: usize = 8;

// Stored per source checkout so Hub can show recent source-build attempts without
// scanning target directories or build logs during normal UI rendering.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBuildRecord {
    pub finished_unix_ms: u64,
    pub status: String,
    pub profile: String,
    #[serde(default)]
    pub jobs: Option<u16>,
    pub output_dir: PathBuf,
    pub detail: String,
    #[serde(default)]
    pub log_excerpt: String,
    #[serde(default)]
    pub command_line: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEngineInstall {
    pub id: String,
    pub display_name: String,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    #[serde(default)]
    pub last_build_unix_ms: Option<u64>,
    #[serde(default)]
    pub build_history: Vec<SourceBuildRecord>,
}

impl SourceEngineInstall {
    pub fn staged_engine_dir(&self) -> PathBuf {
        self.output_dir.join("ZirconEngine")
    }

    pub fn record_build(&mut self, record: SourceBuildRecord) {
        if record.status == "success" {
            self.last_build_unix_ms = Some(record.finished_unix_ms);
        }
        self.build_history.insert(0, record);
        self.build_history.truncate(BUILD_HISTORY_LIMIT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_build_keeps_newest_history_and_last_success() {
        let mut engine = SourceEngineInstall::default();

        for index in 0..10 {
            engine.record_build(SourceBuildRecord {
                finished_unix_ms: index,
                status: if index == 9 { "success" } else { "failed" }.to_string(),
                profile: "debug".to_string(),
                jobs: Some(1),
                output_dir: PathBuf::from("E:/out"),
                detail: format!("run {index}"),
                log_excerpt: format!("log {index}"),
                command_line: vec!["python".to_string(), "tools/zircon_build.py".to_string()],
            });
        }

        assert_eq!(engine.last_build_unix_ms, Some(9));
        assert_eq!(engine.build_history.len(), BUILD_HISTORY_LIMIT);
        assert_eq!(engine.build_history[0].detail, "run 9");
        assert_eq!(engine.build_history[7].detail, "run 2");
    }
}
