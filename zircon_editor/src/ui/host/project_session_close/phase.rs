#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ProjectCloseCoordinatorPhase {
    #[default]
    Decision,
    Quiescing,
    Committing,
    Closed,
    RecoveryRequired,
}
