import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZR_RESOURCE_ROOT = REPO_ROOT / "zircon_runtime/crates/zr_resource"
RUNTIME_RESOURCE_ROOT = REPO_ROOT / "zircon_runtime/src/core/resource"
INTERFACE_RESOURCE_ROOT = REPO_ROOT / "zircon_runtime_interface/src/resource"


def load_toml(relative_path: str) -> dict:
    with (REPO_ROOT / relative_path).open("rb") as stream:
        return tomllib.load(stream)


def rust_sources(root: Path) -> list[Path]:
    return sorted(root.rglob("*.rs")) if root.is_dir() else []


class Frameworks01ResourceCrateBoundaryTests(unittest.TestCase):
    def read_required(self, path: Path) -> str:
        self.assertTrue(
            path.is_file(),
            f"required resource hard-cut owner is missing: {path.relative_to(REPO_ROOT)}",
        )
        return path.read_text(encoding="utf-8")

    def test_workspace_and_runtime_use_the_canonical_resource_owner(self) -> None:
        workspace = load_toml("Cargo.toml")
        runtime = load_toml("zircon_runtime/Cargo.toml")
        runtime_interface = load_toml("zircon_runtime_interface/Cargo.toml")

        self.assertIn(
            "zircon_runtime/crates/zr_resource", workspace["workspace"]["members"]
        )
        self.assertEqual(
            {
                "path": "zircon_runtime/crates/zr_resource",
                "default-features": False,
            },
            workspace["workspace"]["dependencies"]["zr_resource"],
        )
        self.assertEqual({"workspace": True}, runtime["dependencies"]["zr_resource"])
        self.assertNotIn("zr_resource", runtime_interface["dependencies"])

        resource = load_toml("zircon_runtime/crates/zr_resource/Cargo.toml")
        self.assertEqual("zr_resource", resource["package"]["name"])
        self.assertFalse(resource["package"]["publish"])
        self.assertEqual(
            {"blake3", "serde", "thiserror", "toml", "zircon_runtime_interface"},
            set(resource["dependencies"]),
        )
        self.assertNotIn("zircon_runtime", resource["dependencies"])
        self.assertIn("serde_json", resource.get("dev-dependencies", {}))

    def test_resource_implementation_is_physically_owned_by_zr_resource(self) -> None:
        lib = self.read_required(ZR_RESOURCE_ROOT / "src/lib.rs")
        for module in (
            "data",
            "error",
            "event_stream",
            "io",
            "lease",
            "management_generation",
            "manager",
            "mutation",
            "readiness_generation",
            "registry",
            "runtime",
            "snapshot",
        ):
            with self.subTest(module=module):
                self.assertIn(f"mod {module};", lib)

        retired_implementation_owners = (
            "data.rs",
            "error.rs",
            "event_stream.rs",
            "event_stream",
            "lease.rs",
            "management_generation.rs",
            "management_generation",
            "manager.rs",
            "manager",
            "mutation.rs",
            "readiness_generation.rs",
            "readiness_generation",
            "registry.rs",
            "registry",
            "runtime.rs",
            "snapshot.rs",
            "tests",
            "io/atomic_file",
            "io/transaction",
        )
        violations = [
            owner
            for owner in retired_implementation_owners
            if (RUNTIME_RESOURCE_ROOT / owner).exists()
        ]
        self.assertEqual([], violations)

        implementation = "\n".join(
            path.read_text(encoding="utf-8")
            for path in rust_sources(ZR_RESOURCE_ROOT / "src")
        )
        self.assertNotIn("zircon_runtime::", implementation)
        self.assertNotIn("crate::core", implementation)

    def test_runtime_resource_surface_is_a_curated_product_projection(self) -> None:
        facade = self.read_required(RUNTIME_RESOURCE_ROOT / "mod.rs")
        io_facade = self.read_required(RUNTIME_RESOURCE_ROOT / "io/mod.rs")

        self.assertIn("pub use zr_resource::{", facade)
        self.assertNotIn("pub use zr_resource::*;", facade)
        self.assertIn("pub(crate) use zr_resource::assembly::{", facade)
        self.assertNotIn("\npub use zr_resource::assembly", facade)
        self.assertIn("pub mod io;", facade)
        self.assertIn("pub use zr_resource::io::{", io_facade)
        self.assertIn("atomic_write", io_facade)
        self.assertIn("pub(crate) use zr_resource::assembly::io::{", io_facade)
        self.assertNotIn("\npub use zr_resource::assembly", io_facade)
        self.assertNotIn("pub use zr_resource::io::*;", io_facade)

    def test_unimplemented_resource_io_is_not_a_public_product_capability(self) -> None:
        crate_root = self.read_required(ZR_RESOURCE_ROOT / "src/lib.rs")
        io_owner = self.read_required(ZR_RESOURCE_ROOT / "src/io/mod.rs")
        runtime_facade = self.read_required(RUNTIME_RESOURCE_ROOT / "mod.rs")
        runtime_io_facade = self.read_required(RUNTIME_RESOURCE_ROOT / "io/mod.rs")

        self.assertFalse((ZR_RESOURCE_ROOT / "src/io/resource_io.rs").exists())
        self.assertFalse((ZR_RESOURCE_ROOT / "src/io/error.rs").exists())
        for source in (crate_root, io_owner, runtime_facade, runtime_io_facade):
            with self.subTest(source=source[:40]):
                self.assertNotIn("ResourceIo", source)
                self.assertNotIn("ResourceIoError", source)

        self.assertIn("pub use atomic_file::{atomic_write, atomic_write_new};", io_owner)
        self.assertIn("atomic_write", runtime_io_facade)
        self.assertIn("atomic_write_new", runtime_io_facade)

    def test_projection_identity_is_object_backed_and_paired_at_the_authority(self) -> None:
        management = self.read_required(
            ZR_RESOURCE_ROOT / "src/management_generation.rs"
        )
        readiness = self.read_required(
            ZR_RESOURCE_ROOT / "src/readiness_generation.rs"
        )
        manager = self.read_required(
            ZR_RESOURCE_ROOT / "src/manager/resource_manager.rs"
        )
        receipt = self.read_required(ZR_RESOURCE_ROOT / "src/mutation/receipt.rs")

        self.assertIn(
            "pub struct ResourceManagementGenerationIdentity(Arc<ResourceManagementGeneration>)",
            management,
        )
        self.assertIn(
            "pub struct ResourceManagementRowIdentity(Arc<ResourceManagementRow>)",
            management,
        )
        self.assertIn(
            "pub generation: ResourceManagementGenerationIdentity", management
        )
        self.assertIn(
            "pub struct ResourceReadinessGenerationIdentity(Arc<ResourceReadinessGeneration>)",
            readiness,
        )
        self.assertIn(
            "pub struct ResourceReadinessRowIdentity(Arc<ResourceReadinessRow>)",
            readiness,
        )
        self.assertIn("pub struct ResourceProjectionSnapshot", manager)
        self.assertIn("pub fn projection_snapshot(&self)", manager)
        self.assertEqual(1, manager.count("let authority = self\n            .authority\n            .read()"))
        self.assertIn("projections: ResourceProjectionSnapshot", receipt)
        self.assertNotIn("management_generation: u64", receipt)
        self.assertNotIn("readiness_generation: u64", receipt)
        self.assertNotIn("impl Default for ResourceProjectionSnapshot", manager)

    def test_projection_diagnostic_counters_are_not_public_identities(self) -> None:
        management = self.read_required(
            ZR_RESOURCE_ROOT / "src/management_generation.rs"
        )
        readiness = self.read_required(
            ZR_RESOURCE_ROOT / "src/readiness_generation.rs"
        )
        management_projection = self.read_required(
            ZR_RESOURCE_ROOT / "src/manager/management_projection.rs"
        )
        readiness_projection = self.read_required(
            ZR_RESOURCE_ROOT / "src/manager/readiness_projection.rs"
        )

        self.assertNotIn("pub fn sequence(", management)
        self.assertNotIn("pub fn sequence(", readiness)
        self.assertNotIn("pub fn dependency_revision(", readiness)
        self.assertIn("pub publication_count: u64", management)
        self.assertIn("pub publication_count: u64", readiness)
        self.assertNotIn("wrapping_add(1)", management_projection)
        self.assertNotIn("wrapping_add(1)", readiness_projection)

    def test_event_order_is_terminal_and_commit_admitted(self) -> None:
        event_stream = self.read_required(ZR_RESOURCE_ROOT / "src/event_stream.rs")
        event_stream_tests = self.read_required(
            ZR_RESOURCE_ROOT / "src/event_stream/publication_index_tests.rs"
        )
        commit = self.read_required(ZR_RESOURCE_ROOT / "src/manager/commit.rs")

        self.assertIn("next_sequence: Option<u64>", event_stream)
        self.assertIn("cursor: Mutex<Option<u64>>", event_stream)
        self.assertIn("oldest_available_sequence: Option<u64>", event_stream)
        self.assertIn("SequenceExhausted", event_stream)
        self.assertIn("sequence_exhausted: bool", event_stream)
        self.assertIn("rejected_publish_count: u64", event_stream)
        self.assertIn("struct ResourceEventPublishPermit", event_stream)
        self.assertNotIn("publisher_count", event_stream)
        self.assertNotIn("wrapping_add", event_stream)
        self.assertIn("publisher_lifetime.strong_count() == 0", event_stream)
        self.assertNotIn("publisher_lifetime.upgrade()", event_stream)
        self.assertNotIn("pub(crate) fn publish(&self, event", event_stream)
        self.assertIn("recv_timeout(Duration::from_secs(1))", event_stream_tests)
        self.assertIn("publish_permitted", commit)
        self.assertIn("prepare_publish", commit)

    def test_hidden_assembly_surface_is_not_a_product_facade(self) -> None:
        lib = self.read_required(ZR_RESOURCE_ROOT / "src/lib.rs")
        assembly = self.read_required(ZR_RESOURCE_ROOT / "src/assembly.rs")

        self.assertRegex(lib, r"#\[doc\(hidden\)\]\s+pub mod assembly;")
        for symbol in (
            "approximate_event_bytes",
            "PreparedResourceMutation",
            "ResourceReadinessRow",
            "ResourceRegistryStaging",
            "ResourceManagerAssemblyExt",
            "ResourceRegistryAssemblyExt",
        ):
            with self.subTest(symbol=symbol):
                self.assertIn(symbol, assembly)

        self.assertNotIn("ResourceManagementShard", assembly)
        self.assertNotIn("ResourceMutationOperation", assembly)
        self.assertNotIn("pub use crate::data::ResourceData", assembly)
        self.assertNotIn("pub use crate::manager::ResourceManager", assembly)
        self.assertNotIn("pub use crate::snapshot::ResourceSnapshot", assembly)

    def test_products_do_not_depend_directly_on_the_internal_resource_crate(self) -> None:
        product_roots = (
            REPO_ROOT / "zircon_app",
            REPO_ROOT / "zircon_editor",
            REPO_ROOT / "zircon_plugins",
        )
        violations = []
        for root in product_roots:
            for path in rust_sources(root):
                if "zr_resource::" in path.read_text(encoding="utf-8"):
                    violations.append(path.relative_to(REPO_ROOT).as_posix())
        self.assertEqual([], violations)

        interface_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in rust_sources(INTERFACE_RESOURCE_ROOT)
        )
        self.assertNotIn("zr_resource::", interface_sources)

    def test_management_projection_profile_is_reproducible_and_externalized(self) -> None:
        tests = self.read_required(
            ZR_RESOURCE_ROOT / "src/manager/management_projection/tests.rs"
        )
        profile = self.read_required(
            ZR_RESOURCE_ROOT
            / "src/manager/management_projection/tests/profile.rs"
        )

        self.assertIn("mod profile;", tests)
        self.assertIn("#[ignore =", profile)
        self.assertIn("debug_assertions", profile)
        self.assertIn("ZR_RESOURCE_MANAGEMENT_PROFILE_DIR", profile)
        self.assertNotIn("temp_dir()", profile)
        self.assertIn("MIN_PROFILE_SAMPLE_COUNT", profile)
        self.assertIn("31", profile)
        self.assertIn("raw-samples.csv", profile)
        self.assertIn("summary.csv", profile)
        self.assertIn("requested_bytes", profile)
        self.assertIn("peak_live_bytes", profile)
        self.assertIn("projection_source_blake3", profile)
        self.assertIn("generation_source_blake3", profile)
        for scenario in (
            "dense_revision_4096",
            "revision_100000_1",
            "spread_revision_100000_64",
            "spread_revision_100000_4096",
            "add_100000_4096",
            "mixed_100000_4096",
            "remove_100000_4096",
            "rename_100000_4096",
            "dense_revision_100000",
            "no_projected_change_100000",
        ):
            with self.subTest(scenario=scenario):
                self.assertIn(scenario, profile)

    def test_management_projection_only_receives_true_commit_delta(self) -> None:
        commit = self.read_required(ZR_RESOURCE_ROOT / "src/manager/commit.rs")
        projection_window = commit.split("let removed_ids = staged.iter()", 1)[1].split(
            "authority.refresh_readiness_many", 1
        )[0]

        self.assertEqual(1, commit.count(".apply_delta("))
        self.assertIn(
            ".filter(|entry| entry.before != entry.record)", projection_window
        )
        self.assertIn(
            ".apply_delta(removed_ids, changed_records);", projection_window
        )
        self.assertNotIn("authority.registry.iter", projection_window)

    def test_durable_io_identity_is_checked_namespaced_and_version_hard_cut(self) -> None:
        artifact_identity = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/artifact_identity.rs"
        )
        atomic_pathing = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/atomic_file/pathing.rs"
        )
        transaction_pathing = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/transaction/pathing.rs"
        )
        transaction_engine = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/transaction/engine.rs"
        )
        transaction_error = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/transaction/error.rs"
        )
        schema = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/transaction/schema.rs"
        )
        recovery_validation = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/transaction/recovery/validation.rs"
        )

        self.assertIn("NonZeroU64", artifact_identity)
        self.assertIn("compare_exchange_weak", artifact_identity)
        self.assertIn("ArtifactIdentityExhausted", artifact_identity)
        self.assertIn("current.checked_add(1).unwrap_or(0)", artifact_identity)
        self.assertNotIn("fetch_add", atomic_pathing)
        self.assertNotIn("wrapping_add", atomic_pathing)
        self.assertNotIn("fetch_add", transaction_pathing)
        self.assertNotIn("wrapping_add", transaction_pathing)
        self.assertIn("journal_owner_token", transaction_pathing)
        self.assertIn("next_transaction_id(&journal_directory)?", transaction_engine)
        self.assertIn("ArtifactIdentityExhausted", transaction_error)
        self.assertIn("pub(super) const JOURNAL_VERSION: u32 = 7;", schema)
        self.assertIn("valid_transaction_id(&journal.transaction_id, &journal_directory)", recovery_validation)

    def test_durable_io_profile_is_release_only_measured_and_externalized(self) -> None:
        tests = self.read_required(
            ZR_RESOURCE_ROOT / "src/io/transaction/engine/tests.rs"
        )
        profile = self.read_required(
            ZR_RESOURCE_ROOT
            / "src/io/transaction/engine/tests/durable_io_profile.rs"
        )

        self.assertIn("mod durable_io_profile;", tests)
        self.assertIn("#[ignore =", profile)
        self.assertIn("debug_assertions", profile)
        self.assertIn('PROFILE_BATCH_SIZES: [usize; 3] = [1, 16, 256]', profile)
        self.assertIn("MIN_PROFILE_SAMPLE_COUNT: usize = 31", profile)
        self.assertIn("ZR_DURABLE_IO_PROFILE_DIR", profile)
        self.assertNotIn("temp_dir()", profile)
        self.assertIn("assert_profile_directory_is_not_on_c_drive", profile)
        self.assertIn("begin_allocation_profile();", profile)
        self.assertIn("finish_allocation_profile()", profile)
        self.assertIn("commit_prepared_files(", profile)
        self.assertIn("raw-samples.csv", profile)
        self.assertIn("summary.csv", profile)
        self.assertIn("p50_ns", profile)
        self.assertIn("p95_ns", profile)
        self.assertIn("mad_ns", profile)
        self.assertIn("allocation_count", profile)
        self.assertIn("requested_bytes", profile)
        self.assertIn("peak_live_bytes", profile)
        self.assertIn("artifact_identity_source_blake3", profile)
        self.assertIn("transaction_pathing_source_blake3", profile)
        self.assertIn("transaction_engine_source_blake3", profile)
        self.assertIn("journal_schema_source_blake3", profile)
        self.assertIn("power=unavailable", profile)

    def test_readiness_projection_profile_is_isolated_reproducible_and_externalized(self) -> None:
        projection = self.read_required(
            ZR_RESOURCE_ROOT / "src/manager/readiness_projection.rs"
        )
        tests = self.read_required(
            ZR_RESOURCE_ROOT / "src/manager/readiness_projection/tests.rs"
        )
        behavior_red = self.read_required(
            ZR_RESOURCE_ROOT
            / "src/manager/readiness_projection/tests/behavior_red.rs"
        )
        profile = self.read_required(
            ZR_RESOURCE_ROOT / "src/manager/readiness_projection/tests/profile.rs"
        )
        support = self.read_required(ZR_RESOURCE_ROOT / "src/test_profile.rs")

        self.assertIn("mod tests;", projection)
        self.assertIn("mod behavior_red;", tests)
        self.assertIn("mod profile;", tests)
        self.assertIn("cycles must fail closed", behavior_red)
        self.assertIn("deep_chain_10000", behavior_red)
        self.assertIn("#[global_allocator]", support)
        self.assertEqual(1, support.count("#[global_allocator]"))
        self.assertNotIn("#[global_allocator]", profile)
        self.assertIn("resource_readiness_profile_orchestrator", profile)
        self.assertIn("readiness_profile_worker", profile)
        self.assertIn("current_exe()", profile)
        self.assertIn("ZR_RESOURCE_READINESS_PROFILE_DIR", profile)
        self.assertIn("ZR_RESOURCE_READINESS_PROFILE_SCENARIO", profile)
        self.assertIn("ZR_RESOURCE_READINESS_PROFILE_SCOPE", profile)
        self.assertIn("ProfileMeasurementScope", profile)
        self.assertIn("manager_end_to_end", profile)
        self.assertIn("evaluator_only", profile)
        self.assertIn("measurement_scope", profile)
        self.assertIn("PreparedManagerSources", profile)
        self.assertIn("materialize_updates", profile)
        self.assertIn("self.records.get(id).cloned()", profile)
        self.assertIn(".runtime_states", profile)
        self.assertIn(".unwrap_or(RuntimeResourceState::Unloaded)", profile)
        self.assertIn("self.payload_type_ids", profile)
        self.assertNotIn("temp_dir()", profile)
        self.assertIn("MIN_PROFILE_SAMPLE_COUNT", profile)
        self.assertIn("31", profile)
        self.assertIn("raw-samples.csv", profile)
        self.assertIn("summary.csv", profile)
        for field in (
            "allocation_count",
            "requested_bytes",
            "peak_live_bytes",
            "affected_closure_count",
            "edge_visit_count",
            "changed_row_count",
            "projection_source_blake3",
            "generation_source_blake3",
            "profile_source_blake3",
            "allocation_profile_source_blake3",
            "rss=unavailable",
            "power=unavailable",
        ):
            with self.subTest(field=field):
                self.assertIn(field, profile)
        for scenario in (
            "initial_chain_1000",
            "leaf_reload_chain_10000",
            "initial_chain_100000",
            "leaf_reload_fanout_64",
            "leaf_reload_fanout_100000",
            "leaf_reload_diamond_100000",
            "dense_reload_4096",
            "self_cycle",
            "two_node_cycle",
            "cycle_4096",
            "root_edge_replacement_chain_4096",
            "missing_dependency_arrival_chain_4096",
            "no_change_100000",
        ):
            with self.subTest(scenario=scenario):
                self.assertIn(scenario, profile)


if __name__ == "__main__":
    unittest.main()
