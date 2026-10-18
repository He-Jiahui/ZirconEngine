import types
import unittest
import tempfile
from pathlib import Path
from unittest import mock

from tools.build.zircon_build import build_runtime
from tools.build.zircon_build_abi import AbiValidationError
from tools.build.zircon_build_product_output import _validate_product_abis
from tools.build.zircon_build_product_profile import load_product_build_profile


class RuntimeBuildOrchestrationTests(unittest.TestCase):
    def config(self, *, shipping=False):
        return types.SimpleNamespace(
            targets_root=Path("output/targets"),
            runtime_preview_feature_arg="target-client",
            dry_run=False,
            mode="shipping" if shipping else "debug",
            is_shipping=shipping,
        )

    def test_runtime_library_and_preview_have_independent_build_commands(self):
        config = self.config()
        with mock.patch("tools.build.zircon_build.run_cargo") as run, mock.patch(
            "tools.build.zircon_build.copy_artifact"
        ) as copy:
            build_runtime(config, "target-client", include_preview=True)
        library = run.call_args_list[0].args[1]
        preview = run.call_args_list[1].args[1]
        self.assertEqual(
            library[:7],
            ["rustc", "-p", "zircon_runtime", "--lib", "--crate-type", "cdylib", "--no-default-features"],
        )
        self.assertNotIn("--", library)
        self.assertEqual(library[library.index("--features") + 1], "target-client")
        self.assertEqual(
            preview[:5], ["build", "-p", "zircon_app", "--bin", "zircon_runtime"]
        )
        self.assertEqual(copy.call_count, 2)

    def test_development_runtime_keeps_non_api_feature_compositions_available(self):
        config = self.config()
        with mock.patch("tools.build.zircon_build.run_cargo") as run, mock.patch(
            "tools.build.zircon_build.copy_artifact"
        ) as copy:
            build_runtime(config, "target-server", include_preview=False)
        self.assertEqual(run.call_count, 1)
        self.assertEqual(copy.call_count, 1)

    def test_development_features_fail_before_any_build_or_copy(self):
        for features in (
            "dev-dynamic-linking",
            "target-client,zircon_runtime/dev-dynamic-linking",
        ):
            with self.subTest(features=features), mock.patch(
                "tools.build.zircon_build.run_cargo"
            ) as run, mock.patch("tools.build.zircon_build.copy_artifact") as copy:
                with self.assertRaisesRegex(ValueError, "C ABI products"):
                    build_runtime(self.config(shipping=True), features, include_preview=False)
                run.assert_not_called()
                copy.assert_not_called()

    def test_core_only_features_fail_before_any_build_or_copy(self):
        with mock.patch("tools.build.zircon_build.run_cargo") as run, mock.patch(
            "tools.build.zircon_build.copy_artifact"
        ) as copy:
            with self.assertRaisesRegex(ValueError, "dynamic-api"):
                build_runtime(self.config(shipping=True), "core-min", include_preview=False)
            run.assert_not_called()
            copy.assert_not_called()

    def test_product_finalization_requires_runtime_and_host_artifacts(self):
        profile = load_product_build_profile("runtime-windows")
        with tempfile.TemporaryDirectory() as temporary:
            config = types.SimpleNamespace(
                engine_root=Path(temporary) / "ZirconEngine",
                repo_root=Path.cwd(),
            )
            config.engine_root.mkdir()
            with self.assertRaisesRegex(AbiValidationError, "required product artifact"):
                _validate_product_abis(config, profile)


if __name__ == "__main__":
    unittest.main()
