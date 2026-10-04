use serde::{Deserialize, Serialize};

use crate::core::editing::engine::{HistoryContextId, TransactionId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 事务引擎发出的生命周期事实；事件桥接完整保留交易与历史身份，消息消费者不能据此直接执行撤销。
pub enum TransactionMessage {
    Started {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Canceled {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Committed {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Undone {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    Redone {
        transaction: TransactionId,
        history: HistoryContextId,
        label: String,
        timestamp_frame: u64,
    },
    HistoryTrimmed {
        history: HistoryContextId,
    },
}
