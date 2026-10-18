import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.build.zircon_build_size_report import (
    ProductSizeError,
    build_product_size_report,
    enforce_product_size_report,
    write_deterministic_archive,
    write_product_size_report,
    classify_product_path,
    product_report_reference,
    product_report_summary_sha256,
)
from tools.build.zircon_build_product_output import clean_product_output


class ProductSizeReportTests(unittest.TestCase):
    def test_forbidden_tokens_match_path_components_not_resource_substrings(self):
        self.assertEqual(
            classify_product_path("assets/resources/material.bin", forbidden_path_tokens=["source"]),
            "engine_asset",
        )
        self.assertEqual(
            classify_product_path("source/material.bin", forbidden_path_tokens=["source"]),
            "forbidden",
        )

    def test_report_hashes_files_and_keeps_symbols_and_project_pack_external(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            product = root / "ZirconEngine"
            product.mkdir()
            (product / "zircon_runtime.exe").write_bytes(b"host")
            (product / "zircon_runtime.dll").write_bytes(b"dll")
            (product / "staging_manifest.json").write_text("{}", encoding="utf-8")
            symbols = root / "symbols"
            symbols.mkdir()
            (symbols / "zircon_runtime.pdb").write_bytes(b"symbols")
            project = root / "project-pack"
            project.mkdir()
            (project / "assets.zrpack").write_bytes(b"project")
            report = build_product_size_report(
                product,
                build_set_id="a" * 64,
                product_profile="runtime-windows-shipping",
                product_kind="runtime",
                target_triple="x86_64-pc-windows-msvc",
                cargo_profile="shipping",
                runtime_features=["shipping-runtime"],
                app_features=["shipping-runtime"],
                warning_bytes=100,
                hard_limit_bytes=1_000,
                symbols_root=symbols,
                project_pack_root=project,
            )
            self.assertEqual(report["report_kind"], "ProductSizeReportV1")
            self.assertEqual(report["totals"]["uncompressed_bytes"], 9)
            self.assertEqual(report["symbols"]["bytes"], 7)
            self.assertEqual(report["project_pack"]["bytes"], 7)
            self.assertEqual(
                report["target"],
                {
                    "triple": "x86_64-pc-windows-msvc",
                    "target_triple": "x86_64-pc-windows-msvc",
                    "architecture": "x86_64",
                    "operating_system": "windows",
                    "pointer_width": 64,
                },
            )
            dll = next(item for item in report["files"] if item["path"] == "zircon_runtime.dll")
            self.assertEqual(dll["sha256"], hashlib.sha256(b"dll").hexdigest())
            self.assertEqual(report["budget"]["status"], "pass")

    def test_report_rejects_windows_namespace_paths(self):
        with self.assertRaises(ProductSizeError):
            classify_product_path("payload:alternate-stream.bin")

    def test_forbidden_paths_fail_even_when_under_budget(self):
        with tempfile.TemporaryDirectory() as temporary:
            product = Path(temporary) / "product"
            (product / "cache").mkdir(parents=True)
            (product / "cache" / "old.bin").write_bytes(b"x")
            report = build_product_size_report(
                product,
                build_set_id="b" * 64,
                product_profile="runtime-windows-shipping",
                product_kind="runtime",
                target_triple="x86_64-pc-windows-msvc",
                cargo_profile="shipping",
                warning_bytes=100,
                hard_limit_bytes=1_000,
                forbidden_path_tokens=["cache"],
            )
            self.assertEqual(report["budget"]["status"], "fail")
            with self.assertRaisesRegex(ProductSizeError, "product size gate failed"):
                enforce_product_size_report(report)

    def test_project_pack_is_rejected_when_profile_excludes_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            product = Path(temporary) / "product"
            (product / "project-pack").mkdir(parents=True)
            (product / "project-pack" / "game.zrpack").write_bytes(b"pack")
            report = build_product_size_report(
                product,
                build_set_id="d" * 64,
                product_profile="runtime-windows-shipping",
                product_kind="runtime",
                target_triple="x86_64-pc-windows-msvc",
                cargo_profile="shipping",
                warning_bytes=100,
                hard_limit_bytes=1_000,
                include_project_pack=False,
            )
            self.assertEqual(report["budget"]["status"], "fail")

    def test_archive_and_json_report_are_repeatable(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            product = root / "product"
            product.mkdir()
            (product / "b.bin").write_bytes(b"b")
            (product / "a.bin").write_bytes(b"a")
            archive_a = root / "a.zip"
            archive_b = root / "b.zip"
            write_deterministic_archive(product, archive_a)
            write_deterministic_archive(product, archive_b)
            self.assertEqual(archive_a.read_bytes(), archive_b.read_bytes())
            report_path = root / "report.json"
            write_product_size_report(
                report_path,
                product,
                build_set_id="c" * 64,
                product_profile="runtime-windows-shipping",
                product_kind="runtime",
                target_triple="x86_64-pc-windows-msvc",
                cargo_profile="shipping",
                warning_bytes=100,
                hard_limit_bytes=1_000,
            )
            self.assertEqual(
                json.loads(report_path.read_text(encoding="utf-8"))["totals"]["file_count"],
                2,
            )

    def test_clean_product_output_rejects_shared_managed_root(self):
        config = type("Config", (), {})()
        config.out_root = Path("D:/cargo-targets")
        config.engine_root = config.out_root / "ZirconEngine"
        config.symbols_root = config.out_root / "symbols" / "runtime"
        config.reports_root = config.out_root / "reports" / "runtime"
        config.archive_path = config.out_root / "archives" / "runtime.zip"
        config.dry_run = True
        with self.assertRaises(ProductSizeError):
            clean_product_output(config)

    def test_clean_product_output_removes_editor_assets_owned_by_profile(self):
        with tempfile.TemporaryDirectory() as temporary:
            out_root = Path(temporary) / "editor-build"
            editor_assets = out_root / "EditorAssets"
            editor_assets.mkdir(parents=True)
            (editor_assets / "stale.txt").write_text("stale", encoding="utf-8")
            config = type("Config", (), {})()
            config.out_root = out_root
            config.engine_root = out_root / "ZirconEngine"
            config.symbols_root = out_root / "symbols" / "editor"
            config.reports_root = out_root / "reports" / "editor"
            config.archive_path = out_root / "archives" / "editor.zip"
            config.editor_asset_root = editor_assets
            config.dry_run = False
            clean_product_output(config)
            self.assertFalse(editor_assets.exists())

    def test_report_summary_digest_is_stable_and_exposes_needs_split(self):
        with tempfile.TemporaryDirectory() as temporary:
            product = Path(temporary) / "product"
            product.mkdir()
            (product / "payload.bin").write_bytes(b"payload")
            report = build_product_size_report(
                product,
                build_set_id="e" * 64,
                product_profile="runtime-windows-shipping",
                product_kind="runtime",
                target_triple="x86_64-pc-windows-msvc",
                cargo_profile="shipping",
                warning_bytes=100,
                hard_limit_bytes=1_000,
                feature_closure={
                    "status": "needs-split",
                    "heavy_features": ["ui"],
                },
            )
            self.assertEqual(report["publication"]["status"], "needs-split")
            self.assertFalse(report["publication"]["passed"])
            self.assertEqual(
                report["summary_sha256"], product_report_summary_sha256(report)
            )
            reference = product_report_reference(
                Path(temporary) / "reports" / "runtime" / "product_size_report.json",
                report,
            )
            self.assertEqual(reference["sha256"], report["summary_sha256"])
            self.assertEqual(reference["summary"]["publication_status"], "needs-split")

    def test_publication_status_keeps_a_hard_size_failure_visible(self):
        with tempfile.TemporaryDirectory() as temporary:
            product = Path(temporary) / "product"
            product.mkdir()
            (product / "payload.bin").write_bytes(b"payload")
            report = build_product_size_report(
                product,
                build_set_id="1" * 64,
                product_profile="runtime-windows-shipping",
                product_kind="runtime",
                target_triple="x86_64-pc-windows-msvc",
                cargo_profile="shipping",
                warning_bytes=1,
                hard_limit_bytes=1,
                feature_closure={"status": "needs-split", "heavy_features": ["ui"]},
            )
            self.assertEqual(report["budget"]["status"], "fail")
            self.assertEqual(report["publication"]["status"], "fail")
            self.assertFalse(report["publication"]["passed"])

    def test_invalid_report_window_is_rejected_before_walking_tree(self):
        with tempfile.TemporaryDirectory() as temporary:
            product = Path(temporary) / "product"
            product.mkdir()
            with self.assertRaisesRegex(ProductSizeError, "largest_file_count"):
                build_product_size_report(
                    product,
                    build_set_id="f" * 64,
                    product_profile="runtime-windows-shipping",
                    product_kind="runtime",
                    target_triple="x86_64-pc-windows-msvc",
                    cargo_profile="shipping",
                    warning_bytes=100,
                    hard_limit_bytes=1_000,
                    largest_file_count=-1,
                )


if __name__ == "__main__":
    unittest.main()
