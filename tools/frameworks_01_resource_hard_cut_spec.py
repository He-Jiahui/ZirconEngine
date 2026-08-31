from __future__ import annotations

RESOURCE_OWNER = "zircon_runtime/src/core/resource"
ASSEMBLY_VISIBILITY_PATHS = (
    f"{RESOURCE_OWNER}/event_stream.rs",
    f"{RESOURCE_OWNER}/io/atomic_file/directory.rs",
    f"{RESOURCE_OWNER}/io/atomic_file/mod.rs",
    f"{RESOURCE_OWNER}/io/atomic_file/pathing.rs",
    f"{RESOURCE_OWNER}/io/atomic_file/recovery.rs",
    f"{RESOURCE_OWNER}/io/atomic_file/transaction.rs",
    f"{RESOURCE_OWNER}/io/transaction/engine.rs",
    f"{RESOURCE_OWNER}/io/transaction/error.rs",
    f"{RESOURCE_OWNER}/io/transaction/mod.rs",
    f"{RESOURCE_OWNER}/io/transaction/observation.rs",
    f"{RESOURCE_OWNER}/io/transaction/recovery/mod.rs",
    f"{RESOURCE_OWNER}/io/transaction/schema.rs",
    f"{RESOURCE_OWNER}/manager/commit.rs",
    f"{RESOURCE_OWNER}/manager/mod.rs",
    f"{RESOURCE_OWNER}/readiness_generation.rs",
    f"{RESOURCE_OWNER}/registry.rs",
    f"{RESOURCE_OWNER}/tests.rs",
)
REQUIRED_CONSUMER_PATCHES = (
    "zircon_runtime/src/asset/facade/readiness.rs",
    "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs",
    "zircon_runtime/src/asset/project/manager/relocation.rs",
    "zircon_runtime/src/asset/project/manager/scan_and_import.rs",
    "zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs",
    "zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs",
    "zircon_runtime/src/asset/facade/assets.rs",
    "zircon_runtime/src/asset/facade/manager.rs",
)

ASSEMBLY_CONSUMER_RULES = {
    "readiness_row": {
        "anchors": ("ResourceReadinessGeneration", ".readiness_generation()"),
        "paths": (
            "zircon_runtime/src/asset/facade/assets.rs",
            "zircon_runtime/src/asset/facade/manager.rs",
            "zircon_runtime/src/asset/facade/readiness.rs",
        ),
        "usage_pattern": (
            r"(?:\.\s*row\s*\(|ResourceReadinessGeneration\s*::\s*row\s*\()"
        ),
    },
    "resource_manager_prepare": {
        "anchors": ("ResourceMutationBatch",),
        "paths": (
            "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs",
        ),
        "usage_pattern": (
            r"(?:\.\s*prepare_commit\s*\(|ResourceManager\s*::\s*prepare_commit\s*\()"
        ),
    },
    "resource_registry_staging": {
        "anchors": ("ResourceRegistry",),
        "paths": (
            "zircon_runtime/src/asset/project/manager/relocation.rs",
            "zircon_runtime/src/asset/project/manager/scan_and_import.rs",
            "zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs",
            "zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs",
        ),
        "usage_pattern": (
            r"(?:\.\s*begin_staging\s*\(|ResourceRegistry\s*::\s*begin_staging\s*\()"
        ),
    },
}


