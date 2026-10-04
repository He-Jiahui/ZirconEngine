import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE_PATH = (
    ROOT
    / "zircon_plugins"
    / "zr_vm_language"
    / "runtime"
    / "src"
    / "real_backend"
    / "reflection_host.rs"
)

# 读取实现源码约束借用反射宿主字符串：借用字符串辅助函数替换持有提取器，并反射调用 consume 借用字符串视图。
class BorrowedReflectionHostStringsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = SOURCE_PATH.read_text(encoding="utf-8")

    def test_borrowed_string_helper_replaces_owned_extractor(self):
        self.assertIn("fn borrow_string<", self.source)
        self.assertNotIn("fn expect_string(", self.source)
        self.assertIn("visitor: impl FnOnce(&str)", self.source)

    def test_reflection_calls_consume_borrowed_string_views(self):
        self.assertIn("borrow_string(&arguments, 0", self.source)
        self.assertIn("borrow_string(&arguments, 1", self.source)
        self.assertIn("borrow_string(&arguments, 2", self.source)

    def test_borrowed_helper_does_not_materialize_transient_strings(self):
        helper = self.source.split("fn borrow_string<", 1)[-1]
        helper = helper.split("fn expect_int", 1)[0]
        self.assertNotIn("to_owned()", helper)
        self.assertNotIn("-> Result<String", helper)
        self.assertNotIn("record_guest_string_copy", helper)

    def test_rust_regression_covers_borrowed_reflection_arguments(self):
        self.assertIn("reflection.resolve", self.source)
        self.assertIn("reflection.write", self.source)
        self.assertIn("fn reflection_host_error", self.source)


if __name__ == "__main__":
    unittest.main()
