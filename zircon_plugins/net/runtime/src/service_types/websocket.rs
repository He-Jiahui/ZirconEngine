//! 按后端获取、监听、连接、帧收发和关闭拆分 WebSocket manager 实现，统一经 NetManager trait 公开。
//! 真实连接依赖 feature 注入，loopback 仅供本地语义调用和测试。

mod backend;
mod close;
mod connect;
mod frames;
mod listen;
mod loopback;
