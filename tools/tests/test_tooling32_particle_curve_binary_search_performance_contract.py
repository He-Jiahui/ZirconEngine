from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/particles/runtime/src/asset.rs"


class ParticleCurveBinarySearchPerformanceContract(unittest.TestCase):
    def test_scalar_and_color_curves_use_logarithmic_key_lookup(self):
        production = SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        scalar = production.split("fn evaluate_scalar_curve", 1)[1].split(
            "fn evaluate_color_curve", 1
        )[0]
        color = production.split("fn evaluate_color_curve", 1)[1].split(
            "fn normalize_scalar_keys", 1
        )[0]

        for evaluator in (scalar, color):
            self.assertIn("partition_point", evaluator)
            self.assertIn("let right_index", evaluator)
            self.assertNotIn("windows(2)", evaluator)

    def test_curve_lookup_keeps_explicit_endpoint_fast_paths(self):
        production = SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        scalar = production.split("fn evaluate_scalar_curve", 1)[1].split(
            "fn evaluate_color_curve", 1
        )[0]
        color = production.split("fn evaluate_color_curve", 1)[1].split(
            "fn normalize_scalar_keys", 1
        )[0]

        for evaluator in (scalar, color):
            self.assertIn("if t < keys[0].t", evaluator)
            self.assertIn("if right_index == keys.len()", evaluator)


if __name__ == "__main__":
    unittest.main()
