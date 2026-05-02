#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VirtualGeometryPrepareClusterState {
    Resident,
    PendingUpload,
    Missing,
}
