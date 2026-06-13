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
