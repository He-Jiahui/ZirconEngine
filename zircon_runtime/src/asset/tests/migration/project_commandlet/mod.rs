mod command_flow;
mod crash_windows;
mod document_migration;
mod resolver_index;
mod scale_acceptance;
mod source_boundary;
mod transaction_recovery;

use std::fs;

use crate::asset::migration::{
    AssetMigrationIssueKind, AssetMigrationMode, AssetMigrationOptions,
    AssetMigrationTransactionPhase, migrate_project_assets,
    migrate_project_assets_with_commit_fault, migrate_project_assets_with_commit_window_fault,
    migrate_project_assets_with_process_interruption, migrate_project_assets_with_restore_fault,
    migrate_project_assets_with_rollback_cleanup_fault, migrate_project_assets_with_stage_fault,
    migrate_project_assets_with_terminal_interruption,
};
use crate::asset::{
    AssetKind, AssetReference, AssetUri, AssetUuid, ReferenceResolutionError, ZMaterialDocument,
};

use super::{fixture_root, write_manifest, write_registered_source, write_registered_subasset};

fn directory_snapshot(path: &std::path::Path) -> Vec<(std::ffi::OsString, Vec<u8>)> {
    let mut snapshot = fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect::<Vec<_>>();
    snapshot.sort_by(|left, right| left.0.cmp(&right.0));
    snapshot
}
