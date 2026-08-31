import unittest

from tools.runtime_keyboard_admission_pressure import run


class RuntimeKeyboardAdmissionPressureTests(unittest.TestCase):
    def test_all_whitespace_text_halves_character_scans_and_visits(self) -> None:
        text = run()["keyboard_text"]

        self.assertEqual(text["baseline_character_scan_count"], 16_384)
        self.assertEqual(text["candidate_character_scan_count"], 8_192)
        self.assertEqual(text["baseline_character_visit_count"], 67_108_864)
        self.assertEqual(text["candidate_character_visit_count"], 33_554_432)
        self.assertEqual(text["character_visit_reduction_percent"], 50.0)
        self.assertEqual(text["candidate_accepted_text_allocations"], 0)

    def test_near_match_direction_key_normalizes_once_per_check(self) -> None:
        direction = run()["direction_key"]

        self.assertEqual(direction["baseline_normalization_pass_count"], 3_145_728)
        self.assertEqual(direction["candidate_normalization_pass_count"], 262_144)
        self.assertAlmostEqual(
            direction["normalization_pass_reduction_percent"], 91.66666666666667
        )
        self.assertEqual(direction["candidate_heap_allocations"], 0)
        self.assertEqual(direction["candidate_stack_storage_bytes"], 16)

    def test_keyboard_semantic_invariants_are_explicit(self) -> None:
        invariants = run()["invariants"]

        self.assertTrue(invariants["control_character_rejection_preserved"])
        self.assertTrue(invariants["unicode_whitespace_rejection_preserved"])
        self.assertTrue(invariants["accepted_text_remains_borrowed"])
        self.assertTrue(invariants["direction_key_code_fallback_preserved"])
        self.assertTrue(invariants["direction_separator_and_case_folding_preserved"])

    def test_model_is_bound_to_exact_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "630d66c362013e3b5b72f97362ad56fc54ff6d8c",
        )
        self.assertEqual(
            binding["head_baseline_git_blobs"],
            {
                "zircon_runtime/src/ui/surface/input/keyboard_action.rs": (
                    "9e6b49d08bb56da126d4e6942a96a69eb1f2943b"
                ),
                "zircon_runtime/src/ui/surface/input/keyboard_navigation.rs": (
                    "897ba18d086ca8df10f04c3cacd139625c140b77"
                ),
            },
        )
        self.assertEqual(len(binding["source_sha256"]), 3)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_inputs(self) -> None:
        with self.assertRaises(ValueError):
            run(text_checks_per_sample=0)
        with self.assertRaises(ValueError):
            run(text_characters=0)
        with self.assertRaises(ValueError):
            run(direction_checks_per_sample=0)
        with self.assertRaises(ValueError):
            run(baseline_direction_candidates=0)
        with self.assertRaises(ValueError):
            run(normalized_stack_bytes=0)


if __name__ == "__main__":
    unittest.main()
