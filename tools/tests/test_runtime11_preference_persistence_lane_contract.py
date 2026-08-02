from __future__ import annotations

import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TASKS_ROOT = REPO_ROOT / "zircon_runtime/src/core/runtime/tasks"
LANE_ROOT = TASKS_ROOT / "bounded_keyed_io"
PLATFORM_PREFERENCES_ROOT = REPO_ROOT / "zircon_runtime/src/platform/preferences"
PERSISTENCE_ROOT = PLATFORM_PREFERENCES_ROOT / "persistence"
MANAGER_SOURCE = REPO_ROOT / "zircon_runtime/src/platform/service_types/manager.rs"
STORAGE_CONTRACT = (
    REPO_ROOT / "zircon_runtime/src/core/framework/platform/preferences/storage.rs"
)
NEUTRAL_PREFERENCES_ROUTE = (
    REPO_ROOT / "zircon_runtime/src/core/framework/platform/preferences/mod.rs"
)
NEUTRAL_PLATFORM_ROUTE = REPO_ROOT / "zircon_runtime/src/core/framework/platform/mod.rs"
HOST_BACKEND_CONTRACT = PLATFORM_PREFERENCES_ROOT / "backend.rs"
OLD_NEUTRAL_BACKEND_CONTRACT = (
    REPO_ROOT / "zircon_runtime/src/core/framework/platform/preferences/backend.rs"
)
HOST_PREFERENCES_ROUTE = PLATFORM_PREFERENCES_ROOT / "mod.rs"
PLATFORM_ROOT = REPO_ROOT / "zircon_runtime/src/platform"
PLATFORM_ROUTE = PLATFORM_ROOT / "mod.rs"
PERSISTENCE_WORK = PERSISTENCE_ROOT / "work.rs"
PERSISTENCE_TESTS = PERSISTENCE_ROOT / "tests.rs"
ATOMIC_FILE_BACKEND = PLATFORM_PREFERENCES_ROOT / "atomic_file.rs"
EXTERNAL_BACKEND_AUTHORITY_TEST = (
    REPO_ROOT / "zircon_runtime/tests/runtime11_preference_backend_authority.rs"
)
WOC_CLIENT_ROOT = REPO_ROOT / "examples/woc/native/apps/woc_client"
WOC_CLIENT_MANIFEST = WOC_CLIENT_ROOT / "Cargo.toml"
WOC_WORKSPACE_LOCK = WOC_CLIENT_ROOT.parents[1] / "Cargo.lock"
WOC_STORAGE_BRIDGE = WOC_CLIENT_ROOT / "src/preferences/storage.rs"
WOC_STORAGE_TEST_SUPPORT = WOC_CLIENT_ROOT / "tests/preference_storage_support.rs"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def rust_braced_ranges(source: str, header: re.Pattern[str]) -> list[tuple[int, int]]:
    ranges: list[tuple[int, int]] = []
    for match in header.finditer(source):
        opening_brace = source.find("{", match.start(), match.end())
        if opening_brace < 0:
            continue
        depth = 1
        cursor = opening_brace + 1
        while cursor < len(source) and depth > 0:
            if source[cursor] == "{":
                depth += 1
            elif source[cursor] == "}":
                depth -= 1
            cursor += 1
        if depth == 0:
            ranges.append((match.start(), cursor))
    return ranges


def authority_type_names(source: str) -> set[str]:
    names = {"PreferenceBackendWorkAuthority"}
    aliases = list(
        re.finditer(
            r"\btype\s+(?P<alias>[A-Za-z_]\w*)\s*=\s*(?P<target>[^;]+);",
            source,
        )
    )
    changed = True
    while changed:
        changed = False
        for alias in aliases:
            target_tokens = set(re.findall(r"\b[A-Za-z_]\w*\b", alias.group("target")))
            if names.isdisjoint(target_tokens) or alias.group("alias") in names:
                continue
            names.add(alias.group("alias"))
            changed = True
    for alias in re.finditer(
        r"\buse\s+[^;]*\bPreferenceBackendWorkAuthority\s+as\s+"
        r"(?P<alias>[A-Za-z_]\w*)\s*;",
        source,
    ):
        names.add(alias.group("alias"))
    return names


