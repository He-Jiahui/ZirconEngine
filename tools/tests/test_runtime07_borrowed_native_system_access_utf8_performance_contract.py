from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
READ_SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/abi_decode/read.rs"
)
SYSTEM_SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/abi_decode/system.rs"
)


class BorrowedNativeSystemAccessUtf8PerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.read_source = READ_SOURCE.read_text(encoding="utf-8")
        cls.system_source = SYSTEM_SOURCE.read_text(encoding="utf-8")

    def test_utf8_reader_supports_scoped_borrowed_mapping(self) -> None:
        self.assertIn("unsafe fn read_utf8_with<T>", self.read_source)
        self.assertIn("visitor: impl FnOnce(&str) -> T", self.read_source)
        self.assertIn("Ok(visitor(value))", self.read_source)
        self.assertIn("read_utf8_with(slice, str::to_string)", self.read_source)

    def test_system_access_formats_directly_from_borrowed_utf8(self) -> None:
        self.assertIn("read_utf8_with", self.system_source)
        self.assertIn("read_utf8_with(access.stable_id, |stable_id|", self.system_source)
        self.assertIn("system_access_id(mode, domain, stable_id)", self.system_source)
        self.assertIn("fn system_access_id", self.system_source)
        self.assertIn("String::with_capacity", self.system_source)
        self.assertIn("access_id.push_str(mode);", self.system_source)
        self.assertIn("access_id.push_str(stable_id);", self.system_source)
        self.assertNotIn('format!("{mode}:{domain}:{stable_id}")', self.system_source)
        self.assertNotIn(
            "let stable_id = unsafe { read_utf8(access.stable_id) }?;",
            self.system_source,
        )

    def test_rust_guard_preserves_exact_utf8_projection(self) -> None:
        self.assertIn(
            "borrowed_utf8_mapping_preserves_exact_projection",
            self.read_source,
        )
        self.assertIn('assert_eq!(projected, "read:component:weather.velocity");', self.read_source)


if __name__ == "__main__":
    unittest.main()
