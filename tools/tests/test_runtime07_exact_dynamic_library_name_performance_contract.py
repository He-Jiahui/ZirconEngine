from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/dynamic_library_name.rs"
)


class ExactDynamicLibraryNamePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_name_helper_preallocates_all_borrowed_parts(self) -> None:
        self.assertIn(
            "prefix.len() + crate_name.len() + suffix.len()",
            self.source,
        )
        self.assertIn("String::with_capacity(capacity)", self.source)
        self.assertIn("name.push_str(prefix)", self.source)
        self.assertIn("name.push_str(crate_name)", self.source)
        self.assertIn("name.push_str(suffix)", self.source)

    def test_all_platform_branches_share_exact_helper(self) -> None:
        self.assertIn('exact_dynamic_library_name("", crate_name, ".dll")', self.source)
        self.assertIn(
            'exact_dynamic_library_name("lib", crate_name, ".dylib")',
            self.source,
        )
        self.assertIn('exact_dynamic_library_name("lib", crate_name, ".so")', self.source)
        self.assertNotIn("format!(", self.production)

    def test_rust_guard_preserves_all_platform_conventions(self) -> None:
        self.assertIn(
            "exact_dynamic_library_names_preserve_platform_conventions",
            self.source,
        )
        self.assertIn('"zircon_plugin_weather.dll"', self.source)
        self.assertIn('"libzircon_plugin_weather.dylib"', self.source)
        self.assertIn('"libzircon_plugin_weather.so"', self.source)


if __name__ == "__main__":
    unittest.main()
