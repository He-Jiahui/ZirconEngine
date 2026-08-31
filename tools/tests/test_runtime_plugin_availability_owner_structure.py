import unittest
from pathlib import Path


class RuntimePluginAvailabilityOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_06_15_plugin_availability_evaluation_selection_owner_split_"
        "static_passed_cargo_deferred"
    )

    def test_evaluation_and_manifest_selection_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/plugin/runtime_profile/availability_projection.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")
        owner_dir = owner_path.with_suffix("")
        evaluation = (owner_dir / "evaluation.rs").read_text(encoding="utf-8")
        selection = (owner_dir / "selection.rs").read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 330)
        self.assertLessEqual(len(evaluation.splitlines()), 320)
        self.assertLessEqual(len(selection.splitlines()), 130)
        self.assertIn("mod evaluation;", owner)
        self.assertIn("mod selection;", owner)
        self.assertIn(
            "pub(crate) use selection::RuntimePluginAvailabilitySelectionMetrics;",
            owner,
        )
        for moved_anchor in (
            "fn append_plugin_availability",
            "fn project_manifest_plugin_selections",
            "fn merge_runtime_plugin_selection",
        ):
            self.assertNotIn(moved_anchor, owner)

        self._assert_anchors_are_ordered(
            evaluation,
            (
                "pub fn report_for_profile_defaults",
                "pub fn generation_for_profile_defaults",
                "pub fn report_for_manifest",
                "pub fn generation_for_manifest",
                "pub fn report_for_manifest_with_metrics",
                "fn generation_for_runtime_plugins",
                "fn append_plugin_availability",
                "fn builtin_runtime_domain_is_available",
                "fn supports_target",
            ),
        )
        append = evaluation.split("fn append_plugin_availability", 1)[1]
        self._assert_anchors_are_ordered(
            append,
            (
                "BuiltinUnavailable",
                "BuiltinAvailable",
                "MissingCatalog",
                "BlockedByTarget",
                "ExternalizedMissing",
                "RuntimePluginAvailabilityReason::Stub",
                "BlockedByMaturity",
                "RuntimePluginAvailabilityReason::Linked",
                "RuntimePluginAvailabilityReason::NativeDynamic",
                "RuntimePluginAvailabilityReason::MissingProvider",
                "RuntimePluginAvailabilityReason::Available",
            ),
        )

        self._assert_anchors_are_ordered(
            selection,
            (
                "pub(crate) struct RuntimePluginAvailabilitySelectionMetrics",
                "struct RuntimePluginManifestSelectionProjection",
                "fn project_manifest_plugin_selections",
                "fn merge_runtime_plugin_selection",
            ),
        )
        for invariant in (
            "manifest.enabled_for_target(profile.target_mode)",
            "RuntimePluginId::parse_key(&selection.id)",
            "plugins[index].1 = plugins[index].1 || required",
            "positions.insert(runtime_id.clone(), plugins.len())",
        ):
            self.assertIn(invariant, selection)

    def test_owner_split_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
            repo_root
            / "docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/plugin/runtime_profile/availability_projection.rs",
            "zircon_runtime/src/plugin/runtime_profile/availability_projection/evaluation.rs",
            "zircon_runtime/src/plugin/runtime_profile/availability_projection/selection.rs",
            "tools/tests/test_runtime_plugin_availability_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)

    def _assert_anchors_are_ordered(self, source: str, anchors: tuple[str, ...]) -> None:
        positions = [source.index(anchor) for anchor in anchors]
        self.assertEqual(positions, sorted(positions))


if __name__ == "__main__":
    unittest.main()
