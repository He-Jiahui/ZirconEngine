"""Disk-first defaults must apply even when Cargo selects a nested workspace."""
from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[3]
WORKSPACES = ("Cargo.toml", "zircon_plugins/Cargo.toml", "examples/woc/native/Cargo.toml")


class CompactProfileTests(unittest.TestCase):
    def test_dev_and_test_disable_symbols_and_incremental_for_every_package(self):
        for relative in WORKSPACES:
            manifest = tomllib.loads((ROOT / relative).read_text(encoding="utf-8"))
            for name in ("dev", "test"):
                with self.subTest(workspace=relative, profile=name):
                    profile = manifest["profile"][name]
                    self.assertEqual(profile["debug"], 0)
                    self.assertIs(profile["incremental"], False)
                    for package, override in profile.get("package", {}).items():
                        with self.subTest(package=package):
                            self.assertIsNot(override.get("incremental"), True)
                            self.assertIn(override.get("debug", 0), (0, False, "none"))
                    build = profile.get("build-override", {})
                    self.assertIsNot(build.get("incremental"), True)
                    self.assertIn(build.get("debug", 0), (0, False, "none"))

    def test_explicit_diagnostic_symbols_remain_available(self):
        for relative in WORKSPACES[:2]:
            manifest = tomllib.loads((ROOT / relative).read_text(encoding="utf-8"))
            with self.subTest(workspace=relative):
                profile = manifest["profile"]["profiling"]
                self.assertIs(profile["debug"], True)
                self.assertIs(profile["strip"], False)


if __name__ == "__main__":
    unittest.main()
