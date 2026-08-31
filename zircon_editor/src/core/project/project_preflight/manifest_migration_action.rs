/// One explicit operator choice for a manifest that requires schema migration.
///
/// These values describe an admission policy only. Project copy, backup, and conversion remain
/// separate transaction steps and cannot run while preflight is evaluating the manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectManifestMigrationAction {
    OpenCopy,
    ConvertInPlace,
    Cancel,
}

impl ProjectManifestMigrationAction {
    pub const ALL: [Self; 3] = [Self::OpenCopy, Self::ConvertInPlace, Self::Cancel];

    pub const fn mutates_source(self) -> bool {
        matches!(self, Self::ConvertInPlace)
    }

    pub const fn requires_source_backup(self) -> bool {
        matches!(self, Self::ConvertInPlace)
    }

    pub const fn requires_fresh_preflight(self) -> bool {
        matches!(self, Self::OpenCopy | Self::ConvertInPlace)
    }

    pub const fn cancels_launch(self) -> bool {
        matches!(self, Self::Cancel)
    }
}