def authority_forgeability_violations(source: str) -> set[str]:
    violations: set[str] = set()
    authority_declaration = re.search(
        r"(?P<attributes>(?:#\[[^\]]+\]\s*)*)"
        r"pub\s+struct\s+PreferenceBackendWorkAuthority\b",
        source,
    )
    if authority_declaration is None:
        return {"missing authority declaration"}

    derive_traits: set[str] = set()
    for derive in re.findall(
        r"#\[\s*derive\s*\((?P<traits>[^)]*)\)\s*\]",
        authority_declaration.group("attributes"),
    ):
        derive_traits.update(
            token.rsplit("::", 1)[-1].strip()
            for token in derive.split(",")
            if token.strip()
        )
    if not derive_traits.isdisjoint({"Clone", "Copy", "Default"}):
        violations.add("forgeable derive")

    authority_names = authority_type_names(source)
    authority_impl_ranges = rust_braced_ranges(
        source,
        re.compile(
            r"\bimpl(?:\s*<[^>{}]*>)?\s+"
            r"(?:(?:[A-Za-z_]\w*)::)*PreferenceBackendWorkAuthority\s*\{"
        ),
    )

    def is_in_authority_impl(position: int) -> bool:
        return any(start <= position < end for start, end in authority_impl_ranges)

    def mentions_authority(type_source: str, *, include_self: bool) -> bool:
        type_tokens = set(re.findall(r"\b[A-Za-z_]\w*\b", type_source))
        if include_self and "Self" in type_tokens:
            return True
        return not authority_names.isdisjoint(type_tokens)

    public_authority_factory = re.compile(
        r"\bpub(?:\s*\([^)]*\))?\s+"
        r"(?:(?:const|async|unsafe)\s+)*fn\s+\w+"
        r"(?:\s*<[^>{}]*>)?\s*\([^)]*\)\s*->\s*(?P<return>[^;{]*)",
        re.DOTALL,
    )
    for factory in public_authority_factory.finditer(source):
        if mentions_authority(
            factory.group("return"),
            include_self=is_in_authority_impl(factory.start()),
        ):
            violations.add("public authority factory")
            break

    public_authority_value = re.compile(
        r"\bpub(?:\s*\([^)]*\))?\s+(?:const|static)\s+\w+\s*:"
        r"\s*(?P<type>[^;=]*)"
    )
    for value in public_authority_value.finditer(source):
        if mentions_authority(
            value.group("type"),
            include_self=is_in_authority_impl(value.start()),
        ):
            violations.add("public authority value")
            break

    forgeable_trait_impl = re.compile(
        r"\bimpl(?:\s*<[^>{}]*>)?\s+"
        r"(?:::)?(?:(?:[A-Za-z_]\w*)::)*(?:Clone|Copy|Default)\s+for\s+"
        r"(?P<target>(?:(?:[A-Za-z_]\w*)::)*[A-Za-z_]\w*)\b"
    )
    for trait_impl in forgeable_trait_impl.finditer(source):
        target = trait_impl.group("target").rsplit("::", 1)[-1]
        if target in authority_names:
            violations.add("forgeable trait impl")
            break
    return violations


