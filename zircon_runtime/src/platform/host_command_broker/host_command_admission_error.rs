/// Separates broker admission failure from requested-state publication failure
/// without coupling the generic command broker to a concrete state owner.
#[derive(Debug)]
pub(crate) enum HostCommandAdmissionError<E> {
    Broker(super::HostCommandBrokerError),
    RequestedState(E),
}
