import pathlib
import tomllib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


class AstraNetFirstPartyFeatureProviderContractTests(unittest.TestCase):
    def test_catalog_links_http_and_websocket_feature_providers(self) -> None:
        manifest = tomllib.loads(
            (REPO_ROOT / "zircon_plugins/first_party_runtime_catalog/Cargo.toml").read_text(
                encoding="utf-8"
            )
        )
        base_features = manifest["features"]["base-runtime-plugins"]
        self.assertIn("dep:zircon_plugin_net_http_runtime", base_features)
        self.assertIn("dep:zircon_plugin_net_websocket_runtime", base_features)
        self.assertEqual(
            manifest["dependencies"]["zircon_plugin_net_http_runtime"]["path"],
            "../net/features/http/runtime",
        )
        self.assertEqual(
            manifest["dependencies"]["zircon_plugin_net_websocket_runtime"]["path"],
            "../net/features/websocket/runtime",
        )

        source = (
            REPO_ROOT / "zircon_plugins/first_party_runtime_catalog/src/lib.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("pub struct FirstPartyRuntimeCatalogReport", source)
        self.assertIn("pub runtime_plugin_features: Vec<RuntimePluginFeatureRegistrationReport>", source)
        self.assertIn("zircon_plugin_net_http_runtime::plugin_feature_registration()", source)
        self.assertIn("zircon_plugin_net_websocket_runtime::plugin_feature_registration()", source)

    def test_app_ordinary_catalog_path_forwards_feature_reports(self) -> None:
        first_party = (
            REPO_ROOT / "zircon_app/src/entry/first_party_runtime_plugins.rs"
        ).read_text(encoding="utf-8")
        engine_entry = (REPO_ROOT / "zircon_app/src/entry/engine_entry.rs").read_text(
            encoding="utf-8"
        )
        builtin_modules = (REPO_ROOT / "zircon_app/src/entry/builtin_modules.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("first_party_runtime_catalog_for_manifest", first_party)
        self.assertIn("runtime_plugin_features", first_party)
        self.assertIn("first_party_runtime_catalog_for_manifest(", engine_entry)
        self.assertIn("runtime_plugin_features", engine_entry)
        self.assertIn("FirstPartyRuntimeCatalogReports", engine_entry)
        self.assertIn(
            "builtin_modules_for_config_with_effective_manifest_and_runtime_plugin_and_feature_registrations",
            builtin_modules,
        )

    def test_ordinary_app_bootstrap_activates_selected_net_features(self) -> None:
        source = (
            REPO_ROOT
            / "zircon_app/src/entry/tests/profile_bootstrap/first_party_runtime_plugins.rs"
        ).read_text(encoding="utf-8")
        test = source.split(
            "fn ordinary_first_party_bootstrap_admits_selected_net_http_and_websocket_features()",
            1,
        )[1].split("#[cfg(", 1)[0]
        self.assertRegex(test, r"entry\s*\.bootstrap\(\)")
        self.assertIn('name == "NetHttpFeatureModule"', test)
        self.assertIn('name == "NetWebSocketFeatureModule"', test)

    def test_feature_factories_install_backends_on_the_canonical_manager(self) -> None:
        runtime_module = (
            REPO_ROOT / "zircon_plugins/net/runtime/src/module.rs"
        ).read_text(encoding="utf-8")
        runtime_lib = (REPO_ROOT / "zircon_plugins/net/runtime/src/lib.rs").read_text(
            encoding="utf-8"
        )
        service_types = (
            REPO_ROOT / "zircon_plugins/net/runtime/src/service_types.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub const DEFAULT_NET_MANAGER_NAME", runtime_module)
        self.assertIn("DEFAULT_NET_MANAGER_NAME", runtime_lib)
        self.assertIn("pub fn install_http_backend(&self", service_types)
        self.assertIn("pub fn install_websocket_backend(&self", service_types)

        cases = (
            (
                "zircon_plugins/net/features/http/runtime/src/feature.rs",
                "install_http_backend",
                "http_runtime_backend()",
                "http_runtime_manager()",
            ),
            (
                "zircon_plugins/net/features/websocket/runtime/src/feature.rs",
                "install_websocket_backend",
                "websocket_runtime_backend()",
                "websocket_runtime_manager()",
            ),
        )
        for relative_path, installer, backend_factory, private_manager_factory in cases:
            with self.subTest(path=relative_path):
                source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
                descriptor = source.split("pub fn module_descriptor()", 1)[1].split(
                    "pub fn feature_manifest()", 1
                )[0]
                self.assertIn("StartupMode::Immediate", descriptor)
                self.assertIn('"DefaultNetManager"', descriptor)
                self.assertIn(
                    "core.resolve_manager::<DefaultNetManager>(DEFAULT_NET_MANAGER_NAME)",
                    descriptor,
                )
                self.assertIn(f"manager.{installer}({backend_factory})", descriptor)
                self.assertNotIn(private_manager_factory, descriptor)


if __name__ == "__main__":
    unittest.main()
