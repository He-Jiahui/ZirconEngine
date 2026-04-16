#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisibilityBvhUpdateStrategy {
    #[default]
    FullRebuild,
    Incremental,
}
