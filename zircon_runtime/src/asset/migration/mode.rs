#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetMigrationMode {
    DryRun,
    Apply,
}
