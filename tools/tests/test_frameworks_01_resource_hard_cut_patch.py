from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools import frameworks_01_resource_hard_cut_move_manifest as move_owner
from tools import frameworks_01_resource_hard_cut_patch as patch_owner
from tools import frameworks_01_resource_hard_cut_manifest as source_owner


def _test_temp_root() -> Path:
    configured = os.environ.get("ZIRCON_TEST_TEMP_ROOT")
    if configured:
        return Path(configured) / "frameworks01-resource-hard-cut-patch-tests"
    for drive in ("F:/", "D:/", "E:/"):
        root = Path(drive)
        if root.exists():
            return root / "zircon-profiles/frameworks01-resource-hard-cut-patch-tests"
    raise RuntimeError("Frameworks01 tests require an E, D, or F drive temp root")


TEST_TEMP_ROOT = _test_temp_root()

WORKSPACE_MANIFEST_FIXTURE = """[workspace]
members = [
    "zircon_runtime",
    "zircon_runtime/crates/zr_math",
    "zircon_runtime/crates/zr_rhi",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
blake3 = "1.8.2"
serde = { version = "1.0.228", features = ["derive", "rc"] }
serde_json = "1.0.149"
thiserror = "2.0.18"
toml = "1.1.2"
zircon_runtime_interface = { path = "zircon_runtime_interface" }
zr_math = { path = "zircon_runtime/crates/zr_math", default-features = false }
zr_rhi = { path = "zircon_runtime/crates/zr_rhi", default-features = false }
"""

RUNTIME_MANIFEST_FIXTURE = """[package]
name = "zircon_runtime"
version.workspace = true
edition.workspace = true
license.workspace = true

[features]
default = []
profiling = []

[dependencies]
zr_math.workspace = true
zr_rhi.workspace = true

[dev-dependencies]
ttf2woff2 = { version = "0.13.1", default-features = false }
"""

CARGO_LOCK_FIXTURE = """version = 4

[[package]]
name = "zircon_runtime"
version = "0.1.0"
dependencies = [
 "zr_math",
 "zr_rhi",
]

[[package]]
name = "zircon_runtime_interface"
version = "0.1.0"

[[package]]
name = "zr_math"
version = "0.1.0"

[[package]]
name = "zr_rhi"
version = "0.1.0"
"""

RESOURCE_ROOT_FIXTURE = """//! Resource foundation layer.

mod data;
mod error;
mod event_stream;
pub mod io;
mod manager;
mod readiness_generation;
mod registry;

pub use data::ResourceData;
pub use error::{ResourceRegistryError, ResourceResult};
pub(crate) use event_stream::approximate_event_bytes;
pub use event_stream::ResourceEventReceiver;
pub(crate) use manager::PreparedResourceMutation;
pub use manager::ResourceManager;
pub(crate) use readiness_generation::ResourceReadinessRow;
pub use readiness_generation::ResourceReadinessGeneration;
pub use registry::ResourceRegistry;
pub(crate) use registry::ResourceRegistryStaging;
pub use zircon_runtime_interface::resource::{ResourceId, ResourceRecord};

#[cfg(test)]
mod tests;
"""

RESOURCE_IO_ROOT_FIXTURE = """mod atomic_file;
mod error;
mod resource_io;
pub(crate) mod transaction;

#[cfg(test)]
pub(crate) use atomic_file::NEXT_ATOMIC_FILE_ID;
pub use atomic_file::{atomic_write, atomic_write_new};
pub(crate) use atomic_file::{atomic_write_with_fault, stage_atomic_write, AtomicWriteFault};
pub use error::ResourceIoError;
pub use resource_io::ResourceIo;
"""

