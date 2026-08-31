from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
INSTANTIATE = ROOT / "zircon_editor/src/ui/asset_editor/palette/instantiate.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetPaletteMountSetPerformanceContractTests(unittest.TestCase):
    def test_child_mount_validation_uses_one_preallocated_borrowed_set(self) -> None:
        source = INSTANTIATE.read_text(encoding="utf-8")
        validation = function_region(
            source,
            "fn validate_child_mounts_for_component(",
            "fn child_index_in_parent(",
        )

        self.assertIn("HashSet::<&str>::with_capacity(children.len())", validation)
        self.assertIn("child.mount.as_deref().unwrap_or_default()", validation)
        self.assertIn("let first_occupant = occupied.insert(slot_name);", validation)
        self.assertIn("if !slot.multiple && !first_occupant", validation)
        self.assertIn("occupied.contains(slot_name.as_str())", validation)
        self.assertNotIn("BTreeMap", validation)
        self.assertNotIn("child.mount.clone()", validation)
        self.assertNotIn("*count += 1", validation)

        benchmark = (ROOT / "zircon_editor/src/ui/asset_editor/palette/instantiate/child_mount_validation_tests.rs").read_text(encoding="utf-8")
        self.assertIn("RUNTIME75_PALETTE_CHILD_MOUNT_SET_BENCH_V1", benchmark)
        self.assertIn("legacy_tree_nodes_per_check={SLOT_COUNT}", benchmark)
        self.assertIn("optimized_preallocated_sets_per_check=1", benchmark)
        self.assertIn("legacy_p95_ns.saturating_mul(80)", benchmark)


if __name__ == "__main__":
    unittest.main()
