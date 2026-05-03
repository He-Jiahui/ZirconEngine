use serde::{Deserialize, Serialize};

use super::{NetConnectionId, NetListenerId, NetRouteId, NetSessionId, NetSocketId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetError {
    InvalidEndpoint { endpoint: String },
    UnknownSocket { socket: NetSocketId },
    UnknownListener { listener: NetListenerId },
    UnknownConnection { connection: NetConnectionId },
    UnknownSession { session: NetSessionId },
    UnknownRoute { route: NetRouteId },
    RouteNotFound { method: String, path: String },
    InvalidBudget { budget: usize },
    SecurityPolicyViolation { reason: String },
    ProtocolUnavailable { capability: String },
    Io(String),
}
