from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/host/host_registry.rs"


class PreallocatedHostCapabilitySnapshotPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.capabilities = cls.source.split("pub fn capabilities", 1)[1].split(
            "pub fn is_valid", 1
        )[0]

    def test_snapshot_uses_exact_live_record_capacity(self) -> None:
        self.assertIn(
            "state.slots.len().saturating_sub(state.free_slots.len())",
            self.capabilities,
        )
        self.assertIn("Vec::with_capacity(live_record_capacity)", self.capabilities)

    def test_snapshot_extends_preallocated_vector_without_collect_growth(self) -> None:
        self.assertIn("records.extend(", self.capabilities)
        self.assertIn("filter_map(|slot| slot.record.clone())", self.capabilities)
        self.assertNotIn("collect::<Vec<_>>()", self.capabilities)

    def test_rust_guard_preserves_live_sorted_snapshot_contract(self) -> None:
        self.assertIn("let mut records = {", self.capabilities)
        self.assertIn("records.sort_unstable_by_key", self.capabilities)
        self.assertNotIn("records.sort_by_key", self.capabilities)
        self.assertIn(
            "preallocated_capability_snapshot_preserves_live_sorted_contract",
            self.source,
        )
        self.assertIn("registry.revoke(second).unwrap();", self.source)
        self.assertIn("expected.sort_unstable_by_key", self.source)


if __name__ == "__main__":
    unittest.main()
