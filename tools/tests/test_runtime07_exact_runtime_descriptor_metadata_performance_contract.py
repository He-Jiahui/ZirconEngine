from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs"
)

# 读取实现源码约束精确运行时描述符元数据：元数据连接预分配借用部分，并构造器构建两侧元数据字符串不带格式器。
class ExactRuntimeDescriptorMetadataPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.constructor = cls.source.split("pub fn new(", maxsplit=1)[1].split(
            "pub fn with_module_descriptor", maxsplit=1
        )[0]

    def test_metadata_join_preallocates_borrowed_parts(self) -> None:
        self.assertIn("parts.iter().map(|part| part.len()).sum()", self.source)
        self.assertIn("String::with_capacity(capacity)", self.source)
        self.assertIn("joined.push_str(part)", self.source)

    def test_constructor_builds_both_metadata_strings_without_formatter(self) -> None:
        self.assertIn(
            'let module_id = join_string_parts(&[&package_id, ".runtime"]);',
            self.constructor,
        )
        self.assertIn(
            'join_string_parts(&["Runtime plugin module for ", &display_name]);',
            self.constructor,
        )
        self.assertIn(
            "ModuleDescriptor::new(module_id, module_description)",
            self.constructor,
        )
        self.assertNotIn("format!(", self.constructor)

    def test_rust_guard_preserves_module_id_and_description(self) -> None:
        self.assertIn(
            "exact_runtime_descriptor_metadata_preserves_identity_and_description",
            self.source,
        )
        self.assertIn('"weather.runtime"', self.source)
        self.assertIn('"Runtime plugin module for Weather"', self.source)


if __name__ == "__main__":
    unittest.main()