VISIBILITY_REPLACEMENTS: dict[str, tuple[tuple[str, str], ...]] = {
    f"{RESOURCE_OWNER}/event_stream.rs": (
        ("pub(crate) fn approximate_event_bytes", "pub fn approximate_event_bytes"),
    ),
    f"{RESOURCE_OWNER}/readiness_generation.rs": (
        ("pub(crate) struct ResourceReadinessRow", "pub struct ResourceReadinessRow"),
        ("pub(crate) record:", "pub record:"),
        ("pub(crate) load_state:", "pub load_state:"),
        ("pub(crate) direct_dependency_state:", "pub direct_dependency_state:"),
        ("pub(crate) recursive_dependency_state:", "pub recursive_dependency_state:"),
        ("pub(crate) fn typed_load_state", "pub fn typed_load_state"),
    ),
    f"{RESOURCE_OWNER}/registry.rs": (
        ("pub(crate) struct ResourceRegistryStaging", "pub struct ResourceRegistryStaging"),
        ("pub(crate) fn stage_record", "pub fn stage_record"),
        ("pub(crate) fn stage_rename_locator", "pub fn stage_rename_locator"),
        ("pub(crate) fn stage_remove_locator", "pub fn stage_remove_locator"),
        ("pub(crate) fn finish", "pub fn finish"),
    ),
    f"{RESOURCE_OWNER}/manager/mod.rs": (
        (
            "pub(crate) use commit::PreparedResourceMutation",
            "pub use commit::PreparedResourceMutation",
        ),
    ),
    f"{RESOURCE_OWNER}/manager/commit.rs": (
        ("pub(crate) struct PreparedResourceMutation", "pub struct PreparedResourceMutation"),
        ("pub(crate) fn commit(self)", "pub fn commit(self)"),
    ),
    f"{RESOURCE_OWNER}/io/atomic_file/mod.rs": (
        ("pub(crate) use directory::", "pub use directory::"),
        ("pub(crate) use pathing::", "pub use pathing::"),
        ("pub(crate) use recovery::", "pub use recovery::"),
        ("pub(crate) use transaction::", "pub use transaction::"),
        ("pub(crate) static NEXT_ATOMIC_FILE_ID", "pub static NEXT_ATOMIC_FILE_ID"),
        ("pub(crate) enum AtomicWriteFault", "pub enum AtomicWriteFault"),
        ("pub(crate) fn atomic_write_with_fault", "pub fn atomic_write_with_fault"),
        ("pub(crate) type PendingAtomicWrite", "pub type PendingAtomicWrite"),
        ("pub(crate) fn stage_atomic_write", "pub fn stage_atomic_write"),
    ),
    f"{RESOURCE_OWNER}/io/atomic_file/directory.rs": (
        ("pub(crate) fn sync_parent_directory", "pub fn sync_parent_directory"),
        ("pub(crate) fn ensure_parent_directories", "pub fn ensure_parent_directories"),
    ),
    f"{RESOURCE_OWNER}/io/atomic_file/pathing.rs": (
        (
            "pub(crate) fn is_atomic_write_transaction_path",
            "pub fn is_atomic_write_transaction_path",
        ),
    ),
    f"{RESOURCE_OWNER}/io/atomic_file/recovery.rs": (
        (
            "pub(crate) fn recover_missing_target_from_backup",
            "pub fn recover_missing_target_from_backup",
        ),
    ),
    f"{RESOURCE_OWNER}/io/atomic_file/transaction.rs": (
        ("pub(crate) struct PendingAtomicWrite", "pub struct PendingAtomicWrite"),
        ("pub(crate) fn commit(self)", "pub fn commit(self)"),
        ("pub(crate) fn commit_new(self)", "pub fn commit_new(self)"),
        ("pub(crate) fn replace_staged_file", "pub fn replace_staged_file"),
    ),
    f"{RESOURCE_OWNER}/io/transaction/mod.rs": tuple(
        (f"pub(crate) use {owner}::", f"pub use {owner}::")
        for owner in ("engine", "error", "observation", "recovery", "schema")
    ),
    f"{RESOURCE_OWNER}/io/transaction/engine.rs": (
        ("pub(crate) struct PreparedFileWrite", "pub struct PreparedFileWrite"),
        ("pub(crate) fn new", "pub fn new"),
        ("pub(crate) fn retiring(", "pub fn retiring("),
        (
            "pub(crate) fn retiring_with_expected_digest",
            "pub fn retiring_with_expected_digest",
        ),
        ("pub(crate) enum DurableCommitDisposition", "pub enum DurableCommitDisposition"),
        ("pub(crate) fn commit_prepared_files", "pub fn commit_prepared_files"),
    ),
    f"{RESOURCE_OWNER}/io/transaction/error.rs": (
        ("pub(crate) enum TransactionPhase", "pub enum TransactionPhase"),
        ("pub(crate) enum DurableTransactionError", "pub enum DurableTransactionError"),
    ),
    f"{RESOURCE_OWNER}/io/transaction/observation.rs": (
        ("pub(crate) struct DurableCommitReport", "pub struct DurableCommitReport"),
        ("pub(crate) struct DurableRecoveryReport", "pub struct DurableRecoveryReport"),
        ("pub(crate) fn rollback_restore_attempt_count", "pub fn rollback_restore_attempt_count"),
        ("pub(crate) fn rollback_restore_success_count", "pub fn rollback_restore_success_count"),
        ("pub(crate) fn deferred_cleanup_count", "pub fn deferred_cleanup_count"),
        ("pub(crate) fn deferred_commit_recovery_count", "pub fn deferred_commit_recovery_count"),
        ("pub(crate) fn has_commit_activity", "pub fn has_commit_activity"),
        ("pub(crate) fn from_activity_counts", "pub fn from_activity_counts"),
        ("pub(crate) fn new", "pub fn new"),
        ("pub(crate) fn rollback_count", "pub fn rollback_count"),
        ("pub(crate) fn cleanup_count", "pub fn cleanup_count"),
        ("pub(crate) fn intent_orphan_cleanup_count", "pub fn intent_orphan_cleanup_count"),
    ),
    f"{RESOURCE_OWNER}/io/transaction/recovery/mod.rs": (
        ("pub(crate) trait RecoveryPolicy", "pub trait RecoveryPolicy"),
        ("pub(crate) fn detect_pending_transactions", "pub fn detect_pending_transactions"),
        ("pub(crate) fn recover_pending_transactions", "pub fn recover_pending_transactions"),
    ),
    f"{RESOURCE_OWNER}/io/transaction/schema.rs": (
        ("pub(crate) enum TransactionFault", "pub enum TransactionFault"),
        ("pub(crate) struct JournalDocument", "pub struct JournalDocument"),
        ("pub(crate) fn target", "pub fn target"),
        ("pub(crate) fn retired_path", "pub fn retired_path"),
    ),
}

