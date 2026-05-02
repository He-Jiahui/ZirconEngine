use serde::{Deserialize, Serialize};

use super::{NetConnectionId, NetListenerId, NetRouteId, NetSocketId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetError {
    InvalidEndpoint { endpoint: String },
    UnknownSocket { socket: NetSocketId },
    UnknownListener { listener: NetListenerId },
    UnknownConnection { connection: NetConnectionId },
    UnknownRoute { route: NetRouteId },
    RouteNotFound { method: String, path: String },
    InvalidBudget { budget: usize },
    ProtocolUnavailable { capability: String },
    Io(String),
}
