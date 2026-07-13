use serde_json::Value;

use super::MigrateError;

pub type MigrationFn = fn(Value) -> Result<Value, MigrateError>;

/// One value-domain migration from `from_version` to its immediate successor.
#[derive(Clone, Copy)]
pub struct MigrationStep {
    pub(super) from_version: u32,
    pub(super) migrate: MigrationFn,
}

impl MigrationStep {
    pub const fn new(from_version: u32, migrate: MigrationFn) -> Self {
        Self {
            from_version,
            migrate,
        }
    }
}
