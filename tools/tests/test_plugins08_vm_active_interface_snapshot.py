import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANAGER = REPO_ROOT / "zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs"
REGISTRY = REPO_ROOT / "zircon_runtime/src/script/vm/host_interface/registry.rs"
REFLECTION_CATALOG = REPO_ROOT / "zircon_runtime/src/script/vm/reflection/catalog.rs"
PACKAGE_DISCOVERY = (
    REPO_ROOT / "zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs"
)
PAYLOAD_CACHE = (
    REPO_ROOT
    / "zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery/payload_cache.rs"
)
DISCOVERY_IO = (
    REPO_ROOT / "zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery/io.rs"
)
COORDINATOR = REPO_ROOT / "zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs"
GC_OWNER = (
    REPO_ROOT / "zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/gc.rs"
)
GC_SCHEDULE = (
    REPO_ROOT
    / "zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/gc_schedule.rs"
)
GC_DEADLINE = (
    REPO_ROOT
    / "zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/gc_deadline.rs"
)
BEHAVIOR_BRIDGE = REPO_ROOT / "zircon_runtime/src/script/vm/behavior_bridge.rs"


class Plugins08VmActiveInterfaceSnapshotTests(unittest.TestCase):
    def test_stable_interface_queries_do_not_project_full_slot_records(self) -> None:
        source = MANAGER.read_text(encoding="utf-8")
        for method in (
            "registered_systems",
            "registered_behavior_nodes",
            "registered_rpc_handlers",
            "registered_editor_operations",
        ):
            body = self._method_body(source, method)
            self.assertNotIn("list_slots()", body, method)

        run_body = self._method_body(source, "run_registered_systems")
        self.assertIn("systems_snapshot", run_body)

    def test_lifecycle_publishes_one_immutable_active_generation(self) -> None:
        registry = REGISTRY.read_text(encoding="utf-8")
        coordinator = COORDINATOR.read_text(encoding="utf-8")
        manager = MANAGER.read_text(encoding="utf-8")

        self.assertIn("struct VmHostInterfaceActiveSnapshot", registry)
        self.assertIn("fn publish_active_slots", registry)
        self.assertIn("fn rebuild_if_active_generation", registry)
        self.assertIn("fn active_snapshot", registry)
        self.assertIn("fn active_slots", coordinator)
        self.assertGreaterEqual(manager.count("publish_active_interfaces()"), 3)

    def test_package_lookup_and_behavior_callbacks_use_the_active_index(self) -> None:
        manager = MANAGER.read_text(encoding="utf-8")
        behavior_bridge = BEHAVIOR_BRIDGE.read_text(encoding="utf-8")

        lookup = self._method_body(manager, "slot_for_package_name")
        self.assertIn("active_slots_for_package", lookup)
        self.assertNotIn("coordinator.slot_for_package_name", lookup)

        resolve = self._method_body(behavior_bridge, "resolve_callback")
        self.assertIn("active_generation", resolve)
        self.assertNotIn(".slot(slot)", resolve)
        self.assertIn("fn active_slots", COORDINATOR.read_text(encoding="utf-8"))

    def test_gc_step_consumes_a_lifecycle_maintained_next_due_index(self) -> None:
        coordinator = COORDINATOR.read_text(encoding="utf-8")
        gc_owner = GC_OWNER.read_text(encoding="utf-8")
        schedule = GC_SCHEDULE.read_text(encoding="utf-8")
        gc_step = self._method_body(gc_owner, "gc_step")

        self.assertIn("mod gc;", coordinator)
        self.assertIn("GcNextDueSchedule", coordinator)
        self.assertIn("struct GcNextDueSchedule", schedule)
        self.assertIn("due_by_frame: BTreeMap", schedule)
        self.assertIn("fn refresh_gc_schedule", gc_owner)
        self.assertIn("take_due(frame_index)", gc_step)
        self.assertNotIn("gc_policy_is_due(entry, frame_index)", gc_step)
        self.assertNotIn("due_slots.sort", gc_step)

    def test_gc_budget_is_enforced_by_host_wall_clock(self) -> None:
        gc_owner = GC_OWNER.read_text(encoding="utf-8")
        deadline = GC_DEADLINE.read_text(encoding="utf-8")
        gc_step = self._method_body(gc_owner, "gc_step")

        self.assertIn("struct GcFrameDeadline", deadline)
        self.assertIn("GcFrameDeadline::start", gc_step)
        self.assertIn("remaining_micros()", gc_step)
        self.assertNotIn("saturating_sub(pause_micros)", gc_step)
        self.assertIn("host_elapsed_micros", gc_step)

    def test_export_panic_restores_instance_before_resuming_unwind(self) -> None:
        coordinator = COORDINATOR.read_text(encoding="utf-8")
        call_export = self._method_body(coordinator, "call_slot_export")

        catch_index = call_export.index("catch_unwind")
        restore_index = call_export.index("self.restore_slot_instance")
        resume_index = call_export.index("Err(payload) => resume_unwind(payload)")
        self.assertLess(catch_index, restore_index)
        self.assertLess(restore_index, resume_index)

    def test_prepared_reflection_registry_is_published_and_reused(self) -> None:
        catalog = REFLECTION_CATALOG.read_text(encoding="utf-8")
        commit = self._method_body(catalog, "commit_prepared")
        current_snapshot = self._method_body(catalog, "current_snapshot")

        self.assertIn("registry: Arc<TypeRegistry>", catalog)
        self.assertIn("committed_snapshot", catalog)
        self.assertNotIn("registry_for_state", commit)
        self.assertNotIn("validate_candidate", commit)
        self.assertNotIn("registry_for_state", current_snapshot)

    def test_discovery_is_bounded_and_bytecode_is_lazy_single_flight(self) -> None:
        discovery = PACKAGE_DISCOVERY.read_text(encoding="utf-8")
        payload_cache = PAYLOAD_CACHE.read_text(encoding="utf-8")
        manager = MANAGER.read_text(encoding="utf-8")

        self.assertIn("VmPluginDiscoveryLimits", discovery)
        self.assertIn("discover_vm_plugin_packages_with_limits", discovery)
        self.assertNotIn("fs::read(&bytecode_path)", discovery)
        self.assertIn("OnceLock", payload_cache)
        self.assertIn("max_bytecode_bytes", payload_cache)
        self.assertIn("materialize(package)", self._method_body(manager, "load_discovered_package"))
        self.assertIn(
            "materialize(package)",
            self._method_body(manager, "hot_reload_discovered_slot"),
        )

    def test_discovery_runs_on_runtime11_bounded_io_lane(self) -> None:
        discovery_io = DISCOVERY_IO.read_text(encoding="utf-8")
        manager = MANAGER.read_text(encoding="utf-8")

        self.assertIn("BoundedKeyedIoLane", discovery_io)
        self.assertIn("JobScheduler::process_io()", discovery_io)
        self.assertIn("BoundedKeyedIoWorkDeadline::at", discovery_io)
        self.assertIn("cancel_before_start", discovery_io)
        self.assertIn("AtomicBool", discovery_io)
        self.assertIn("discover_vm_plugin_packages_cancellable", discovery_io)
        self.assertIn("submit_package_discovery", manager)
        discover = self._method_body(manager, "discover_packages")
        self.assertIn("submit_package_discovery", discover)
        self.assertNotIn("discover_vm_plugin_packages(root)", discover)

    @staticmethod
    def _method_body(source: str, method: str) -> str:
        match = re.search(
            rf"(?:pub(?:\([^)]*\))?\s+)?fn\s+{method}\s*\([^{{]*\)\s*(?:->\s*[^{{]+)?\{{",
            source,
        )
        if match is None:
            raise AssertionError(f"missing method {method}")
        depth = 1
        index = match.end()
        while index < len(source) and depth:
            if source[index] == "{":
                depth += 1
            elif source[index] == "}":
                depth -= 1
            index += 1
        return source[match.end() : index - 1]


if __name__ == "__main__":
    unittest.main()
