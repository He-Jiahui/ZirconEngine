#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VirtualGeometryPageResidencyState {
    Resident,
    PendingUpload,
}
