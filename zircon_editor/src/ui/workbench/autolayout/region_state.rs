use super::PaneConstraints;

#[derive(Clone, Copy, Debug)]
pub(super) struct RegionState {
    pub(super) visible: bool,
    pub(super) expanded: bool,
    pub(super) constraints: PaneConstraints,
}
