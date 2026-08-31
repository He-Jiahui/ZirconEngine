from pathlib import Path
import unittest

from tools.runtime_ui_style_delta_merge_pressure import run


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_STATE = (
    REPO_ROOT / "zircon_runtime/src/ui/v2/style/runtime_state.rs"
)
PROFILE_MANIFEST = REPO_ROOT / "tools/profile-capture-manifest.ps1"


class RuntimeUiStyleDeltaMergePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = RUNTIME_STATE.read_text(encoding="utf-8")
        cls.delta_body = cls.source.split(
            "pub(super) fn dirty_for_runtime_style_delta(", 1
        )[1].split("fn is_text_affecting_style_key", 1)[0]

    def test_delta_uses_one_pass_ordered_merge_without_key_collection(self) -> None:
        self.assertGreaterEqual(self.delta_body.count(".peekable()"), 2)
        self.assertIn("Ordering::Less", self.delta_body)
        self.assertIn("Ordering::Equal", self.delta_body)
        self.assertIn("Ordering::Greater", self.delta_body)
        self.assertNotIn(".chain(new_attributes.keys())", self.delta_body)
        self.assertNotIn("collect::<BTreeSet", self.delta_body)

    def test_delta_can_stop_after_all_dirty_domains_are_known(self) -> None:
        self.assertIn("dirty.text && dirty.style", self.delta_body)
        self.assertIn("break", self.delta_body)

    def test_behavior_regressions_cover_domain_classification(self) -> None:
        self.assertIn(
            "runtime_style_delta_preserves_dirty_domain_classification",
            self.source,
        )

    def test_profile_manifest_binds_the_runtime_style_delta_source(self) -> None:
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")
        self.assertIn(
            '"zircon_runtime/src/ui/v2/style/runtime_state.rs"',
            manifest,
        )

    def test_pressure_model_counts_conservative_map_operations(self) -> None:
        result = run(
            node_update_count=4096,
            attributes_per_map=256,
            changed_existing_key_count=2,
            early_domain_resolution_rank=2,
        )

        self.assertEqual(result["retired_chain_filter"]["chained_key_visits"], 2097152)
        self.assertEqual(result["retired_chain_filter"]["btree_get_calls"], 4194304)
        self.assertEqual(
            result["retired_chain_filter"]["temporary_changed_key_clones"],
            16384,
        )
        self.assertEqual(
            result["ordered_merge_full_scan"]["decision_work_reduction_ratio"],
            3.0,
        )
        self.assertEqual(
            result["ordered_merge_early_exit"]["decision_work_reduction_ratio"],
            384.0,
        )
        self.assertEqual(
            result["ordered_merge_early_exit"]["temporary_changed_key_clones"],
            0,
        )
        self.assertIn(
            "runtime_style_delta_keeps_state_and_render_only_changes_render_scoped",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
