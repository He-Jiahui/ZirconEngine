import unittest

from tools.analysis.performance.runtime.runtime_ui_svg_document_cache_pressure import run

# 用稳定工作集模型验证缓存消除重复 SVG 解析，并拒绝容量不足及非法输入。
class RuntimeUiSvgDocumentCachePressureTests(unittest.TestCase):
    def test_stable_working_set_eliminates_repeated_parses(self):
        result = run(1_000, 64, 512)
        self.assertEqual(result["old_parse_count"], 64_000)
        self.assertEqual(result["new_parse_count"], 64)
        self.assertEqual(result["eliminated_parse_count"], 63_936)
        self.assertEqual(result["parse_reduction_ratio"], 1_000.0)

    def test_model_rejects_working_set_larger_than_capacity(self):
        with self.assertRaises(ValueError):
            run(10, 513, 512)

    def test_model_rejects_invalid_inputs(self):
        with self.assertRaises(ValueError):
            run(0, 1, 512)
        with self.assertRaises(ValueError):
            run(1, 0, 512)
        with self.assertRaises(ValueError):
            run(1, 1, 0)


if __name__ == "__main__":
    unittest.main()
