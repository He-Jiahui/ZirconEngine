import tomllib
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
DEV_DEPS_PACKAGE = "zircon_runtime/crates/zr_dev_deps_dylib"
RUNTIME_DEV_PACKAGE = "zircon_runtime/crates/zr_runtime_dev_dylib"


def load_manifest(relative_path: str) -> dict:
    with (REPOSITORY_ROOT / relative_path).open("rb") as source:
        return tomllib.load(source)


class DevDynamicLinkingContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workspace = load_manifest("Cargo.toml")
        cls.runtime = load_manifest("zircon_runtime/Cargo.toml")
        cls.app = load_manifest("zircon_app/Cargo.toml")
        cls.editor = load_manifest("zircon_editor/Cargo.toml")
        cls.dev_deps = load_manifest(f"{DEV_DEPS_PACKAGE}/Cargo.toml")
        cls.runtime_dev = load_manifest(f"{RUNTIME_DEV_PACKAGE}/Cargo.toml")

    def test_helper_crates_are_internal_workspace_dylibs(self) -> None:
        members = self.workspace["workspace"]["members"]
        self.assertIn(DEV_DEPS_PACKAGE, members)
        self.assertIn(RUNTIME_DEV_PACKAGE, members)

        for manifest in (self.dev_deps, self.runtime_dev):
            self.assertFalse(manifest["package"]["publish"])
            self.assertEqual(manifest["lib"]["crate-type"], ["dylib"])

    def test_runtime_keeps_rlib_as_its_only_default_library_type(self) -> None:
        self.assertEqual(self.runtime["lib"]["crate-type"], ["rlib"])

    def test_heavy_dependency_dylib_has_bounded_feature_forwarding(self) -> None:
        dependencies = self.dev_deps["dependencies"]
        self.assertEqual(set(dependencies), {"gltf", "image", "meshopt", "naga", "wgpu"})
        for dependency in dependencies.values():
            self.assertTrue(dependency["optional"])
        self.assertEqual(
            self.dev_deps["features"],
            {
                "default": [],
                "naga": ["dep:naga"],
                "runtime-assets": ["dep:gltf", "dep:image", "dep:meshopt"],
                "wgpu": ["dep:wgpu"],
            },
        )

        features = self.runtime["features"]
        self.assertEqual(
            features["dev-dynamic-linking"],
            ["dep:zr_dev_deps_dylib", "zr_dev_deps_dylib/runtime-assets"],
        )
        self.assertIn("zr_dev_deps_dylib?/naga", features["text"])
        self.assertIn("zr_dev_deps_dylib?/wgpu", features["text"])
        self.assertIn("zr_dev_deps_dylib?/naga", features["target-server"])
        self.assertNotIn("dev-dynamic-linking", features["default"])
        self.assertTrue(self.runtime["dependencies"]["zr_dev_deps_dylib"]["optional"])

    def test_product_features_activate_only_the_runtime_dylib_anchor(self) -> None:
        for manifest in (self.app, self.editor):
            self.assertEqual(
                manifest["features"]["dev-dynamic-linking"],
                ["dep:zr_runtime_dev_dylib", "zr_runtime_dev_dylib/runtime"],
            )
            self.assertNotIn("dev-dynamic-linking", manifest["features"]["default"])
            self.assertTrue(manifest["dependencies"]["zr_runtime_dev_dylib"]["optional"])
            self.assertFalse(manifest["dependencies"]["zircon_runtime"]["default-features"])

        runtime_dependency = self.runtime_dev["dependencies"]["zircon_runtime"]
        self.assertTrue(runtime_dependency["optional"])
        self.assertEqual(
            self.runtime_dev["features"]["runtime"],
            ["dep:zircon_runtime", "zircon_runtime/dev-dynamic-linking"],
        )
        self.assertFalse(
            self.workspace["workspace"]["dependencies"]["zircon_runtime"][
                "default-features"
            ]
        )

    def test_crate_roots_force_link_without_exporting_helpers(self) -> None:
        source_contracts = {
            "zircon_runtime/src/lib.rs": "use zr_dev_deps_dylib as _;",
            "zircon_app/src/lib.rs": "use zr_runtime_dev_dylib as _;",
            "zircon_editor/src/lib.rs": "use zr_runtime_dev_dylib as _;",
        }
        for relative_path, expected_import in source_contracts.items():
            source = (REPOSITORY_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn('#[cfg(feature = "dev-dynamic-linking")]', source)
            self.assertIn(expected_import, source)

        for package in (DEV_DEPS_PACKAGE, RUNTIME_DEV_PACKAGE):
            source = (REPOSITORY_ROOT / package / "src/lib.rs").read_text(
                encoding="utf-8"
            )
            self.assertNotIn("pub use", source)

        runtime_dev_source = (
            REPOSITORY_ROOT / RUNTIME_DEV_PACKAGE / "src/lib.rs"
        ).read_text(encoding="utf-8")
        self.assertIn('#[cfg(feature = "runtime")]', runtime_dev_source)
        self.assertIn("use zircon_runtime as _;", runtime_dev_source)

    def test_dev_and_test_profiles_limit_incremental_codegen(self) -> None:
        profiles = self.workspace["profile"]
        for profile_name in ("dev", "test"):
            profile = profiles[profile_name]
            self.assertEqual(profile["debug"], 0)
            self.assertFalse(profile["incremental"])
            self.assertEqual(
                set(profile["package"]),
                {"zircon_runtime", "zircon_editor", "zircon_app"},
            )
            for package in profile["package"].values():
                self.assertTrue(package["incremental"])

        self.assertEqual(
            profiles["profiling"],
            {"inherits": "release", "debug": True, "strip": False},
        )


if __name__ == "__main__":
    unittest.main()
