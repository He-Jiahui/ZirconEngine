import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.build.zircon_build_product_profile import (
    DEFAULT_TARGET_TRIPLE,
    ProductBuildProfileError,
    build_set_id_for,
    load_product_build_profiles,
)
from tools.build.zircon_build import parse_args, resolve_config


class ProductBuildProfileTests(unittest.TestCase):
    def test_shipping_products_preserve_host_panic_isolation(self):
        root = Path(__file__).resolve().parents[2]
        cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
        profiles = cargo["profile"]

        def panic_strategy(name):
            profile = profiles.get(name, {})
            if "panic" in profile:
                return profile["panic"]
            parent = profile.get("inherits")
            return panic_strategy(parent) if parent else "unwind"

        products = load_product_build_profiles(
            root / "tools" / "export" / "product_build_profiles.toml"
        )
        for name in ("runtime-windows", "editor-windows"):
            profile = products.by_name(name)
            with self.subTest(product=name):
                self.assertEqual(panic_strategy(profile.cargo_profile), "unwind")
        self.assertEqual(panic_strategy("shipping-symbols"), "unwind")

    def test_target_alias_accepts_cargo_triple_in_equals_form(self):
        args = parse_args(["--target=x86_64-pc-windows-msvc", "--out", "out"])
        self.assertEqual(args.target_triple, "x86_64-pc-windows-msvc")

    def test_repository_profiles_resolve_aliases_and_keep_build_identity_separate(self):
        profiles = load_product_build_profiles(
            Path(__file__).resolve().parents[1]
            / "export"
            / "product_build_profiles.toml"
        )
        runtime = profiles.by_name("runtime-windows")
        self.assertEqual(runtime.name, "runtime-windows-shipping")
        self.assertEqual(runtime.target_triple, DEFAULT_TARGET_TRIPLE)
        self.assertEqual(runtime.hard_limit_bytes, 200_000_000)
        self.assertEqual(runtime.warning_bytes, 300_000_000)
        self.assertNotIn("target-client", runtime.runtime_features)
        self.assertEqual(runtime.build_set_id, build_set_id_for(runtime))
        self.assertNotEqual(
            runtime.build_set_id,
            build_set_id_for(runtime, selected_plugins=["optional-plugin"]),
        )
        editor = profiles.by_name("editor-windows")
        self.assertEqual(editor.hard_limit_bytes, 400_000_000)
        self.assertEqual(editor.warning_bytes, 500_000_000)
        self.assertEqual(editor.asset_scope, "editor")
        self.assertEqual(editor.product_kind, "editor")

    def test_profile_loader_rejects_unknown_fields(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "profiles.toml"
            path.write_text(
                """
schema_version = 1
default_target_triple = "x86_64-pc-windows-msvc"
[profiles.example]
product_kind = "runtime"
target_triple = "x86_64-pc-windows-msvc"
cargo_profile = "shipping"
app_package = "zircon_app"
app_binary = "zircon_runtime"
runtime_package = "zircon_runtime"
runtime_library = "zircon_runtime.dll"
runtime_features = ["shipping-runtime"]
app_features = ["shipping-runtime"]
plugin_carrier = "none"
asset_scope = "runtime"
include_project_pack = false
symbols = "external"
warning_bytes = 3
hard_limit_bytes = 4
hard_gate = true
archive = true
forbidden_path_tokens = ["cache"]
unexpected = true
""",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ProductBuildProfileError, "unknown field"):
                load_product_build_profiles(path)

    def test_product_profile_rejects_feature_drift(self):
        profile = load_product_build_profiles(
            Path(__file__).resolve().parents[1]
            / "export"
            / "product_build_profiles.toml"
        ).by_name("runtime-windows")
        args = parse_args(
            [
                "--product-profile",
                "runtime-windows",
                "--out",
                "D:/cargo-targets/profile-feature-drift",
                "--runtime-features",
                "shipping-runtime,text",
                "--mode",
                "shipping",
            ]
        )
        with self.assertRaisesRegex(SystemExit, "ad-hoc --runtime-features"):
            resolve_config(args, Path.cwd(), ())

    def test_build_set_identity_is_order_independent_for_sets(self):
        profile = load_product_build_profiles(
            Path(__file__).resolve().parents[1]
            / "export"
            / "product_build_profiles.toml"
        ).by_name("runtime-windows")
        self.assertEqual(
            build_set_id_for(
                profile,
                selected_plugins=["z-plugin", "a-plugin", "z-plugin"],
                feature_overrides=["b", "a", "b"],
            ),
            build_set_id_for(
                profile,
                selected_plugins=["a-plugin", "z-plugin"],
                feature_overrides=["a", "b"],
            ),
        )

    def test_product_profile_file_requires_profile_selector(self):
        args = parse_args(
            [
                "--product-profile-file",
                "profiles.toml",
                "--out",
                "D:/cargo-targets/profile-file-without-name",
            ]
        )
        with self.assertRaisesRegex(SystemExit, "requires --product-profile"):
            resolve_config(args, Path.cwd(), ())

    def test_runtime_product_uses_a_separate_cargo_bin_gate(self):
        cargo_text = (
            Path(__file__).resolve().parents[2] / "zircon_app" / "Cargo.toml"
        ).read_text(encoding="utf-8")
        self.assertIn(
            'name = "zircon_runtime_product"',
            cargo_text,
        )
        self.assertIn(
            'required-features = ["runtime-product"]',
            cargo_text,
        )
        self.assertIn(
            'name = "zircon_runtime"',
            cargo_text,
        )
        self.assertIn(
            'required-features = ["target-client"]',
            cargo_text,
        )
        self.assertIn(
            'name = "zircon_runtime_product"\npath = "src/bin/runtime_product.rs"',
            cargo_text,
            "shipping Runtime must use a distinct Cargo entry file from the development preview",
        )


if __name__ == "__main__":
    unittest.main()
