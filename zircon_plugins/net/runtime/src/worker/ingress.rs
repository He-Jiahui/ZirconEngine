//! 将 worker 生命周期 NetEvent 送入 manager 事件队列；消息枚举为后续 ingress 扩展保留边界。
//! 数据 payload 仍通过 TCP/UDP 显式 poll 返回，不经过该通道。

use zircon_runtime::core::framework::net::NetEvent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NetIngress {
    Event(NetEvent),
}

impl NetIngress {
    pub(crate) fn into_event(self) -> NetEvent {
        match self {
            Self::Event(event) => event,
        }
    }
}
