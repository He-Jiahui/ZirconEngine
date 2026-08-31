import unittest
from pathlib import Path

from tools.runtime_ui_dispatch_route_sharing_pressure import run, validate_output_path


ROOT = Path(__file__).resolve().parents[2]


class RuntimeUiDispatchRouteSharingPressureTests(unittest.TestCase):
    def test_depth_matrix_eliminates_event_route_deep_copies(self) -> None:
        report = run(event_count=1_000_000)
        cases = {
            (case["route_depth"], case["handler_count"]): case
            for case in report["cases"]
        }

        depth_100 = cases[(100, 4)]
        self.assertEqual(depth_100["head_pointer_route_clone_count"], 2_000_000)
        self.assertEqual(depth_100["candidate_pointer_route_clone_count"], 0)
        self.assertEqual(
            depth_100["head_pointer_total_identity_copies"], 616_000_000
        )
        self.assertEqual(
            depth_100["head_pointer_vector_allocations_lower_bound"], 11_000_000
        )
        self.assertEqual(
            depth_100["candidate_pointer_vector_allocations_lower_bound"], 0
        )
        self.assertEqual(
            depth_100["head_pointer_candidate_vector_copy_count"], 1_000_000
        )
        self.assertEqual(
            depth_100["candidate_pointer_candidate_vector_copy_count"], 0
        )
        self.assertEqual(
            depth_100["head_pointer_payload_bytes_lower_bound"], 4_928_000_000
        )
        self.assertEqual(
            depth_100["head_navigation_route_clone_count"], 2_000_000
        )
        self.assertEqual(
            depth_100["head_navigation_vector_allocations_lower_bound"], 3_000_000
        )
        self.assertEqual(
            depth_100["candidate_navigation_route_clone_count"], 0
        )
        self.assertEqual(
            depth_100["head_pointer_visited_heap_allocation_count"], 1_000_000
        )
        self.assertEqual(
            depth_100["candidate_pointer_visited_heap_allocation_count"],
            1_000_000,
        )
        self.assertEqual(
            depth_100["head_navigation_visited_heap_allocation_count"],
            1_000_000,
        )
        self.assertEqual(
            depth_100["candidate_navigation_visited_heap_allocation_count"],
            1_000_000,
        )
        self.assertEqual(depth_100["head_visited_node_insert_count"], 100_000_000)
        self.assertEqual(
            depth_100["candidate_visited_node_insert_count"], 100_000_000
        )

    def test_inline_visited_set_removes_heap_work_for_typical_route_depths(self) -> None:
        report = run(event_count=1_000_000)
        cases = {
            (case["route_depth"], case["handler_count"]): case
            for case in report["cases"]
        }

        for route_depth in (1, 10):
            case = cases[(route_depth, 4)]
            self.assertEqual(
                case["head_pointer_visited_heap_allocation_count"], 1_000_000
            )
            self.assertEqual(
                case["candidate_pointer_visited_heap_allocation_count"], 0
            )
            self.assertEqual(
                case["head_navigation_visited_heap_allocation_count"], 1_000_000
            )
            self.assertEqual(
                case["candidate_navigation_visited_heap_allocation_count"], 0
            )
        self.assertEqual(
            cases[(10, 4)][
                "candidate_pointer_disjoint_ancestry_heap_allocation_upper_bound"
            ],
            1_000_000,
        )
        self.assertEqual(
            cases[(10, 4)]["pointer_shared_ancestry_unique_node_count"], 14
        )
        self.assertEqual(
            cases[(10, 4)]["pointer_disjoint_ancestry_unique_node_upper_bound"],
            40,
        )
        self.assertEqual(report["inputs"]["inline_visited_capacity"], 16)
        self.assertTrue(report["invariants"]["typical_route_visited_set_is_inline"])

    def test_handler_count_does_not_change_route_copy_work(self) -> None:
        report = run(event_count=10, route_depths=(10,), handler_counts=(1, 4))
        one_handler, four_handlers = report["cases"]

        for key in (
            "head_pointer_route_clone_count",
            "candidate_pointer_route_clone_count",
            "head_pointer_payload_bytes_lower_bound",
            "candidate_pointer_payload_bytes_lower_bound",
            "head_navigation_route_clone_count",
            "candidate_navigation_route_clone_count",
        ):
            self.assertEqual(one_handler[key], four_handlers[key])
        self.assertFalse(
            report["invariants"]["handler_count_changes_route_clone_bytes"]
        )

    def test_head_baseline_and_handler_topology_are_explicitly_bound(self) -> None:
        report = run(
            event_count=10,
            route_depths=(10,),
            handler_counts=(4,),
            handler_bearing_node_phase_count=2,
            inline_visited_capacity=32,
        )
        case = report["cases"][0]

        self.assertEqual(case["head_pointer_route_clone_count"], 30)
        self.assertEqual(case["head_navigation_route_clone_count"], 30)
        self.assertEqual(len(report["source_binding"]["git_revision"]), 40)
        self.assertEqual(
            report["source_binding"]["baseline_git_revision"],
            "5ffc4945095a6fc734bcbb2e632958026350b760",
        )
        self.assertTrue(
            report["source_binding"]["candidate_contract"]["ready"],
            report["source_binding"]["candidate_contract"]["blockers"],
        )
        self.assertEqual(
            set(report["source_binding"]["head_baseline_files"]),
            {
                "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs",
                "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs",
                "zircon_runtime_interface/src/ui/dispatch/pointer/context.rs",
                "zircon_runtime_interface/src/ui/dispatch/navigation/context.rs",
                "zircon_runtime_interface/src/ui/surface/hit.rs",
                "zircon_runtime_interface/src/ui/surface/pointer/route.rs",
            },
        )
        self.assertIn(
            "managed Rust clone/allocation counters",
            report["interpretation"]["dynamic_acceptance_pending"],
        )

    def test_rejects_invalid_inputs(self) -> None:
        with self.assertRaises(ValueError):
            run(event_count=0)
        with self.assertRaises(ValueError):
            run(event_count=1, route_depths=(0,))
        with self.assertRaises(ValueError):
            run(event_count=1, handler_counts=())
        with self.assertRaises(ValueError):
            run(event_count=1, stacked_candidate_count=-1)
        with self.assertRaises(ValueError):
            run(event_count=1, node_identity_bytes=0)
        with self.assertRaises(ValueError):
            run(event_count=1, handler_bearing_node_phase_count=0)
        with self.assertRaises(ValueError):
            run(event_count=1, inline_visited_capacity=0)

    def test_current_dispatchers_use_inline_visited_storage_with_deep_fallback(self) -> None:
        pointer = (
            ROOT / "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs"
        ).read_text(encoding="utf-8")
        navigation = (
            ROOT / "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs"
        ).read_text(encoding="utf-8")
        visited = (
            ROOT / "zircon_runtime/src/ui/dispatch/visited_node_set.rs"
        ).read_text(encoding="utf-8")

        for dispatcher in (pointer, navigation):
            self.assertIn("UiDispatchVisitedNodeSet::with_expected_len", dispatcher)
            self.assertNotIn("HashSet::with_capacity", dispatcher)
        self.assertIn("UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY: usize = 16", visited)
        self.assertIn("HashSet::with_capacity", visited)
        self.assertIn("overflow.extend(self.inline.iter().copied())", visited)

    def test_output_is_restricted_to_profile_drives(self) -> None:
        for path in (
            Path("D:/profiles/dispatch.json"),
            Path("E:/profiles/dispatch.json"),
            Path("F:/profiles/dispatch.json"),
        ):
            self.assertEqual(validate_output_path(path), path)
        for path in (Path("C:/profiles/dispatch.json"), Path("dispatch.json")):
            with self.assertRaises(ValueError):
                validate_output_path(path)


if __name__ == "__main__":
    unittest.main()
