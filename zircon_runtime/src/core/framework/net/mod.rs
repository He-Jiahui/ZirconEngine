//! Networking framework contracts for the minimal socket and message-loop surface.

mod endpoint;
mod error;
mod manager;
mod packet;
mod socket_id;

pub use endpoint::NetEndpoint;
pub use error::NetError;
pub use manager::NetManager;
pub use packet::NetPacket;
pub use socket_id::NetSocketId;
