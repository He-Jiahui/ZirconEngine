use std::path::{Path, PathBuf};

use crate::core::recovery::{RestorePlan, RestoreStartup};

/// Fully specified, project-bound work that may be admitted to the editor job system.
///
/// The decision coordinator constructs this only after every recovery candidate has one explicit
/// operator resolution. It owns no UI state and is intentionally consumed by the worker adapter.
pub(super) struct RecoveryRestoreWork {
    project_root: PathBuf,
    startup: RestoreStartup,
    plan: RestorePlan,
}

impl RecoveryRestoreWork {
    pub(super) fn new(project_root: PathBuf, startup: RestoreStartup, plan: RestorePlan) -> Self {
        Self {
            project_root,
            startup,
            plan,
        }
    }

    pub(super) fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub(super) fn startup(&self) -> &RestoreStartup {
        &self.startup
    }

    pub(super) fn plan(&self) -> &RestorePlan {
        &self.plan
    }
}
