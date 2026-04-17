#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HybridGiProbeResidencyState {
    Resident,
    PendingUpdate,
}
