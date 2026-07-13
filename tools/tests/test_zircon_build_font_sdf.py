import json
import tempfile
import types
import unittest
from pathlib import Path

from tools.zircon_build_font_sdf import (
    FontSdfBakeManifestError,
    build_font_sdf_command,
    load_font_sdf_manifest,
)


class ZirconBuildFontSdfTests(unittest.TestCase):
    def test_subset_command_forwards_identity_params_and_managed_target(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest_path = self._write_manifest(
                root,
                {
                    "format_version": 1,
                    "bakes": [
                        {
                            "font": "assets/fonts/FiraSans-Regular.ttf",
                            "cache_root": ".zircon/cache",
                            "asset_guid": "12345678-90ab-4cde-8f01-234567890abc",
                            "face_index": 1,
                            "mode": "msdf",
                            "codepoints": ["U+0041", "U+004D", "U+4E2D"],
                            "page_size": 512,
                            "bake_em_px": 48,
                            "spread_px_milli": 8000,
                        }
                    ],
                },
            )
            spec = load_font_sdf_manifest(manifest_path, root)[0]

            command = build_font_sdf_command(self._config(root), spec)

            self.assertEqual("cargo", command[0])
            self.assertIn("--locked", command)
            self.assertEqual(("1",), _flag_values(command, "--jobs"))
            self.assertEqual(
                (str(root / "targets" / "font-sdf"),),
                _flag_values(command, "--target-dir"),
            )
            self.assertEqual(("msdf",), _flag_values(command, "--mode"))
            self.assertEqual(
                ("U+0041", "U+004D", "U+4E2D"),
                _flag_values(command, "--codepoint"),
            )
            self.assertNotIn("--all-cmap", command)

    def test_all_cmap_command_uses_single_mode_flag(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest_path = self._write_manifest(
                root,
                {
                    "format_version": 1,
                    "bakes": [
                        {
                            "font": "font.ttf",
                            "cache_root": "cache",
                            "asset_guid": "12345678-90ab-4cde-8f01-234567890abc",
                            "mode": "mtsdf",
                            "all_cmap": True,
                        }
                    ],
                },
            )
            spec = load_font_sdf_manifest(manifest_path, root)[0]

            command = build_font_sdf_command(self._config(root), spec)

            self.assertIn("--all-cmap", command)
            self.assertEqual((), _flag_values(command, "--codepoint"))
            self.assertEqual(("mtsdf",), _flag_values(command, "--mode"))

    def test_subset_ranges_expand_in_scalar_order_and_deduplicate(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest_path = self._write_manifest(
                root,
                {
                    "format_version": 1,
                    "bakes": [
                        {
                            "font": "font.ttf",
                            "cache_root": "cache",
                            "asset_guid": "12345678-90ab-4cde-8f01-234567890abc",
                            "mode": "sdf",
                            "codepoints": ["U+0041-U+0043", "U+0042"],
                        }
                    ],
                },
            )

            spec = load_font_sdf_manifest(manifest_path, root)[0]
            command = build_font_sdf_command(self._config(root), spec)

            self.assertEqual(
                ("U+0041", "U+0042", "U+0043"),
                _flag_values(command, "--codepoint"),
            )

    def test_manifest_rejects_missing_or_ambiguous_glyph_selection(self):
        invalid_bakes = [
            {
                "font": "font.ttf",
                "cache_root": "cache",
                "asset_guid": "12345678-90ab-4cde-8f01-234567890abc",
                "mode": "sdf",
            },
            {
                "font": "font.ttf",
                "cache_root": "cache",
                "asset_guid": "12345678-90ab-4cde-8f01-234567890abc",
                "mode": "sdf",
                "all_cmap": True,
                "codepoints": ["U+0041"],
            },
        ]
        for bake in invalid_bakes:
            with self.subTest(bake=bake), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                manifest_path = self._write_manifest(
                    root, {"format_version": 1, "bakes": [bake]}
                )
                with self.assertRaises(FontSdfBakeManifestError):
                    load_font_sdf_manifest(manifest_path, root)

    def test_build_root_does_not_absorb_font_bake_policy(self):
        repo_root = Path(__file__).resolve().parents[2]
        root_text = (repo_root / "tools/zircon_build.py").read_text(encoding="utf-8")
        child_text = (repo_root / "tools/zircon_build_font_sdf.py").read_text(
            encoding="utf-8"
        )

        for owner in (
            "FontSdfBakeSpec",
            "load_font_sdf_manifest",
            "build_font_sdf_command",
            "validate_font_sdf_spec",
        ):
            self.assertNotIn(f"def {owner}(", root_text)
            self.assertIn(owner, child_text)
        self.assertLessEqual(len(root_text.splitlines()), 1000)

    @staticmethod
    def _write_manifest(root: Path, document: dict) -> Path:
        path = root / "font-sdf.json"
        path.write_text(json.dumps(document), encoding="utf-8")
        return path

    @staticmethod
    def _config(root: Path):
        return types.SimpleNamespace(
            cargo="cargo",
            jobs="1",
            locked=True,
            repo_root=root,
            targets_root=root / "targets",
        )


def _flag_values(command: list[str], flag: str) -> tuple[str, ...]:
    return tuple(command[index + 1] for index, value in enumerate(command) if value == flag)


if __name__ == "__main__":
    unittest.main()
