from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks05PreferenceStorageBoundaryTests(unittest.TestCase):
    def test_neutral_contract_and_runtime_owners_are_folder_backed(self) -> None:
        required = (
            "zircon_runtime/src/core/framework/platform/preferences/mod.rs",
            "zircon_runtime/src/core/framework/platform/preferences/backend.rs",
            "zircon_runtime/src/core/framework/platform/preferences/error.rs",
            "zircon_runtime/src/core/framework/platform/preferences/key.rs",
            "zircon_runtime/src/core/framework/platform/preferences/storage.rs",
            "zircon_runtime/src/platform/preferences/atomic_file.rs",
            "zircon_runtime/src/platform/preferences/unavailable.rs",
            "zircon_runtime/src/platform/service_types/driver.rs",
            "zircon_runtime/src/platform/service_types/manager.rs",
        )
        for relative in required:
            self.assertTrue((REPO_ROOT / relative).is_file(), relative)
        self.assertFalse(
            (REPO_ROOT / "zircon_runtime/src/platform/service_types.rs").exists()
        )

    def test_runtime_contract_has_no_woc_or_editor_special_case(self) -> None:
        roots = (
            REPO_ROOT / "zircon_runtime/src/core/framework/platform/preferences",
            REPO_ROOT / "zircon_runtime/src/platform/preferences",
            REPO_ROOT / "zircon_runtime/src/platform/service_types",
        )
        source = "\n".join(
            path.read_text(encoding="utf-8")
            for root in roots
            for path in sorted(root.rglob("*.rs"))
        )
        for forbidden in ("woc_", "world_of_claudecraft", "zircon_editor", "localStorage"):
            self.assertNotIn(forbidden, source)

    def test_manager_access_is_versioned_and_contract_typed(self) -> None:
        resolver = (
            REPO_ROOT / "zircon_runtime/src/core/manager/resolver.rs"
        ).read_text(encoding="utf-8")
        service_names = (
            REPO_ROOT / "zircon_runtime/src/core/manager/service_names.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("PreferenceStorage", resolver)
        self.assertIn("platform_preference_storage_handle", resolver)
        self.assertIn("PLATFORM_MANAGER_NAME", service_names)
        self.assertNotIn("Arc<PlatformManager>", resolver)

    def test_production_host_installs_desktop_backend_after_platform_activation(self) -> None:
        host = (
            REPO_ROOT / "zircon_app/src/entry/platform_preferences.rs"
        ).read_text(encoding="utf-8")
        entry = (REPO_ROOT / "zircon_app/src/entry/engine_entry.rs").read_text(
            encoding="utf-8"
        )
        entry_root = (REPO_ROOT / "zircon_app/src/entry/mod.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("LOCALAPPDATA", host)
        self.assertIn("XDG_DATA_HOME", host)
        self.assertIn("Application Support", host)
        self.assertIn("AtomicFilePreferenceStorageBackend", host)
        self.assertIn("install_preference_storage_backend", host)
        self.assertIn("install_default_preference_storage", entry)
        self.assertIn("with_preference_storage_backend", entry)
        self.assertIn("mod platform_preferences;", entry_root)
        self.assertGreater(
            entry.rfind("install_default_preference_storage("),
            entry.rfind("runtime.activate_registered_modules()?"),
        )
        production_host = host.split("#[cfg(test)]", maxsplit=1)[0]
        self.assertNotIn("activate_module(PLATFORM_MODULE_NAME)", production_host)
        self.assertLess(
            host.find("if !config.enabled"),
            host.find("if let Some(host_backend)"),
        )

    def test_backend_install_and_error_mapping_are_explicit(self) -> None:
        driver = (
            REPO_ROOT / "zircon_runtime/src/platform/service_types/driver.rs"
        ).read_text(encoding="utf-8")
        atomic = (
            REPO_ROOT / "zircon_runtime/src/platform/preferences/atomic_file.rs"
        ).read_text(encoding="utf-8")
        error = (
            REPO_ROOT
            / "zircon_runtime/src/core/framework/platform/preferences/error.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("AlreadyInstalled", driver)
        self.assertIn("UnavailableBackend", driver)
        self.assertIn("QuotaExceeded", atomic)
        self.assertNotIn("FilesystemQuotaExceeded", atomic)
        self.assertIn("ReadOnlyFilesystem", atomic)
        self.assertIn("sync_committed_value", atomic)
        self.assertIn("from_source", error)
        self.assertIn("fn source(&self)", error)

    def test_capability_diagnostics_accept_runtime_backend_truth(self) -> None:
        config = (REPO_ROOT / "zircon_runtime/src/platform/config.rs").read_text(
            encoding="utf-8"
        )
        report = (
            REPO_ROOT / "zircon_runtime/src/platform/capability/report.rs"
        ).read_text(encoding="utf-8")
        entry = (REPO_ROOT / "zircon_app/src/entry/engine_entry.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("diagnostic_lines_with_preference_storage_backend", config)
        self.assertIn("with_preference_storage_backend", report)
        self.assertIn("preference_storage_backend", entry)

    def test_no_silent_memory_fallback_or_platform_cfg_branch(self) -> None:
        source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(
                (REPO_ROOT / "zircon_runtime/src/platform/preferences").rglob("*.rs")
            )
        )
        self.assertIn("UnavailablePreferenceStorageBackend", source)
        self.assertNotIn("HashMap", source)
        self.assertNotIn("cfg(target_os", source)


if __name__ == "__main__":
    unittest.main()
