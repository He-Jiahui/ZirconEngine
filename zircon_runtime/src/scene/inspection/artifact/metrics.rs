use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Cumulative complete-view rows materialized after sparse hierarchy publications.
#[derive(Debug, Default)]
pub(super) struct HierarchyRowMaterializations {
    totals: RwLock<(u64, u64)>,
}

impl HierarchyRowMaterializations {
    pub(super) fn from_totals(totals: (u64, u64)) -> Self {
        Self {
            totals: RwLock::new(totals),
        }
    }

    pub(super) fn record(&self, row_count: usize) {
        let mut totals = write_metrics(&self.totals);
        totals.0 = totals.0.saturating_add(1);
        totals.1 = totals.1.saturating_add(row_count as u64);
    }

    pub(super) fn snapshot(&self) -> (u64, u64) {
        *read_metrics(&self.totals)
    }

    pub(super) fn with_snapshot<T>(&self, read: impl FnOnce((u64, u64)) -> T) -> T {
        let totals = read_metrics(&self.totals);
        read(*totals)
    }
}

fn read_metrics<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_metrics<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Cumulative producer and on-demand materialization work recorded by the runtime inspection cache.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldInspectionArtifactDiagnostics {
    pub(super) hierarchy_builds: u64,
    pub(super) hierarchy_rows_built: u64,
    pub(super) hierarchy_child_hash_updates: u64,
    pub(super) hierarchy_full_materializations: u64,
    pub(super) hierarchy_rows_materialized: u64,
    pub(super) focused_field_builds: u64,
    pub(super) focused_fields_built: u64,
}

impl WorldInspectionArtifactDiagnostics {
    pub const fn hierarchy_builds(self) -> u64 {
        self.hierarchy_builds
    }

    pub const fn hierarchy_rows_built(self) -> u64 {
        self.hierarchy_rows_built
    }

    /// Incremental parent aggregates updated after changed child subtree hashes.
    pub const fn hierarchy_child_hash_updates(self) -> u64 {
        self.hierarchy_child_hash_updates
    }

    /// Full hierarchy views materialized after a sparse generation was published.
    pub const fn hierarchy_full_materializations(self) -> u64 {
        self.hierarchy_full_materializations
    }

    /// Total rows in complete hierarchy views materialized from sparse generations.
    pub const fn hierarchy_rows_materialized(self) -> u64 {
        self.hierarchy_rows_materialized
    }

    pub const fn focused_field_builds(self) -> u64 {
        self.focused_field_builds
    }

    pub const fn focused_fields_built(self) -> u64 {
        self.focused_fields_built
    }
}
