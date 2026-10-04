from __future__ import annotations

import argparse
import unittest
from pathlib import Path

from tools.export.cli_arguments import parse_args
from tools.export.compile_host import (
    compile_host_command,
    compile_host_executable_path,
    product_compile_host_plan,
    resolve_product_profile,
)
from tools.build.zircon_build_product_profile import load_product_build_profile


class ProductCompileHostTests(unittest.TestCase):
    def test_cli_accepts_product_profile_without_changing_export_profile(self) -> None:
        args = parse_args(
            [
                "--profile",
                "windows-release",
                "--product-profile",
                "runtime-windows",
                "--target-triple",
                "x86_64-pc-windows-msvc",
            ]
        )
        self.assertEqual(args.profile, "windows-release")
        self.assertEqual(args.product_profile, "runtime-windows")
        self.assertEqual(args.target_triple, "x86_64-pc-windows-msvc")

    def test_product_plan_adds_shipping_profile_and_target(self) -> None:
        profile = load_product_build_profile("runtime-windows")
        plan = product_compile_host_plan(profile)
        args = argparse.Namespace(
            cargo="cargo",
            no_locked=False,
            offline=False,
            target_dir=None,
        )
        command = compile_host_command(args, Path("E:/export"), plan)
        self.assertIn("--profile", command)
        self.assertEqual(command[command.index("--profile") + 1], "shipping")
        self.assertIn("--target", command)
        self.assertEqual(
            command[command.index("--target") + 1], "x86_64-pc-windows-msvc"
        )
        self.assertIn("runtime-product", command)
        self.assertEqual(command[command.index("--bin") + 1], "zircon_runtime_product")

    def test_product_executable_path_uses_target_specific_layout(self) -> None:
        profile = load_product_build_profile("runtime-windows")
        plan = product_compile_host_plan(profile)
        path = compile_host_executable_path(
            Path("E:/export"),
            plan,
            target_dir=Path("E:/export/stages/compile_host/target"),
        )
        self.assertEqual(
            path,
            Path(
                "E:/export/stages/compile_host/target/"
                "x86_64-pc-windows-msvc/shipping/zircon_runtime.exe"
            ),
        )

    def test_product_profile_target_mismatch_is_diagnostic(self) -> None:
        args = argparse.Namespace(
            product_profile="runtime-windows",
            product_profile_file=None,
            target_triple="aarch64-unknown-linux-gnu",
        )
        diagnostics: list[str] = []
        self.assertIsNone(resolve_product_profile(args, diagnostics))
        self.assertTrue(any("requires target" in item for item in diagnostics))


if __name__ == "__main__":
    unittest.main()