class Runtime11PreferencePersistenceLaneContractTests(unittest.TestCase):
    def test_runtime_lane_is_folder_backed_bounded_and_domain_neutral(self) -> None:
        expected_files = {
            "mod.rs",
            "admission.rs",
            "diagnostics.rs",
            "fence.rs",
            "lane.rs",
            "ticket.rs",
        }
        self.assertTrue(LANE_ROOT.is_dir(), "Runtime11 bounded keyed I/O owner is missing")
        self.assertEqual(
            expected_files,
            {path.name for path in LANE_ROOT.glob("*.rs")} & expected_files,
        )

        source = "\n".join(read(path) for path in sorted(LANE_ROOT.rglob("*.rs")))
        for anchor in (
            "BoundedKeyedIoLimits",
            "BoundedKeyedIoLane",
            "BoundedKeyedIoTicket",
            "BoundedKeyedIoFence",
            "BoundedKeyedIoTerminal",
            "BoundedKeyedIoDiagnostics",
            "BoundedKeyedIoShutdownGuard",
            "BoundedKeyedIoWorkDeadline",
            "wait_until",
            "cancel_before_start",
            "GlobalAdmissionEpoch",
            "FencePrerequisite",
            "TaskTimer",
            "Condvar",
            "catch_unwind",
            "work_panicked",
            "plan_fence_prerequisites",
            "size_of::<FencePrerequisite>()",
            "BoundedKeyedIoShutdownReport",
            "marks_active_work_started_before_shutdown_can_linearize",
            "shutdown_waits_for_terminal_observer_and_pump_handle",
            "admission_matrix_stays_bounded_at_one_thousand_and_hundred_thousand",
            "same_key_storm_does_not_starve_an_interleaved_key",
            "charges_fence_prerequisites_before_capture",
            "consecutive_fences_retain_linear_prerequisite_records",
            "max_entries",
            "max_retained_bytes",
            "Superseded",
        ):
            self.assertIn(anchor, source)

        for forbidden in (
            "Preference",
            "core::framework::platform",
            "platform::preferences",
            "std::thread",
            "spawn_named_thread",
            "JoinHandle",
            "ThreadPoolBuilder",
        ):
            self.assertNotIn(forbidden, source)
        self.assertIn("JobHandle", source)

        tasks_route = read(TASKS_ROOT / "mod.rs")
        self.assertIn("mod bounded_keyed_io;", tasks_route)
        self.assertIn("pub use bounded_keyed_io::", tasks_route)

    def test_platform_adapter_owns_overlay_quotes_and_backend_work(self) -> None:
        expected_files = {"mod.rs", "adapter.rs", "overlay.rs", "work.rs"}
        self.assertTrue(
            PERSISTENCE_ROOT.is_dir(),
            "Platform preference persistence adapter owner is missing",
        )
        self.assertEqual(
            expected_files,
            {path.name for path in PERSISTENCE_ROOT.glob("*.rs")} & expected_files,
        )

        source = "\n".join(read(path) for path in sorted(PERSISTENCE_ROOT.glob("*.rs")))
        for anchor in (
            "PreferencePersistenceAdapter",
            "PreferenceReadSnapshot",
            "PreferenceDurabilityState",
            "PreferenceOverlayLimits",
            "PreferenceOverlayReservation",
            "PreferencePersistenceQuote",
            "PreferencePersistenceDiagnostics",
            "PreferenceOverlayDiagnostics",
            "PreferencePersistenceLimitsError",
            "overlay_retained_bytes",
            "lane_retained_bytes",
            "checked_add",
            "MAX_PREFERENCE_VALUE_BYTES",
            "MAX_PREFERENCE_FAILURE_DETAIL_BYTES",
            "PreferencePersistenceFailureProjection",
            "truncate_utf8_detail",
            "visible_not_durable",
            "quote_retained_bytes",
            "install_generation_before_runnable",
            "complete_if_generation",
            "known_non_durable_failure",
            "overlay replacement quote underflow",
            "entry.durability != PreferenceDurabilityState::VisibleNotDurable",
            "submission: Mutex<()>",
            "observe_terminal",
            "caller_filesystem_wall",
            "flush_before_later_different_key",
            "default_limits_allow_maximum_value_failure_retry",
            "rejects_pending_and_durable_eviction",
            "one_second_backend_stall_remains_off_caller_filesystem_wall",
            "PreferenceStorageBackend",
        ):
            self.assertIn(anchor, source)

    def test_sync_backend_spi_is_hard_cut_from_neutral_framework_surface(self) -> None:
        neutral_source = read(NEUTRAL_PREFERENCES_ROUTE) + read(NEUTRAL_PLATFORM_ROUTE)
        backend_trait = re.compile(r"\bPreferenceStorageBackend\b")
        self.assertIsNone(backend_trait.search(neutral_source))
        self.assertNotIn("mod backend;", neutral_source)
        self.assertFalse(
            OLD_NEUTRAL_BACKEND_CONTRACT.exists(),
            "retired neutral backend SPI file must be deleted, not orphaned",
        )

        self.assertTrue(
            HOST_BACKEND_CONTRACT.is_file(),
            "sync host backend SPI must move to platform::preferences",
        )
        host_route = read(HOST_PREFERENCES_ROUTE)
        self.assertIn("mod backend;", host_route)
        self.assertIn("mod persistence;", host_route)
        self.assertIn("backend::PreferenceStorageBackend", host_route)
        host_source = read(HOST_BACKEND_CONTRACT)
        self.assertIn("pub trait PreferenceStorageBackend", host_source)
        for primitive in ("fn open_read(", "fn write(", "fn remove(", "fn flush("):
            self.assertIn(primitive, host_source)
        self.assertGreaterEqual(host_source.count("PreferenceBackendWorkAuthority"), 4)
        self.assertIn("Box<dyn Read + Send>", host_source)

        platform_route = read(PLATFORM_ROUTE)
        external_contract = read(EXTERNAL_BACKEND_AUTHORITY_TEST)
        for internal_type in (
            "PreferencePersistenceAdapter",
            "PreferencePersistenceLimits",
            "PreferencePersistenceQuote",
            "PreferencePersistenceDiagnostics",
            "PreferenceOverlayDiagnostics",
        ):
            self.assertNotIn(internal_type, platform_route)
            self.assertNotIn(internal_type, external_contract)
        self.assertIn("pub(crate) use persistence::", host_route)
        self.assertNotIn("impl Default for PlatformManager", read(MANAGER_SOURCE))

    def test_consumer_contract_is_hard_cut_to_submission_snapshot_and_fence(self) -> None:
        source = read(STORAGE_CONTRACT)
        for retired_signature in (
            "fn read(&self, key: &PreferenceKey) -> Result<Option<Vec<u8>>",
            "fn write(&self, key: &PreferenceKey, value: &[u8]) -> Result<(),",
            "fn remove(&self, key: &PreferenceKey) -> Result<(),",
            "fn flush(&self) -> Result<(),",
        ):
            self.assertNotIn(retired_signature, source)

        for anchor in (
            "PreferenceReadSnapshot",
            "PreferenceMutationTicket",
            "PreferenceFlushTicket",
            "submit_write",
            "submit_remove",
            "flush_fence",
        ):
            self.assertIn(anchor, source)

    def test_woc_consumes_runtime_preference_service_without_sync_compat_trait(self) -> None:
        manifest = read(WOC_CLIENT_MANIFEST)
        self.assertIn(
            'zircon_runtime = { path = "../../../../../zircon_runtime", '
            'default-features = false }',
            manifest,
        )

        lock = tomllib.loads(read(WOC_WORKSPACE_LOCK))
        packages = lock.get("package", [])
        woc_client = next(
            package for package in packages if package.get("name") == "woc_client"
        )
        self.assertIn("zircon_runtime", woc_client.get("dependencies", []))
        self.assertTrue(
            any(package.get("name") == "zircon_runtime" for package in packages),
            "nested WOC lock must include the engine dependency package",
        )

        bridge = read(WOC_STORAGE_BRIDGE)
        self.assertNotIn("pub trait PreferenceStorage", bridge)
        self.assertNotIn("fn read(&self, key: &str)", bridge)
        self.assertNotIn("fn write(&self, key: &str", bridge)
        for anchor in (
            "zircon_runtime::core::framework::platform",
            "storage.snapshot(&key)",
            ".submit_write(",
            "PreferenceWorkDeadline::none()",
            "PreferenceRead::Pending",
            "PreferenceDurabilityState::Pending",
        ):
            self.assertIn(anchor, bridge)

        production = "\n".join(
            read(path) for path in sorted((WOC_CLIENT_ROOT / "src").rglob("*.rs"))
        )
        self.assertNotIn("crate::preferences::PreferenceStorage", production)
        self.assertNotIn("S: PreferenceStorage", production)
        self.assertNotIn(
            "pub use storage::*", read(WOC_CLIENT_ROOT / "src/preferences/mod.rs")
        )
        self.assertIn("S: AsRef<dyn PreferenceStorage>", production)
        for anchor in (
            "refresh_from_storage",
            "take_persistence_submission",
            "last_persistence_submission",
            "try_load_inventory_filter",
        ):
            self.assertIn(anchor, production)

        support = read(WOC_STORAGE_TEST_SUPPORT)
        for anchor in (
            "CoreRuntime::new()",
            "platform_preference_storage_handle",
            "resolve_manager_service",
            "impl PreferenceStorageBackend for MemoryPreferenceBackend",
            "seed_persisted",
            "block_read",
            "wait_until_read_started",
            "release_read",
            "wait_until_loaded",
        ):
            self.assertIn(anchor, support)

    def test_platform_manager_never_executes_backend_primitive_directly(self) -> None:
        source = read(MANAGER_SOURCE)
        backend_trait = re.compile(r"\bPreferenceStorageBackend\b")
        for direct_call in (
            "preference_storage_backend().read(",
            "preference_storage_backend().write(",
            "preference_storage_backend().remove(",
            "preference_storage_backend().flush(",
        ):
            self.assertNotIn(direct_call, source)
        self.assertIsNone(backend_trait.search(source))
        self.assertIn("PreferencePersistenceAdapter", source)

        backend_type_allowlist = {
            "mod.rs",
            "preferences/atomic_file.rs",
            "preferences/backend.rs",
            "preferences/mod.rs",
            "preferences/persistence/adapter.rs",
            "preferences/persistence/tests.rs",
            "preferences/persistence/work.rs",
            "preferences/unavailable.rs",
            "service_types/driver.rs",
            "tests/preferences.rs",
        }
        primitive_call_allowlist = {"preferences/persistence/work.rs"}
        primitive_call = re.compile(
            r"(?<!\.)\bbackend\s*\.\s*(?:open_read|write|remove|flush)\s*\("
        )
        for path in PLATFORM_ROOT.rglob("*.rs"):
            relative = path.relative_to(PLATFORM_ROOT).as_posix()
            production = read(path)
            if backend_trait.search(production):
                self.assertIn(relative, backend_type_allowlist)
            if backend_trait.search(production) and primitive_call.search(production):
                self.assertIn(relative, primitive_call_allowlist)

    def test_backend_work_authority_blocks_local_receiver_and_helper_bypass(self) -> None:
        safe_declaration = "pub struct PreferenceBackendWorkAuthority { _private: () }"
        self.assertEqual(set(), authority_forgeability_violations(safe_declaration))
        unrelated_self_factory = f"""
            {safe_declaration}
            pub struct PreferenceBackendWork;
            impl PreferenceBackendWork {{
                pub(super) fn new() -> Self {{ Self }}
            }}
        """
        self.assertEqual(
            set(), authority_forgeability_violations(unrelated_self_factory)
        )
        bypasses = {
            "unordered derive": """
                #[derive(Debug, Clone, Copy, Default)]
                pub struct PreferenceBackendWorkAuthority { _private: () }
            """,
            "wrapped public factory": f"""
                {safe_declaration}
                impl PreferenceBackendWorkAuthority {{
                    pub(crate) fn issue() -> Option<Self> {{ None }}
                }}
            """,
            "public wrapped static": f"""
                {safe_declaration}
                pub static AUTHORITY: Option<PreferenceBackendWorkAuthority> = None;
            """,
            "qualified default impl": f"""
                {safe_declaration}
                impl ::std::default::Default for self::PreferenceBackendWorkAuthority {{
                    fn default() -> Self {{ Self {{ _private: () }} }}
                }}
            """,
            "associated public authority const": f"""
                {safe_declaration}
                impl PreferenceBackendWorkAuthority {{
                    pub const INSTANCE: Self = Self {{ _private: () }};
                }}
            """,
            "aliased public authority factory": f"""
                {safe_declaration}
                type BackendAuthority = PreferenceBackendWorkAuthority;
                pub(super) fn issue() -> Option<BackendAuthority> {{ None }}
            """,
        }
        for label, bypass in bypasses.items():
            self.assertTrue(
                authority_forgeability_violations(bypass),
                f"authority guard accepted safe-Rust bypass: {label}",
            )

        self.assertTrue(
            HOST_BACKEND_CONTRACT.is_file(),
            "host backend SPI is missing",
        )
        self.assertTrue(
            PERSISTENCE_WORK.is_file(),
            "persistence worker authority owner is missing",
        )
        self.assertTrue(
            EXTERNAL_BACKEND_AUTHORITY_TEST.is_file(),
            "external host backend authority compile fixture is missing",
        )
        host_source = read(HOST_BACKEND_CONTRACT)
        work_source = read(PERSISTENCE_WORK)
        host_route = read(HOST_PREFERENCES_ROUTE)
        platform_route = read(PLATFORM_ROUTE)
        external_fixture = read(EXTERNAL_BACKEND_AUTHORITY_TEST)
        self.assertIn("PreferenceBackendWorkAuthority", host_source)
        self.assertIn("pub mod preferences;", platform_route)
        self.assertIn(
            "pub use persistence::PreferenceBackendWorkAuthority;",
            host_route,
        )
        self.assertIn("pub struct PreferenceBackendWorkAuthority", work_source)
        self.assertIn("_private: ()", work_source)
        self.assertIn("fn new() -> Self", work_source)
        self.assertEqual(set(), authority_forgeability_violations(work_source))
        self.assertIn("```compile_fail", work_source)
        self.assertIn("PreferenceBackendWorkAuthority { _private: () }", work_source)

        for primitive in ("open_read", "write", "remove", "flush"):
            self.assertRegex(
                host_source,
                rf"fn\s+{primitive}\s*\(\s*&self\s*,\s*authority\s*:\s*&PreferenceBackendWorkAuthority",
            )
        self.assertIn("impl PreferenceStorageBackend for ExternalBackend", external_fixture)
        self.assertIn("&PreferenceBackendWorkAuthority", external_fixture)
        self.assertIn("external_backend_can_implement_host_spi", external_fixture)
        self.assertIn("authority_is_issued_only_by_persistence_worker", external_fixture)

        authority_construction_sites = []
        for path in PLATFORM_ROOT.rglob("*.rs"):
            source = read(path)
            if (
                "PreferenceBackendWorkAuthority::new()" in source
                or "PreferenceBackendWorkAuthority {" in source
            ):
                authority_construction_sites.append(
                    path.relative_to(PLATFORM_ROOT).as_posix()
                )
        self.assertEqual(["preferences/persistence/work.rs"], authority_construction_sites)
        self.assertNotIn("PreferenceBackendWorkAuthority", read(MANAGER_SOURCE))

        self.assertTrue(PERSISTENCE_TESTS.is_file(), "bounded read tests are missing")
        persistence_tests = read(PERSISTENCE_TESTS)
        self.assertIn(
            "bounded_backend_read_consumes_at_most_max_plus_one_bytes",
            persistence_tests,
        )
        self.assertIn("CountingRead", persistence_tests)
        self.assertIn("read_bounded", work_source)
        self.assertIn("checked_add(1)", work_source)
        self.assertIn(".take(", work_source)
        atomic_source = read(ATOMIC_FILE_BACKEND)
        self.assertNotIn("fs::read(", atomic_source)
        self.assertIn("File::open(", atomic_source)


if __name__ == "__main__":
    unittest.main()