RESOURCE_GUARD_FIXTURE = """use std::path::Path;

use super::super::*;
use super::support::*;

fn is_higher_layer_runtime_path(path: &[String]) -> bool {
    path.first().is_some_and(|segment| segment == "crate")
        && !path.starts_with(&["crate".to_owned(), "core".to_owned(), "resource".to_owned()])
}

const ASSET_CONTRACT: &str = include_str!("../../../framework/asset.rs");
const FRAMEWORK_ROOT: &str = include_str!("../../../framework/mod.rs");
const MANAGEMENT_PROJECTION: &str = include_str!("../../manager/management_projection.rs");
const RESOURCE_MANAGER: &str = include_str!("../../manager/resource_manager.rs");

fn owner_paths() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let owner_path = repo_root.join("zircon_runtime/src/core/resource/management_generation.rs");
    let resource_root = repo_root.join("zircon_runtime/src/core/resource");
    let module_path = ["crate", "core", "resource", "management_generation"].map(str::to_owned);
    let _ = (owner_path, resource_root, module_path);
}

fn dependency_examples() {
    let _ = "use crate::asset::AssetUri;";
    let _ = "use super::super::diagnostics::profiling;";
    let _ = "use crate as runtime_root;";
    let _ = "use crate::core::resource::{ResourceId, ResourceRecord};";
}
"""

RESOURCE_GUARD_SUPPORT_FIXTURE = """pub(super) fn normalize_module_path(
    path: &[String],
    _module_path: &[String],
) -> Vec<String> {
    if path
        .first()
        .is_some_and(|segment| segment == "zircon_runtime")
    {
        let mut normalized = vec!["crate".to_owned()];
        normalized.extend_from_slice(&path[1..]);
        return normalized;
    }
    path.to_vec()
}
"""

CONSUMER_FIXTURES = {
    patch_owner.REQUIRED_CONSUMER_PATCHES[0]: (
        "use crate::core::resource::{\n"
        "    ResourceDiagnostic, ResourceMarker, ResourceReadinessGeneration, ResourceReadinessRow,\n"
        "};\n"
        "fn read(generation: &ResourceReadinessGeneration) { let _ = generation.row(0); }\n"
    ),
    patch_owner.REQUIRED_CONSUMER_PATCHES[1]: (
        "use crate::core::resource::{ResourceMutationBatch, ResourceRecord};\n"
        "fn publish(manager: &ResourceManager, batch: ResourceMutationBatch) {\n"
        "    let prepared = manager.prepare_commit(batch).unwrap();\n"
        "    prepared.commit();\n"
        "}\n"
    ),
    patch_owner.REQUIRED_CONSUMER_PATCHES[2]: (
        "use crate::core::resource::ResourceRecord;\n"
        "fn stage(registry: &ResourceRegistry) { let _ = registry.begin_staging(); }\n"
    ),
    patch_owner.REQUIRED_CONSUMER_PATCHES[3]: (
        "use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistryStaging};\n"
        "fn stage(registry: &ResourceRegistry) { let _ = registry.begin_staging(); }\n"
    ),
    patch_owner.REQUIRED_CONSUMER_PATCHES[4]: (
        "use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState};\n"
        "fn stage() { let _ = ResourceRegistry::default().begin_staging(); }\n"
    ),
    patch_owner.REQUIRED_CONSUMER_PATCHES[5]: (
        "use crate::core::resource::{\n"
        "    ResourceDiagnostic, ResourceRecord, ResourceRegistryStaging, ResourceState,\n"
        "};\n"
        "fn stage(registry: &ResourceRegistry) { let _ = registry.begin_staging(); }\n"
    ),
    "zircon_runtime/src/asset/facade/assets.rs": (
        "use crate::core::resource::{\n"
        "    ResourceLease, ResourceManager, ResourceMarker, ResourceMutationBatch, ResourceRecord,\n"
        "    ResourceRegistryError, ResourceResult,\n"
        "};\n"
        "fn load(manager: &ResourceManager) {\n"
        "    let generation = manager.readiness_generation();\n"
        "    let _ = generation.row(0);\n"
        "}\n"
    ),
    "zircon_runtime/src/asset/facade/manager.rs": (
        "use crate::core::resource::{\n"
        "    ResourceHandle, ResourceMarker, ResourceReadinessGeneration, ResourceState,\n"
        "};\n"
        "fn load(generation: &ResourceReadinessGeneration) { let _ = generation.row(0); }\n"
    ),
}


