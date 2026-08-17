//! Stable metric categories for host-consumed runtime output.

pub(super) const RUNTIME_FOREIGN_OUTPUT_KIND_COUNT: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeForeignOutputKind {
    SessionProtocol,
    HostRequests,
    ProfileResponse,
    OperationResult,
    PluginEvents,
    WorldQuery,
    WorldInvalidations,
}

impl RuntimeForeignOutputKind {
    pub(super) const ALL: [Self; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT] = [
        Self::SessionProtocol,
        Self::HostRequests,
        Self::ProfileResponse,
        Self::OperationResult,
        Self::PluginEvents,
        Self::WorldQuery,
        Self::WorldInvalidations,
    ];

    pub(super) const fn index(self) -> usize {
        match self {
            Self::SessionProtocol => 0,
            Self::HostRequests => 1,
            Self::ProfileResponse => 2,
            Self::OperationResult => 3,
            Self::PluginEvents => 4,
            Self::WorldQuery => 5,
            Self::WorldInvalidations => 6,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::SessionProtocol => "session_protocol",
            Self::HostRequests => "host_requests",
            Self::ProfileResponse => "profile_response",
            Self::OperationResult => "operation_result",
            Self::PluginEvents => "plugin_events",
            Self::WorldQuery => "world_query",
            Self::WorldInvalidations => "world_invalidations",
        }
    }
}
