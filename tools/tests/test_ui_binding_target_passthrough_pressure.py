import unittest

from tools.ui_binding_target_passthrough_pressure import model_pressure


class UiBindingTargetPassthroughPressureTests(unittest.TestCase):
    def test_empty_targets_skip_receipt_preparation_at_scale(self) -> None:
        result = model_pressure(
            event_count=65_536,
            binding_id_bytes=64,
            asset_id_bytes=128,
        )

        self.assertEqual(result["retired"]["timer_reads"], 65_536)
        self.assertEqual(result["retired"]["compiled_binding_lookups"], 131_072)
        self.assertEqual(result["retired"]["temporary_identifier_allocations"], 131_072)
        self.assertEqual(result["retired"]["temporary_identifier_bytes"], 12_582_912)
        self.assertEqual(result["passthrough"]["compiled_binding_lookups"], 65_536)
        self.assertEqual(result["passthrough"]["temporary_identifier_allocations"], 0)

    def test_empty_event_set_has_zero_delta(self) -> None:
        result = model_pressure(event_count=0)

        self.assertEqual(result["delta"]["eliminated_timer_reads"], 0)
        self.assertEqual(result["delta"]["eliminated_temporary_identifier_bytes"], 0)


if __name__ == "__main__":
    unittest.main()