def _sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _manifest_sha256(value: object) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return _sha256(payload)


class Frameworks01ResourceHardCutPatchTests(unittest.TestCase):
    def setUp(self) -> None:
        TEST_TEMP_ROOT.mkdir(parents=True, exist_ok=True)
        self.temporary_directory = tempfile.TemporaryDirectory(dir=TEST_TEMP_ROOT)
        self.repo_root = Path(self.temporary_directory.name)
        self._git("init", "--quiet")
        self._write_fixture()
        self._git("add", ".")
        self._git(
            "-c",
            "core.hooksPath=NUL",
            "-c",
            "user.name=Zircon Fixture",
            "-c",
            "user.email=zircon-fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _git(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", "-C", str(self.repo_root), *arguments],
            check=True,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )

    def _write(self, relative_path: str, source: str) -> Path:
        path = self.repo_root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(source.encode("utf-8"))
        return path

    def _write_fixture(self) -> None:
        sources = {
            "Cargo.toml": WORKSPACE_MANIFEST_FIXTURE,
            "Cargo.lock": CARGO_LOCK_FIXTURE,
            "zircon_runtime/Cargo.toml": RUNTIME_MANIFEST_FIXTURE,
            "zircon_runtime/src/core/mod.rs": "pub mod resource;\n",
            "zircon_runtime/src/lib.rs": "pub mod core;\n",
            "zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs": (
                '#[path = "resource_foundation/runtime_surface.rs"]\n'
                "mod runtime_surface;\n"
            ),
            "zircon_runtime_interface/Cargo.toml": (
                "[package]\n"
                'name = "zircon_runtime_interface"\n'
                "version.workspace = true\n"
                "edition.workspace = true\n"
                "license.workspace = true\n"
            ),
            "zircon_runtime_interface/src/lib.rs": "pub mod resource;\n",
            "zircon_runtime_interface/src/resource/mod.rs": "pub struct ResourceId;\n",
            "zircon_runtime_interface/src/tests/mod.rs": "mod resource_contracts;\n",
            "zircon_runtime_interface/src/tests/resource_contracts.rs": (
                "#[test]\nfn resource_contract() {}\n"
            ),
            "zircon_runtime/src/core/resource/mod.rs": RESOURCE_ROOT_FIXTURE,
            "zircon_runtime/src/core/resource/io/mod.rs": RESOURCE_IO_ROOT_FIXTURE,
            "zircon_runtime/src/core/resource/event_stream.rs": (
                "pub(crate) fn approximate_event_bytes() -> usize { 0 }\n"
                '// pub(crate) fn approximate_event_bytes() { hidden }\n'
            ),
            "zircon_runtime/src/core/resource/readiness_generation.rs": (
                "pub struct ResourceReadinessGeneration;\n"
                "pub(crate) struct ResourceReadinessRow {\n"
                "    pub(crate) record: usize,\n"
                "    pub(crate) load_state: usize,\n"
                "    pub(crate) direct_dependency_state: usize,\n"
                "    pub(crate) recursive_dependency_state: usize,\n"
                "    pub(crate) dependency_revision: u64,\n"
                "}\n"
                "impl ResourceReadinessRow {\n"
                "    pub(crate) fn typed_load_state(&self) -> usize { self.load_state }\n"
                "}\n"
                "impl ResourceReadinessGeneration {\n"
                "    pub(crate) fn row(&self, _id: usize) -> Option<&ResourceReadinessRow> { None }\n"
                "}\n"
            ),
            "zircon_runtime/src/core/resource/registry.rs": (
                "pub struct ResourceRegistry;\n"
                "pub(crate) struct ResourceRegistryStaging;\n"
                "impl ResourceRegistry {\n"
                "    pub(crate) fn begin_staging(&self) -> ResourceRegistryStaging { ResourceRegistryStaging }\n"
                "}\n"
                "impl ResourceRegistryStaging {\n"
                "    pub(crate) fn stage_record(&mut self) {}\n"
                "    pub(crate) fn stage_rename_locator(&mut self) {}\n"
                "    pub(crate) fn stage_remove_locator(&mut self) {}\n"
                "    pub(crate) fn finish(self) -> ResourceRegistry { ResourceRegistry }\n"
                "}\n"
            ),
            "zircon_runtime/src/core/resource/tests.rs": (
                "fn resource_manager_hot_paths_avoid_redundant_record_projection() {\n"
                '    let registry = include_str!("registry.rs");\n'
                '    assert!(registry.contains("pub(crate) struct ResourceRegistryStaging"));\n'
                "}\n"
            ),
            "zircon_runtime/src/core/resource/manager/mod.rs": (
                "mod commit;\nmod resource_manager;\n"
                "pub(crate) use commit::PreparedResourceMutation;\n"
                "pub use resource_manager::ResourceManager;\n"
            ),
            "zircon_runtime/src/core/resource/manager/commit.rs": (
                "pub(crate) struct PreparedResourceMutation;\n"
                "impl PreparedResourceMutation { pub(crate) fn commit(self) {} }\n"
            ),
            "zircon_runtime/src/core/resource/io/atomic_file/mod.rs": (
                "mod directory;\nmod pathing;\nmod recovery;\nmod transaction;\n"
                "pub(crate) use directory::{ensure_parent_directories, sync_parent_directory};\n"
                "pub(crate) use pathing::is_atomic_write_transaction_path;\n"
                "pub(crate) use recovery::recover_missing_target_from_backup;\n"
                "pub(crate) use transaction::replace_staged_file;\n"
                "pub(crate) static NEXT_ATOMIC_FILE_ID: usize = 1;\n"
                "pub(crate) enum AtomicWriteFault { None }\n"
                "pub fn atomic_write() {}\n"
                "pub fn atomic_write_new() {}\n"
                "pub(crate) fn atomic_write_with_fault() {}\n"
                "pub(crate) type PendingAtomicWrite = transaction::PendingAtomicWrite;\n"
                "pub(crate) fn stage_atomic_write() -> PendingAtomicWrite { transaction::PendingAtomicWrite }\n"
            ),
            "zircon_runtime/src/core/resource/io/atomic_file/directory.rs": (
                "pub(crate) fn sync_parent_directory() {}\n"
                "pub(crate) fn ensure_parent_directories() {}\n"
            ),
            "zircon_runtime/src/core/resource/io/atomic_file/pathing.rs": (
                "pub(crate) fn is_atomic_write_transaction_path() -> bool { false }\n"
            ),
            "zircon_runtime/src/core/resource/io/atomic_file/recovery.rs": (
                "pub(crate) fn recover_missing_target_from_backup() {}\n"
            ),
            "zircon_runtime/src/core/resource/io/atomic_file/transaction.rs": (
                "pub(crate) struct PendingAtomicWrite;\n"
                "impl PendingAtomicWrite {\n"
                "    pub(crate) fn commit(self) {}\n"
                "    pub(crate) fn commit_new(self) {}\n"
                "}\n"
                "pub(crate) fn replace_staged_file() {}\n"
            ),
            "zircon_runtime/src/core/resource/io/transaction/mod.rs": (
                "mod engine;\nmod error;\nmod observation;\nmod recovery;\nmod schema;\n"
                "pub(crate) use engine::{commit_prepared_files, DurableCommitDisposition, PreparedFileWrite};\n"
                "pub(crate) use error::{DurableTransactionError, TransactionPhase};\n"
                "pub(crate) use observation::{DurableCommitReport, DurableRecoveryReport};\n"
                "pub(crate) use recovery::{detect_pending_transactions, recover_pending_transactions, RecoveryPolicy};\n"
                "pub(crate) use schema::{JournalDocument, TransactionFault};\n"
            ),
            "zircon_runtime/src/core/resource/io/transaction/engine.rs": (
                "pub(crate) struct PreparedFileWrite;\n"
                "impl PreparedFileWrite {\n"
                "    pub(crate) fn new() -> Self { Self }\n"
                "    pub(crate) fn retiring(self) -> Self { self }\n"
                "    pub(crate) fn retiring_with_expected_digest(self) -> Self { self }\n"
                "}\n"
                "pub(crate) enum DurableCommitDisposition { Durable }\n"
                "pub(crate) fn commit_prepared_files() {}\n"
                "#[cfg(test)]\nfn injected_fault_path() {}\n"
                "#[cfg(test)]\nmod tests {}\n"
            ),
            "zircon_runtime/src/core/resource/io/transaction/error.rs": (
                "pub(crate) enum DurableTransactionError { Failure }\n"
                "pub(crate) enum TransactionPhase { Commit }\n"
            ),
            "zircon_runtime/src/core/resource/io/transaction/observation.rs": (
                "pub(crate) struct DurableCommitReport;\n"
                "impl DurableCommitReport {\n"
                "    pub(crate) fn rollback_restore_attempt_count(&self) -> usize { 0 }\n"
                "    pub(crate) fn rollback_restore_success_count(&self) -> usize { 0 }\n"
                "    pub(crate) fn deferred_cleanup_count(&self) -> usize { 0 }\n"
                "    pub(crate) fn deferred_commit_recovery_count(&self) -> usize { 0 }\n"
                "    pub(crate) fn has_commit_activity(&self) -> bool { false }\n"
                "    pub(crate) fn from_activity_counts() -> Self { Self }\n"
                "}\n"
                "pub(crate) struct DurableRecoveryReport;\n"
                "impl DurableRecoveryReport {\n"
                "    pub(crate) fn new() -> Self { Self }\n"
                "    pub(crate) fn rollback_count(&self) -> usize { 0 }\n"
                "    pub(crate) fn cleanup_count(&self) -> usize { 0 }\n"
                "    pub(crate) fn intent_orphan_cleanup_count(&self) -> usize { 0 }\n"
                "}\n"
            ),
            "zircon_runtime/src/core/resource/io/transaction/recovery/mod.rs": (
                "pub(crate) trait RecoveryPolicy {}\n"
                "pub(crate) fn detect_pending_transactions() {}\n"
                "pub(crate) fn recover_pending_transactions() {}\n"
            ),
            "zircon_runtime/src/core/resource/io/transaction/schema.rs": (
                "pub(crate) enum TransactionFault {\n"
                "    None,\n"
                "    #[cfg(test)]\n"
                "    CrashAfterCommit(usize),\n"
                "}\n"
                "pub(crate) struct JournalDocument;\n"
                "impl JournalDocument {\n"
                "    pub(crate) fn target(&self) {}\n"
                "    pub(crate) fn retired_path(&self) {}\n"
                "}\n"
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/mod.rs": (
                "mod hard_cut;\nmod projection;\nmod support;\n"
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/hard_cut.rs": (
                RESOURCE_GUARD_FIXTURE
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/support.rs": (
                RESOURCE_GUARD_SUPPORT_FIXTURE
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/projection.rs": (
                "#[test]\nfn projection() {}\n"
            ),
        }
        sources.update(CONSUMER_FIXTURES)
        for path, source in sources.items():
            self._write(path, source)

    def _source_report(self) -> dict[str, object]:
        return source_owner.build_resource_hard_cut_manifest(self.repo_root)

    def _reports(self) -> tuple[dict[str, object], dict[str, object]]:
        source_report = self._source_report()
        move_report = move_owner.build_resource_hard_cut_move_manifest(
            self.repo_root, source_report
        )
        return source_report, move_report

    def test_code_rewrite_preserves_comments_strings_and_spacing_independent_paths(self) -> None:
        source = (
            "use crate :: core::resource::{ResourceId, ResourceRecord};\n"
            '// crate::core::resource::Comment\n'
            'const EXAMPLE: &str = "crate::core::resource::String";\n'
        )

        rewritten = patch_owner.rewrite_resource_crate_root(source)

        self.assertIn("use crate::{ResourceId, ResourceRecord};", rewritten)
        self.assertIn("// crate::core::resource::Comment", rewritten)
        self.assertIn('"crate::core::resource::String"', rewritten)

    def test_visibility_promotions_are_exact_and_keep_product_methods_private(self) -> None:
        outputs = patch_owner.promote_assembly_visibility(
            {
                path: (self.repo_root / path).read_text(encoding="utf-8")
                for path in patch_owner.ASSEMBLY_VISIBILITY_PATHS
            }
        )

        self.assertIn(
            "pub struct PreparedResourceMutation",
            outputs["zircon_runtime/src/core/resource/manager/commit.rs"],
        )
        self.assertIn(
            "pub fn commit(self)",
            outputs["zircon_runtime/src/core/resource/manager/commit.rs"],
        )
        self.assertIn(
            "pub(crate) fn row",
            outputs["zircon_runtime/src/core/resource/readiness_generation.rs"],
        )
        self.assertIn(
            "pub direct_dependency_state: usize",
            outputs["zircon_runtime/src/core/resource/readiness_generation.rs"],
        )
        self.assertIn(
            "pub recursive_dependency_state: usize",
            outputs["zircon_runtime/src/core/resource/readiness_generation.rs"],
        )
        self.assertIn(
            "pub(crate) dependency_revision: u64",
            outputs["zircon_runtime/src/core/resource/readiness_generation.rs"],
        )
        self.assertIn(
            "pub(crate) fn begin_staging",
            outputs["zircon_runtime/src/core/resource/registry.rs"],
        )
        self.assertIn(
            "pub use directory::",
            outputs["zircon_runtime/src/core/resource/io/atomic_file/mod.rs"],
        )
        tests = outputs["zircon_runtime/src/core/resource/tests.rs"]
        self.assertIn('let crate_root = include_str!("lib.rs");', tests)
        self.assertIn('let assembly = include_str!("assembly.rs");', tests)
        self.assertIn(
            'assert!(!crate_root.contains("pub use registry::ResourceRegistryStaging"));',
            tests,
        )
        self.assertIn(
            'assert!(assembly.contains("pub use crate::registry::ResourceRegistryStaging;"));',
            tests,
        )
        self.assertIn(
            '#[cfg(any(test, feature = "test-support"))]\nfn injected_fault_path',
            outputs["zircon_runtime/src/core/resource/io/transaction/engine.rs"],
        )
        self.assertIn(
            "#[cfg(test)]\nmod tests",
            outputs["zircon_runtime/src/core/resource/io/transaction/engine.rs"],
        )

    def test_rust_outputs_are_formatted_with_the_pinned_toolchain(self) -> None:
        outputs = {
            "zircon_runtime/crates/zr_resource/src/lib.rs": (
                "pub fn sample(){\n"
                "    let _ = crate::io::NEXT_ATOMIC_FILE_ID\n"
                "        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);\n"
                "}\n"
            ),
            "zircon_runtime/crates/zr_resource/Cargo.toml": "[package]\nname = \"zr_resource\"\n",
        }

        formatted = patch_owner._format_rust_outputs(self.repo_root, outputs)

        self.assertEqual(
            "pub fn sample() {\n"
            "    let _ = crate::io::NEXT_ATOMIC_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);\n"
            "}\n",
            formatted["zircon_runtime/crates/zr_resource/src/lib.rs"],
        )
        self.assertEqual(
            outputs["zircon_runtime/crates/zr_resource/Cargo.toml"],
            formatted["zircon_runtime/crates/zr_resource/Cargo.toml"],
        )

    def test_rust_outputs_do_not_double_crlf_source_lines(self) -> None:
        path = "zircon_runtime/crates/zr_resource/src/data.rs"
        outputs = {
            path: (
                "pub fn sample() {\r\n"
                "    let value = 1;\r\n"
                "    let _ = value;\r\n"
                "}\r\n"
            )
        }

        formatted = patch_owner._format_rust_outputs(self.repo_root, outputs)

        self.assertEqual(
            "pub fn sample() {\n"
            "    let value = 1;\n"
            "    let _ = value;\n"
            "}\n",
            formatted[path],
        )

    def test_generated_surfaces_keep_product_and_assembly_separate(self) -> None:
        lib, assembly, resource_facade, io_root, io_facade = (
            patch_owner.generated_resource_surfaces(
                RESOURCE_ROOT_FIXTURE,
                RESOURCE_IO_ROOT_FIXTURE,
            )
        )

        self.assertIn("#[doc(hidden)]\npub mod assembly;", lib)
        self.assertIn(
            "pub(crate) use manager::PreparedResourceMutation;",
            lib,
        )
        self.assertIn(
            "pub(crate) use readiness_generation::ResourceReadinessRow;",
            lib,
        )
        self.assertIn("pub trait ResourceManagerAssemblyExt", assembly)
        self.assertIn("pub trait ResourceRegistryAssemblyExt", assembly)
        self.assertIn("pub trait ResourceReadinessGenerationAssemblyExt", assembly)
        self.assertNotIn("pub use zr_resource::assembly", resource_facade)
        self.assertIn("pub(crate) use zr_resource::assembly::{", resource_facade)
        self.assertNotIn("pub use zr_resource::*", resource_facade)
        self.assertIn("pub(crate) mod atomic_file;", io_root)
        self.assertIn(
            "pub(crate) use atomic_file::{atomic_write_with_fault, stage_atomic_write, AtomicWriteFault};",
            io_root,
        )
        self.assertIn(
            "pub use zr_resource::io::{atomic_write, atomic_write_new};",
            io_facade,
        )
        self.assertNotIn(
            "pub(crate) use zr_resource::io::{atomic_write, atomic_write_new};",
            io_facade,
        )
        self.assertIn("zr_resource::assembly::io::transaction", io_facade)

    def test_end_to_end_patch_is_deterministic_and_applies_in_fixture_repo(self) -> None:
        source_report, move_report = self._reports()
        first_report, first_patch = patch_owner.compose_resource_hard_cut_patch(
            self.repo_root, source_report, move_report
        )
        second_report, second_patch = patch_owner.compose_resource_hard_cut_patch(
            self.repo_root, source_report, move_report
        )
        self.assertEqual(first_report, second_report)
        self.assertEqual(first_patch, second_patch)
        self.assertEqual(_sha256(first_patch), first_report["patch_sha256"])
        self.assertEqual(8, first_report["consumer_patch_count"])
        planned_paths = [entry["path"] for entry in move_report["write_paths"]]
        changed_paths = [entry["path"] for entry in first_report["changes"]]
        self.assertEqual(planned_paths, changed_paths)
        self.assertEqual(
            move_report["write_path_manifest_sha256"],
            first_report["write_path_manifest_sha256"],
        )
        self.assertTrue(first_report["stability"]["source_manifest"])
        patch_path = self.repo_root / "hard-cut.patch"
        patch_path.write_bytes(first_patch)
        self._git("apply", "--check", str(patch_path))
        self._git("apply", str(patch_path))
        self.assertTrue(
            (self.repo_root / "zircon_runtime/crates/zr_resource/src/lib.rs").is_file()
        )
        self.assertFalse(
            (self.repo_root / "zircon_runtime/src/core/resource/event_stream.rs").exists()
        )
        runtime_facade = (
            self.repo_root / "zircon_runtime/src/core/resource/mod.rs"
        ).read_text(encoding="utf-8")
        self.assertNotIn("pub use zr_resource::assembly", runtime_facade)
        self.assertIn("pub(crate) use zr_resource::assembly", runtime_facade)
        runtime_manifest = (self.repo_root / "zircon_runtime/Cargo.toml").read_text(
            encoding="utf-8"
        )
        self.assertIn('profiling = ["zr_resource/profiling"]', runtime_manifest)
        self.assertIn(
            'zr_resource = { workspace = true, features = ["test-support"] }',
            runtime_manifest,
        )
        guard = (
            self.repo_root
            / "zircon_runtime/src/tests/runtime_absorption/resource_foundation/resource_owner_boundary/mod.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("zircon_runtime/crates/zr_resource/src", guard)
        for path in (
            "zircon_runtime/src/asset/facade/assets.rs",
            "zircon_runtime/src/asset/facade/manager.rs",
            "zircon_runtime/src/asset/facade/readiness.rs",
        ):
            consumer = (self.repo_root / path).read_text(encoding="utf-8")
            self.assertIn("ResourceReadinessGenerationAssemblyExt", consumer)

    def test_patch_accepts_unrelated_head_drift_after_source_seal(self) -> None:
        source_report, move_report = self._reports()
        self._write(".github/unrelated.txt", "unrelated repository metadata\n")
        self._git("add", ".github/unrelated.txt")
        self._git(
            "-c",
            "core.hooksPath=NUL",
            "-c",
            "user.name=Zircon Fixture",
            "-c",
            "user.email=zircon-fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "unrelated head",
        )
        report, patch = patch_owner.compose_resource_hard_cut_patch(
            self.repo_root, source_report, move_report
        )

        self.assertNotIn("source_head", report)
        self.assertGreater(len(patch), 0)

    def test_patch_rejects_unregistered_readiness_row_assembly_consumer(self) -> None:
        self._write(
            "zircon_runtime/src/asset/facade/new_readiness_consumer.rs",
            "use crate::core::resource::ResourceReadinessGeneration;\n"
            "fn load(generation: &ResourceReadinessGeneration) {\n"
            "    let _ = ResourceReadinessGeneration :: row(generation, 0);\n"
            "}\n",
        )
        source_report, move_report = self._reports()

        with self.assertRaises(patch_owner.HardCutPatchError) as raised:
            patch_owner.compose_resource_hard_cut_patch(
                self.repo_root, source_report, move_report
            )

        self.assertIn("unregistered assembly consumer", str(raised.exception))

    def test_patch_rejects_tampered_move_manifest(self) -> None:
        source_report, move_report = self._reports()
        move_report["operation_manifest_sha256"] = "0" * 64

        with self.assertRaises(patch_owner.HardCutPatchError) as raised:
            patch_owner.compose_resource_hard_cut_patch(
                self.repo_root, source_report, move_report
            )

        self.assertIn("move manifest does not match", str(raised.exception))

    def test_patch_rejects_tampered_full_source_fence(self) -> None:
        source_report, move_report = self._reports()
        source_report["supplemental_candidate_manifest_sha256"] = "0" * 64

        with self.assertRaises(patch_owner.HardCutPatchStabilityError) as raised:
            patch_owner.compose_resource_hard_cut_patch(
                self.repo_root, source_report, move_report
            )

        self.assertIn("sealed current source changed", str(raised.exception))

    def test_patch_rejects_consumer_without_required_role(self) -> None:
        source_report, move_report = self._reports()
        target = next(
            entry
            for entry in source_report["inputs"]
            if entry["path"] == patch_owner.REQUIRED_CONSUMER_PATCHES[0]
        )
        target["roles"] = ["textual_reference"]
        source_report["atomic_input_manifest_sha256"] = _manifest_sha256(
            source_report["inputs"]
        )

        with self.assertRaises(patch_owner.HardCutPatchError) as raised:
            patch_owner.compose_resource_hard_cut_patch(
                self.repo_root, source_report, move_report
            )

        self.assertIn("required Rust consumer", str(raised.exception))

    def test_patch_rejects_unexpected_consumer_shape(self) -> None:
        source_report, move_report = self._reports()
        path = patch_owner.REQUIRED_CONSUMER_PATCHES[0]
        self._write(path, "use crate::core::resource::ResourceMutationBatch;\n")
        changed = (self.repo_root / path).read_bytes()
        entry = next(
            entry for entry in source_report["inputs"] if entry["path"] == path
        )
        entry["bytes"] = len(changed)
        entry["sha256"] = _sha256(changed)
        source_report["atomic_input_manifest_sha256"] = _manifest_sha256(
            source_report["inputs"]
        )
        move_report = move_owner.build_resource_hard_cut_move_manifest(
            self.repo_root, source_report
        )

        with self.assertRaises(patch_owner.HardCutPatchError) as raised:
            patch_owner.compose_resource_hard_cut_patch(
                self.repo_root, source_report, move_report
            )

        self.assertIn("consumer source shape", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
