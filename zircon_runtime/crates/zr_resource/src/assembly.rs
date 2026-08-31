//! Workspace-internal Resource assembly surface.

pub use crate::event_stream::approximate_event_bytes;
pub use crate::manager::PreparedResourceMutation;
pub use crate::readiness_generation::ResourceReadinessRow;
pub use crate::registry::ResourceRegistryStaging;

use crate::{
    ResourceManager, ResourceMutationBatch, ResourceReadinessGeneration, ResourceRegistry,
    ResourceResult,
};

pub trait ResourceManagerAssemblyExt {
    fn prepare_commit(
        &self,
        batch: ResourceMutationBatch,
    ) -> ResourceResult<PreparedResourceMutation<'_>>;
}

impl ResourceManagerAssemblyExt for ResourceManager {
    fn prepare_commit(
        &self,
        batch: ResourceMutationBatch,
    ) -> ResourceResult<PreparedResourceMutation<'_>> {
        ResourceManager::prepare_commit(self, batch)
    }
}

pub trait ResourceRegistryAssemblyExt {
    fn begin_staging(&self) -> ResourceRegistryStaging;
}

impl ResourceRegistryAssemblyExt for ResourceRegistry {
    fn begin_staging(&self) -> ResourceRegistryStaging {
        ResourceRegistry::begin_staging(self)
    }
}

pub trait ResourceReadinessGenerationAssemblyExt {
    fn row(&self, id: crate::ResourceId) -> Option<&std::sync::Arc<ResourceReadinessRow>>;
}

impl ResourceReadinessGenerationAssemblyExt for ResourceReadinessGeneration {
    fn row(&self, id: crate::ResourceId) -> Option<&std::sync::Arc<ResourceReadinessRow>> {
        ResourceReadinessGeneration::row(self, id)
    }
}

pub mod io {
    pub use crate::io::atomic_file::{
        AtomicWriteFault, PendingAtomicWrite, atomic_write_with_fault, ensure_parent_directories,
        is_atomic_write_transaction_path, recover_missing_target_from_backup, replace_staged_file,
        stage_atomic_write, sync_parent_directory,
    };

    pub mod transaction {
        pub use crate::io::transaction::{
            DurableCommitDisposition, DurableCommitReport, DurableRecoveryReport,
            DurableTransactionError, JournalDocument, PreparedFileWrite, RecoveryPolicy,
            TransactionFault, TransactionPhase, commit_prepared_files, detect_pending_transactions,
            recover_pending_transactions,
        };
    }
}
