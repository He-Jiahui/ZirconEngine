use std::sync::{Arc, Mutex, MutexGuard};

use super::{ProductFailureRecord, ProductFailureReport, ProductFailureSeverity, ProductHostPhase};

pub(crate) const PRODUCT_FAILURE_LEDGER_CAPACITY: usize = 16;
pub(crate) const PRODUCT_FAILURE_MESSAGE_BYTES: usize = 512;
const PRODUCT_FAILURE_TRUNCATION_SUFFIX: &str = "...";

#[derive(Debug)]
struct ProductFailureLedgerState {
    next_sequence: u64,
    records: Vec<ProductFailureRecord>,
    suppressed_count: u64,
}

impl Default for ProductFailureLedgerState {
    fn default() -> Self {
        Self {
            next_sequence: 0,
            records: Vec::with_capacity(PRODUCT_FAILURE_LEDGER_CAPACITY),
            suppressed_count: 0,
        }
    }
}

/// Cloneable cold-path sink for ordered product failures.
/// 启动失败可即时读取快照；持有动态会话的终结路径应在清理正常返回后读取最终账本。
/// 会话 destroy 在 Drop 中失败会记录后终止进程，无法返回入口形成最终快照。
#[derive(Clone, Debug, Default)]
pub(crate) struct ProductFailureLedger(Arc<Mutex<ProductFailureLedgerState>>);

impl ProductFailureLedger {
    /// 按观察顺序保留最早的有限条目；容量满时仍计数丢失项，不能把快照视作完整日志。
    pub(crate) fn record(
        &self,
        phase: ProductHostPhase,
        severity: ProductFailureSeverity,
        owner: &'static str,
        message: impl ToString,
    ) {
        let message = bounded_failure_message(message.to_string());
        let mut state = self.lock();
        let sequence = state.next_sequence;
        state.next_sequence = state.next_sequence.saturating_add(1);
        if state.records.len() == PRODUCT_FAILURE_LEDGER_CAPACITY {
            state.suppressed_count = state.suppressed_count.saturating_add(1);
            return;
        }
        state.records.push(ProductFailureRecord::new(
            sequence, phase, severity, owner, message,
        ));
    }

    pub(crate) fn snapshot(&self) -> ProductFailureReport {
        let state = self.lock();
        ProductFailureReport::new(state.records.clone(), state.suppressed_count)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.lock().records.is_empty()
    }

    fn lock(&self) -> MutexGuard<'_, ProductFailureLedgerState> {
        match self.0.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

// 终结诊断使用单行定界文本，必须先转义控制字符和分隔符，再按 UTF-8 字节边界截断。
fn bounded_failure_message(message: String) -> String {
    let mut escaped = String::with_capacity(message.len().min(PRODUCT_FAILURE_MESSAGE_BYTES));
    for character in message.chars() {
        let replacement = match character {
            '\\' => Some("\\\\"),
            '\r' => Some("\\r"),
            '\n' => Some("\\n"),
            '\t' => Some("\\t"),
            '|' => Some("\\|"),
            '=' => Some("\\="),
            _ => None,
        };
        if let Some(replacement) = replacement {
            if escaped.len() + replacement.len() > PRODUCT_FAILURE_MESSAGE_BYTES {
                return truncate_failure_message(escaped);
            }
            escaped.push_str(replacement);
            continue;
        }
        if character.is_control() {
            let replacement = character.escape_default().to_string();
            if escaped.len() + replacement.len() > PRODUCT_FAILURE_MESSAGE_BYTES {
                return truncate_failure_message(escaped);
            }
            escaped.push_str(&replacement);
            continue;
        }
        if escaped.len() + character.len_utf8() > PRODUCT_FAILURE_MESSAGE_BYTES {
            return truncate_failure_message(escaped);
        }
        escaped.push(character);
    }
    escaped
}

fn truncate_failure_message(mut message: String) -> String {
    let mut boundary = PRODUCT_FAILURE_MESSAGE_BYTES - PRODUCT_FAILURE_TRUNCATION_SUFFIX.len();
    boundary = boundary.min(message.len());
    while !message.is_char_boundary(boundary) {
        boundary -= 1;
    }
    message.truncate(boundary);
    message.push_str(PRODUCT_FAILURE_TRUNCATION_SUFFIX);
    message
}
