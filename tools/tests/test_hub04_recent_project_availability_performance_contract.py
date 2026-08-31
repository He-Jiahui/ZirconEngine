import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
HUB_SNAPSHOT = REPO_ROOT / "zircon_hub/src/state/hub_snapshot.rs"
RUNTIME_STATE = REPO_ROOT / "zircon_hub/src/tauri_app/runtime_state.rs"
VIEW_MODEL = REPO_ROOT / "zircon_hub/src/tauri_app/view_model.rs"


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class HubRecentProjectAvailabilityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.snapshot = HUB_SNAPSHOT.read_text(encoding="utf-8")
        cls.runtime_state = RUNTIME_STATE.read_text(encoding="utf-8")
        cls.view_model = VIEW_MODEL.read_text(encoding="utf-8")

    def test_repeated_view_model_projection_reuses_availability_snapshot(self) -> None:
        self.assertIn("ProjectAvailabilitySnapshot", self.snapshot)
        self.assertIn(
            "project_availability: RefCell<ProjectAvailabilitySnapshot>",
            self.runtime_state,
        )
        view_model = function_body(self.runtime_state, "view_model")
        self.assertIn(".synchronize_with_selected(", view_model)
        self.assertIn("snapshot.selected_project_path.as_deref()", view_model)
        self.assertIn("HubViewModel::from_snapshot_with_availability", view_model)
        self.assertNotIn(".exists()", view_model)

        synchronize = function_body(
            self.snapshot,
            "synchronize_with_selected_and_probe",
        )
        normalized = " ".join(synchronize.split())
        self.assertIn("self.by_path.len() == expected_len", normalized)
        self.assertIn("self.by_path.contains_key(&project.path)", normalized)
        self.assertNotIn("projects .iter() .any", normalized)

    def test_filter_and_detail_projection_do_not_probe_the_filesystem(self) -> None:
        filtered = function_body(self.snapshot, "filtered_recent_projects_with_availability")
        includes = function_body(self.snapshot, "includes")
        detail = function_body(self.view_model, "project_detail_from_parts")
        self.assertIn("availability.path_exists(&project.path)", includes)
        self.assertIn("availability.path_exists(path)", detail)
        self.assertNotIn(".exists()", filtered)
        self.assertNotIn(".exists()", includes)
        self.assertNotIn(".exists()", detail)

    def test_focus_refresh_rechecks_cached_path_statuses(self) -> None:
        refresh = function_body(
            self.runtime_state,
            "refresh_shared_recent_projects_on_focus_unchecked",
        )
        self.assertIn("project_availability", refresh)
        self.assertIn(".refresh_with_selected(", refresh)
        self.assertIn("self.selected_project_path.as_deref()", refresh)
        self.assertIn("recent_projects_changed || availability_changed", refresh)

    def test_uncached_selected_project_is_admitted_to_the_same_snapshot(self) -> None:
        constructor = function_body(self.view_model, "from_snapshot")
        self.assertIn("ProjectAvailabilitySnapshot::capture_with_selected", constructor)
        self.assertIn("snapshot.selected_project_path.as_deref()", constructor)

        synchronize = function_body(self.snapshot, "synchronize_with_selected_and_probe")
        self.assertIn("selected_path", synchronize)
        self.assertIn("synchronized.contains_key(path)", synchronize)
        self.assertIn("probe(path)", synchronize)
        self.assertIn(
            "hub04_project_availability_caches_selected_path_outside_recents",
            self.snapshot,
        )

    def test_behavior_and_release_performance_evidence_are_pinned(self) -> None:
        for test_name in (
            "hub04_project_availability_synchronize_only_probes_added_paths",
            "hub04_project_availability_filter_uses_cached_snapshot",
            "hub04_project_availability_filter_release_benchmark_evidence",
        ):
            self.assertIn(test_name, self.snapshot)
        self.assertIn(
            "PERF_RESULT hub04_project_availability_cache",
            self.snapshot,
        )
        self.assertIn("sample_pairs=21", self.snapshot)
        self.assertIn("legacy_filesystem_probes_per_projection=10000", self.snapshot)
        self.assertIn("optimized_filesystem_probes_per_projection=0", self.snapshot)
        self.assertIn("threshold_percent=40", self.snapshot)
        benchmark = function_body(
            self.snapshot,
            "hub04_project_availability_filter_release_benchmark_evidence",
        )
        self.assertIn("availability.synchronize(&projects)", benchmark)


if __name__ == "__main__":
    unittest.main()
