#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VirtualGeometryPrepareClusterState {
    Resident,
    PendingUpload,
    Missing,
}