OWNER_SOURCE_REPLACEMENTS: dict[str, tuple[tuple[str, str], ...]] = {
    f"{RESOURCE_OWNER}/tests.rs": (
        (
            '    let registry = include_str!("registry.rs");\n',
            '    let registry = include_str!("registry.rs");\n'
            '    let crate_root = include_str!("lib.rs");\n'
            '    let assembly = include_str!("assembly.rs");\n',
        ),
        (
            '    assert!(registry.contains("pub(crate) struct ResourceRegistryStaging"));',
            '    assert!(registry.contains("pub struct ResourceRegistryStaging"));\n'
            '    assert!(!crate_root.contains("pub use registry::ResourceRegistryStaging"));\n'
            '    assert!(assembly.contains("pub use crate::registry::ResourceRegistryStaging;"));',
        ),
    ),
}


ASSEMBLY_SOURCE = """//! Workspace-internal Resource assembly surface.

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
        atomic_write_with_fault, ensure_parent_directories, is_atomic_write_transaction_path,
        recover_missing_target_from_backup, replace_staged_file, stage_atomic_write,
        sync_parent_directory, AtomicWriteFault, PendingAtomicWrite, NEXT_ATOMIC_FILE_ID,
    };

    pub mod transaction {
        pub use crate::io::transaction::{
            commit_prepared_files, detect_pending_transactions, recover_pending_transactions,
            DurableCommitDisposition, DurableCommitReport, DurableRecoveryReport,
            DurableTransactionError, JournalDocument, PreparedFileWrite, RecoveryPolicy,
            TransactionFault, TransactionPhase,
        };
    }
}
"""

ROOT_ASSEMBLY_PROJECTION = """pub(crate) use zr_resource::assembly::{
    approximate_event_bytes, PreparedResourceMutation, ResourceManagerAssemblyExt,
    ResourceReadinessGenerationAssemblyExt, ResourceReadinessRow,
    ResourceRegistryAssemblyExt, ResourceRegistryStaging,
};
"""

IO_ASSEMBLY_PROJECTION = """pub(crate) use zr_resource::assembly::io::{
    atomic_write_with_fault, ensure_parent_directories, is_atomic_write_transaction_path,
    recover_missing_target_from_backup, replace_staged_file, stage_atomic_write,
    sync_parent_directory, AtomicWriteFault, PendingAtomicWrite,
};

#[cfg(test)]
pub(crate) use zr_resource::assembly::io::NEXT_ATOMIC_FILE_ID;

pub(crate) mod transaction {
    pub(crate) use zr_resource::assembly::io::transaction::{
        commit_prepared_files, detect_pending_transactions, recover_pending_transactions,
        DurableCommitDisposition, DurableCommitReport, DurableRecoveryReport,
        DurableTransactionError, JournalDocument, PreparedFileWrite, RecoveryPolicy,
        TransactionFault, TransactionPhase,
    };
}
"""


