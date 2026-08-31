from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT / "zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs"
)


class BorrowedVmBytecodeFileNamePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.payload = cls.source.split("fn resolve_package_payload", maxsplit=1)[
            1
        ].split("fn collect_plugin_manifests", maxsplit=1)[0]

    def test_bytecode_file_name_borrows_custom_or_default_value(self) -> None:
        self.assertIn(
            "bytecode.unwrap_or(DEFAULT_BYTECODE_FILE)",
            self.source,
        )
        self.assertIn(
            "bytecode_file_name(disk_manifest.bytecode.as_deref())",
            self.payload,
        )

    def test_payload_resolution_does_not_clone_a_temporary_file_name(self) -> None:
        self.assertNotIn("disk_manifest.bytecode.clone()", self.payload)
        self.assertNotIn("default_bytecode_file", self.source)

    def test_rust_guard_preserves_custom_and_default_file_names(self) -> None:
        self.assertIn(
            "bytecode_file_name_borrows_custom_and_default_values",
            self.source,
        )
        self.assertIn('Some("module/runtime.zrbc")', self.source)
        self.assertIn('"plugin.bin"', self.source)


if __name__ == "__main__":
    unittest.main()
