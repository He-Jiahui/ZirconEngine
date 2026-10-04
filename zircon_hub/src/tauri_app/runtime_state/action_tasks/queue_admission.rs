//! 限制已有后台工作者运行期间等待的请求数量，以保留有界的内存和任务积压。
//! 这里只拥有队列准入；目标解析、动作校验和取消语义由运行时调度路径处理。

use std::collections::VecDeque;

use crate::error::HubError;
use crate::tauri_app::HubActionRequest;

// 上限约束等待请求；正在执行的一个工作者不计入此容量。
pub(super) const BACKGROUND_ACTION_QUEUE_CAPACITY: usize = 64;

/// 由持有运行时队列独占访问的调度入口调用；满载时拒绝新请求，已排队请求保持 FIFO。
/// 保存的是请求副本；排队成功不代表目标或负载已通过真正执行前的校验。
pub(super) fn enqueue_background_action(
    queue: &mut VecDeque<HubActionRequest>,
    request: &HubActionRequest,
) -> Result<(), HubError> {
    if queue.len() >= BACKGROUND_ACTION_QUEUE_CAPACITY {
        return Err(HubError::BackgroundActionQueueFull {
            capacity: BACKGROUND_ACTION_QUEUE_CAPACITY,
        });
    }
    queue.push_back(request.clone());
    Ok(())
}

#[cfg(test)]
#[path = "tests/queue_admission.rs"]
mod tests;
