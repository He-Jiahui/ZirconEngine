#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentIblSourceStagingStatus {
    Skipped,
    Reused,
    Written,
}
