//! 按后端获取、监听、连接、帧收发和关闭拆分 WebSocket manager 实现，统一经 NetManager trait 公开。
//! 真实连接依赖 feature 注入，loopback 仅供本地语义调用和测试。

#[path = "websocket/backend.rs"]
mod backend;
#[path = "websocket/close.rs"]
mod close;
#[path = "websocket/connect.rs"]
mod connect;
#[path = "websocket/frames.rs"]
mod frames;
#[path = "websocket/listen.rs"]
mod listen;
#[path = "websocket/loopback.rs"]
mod loopback;
