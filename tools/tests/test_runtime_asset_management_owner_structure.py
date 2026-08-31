import unittest
from pathlib import Path


class RuntimeAssetManagementOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_04_15_asset_management_generation_static_"
        "implemented_cargo_deferred"
    )

    def test_family_and_record_set_behaviors_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/asset/management.rs"
        owner = owner_path.read_text(encoding="utf-8")
        owner_dir = owner_path.with_suffix("")
        family = (owner_dir / "family.rs").read_text(encoding="utf-8")
        record_sets = (owner_dir / "record_sets.rs").read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 200)
        self.assertLessEqual(len(family.splitlines()), 220)
        self.assertLessEqual(len(record_sets.splitlines()), 300)
        self.assertIn("mod family;", owner)
        self.assertIn("mod record_sets;", owner)
        self.assertNotIn("impl AssetManagement", owner)

        self._assert_anchors_are_ordered(
            owner,
            (
                "pub struct AssetManagementRecordSetSummary",
                "pub enum AssetManagementFamilyKind",
                "pub enum AssetManagementFamilyStatus",
                "pub struct AssetManagementFamilySummary",
                "pub struct AssetManagementFamilyStatusIndex",
                "pub struct AssetManagementFamilyStatusView",
                "pub enum AssetManagementFamilyIssueBucket",
                "pub struct AssetManagementFamilyIssueIndex",
                "pub struct AssetManagementFamilyIssueView",
                "pub struct AssetManagementOverview",
                "pub struct AssetManagementRecordSets",
            ),
        )

        self._assert_anchors_are_ordered(
            family,
            (
                "impl AssetManagementFamilySummary",
                "impl AssetManagementFamilyStatus",
                "impl AssetManagementFamilyStatusIndex",
                "impl AssetManagementFamilyStatusView",
                "impl AssetManagementFamilyIssueBucket",
                "impl AssetManagementFamilyIssueIndex",
                "impl AssetManagementFamilyIssueView",
            ),
        )
        for invariant in (
            "if total_record_count == 0",
            "else if degraded_record_count > 0",
            "AssetManagementFamilyStatus::Empty => index.empty.push(family.kind)",
            "let total_record_count = rows.iter().map(|family| family.total_record_count).sum();",
            "Self::WithIssues => family.issue_row_count > 0",
        ):
            self.assertIn(invariant, family)

        self._assert_anchors_are_ordered(
            record_sets,
            (
                "impl AssetManagementOverview",
                "impl AssetManagementRecordSetSummary",
                "impl AssetManagementRecordSets",
            ),
        )
        self._assert_anchors_are_ordered(
            record_sets.split("pub fn family_summaries(&self) -> Vec", 1)[1],
            (
                "AssetManagementFamilyKind::Model",
                "AssetManagementFamilyKind::Mesh",
                "AssetManagementFamilyKind::Scene",
                "AssetManagementFamilyKind::Entity",
                "AssetManagementFamilyKind::Material",
                "AssetManagementFamilyKind::Shader",
            ),
        )
        for invariant in (
            "let family_status_index = AssetManagementFamilyStatusIndex::from_families(&families);",
            "let family_issue_index = AssetManagementFamilyIssueIndex::from_families(&families);",
            "managed_record_count: models.summary.model_count",
            "degraded_record_count: meshes.summary.invalid_mesh_count",
        ):
            self.assertIn(invariant, record_sets)

    def test_owner_split_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/asset/management.rs",
            "zircon_runtime/src/asset/management/family.rs",
            "zircon_runtime/src/asset/management/record_sets.rs",
            "tools/tests/test_runtime_asset_management_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)

    def _assert_anchors_are_ordered(self, source: str, anchors: tuple[str, ...]) -> None:
        positions = [source.index(anchor) for anchor in anchors]
        self.assertEqual(positions, sorted(positions))


if __name__ == "__main__":
    unittest.main()
