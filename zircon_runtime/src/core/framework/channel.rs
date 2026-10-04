//! Neutral channel aliases and receive helpers shared by runtime framework contracts.

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};

pub type ChannelSender<T> = Sender<T>;
pub type ChannelReceiver<T> = Receiver<T>;
/// 生产者唤醒宿主事件循环的通知钩子；消息仍由接收方从对应通道读取。
pub type ChannelWakeCallback = Arc<dyn Fn() + Send + Sync + 'static>;

/// 丢弃已排队的中间值，仅取当前可见的最后一个值；适合状态收敛，不适合逐条事件处理。
/// None 同时表示尚无消息或通道已断开，调用方如需区分必须直接使用 Receiver。
pub fn recv_latest<T>(receiver: &Receiver<T>) -> Option<T> {
    let mut latest = None;
    while let Ok(value) = receiver.try_recv() {
        latest = Some(value);
    }
    latest
}

pub fn wait_for<T>(receiver: &Receiver<T>, timeout: Duration) -> Result<T, RecvTimeoutError> {
    receiver.recv_timeout(timeout)
}
