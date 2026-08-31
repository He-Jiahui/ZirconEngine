from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LOCALIZATION_ROOT = ROOT / "zircon_runtime/src/ui/template/asset/localization"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime83LocalizationPathPerformanceContractTests(unittest.TestCase):
    def test_locale_key_flattening_reuses_one_path_buffer(self) -> None:
        source = (LOCALIZATION_ROOT / "resolve.rs").read_text(encoding="utf-8")
        entry = function_body(
            source,
            "pub fn localization_table_keys_from_toml_str(",
            "fn validate_dependency<'dependency>(",
        )
        collector = source[source.index("fn collect_locale_keys(") :]

        self.assertIn("let mut path = String::new();", entry)
        self.assertIn("collect_locale_keys(&mut path, &value, &mut keys);", entry)
        self.assertIn("path.truncate(prefix_len);", collector)
        self.assertIn("path.push_str(key);", collector)
        self.assertNotIn("fn join_key(", source)
        self.assertNotIn("format!(\"{prefix}.{key}\")", source)
        performance = (
            LOCALIZATION_ROOT / "resolve/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("RUNTIME83_LOCALIZATION_RESOLVE_PERF", performance)
        self.assertIn("legacy_temporary_path_allocations, 10_100", performance)

    def test_document_collection_reuses_path_storage_during_recursion(self) -> None:
        source = (LOCALIZATION_ROOT / "collect.rs").read_text(encoding="utf-8")
        entry = function_body(
            source,
            "pub fn collect_document_localization_report(",
            "pub fn validate_document_localization(",
        )
        collector = function_body(source, "fn collect_values(", "fn localized_text_ref(")

        self.assertIn("let mut path = String::new();", entry)
        self.assertIn("path.truncate(prefix_len);", collector)
        self.assertIn("write!(path, \"[{index}]\")", collector)
        self.assertNotIn("&format!", entry)
        self.assertNotIn("format!(\"{path}.", collector)
        self.assertNotIn("format!(\"{path}[", collector)
        performance = (
            LOCALIZATION_ROOT / "collect/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("RUNTIME83_LOCALIZATION_COLLECT_PERF", performance)
        self.assertIn("legacy_temporary_path_allocations, 20_100", performance)

    def test_catalog_validation_borrows_locale_table_map_once_per_batch(self) -> None:
        source = (LOCALIZATION_ROOT / "resolve.rs").read_text(encoding="utf-8")
        entry = function_body(
            source,
            "pub fn validate_localization_report_against_catalog(",
            "pub fn localization_table_keys_from_toml_str(",
        )
        dependency = function_body(
            source,
            "fn validate_dependency<'dependency>(",
            "fn missing_ref_severity(",
        )

        self.assertIn("let locale_tables = catalog.tables.get(locale);", entry)
        self.assertIn("validate_dependency(", entry)
        self.assertIn("locale_tables,", entry)
        self.assertIn("&mut emitted_diagnostics", entry)
        self.assertIn(
            "locale_tables: Option<&HashMap<String, UiLocalizationTableEntry>>",
            dependency,
        )
        self.assertNotIn("catalog.table(locale, table_name)", dependency)


if __name__ == "__main__":
    unittest.main()
