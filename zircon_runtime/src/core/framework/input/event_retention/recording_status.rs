use serde::{Deserialize, Serialize};

/// 与记录排空一起读取时用于判断回放是否完整；丢弃数累计到重新配置为关闭或重新启用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventRecordingStatus {
    pub enabled: bool,
    pub capacity: u32,
    pub retained_records: u32,
    pub discarded_records: u64,
}
