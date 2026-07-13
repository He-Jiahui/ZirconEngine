use std::marker::PhantomData;

use super::MigrationStep;

/// Ordered migration steps for one schema type.
pub struct MigrationChain<T> {
    pub(super) steps: &'static [MigrationStep],
    marker: PhantomData<fn() -> T>,
}

impl<T> MigrationChain<T> {
    pub const fn new(steps: &'static [MigrationStep]) -> Self {
        Self {
            steps,
            marker: PhantomData,
        }
    }
}
