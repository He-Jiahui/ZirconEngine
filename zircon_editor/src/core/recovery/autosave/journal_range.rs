//! 声明快照与耐久日志覆盖范围的关系；当前捕获仅记录Unavailable，恢复不能据此认定任何日志前缀已被快照覆盖。

use serde::{Deserialize, Serialize};

/// The durable transaction span represented by an autosave snapshot.
///
/// The P1-10 durable journal does not yet link autosave commit records to
/// journal coverage, so current captures record that absence explicitly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutosaveJournalRange {
    #[default]
    Unavailable,
}
