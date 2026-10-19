from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def section(source: str, start: str, end: str) -> str:
    start_index = source.index(start)
    end_index = source.index(end, start_index + len(start))
    return source[start_index:end_index]


class AstraPluginSelectionAndTrustContractTests(unittest.TestCase):
    def test_selection_resolution_preserves_outcomes_and_fails_closed(self) -> None:
        source = read(
            "zircon_runtime/src/core/framework/project/plugin_selection_resolution.rs"
        )

        for status in (
            "Resolved",
            "InvalidId",
            "Duplicate",
            "Unsupported",
            "SkippedDisabled",
            "SkippedTargetMismatch",
        ):
            self.assertIn(f"PluginSelectionResolutionStatus::{status}", source)
        self.assertIn("pub struct PluginSelectionResolutionReport<T>", source)
        self.assertIn("for selection in &manifest.selections", source)
        self.assertIn("let mut outcomes = Vec::with_capacity(manifest.selections.len())", source)
        self.assertIn("outcomes.push(PluginSelectionResolution", source)
        self.assertIn("into_registrations_if_required_resolved", source)
        self.assertIn("RequiredPluginSelectionResolutionError", source)
        self.assertIn("if failures.is_empty()", source)

    def test_first_party_catalogs_share_the_resolution_report(self) -> None:
        runtime = read("zircon_plugins/first_party_runtime_catalog/src/lib.rs")
        editor = read("zircon_plugins/first_party_editor_catalog/src/catalog.rs")
        app_runtime = read("zircon_app/src/entry/first_party_runtime_plugins.rs")
        app_editor = read("zircon_app/src/entry/first_party_editor_plugins.rs")

        for source in (runtime, editor, app_runtime, app_editor):
            self.assertIn("PluginSelectionResolutionReport", source)
        for source in (runtime, editor):
            self.assertIn("resolve_plugin_selections(target_mode, manifest", source)
        self.assertIn("target_mode != RuntimeTargetMode::EditorHost", editor)
        self.assertIn("resolve_plugin_selections(target_mode, manifest, |_| None)", app_runtime)

    def test_app_ready_paths_check_required_runtime_and_editor_outcomes(self) -> None:
        engine_entry = read("zircon_app/src/entry/engine_entry.rs")
        editor_runner = read("zircon_app/src/entry/entry_runner/editor.rs")

        self.assertIn("first_party_runtime_catalog_for_manifest", engine_entry)
        self.assertIn(".into_registrations_if_required_resolved()", engine_entry)
        self.assertIn("zircon_app first-party runtime plugin selection", engine_entry)
        self.assertIn("first_party_runtime_plugin_registrations_for_config", editor_runner)
        self.assertIn("first_party_editor_plugin_registrations_for_config", editor_runner)
        self.assertIn(".into_registrations_if_required_resolved_where", editor_runner)
        self.assertIn("selection.editor_crate.is_some()", editor_runner)

    def test_editor_product_catalog_uses_role_admission_not_name_blacklists(self) -> None:
        source = read("zircon_editor/build.rs")
        eligibility = section(source, "fn editor_catalog_entry_from_manifest", "fn required_manifest_str")
        role = section(source, "enum PluginPackageRole", "#[cfg(test)]")

        self.assertIn("PluginPackageRole::parse", eligibility)
        self.assertIn("if !package_role.is_product_catalog_eligible()", eligibility)
        self.assertIn("fn is_product_catalog_eligible", role)
        self.assertIn("Self::Production | Self::DeveloperTool", role)
        self.assertIn("Self::Sample", role)
        self.assertIn("Self::TestFixture", role)
        self.assertNotIn("contains(\"fixture\")", eligibility.lower())
        self.assertNotIn("contains(\"sample\")", eligibility.lower())

    def test_native_loader_admits_before_descriptor_probe_or_library_execution(self) -> None:
        source = read("zircon_runtime/src/plugin/loader/load_discovered.rs")
        loader = section(source, "fn load_candidate_library", "fn retain_requested_entry")
        admit = loader.index("authority.admit(candidate, &library_path, module_kinds)")
        load = loader.index("load_admitted_library(&admission_receipt)")
        probe = loader.index("probe_native_plugin_descriptor")

        self.assertLess(admit, load)
        self.assertLess(load, probe)
        self.assertIn("LoadedNativePlugin::admitted_library(library, admission_receipt)", loader)
        admitted_loader = section(source, "unsafe fn load_admitted_library", "fn retain_requested_entry")
        self.assertIn("receipt.admitted_library_path()", admitted_loader)
        self.assertNotIn("Library::new(library_path)", admitted_loader)

    def test_native_authority_pins_identity_trust_target_capabilities_and_digests(self) -> None:
        source = read("zircon_runtime/src/plugin/loader/native_artifact_trust.rs")
        expectation = section(source, "pub struct NativePluginArtifactExpectation", "impl NativePluginArtifactExpectation")
        admit = section(source, "pub(crate) fn admit", "#[derive(Clone, Debug)]\npub struct NativePluginArtifactAdmissionReceipt")

        for field in (
            "pub plugin_id",
            "pub package_id",
            "pub manifest_digest",
            "pub library_digest",
            "pub trust",
            "pub target",
            "pub capabilities",
        ):
            self.assertIn(field, expectation)
        for check in (
            "expectation.trust.in_process_identity()",
            "candidate.package_manifest.id != expectation.plugin_id",
            "ensure_target_supported(&candidate.package_manifest, &expectation.target)",
            "actual_capabilities",
            "verify_manifest(candidate, &expectation.manifest_digest)",
            "NativePluginArtifactStaging::prepare",
            "digest_file(&library_guard, &staged_path)",
            "NativePluginArtifactAdmissionReceipt",
        ):
            self.assertIn(check, admit)
        self.assertLess(admit.index("verify_manifest"), admit.index("NativePluginArtifactStaging::prepare"))
        self.assertLess(admit.index("NativePluginArtifactStaging::prepare"), admit.index("digest_file"))
        self.assertIn("pub fn deny_all() -> Self", source)

    def test_hot_reload_reuses_authority_before_mutating_live_generation(self) -> None:
        source = read(
            "zircon_runtime/src/plugin/loader/live/lifecycle.rs"
        )
        reload = section(
            source,
            "fn hot_reload_plugin_result_with_authority",
            "pub(super) fn hot_reload_reported_plugin",
        )
        load = source.index("load_discovered_runtime_with_authority")
        state = source.index("NativePluginHotReloadState::new")

        self.assertIn("authority: &super::super::NativePluginArtifactAuthority", reload)
        self.assertIn("load_discovered_editor_with_authority(root, authority)", reload)
        self.assertLess(load, state)


if __name__ == "__main__":
    unittest.main()
