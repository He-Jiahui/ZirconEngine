use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::HubMessage;

const BUILD_HISTORY_LIMIT: usize = 8;

// Stored per source checkout so Hub can show recent source-build attempts without
// scanning target directories or build logs during normal UI rendering.
/// 保存在 Hub 配置中的一次源码构建摘要，供历史视图诊断，不能代替产物资格验证。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBuildRecord {
    pub finished_unix_ms: u64,
    pub status: String,
    pub profile: String,
    #[serde(default)]
    pub jobs: Option<u16>,
    pub output_dir: PathBuf,
    pub detail: HubMessage,
    #[serde(default = "HubMessage::empty")]
    pub log_excerpt: HubMessage,
    #[serde(default)]
    pub command_line: Vec<String>,
}

/// 一个源码检出与其发布输出根的稳定登记项；项目绑定引用 `id`，构建历史随登记项持久化。
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
    /// 返回已发布引擎所在位置；启动 Editor 前仍须验证对应的分阶段构建产物。
    pub fn staged_engine_dir(&self) -> PathBuf {
        self.output_dir.join("ZirconEngine")
    }

    /// 构建终结时追加有限历史；失败记录不覆盖最近一次成功构建时间。
    pub fn record_build(&mut self, record: SourceBuildRecord) {
        if record.status == "success" {
            self.last_build_unix_ms = Some(record.finished_unix_ms);
        }
        self.build_history.insert(0, record);
        self.build_history.truncate(BUILD_HISTORY_LIMIT);
    }
}

#[cfg(test)]
#[path = "tests/source_engine_install.rs"]
mod tests;
