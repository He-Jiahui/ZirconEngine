use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEvent, NetTransportKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransportStateMachine {
    connection: NetConnectionId,
    transport: NetTransportKind,
    state: NetConnectionState,
}

impl TransportStateMachine {
    pub(crate) fn new(
        connection: NetConnectionId,
        transport: NetTransportKind,
        state: NetConnectionState,
    ) -> Self {
        Self {
            connection,
            transport,
            state,
        }
    }

    pub(crate) fn state(&self) -> NetConnectionState {
        self.state
    }

    pub(crate) fn current_event(&self) -> NetEvent {
        Self::event_for(self.connection, self.transport, self.state)
    }

    pub(crate) fn transition(&mut self, next: NetConnectionState) -> Option<NetEvent> {
        if self.state == next {
            return None;
        }

        self.state = next;
        Some(self.current_event())
    }

    pub(crate) fn event_for(
        connection: NetConnectionId,
        transport: NetTransportKind,
        state: NetConnectionState,
    ) -> NetEvent {
        NetEvent::ConnectionStateChanged {
            connection,
            transport,
            state,
        }
    }
}