CONSUMER_REPLACEMENTS: dict[str, tuple[tuple[str, str], ...]] = {
    REQUIRED_CONSUMER_PATCHES[0]: (
        (
            "ResourceDiagnostic, ResourceMarker, ResourceReadinessGeneration, ResourceReadinessRow,",
            "ResourceDiagnostic, ResourceMarker, ResourceReadinessGeneration,\n"
            "    ResourceReadinessGenerationAssemblyExt, ResourceReadinessRow,",
        ),
    ),
    REQUIRED_CONSUMER_PATCHES[1]: (
        (
            "use crate::core::resource::{ResourceMutationBatch, ResourceRecord};",
            "use crate::core::resource::{\n"
            "    ResourceManagerAssemblyExt, ResourceMutationBatch, ResourceRecord,\n"
            "};",
        ),
    ),
    REQUIRED_CONSUMER_PATCHES[2]: (
        (
            "use crate::core::resource::ResourceRecord;",
            "use crate::core::resource::{ResourceRecord, ResourceRegistryAssemblyExt};",
        ),
    ),
    REQUIRED_CONSUMER_PATCHES[3]: (
        (
            "ResourceDiagnostic, ResourceRecord, ResourceRegistryStaging",
            "ResourceDiagnostic, ResourceRecord, ResourceRegistryAssemblyExt,\n"
            "    ResourceRegistryStaging",
        ),
    ),
    REQUIRED_CONSUMER_PATCHES[4]: (
        (
            "ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState",
            "ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceRegistryAssemblyExt,\n"
            "    ResourceState",
        ),
    ),
    REQUIRED_CONSUMER_PATCHES[5]: (
        (
            "ResourceDiagnostic, ResourceRecord, ResourceRegistryStaging, ResourceState,",
            "ResourceDiagnostic, ResourceRecord, ResourceRegistryAssemblyExt,\n"
            "    ResourceRegistryStaging, ResourceState,",
        ),
    ),
    REQUIRED_CONSUMER_PATCHES[6]: (
        (
            "ResourceLease, ResourceManager, ResourceMarker, ResourceMutationBatch, ResourceRecord,",
            "ResourceLease, ResourceManager, ResourceReadinessGenerationAssemblyExt, ResourceMarker,\n"
            "    ResourceMutationBatch, ResourceRecord,",
        ),
    ),
    REQUIRED_CONSUMER_PATCHES[7]: (
        (
            "ResourceHandle, ResourceMarker, ResourceReadinessGeneration, ResourceState,",
            "ResourceHandle, ResourceMarker, ResourceReadinessGeneration,\n"
            "    ResourceReadinessGenerationAssemblyExt, ResourceState,",
        ),
    ),
}

CONSUMER_USAGE_MARKERS = {
    REQUIRED_CONSUMER_PATCHES[0]: ".row(",
    REQUIRED_CONSUMER_PATCHES[1]: ".prepare_commit(",
    REQUIRED_CONSUMER_PATCHES[2]: ".begin_staging(",
    REQUIRED_CONSUMER_PATCHES[3]: ".begin_staging(",
    REQUIRED_CONSUMER_PATCHES[4]: ".begin_staging(",
    REQUIRED_CONSUMER_PATCHES[5]: ".begin_staging(",
    REQUIRED_CONSUMER_PATCHES[6]: ".row(",
    REQUIRED_CONSUMER_PATCHES[7]: ".row(",
}

ZR_RESOURCE_LOCK_PACKAGE = """[[package]]
name = "zr_resource"
version = "0.1.0"
dependencies = [
 "blake3",
 "serde",
 "serde_json",
 "thiserror 2.0.18",
 "toml 1.1.2+spec-1.1.0",
 "zircon_runtime_interface",
]

"""

ZR_RESOURCE_MANIFEST = """[package]
name = "zr_resource"
version.workspace = true
edition.workspace = true
license.workspace = true
publish = false
description = "Canonical low-dependency Resource implementation for Zircon runtime products."

[features]
default = []
profiling = []
test-support = []

[dependencies]
blake3.workspace = true
serde.workspace = true
thiserror.workspace = true
toml.workspace = true
zircon_runtime_interface.workspace = true

[dev-dependencies]
serde_json = "1.0.149"
"""
