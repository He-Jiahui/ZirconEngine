from __future__ import annotations

import tempfile
import unittest
import subprocess
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tools.validate_cargo_test_reachability import _cargo_metadata, audit_test_reachability


class CargoTestReachabilityTests(unittest.TestCase):
    @patch("tools.validate_cargo_test_reachability.subprocess.run")
    def test_cargo_metadata_decodes_cargo_output_as_utf8(self, run: object) -> None:
        run.return_value = SimpleNamespace(stdout='{"packages": []}')

        _cargo_metadata(Path("Cargo.toml"))

        self.assertEqual(run.call_args.kwargs["encoding"], "utf-8")
        self.assertEqual(run.call_args.kwargs["errors"], "replace")

    @patch("tools.validate_cargo_test_reachability.subprocess.run")
    def test_cargo_metadata_reports_cargo_stderr(self, run: object) -> None:
        run.side_effect = subprocess.CalledProcessError(
            101,
            ["cargo", "metadata"],
            stderr="error: workspace manifest is invalid",
        )

        with self.assertRaisesRegex(RuntimeError, "workspace manifest is invalid"):
            _cargo_metadata(Path("Cargo.toml"))

    def test_rejects_a_library_target_that_disables_inline_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            source_root.mkdir(parents=True)
            (source_root / "lib.rs").write_text(
                "#[cfg(test)]\nmod tests {\n    #[test]\n    fn reachable() {}\n}\n",
                encoding="utf-8",
            )

            report = audit_test_reachability(
                self._metadata(package_root, source_root / "lib.rs", test_enabled=False)
            )

        self.assertFalse(report["passed"])
        self.assertEqual(report["violation_count"], 1)
        self.assertEqual(report["violations"][0]["package"], "fixture")
        self.assertEqual(report["violations"][0]["target"], "fixture")
        self.assertEqual(report["violations"][0]["inline_test_count"], 1)

    def test_allows_a_disabled_library_target_without_inline_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            source_root.mkdir(parents=True)
            (source_root / "lib.rs").write_text("pub fn live() {}\n", encoding="utf-8")

            report = audit_test_reachability(
                self._metadata(package_root, source_root / "lib.rs", test_enabled=False)
            )

        self.assertTrue(report["passed"])
        self.assertEqual(report["violation_count"], 0)

    def test_allows_inline_tests_when_the_library_target_is_enabled(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            source_root.mkdir(parents=True)
            (source_root / "lib.rs").write_text(
                "#[cfg(test)]\nmod tests {\n    #[test]\n    fn reachable() {}\n}\n",
                encoding="utf-8",
            )

            report = audit_test_reachability(
                self._metadata(package_root, source_root / "lib.rs", test_enabled=True)
            )

        self.assertTrue(report["passed"])
        self.assertEqual(report["checked_target_count"], 1)

    def test_rejects_inline_tests_in_a_nested_library_module(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            nested_root = source_root / "nested"
            nested_root.mkdir(parents=True)
            (source_root / "lib.rs").write_text(
                "pub mod nested;\n", encoding="utf-8"
            )
            (nested_root / "mod.rs").write_text(
                "#[cfg(test)]\nmod tests {\n    #[test]\n    fn reachable() {}\n}\n",
                encoding="utf-8",
            )

            report = audit_test_reachability(
                self._metadata(package_root, source_root / "lib.rs", test_enabled=False)
            )

        self.assertFalse(report["passed"])
        self.assertEqual(report["violations"][0]["inline_test_count"], 1)
        self.assertEqual(
            report["violations"][0]["inline_test_sources"],
            [(nested_root / "mod.rs").resolve().as_posix()],
        )

    def test_rejects_tests_loaded_through_a_path_attribute(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            support_root = source_root / "support"
            support_root.mkdir(parents=True)
            (source_root / "lib.rs").write_text(
                '#[path = "support/hidden.rs"]\nmod hidden; // custom source\n',
                encoding="utf-8",
            )
            hidden_source = support_root / "hidden.rs"
            hidden_source.write_text(
                "#[test]\nfn hidden() {}\n", encoding="utf-8"
            )

            report = audit_test_reachability(
                self._metadata(package_root, source_root / "lib.rs", test_enabled=False)
            )

        self.assertFalse(report["passed"])
        self.assertEqual(
            report["violations"][0]["inline_test_sources"],
            [hidden_source.resolve().as_posix()],
        )

    def test_rejects_tests_loaded_through_include(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            source_root.mkdir(parents=True)
            (source_root / "lib.rs").write_text(
                'include!("generated_tests.rs");\n', encoding="utf-8"
            )
            generated_source = source_root / "generated_tests.rs"
            generated_source.write_text(
                "#[test]\nfn generated() {}\n", encoding="utf-8"
            )

            report = audit_test_reachability(
                self._metadata(package_root, source_root / "lib.rs", test_enabled=False)
            )

        self.assertFalse(report["passed"])
        self.assertEqual(
            report["violations"][0]["inline_test_count"], 1)

    def test_rejects_a_binary_target_that_disables_inline_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            source_root.mkdir(parents=True)
            (source_root / "main.rs").write_text(
                "#[test]\nfn executable() {}\n", encoding="utf-8"
            )

            report = audit_test_reachability(
                self._metadata(
                    package_root,
                    source_root / "main.rs",
                    test_enabled=False,
                    target_kind="bin",
                )
            )

        self.assertFalse(report["passed"])
        self.assertEqual(report["violations"][0]["target_kind"], "bin")

    def test_ignores_a_custom_build_target(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "fixture"
            source_root = package_root / "src"
            source_root.mkdir(parents=True)
            build_source = package_root / "build.rs"
            build_source.write_text("#[test]\nfn build() {}\n", encoding="utf-8")

            report = audit_test_reachability(
                self._metadata(
                    package_root,
                    build_source,
                    test_enabled=False,
                    target_kind="custom-build",
                )
            )

        self.assertTrue(report["passed"])
        self.assertEqual(report["checked_target_count"], 0)

    @staticmethod
    def _metadata(
        package_root: Path,
        source_path: Path,
        *,
        test_enabled: bool,
        target_kind: str = "lib",
    ) -> dict[str, object]:
        package_id = "path+file:///fixture#0.1.0"
        return {
            "workspace_members": [package_id],
            "packages": [
                {
                    "id": package_id,
                    "name": "fixture",
                    "manifest_path": str(package_root / "Cargo.toml"),
                    "targets": [
                        {
                            "name": "fixture",
                            "kind": [target_kind],
                            "src_path": str(source_path),
                            "test": test_enabled,
                        }
                    ],
                }
            ],
        }


if __name__ == "__main__":
    unittest.main()
