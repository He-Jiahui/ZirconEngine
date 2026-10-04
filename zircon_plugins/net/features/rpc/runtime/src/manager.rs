//! 组织 RPC 的 channel、dispatch、握手、配额、注册和会话状态；公开独立内存 manager 构造器。
//! 调用方负责从真实连接导入控制/调用消息并发布响应，当前 feature factory 未取得根 NetManager。

mod channel;
mod dispatch;
mod handshake;
mod quota;
mod registry;
mod session;
mod state;

use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::core::framework::net::RpcInvocationDescriptor;

pub use channel::{RpcChannelMessage, RPC_CHANNEL_RELIABLE_ORDERED, RPC_CHANNEL_UNRELIABLE};
pub use handshake::{NetRpcHandshakeFrame, RPC_HANDSHAKE_CAPABILITY_NET_RPC, RPC_HANDSHAKE_MAGIC};
pub use state::NetRpcRuntimeManager;

const RPC_PROTOCOL_VERSION: u32 = 1;
const RPC_QUOTA_WINDOW: Duration = Duration::from_secs(1);
const DEFAULT_RPC_QUEUE_DEPTH: usize = 256;

pub(in crate::manager) type RpcSchemaValidator = Arc<dyn Fn(&[u8]) -> bool + Send + Sync>;
pub(in crate::manager) type RpcHandler =
    Arc<dyn Fn(&RpcInvocationDescriptor) -> Result<Vec<u8>, String> + Send + Sync>;

pub fn net_rpc_runtime_manager() -> NetRpcRuntimeManager {
    NetRpcRuntimeManager::default()
}
