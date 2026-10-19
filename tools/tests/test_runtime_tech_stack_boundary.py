import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.tech_stack_boundary import (  # noqa: E402
    _is_zr_vm_binding_dependency,
    _is_zr_vm_feature_name,
    _path_is_within,
    tech_stack_boundary_audit,
)
from runtime_structure_audits.plugin_surface_lifecycle_boundary import (  # noqa: E402
    plugin_surface_lifecycle_boundary_audit,
)


def _assert_canonical_zr_vm_command(command: str) -> None:
    feature_prefix = '--features "'
    assert feature_prefix in command, "ZrVM launch command must declare Cargo features"
    feature_list = command.split(feature_prefix, 1)[1].split('"', 1)[0]
    features = {feature.strip() for feature in feature_list.split(",")}

    assert "backend-zr-vm" in features, "canonical ZrVM backend feature is missing"
    assert "first-party-zr-vm-language-runtime-plugin" in features, (
        "canonical ZrVM language provider feature is missing"
    )
    assert "first-party-zr-vm-real-backend" not in features, (
        "retired ZrVM backend feature must not return"
    )

# 调用技术栈边界审计器核对 VM 后端特性命令与目录镜像守卫归属。
class RuntimeTechStackBoundaryTests(unittest.TestCase):
    def test_vampire_executable_commands_use_canonical_zr_vm_backend_feature(
        self,
    ) -> None:
        command_owners = (
            REPO_ROOT / "examples/vampire/README.md",
            REPO_ROOT
            / "docs/crates/zircon_runtime/script/vm/vampire_gameplay.md",
        )

        for owner in command_owners:
            with self.subTest(owner=owner.relative_to(REPO_ROOT)):
                source = owner.read_text(encoding="utf-8")
                command = next(
                    line.strip()
                    for line in source.splitlines()
                    if "cargo run -p zircon_app" in line
                )
                _assert_canonical_zr_vm_command(command)

        with self.assertRaises(AssertionError):
            _assert_canonical_zr_vm_command(
                'cargo run -p zircon_app --features "first-party-zr-vm-real-backend"'
            )

    def test_folder_backed_mirror_guard_is_discovered(self) -> None:
        report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertTrue(report["mirror_docs_guard_present"])
        self.assertNotIn(
            "Runtime 01 tech-stack mirror-doc aggregate guard is missing.",
            report["risks"],
        )

    def test_current_optional_text_and_backend_feature_declarations_are_clean(self) -> None:
        report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_version_anchors"])
        self.assertEqual([], report["dependency_boundary_violations"])
        self.assertEqual(2, report["jolt_feature_slot_count"])
        self.assertEqual(1, report["runtime_jolt_feature_slot_count"])
        self.assertEqual(1, report["physics_jolt_dependency_feature_slot_count"])
        self.assertTrue(report["joltc_sys_optional_dependency_present"])
        self.assertTrue(report["physics_jolt_backend_files_present"])
        self.assertTrue(report["jolt_backend_feature_gated"])
        self.assertTrue(report["runtime_joltc_sys_dependency_absent"])
        self.assertEqual([], report["risks"])

    def test_zr_vm_backend_is_plugin_owned_and_runtime_binding_free(self) -> None:
        report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertTrue(report["zr_vm_plugin_manifest_present"])
        self.assertTrue(report["zr_vm_plugin_backend_feature_present"])
        self.assertGreater(report["zr_vm_plugin_binding_dependency_count"], 0)
        self.assertTrue(report["zr_vm_plugin_binding_dependencies_optional"])
        self.assertTrue(report["zr_vm_plugin_binding_dependencies_external"])
        self.assertTrue(report["zr_vm_plugin_backend_feature_gates_bindings"])
        self.assertTrue(report["runtime_zr_vm_owner_absent"])

    def test_zr_vm_dependency_rename_and_path_traversal_cannot_bypass_owner_guard(
        self,
    ) -> None:
        self.assertTrue(
            _is_zr_vm_binding_dependency(
                "vm_binding",
                {"package": "zr_vm_rust_binding", "optional": True},
            )
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / "zr_vm"
            self.assertTrue(_path_is_within(owner / "binding", owner))
            self.assertFalse(_path_is_within(owner / ".." / "other", owner))
        self.assertTrue(_is_zr_vm_feature_name("zr-vm-real-backend"))
        self.assertTrue(_is_zr_vm_feature_name("zr_vm_backend"))
        self.assertTrue(_is_zr_vm_feature_name("backend-zrvm"))
        self.assertFalse(_is_zr_vm_feature_name("backend-jolt"))

    def test_runtime_zr_vm_renamed_dependency_is_reported_as_owner_drift(self) -> None:
        runtime_manifest = REPO_ROOT / "zircon_runtime/Cargo.toml"
        original = runtime_manifest.read_text(encoding="utf-8")
        drifted = original.replace(
            "[dependencies]",
            "[dependencies]\nvm_binding = { package = \"zr_vm_rust_binding\", path = \"../../zr_vm\", optional = true }",
            1,
        )
        real_read_toml = sys.modules[
            "runtime_structure_audits.tech_stack_boundary"
        ]._read_toml

        def read_toml_with_runtime_drift(path: Path) -> dict[str, object]:
            if path == runtime_manifest:
                import tomllib

                return tomllib.loads(drifted)
            return real_read_toml(path)

        with patch(
            "runtime_structure_audits.tech_stack_boundary._read_toml",
            side_effect=read_toml_with_runtime_drift,
        ):
            report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertFalse(report["runtime_zr_vm_owner_absent"])
        self.assertIn(
            "zircon_runtime declares plugin-owned ZrVM backend state",
            report["dependency_boundary_violations"],
        )

    def test_jolt_backend_is_feature_gated_and_plugin_owned(self) -> None:
        backend_root = REPO_ROOT / "zircon_plugins/physics/runtime/src/backend"

        self.assertFalse((backend_root.parent / "backend.rs").exists())
        self.assertIn(
            'JOLT_BACKEND_AVAILABLE: bool = cfg!(feature = "backend-jolt")',
            (backend_root / "selection.rs").read_text(encoding="utf-8"),
        )
        self.assertTrue((backend_root / "jolt/mod.rs").exists())
        self.assertTrue((backend_root / "jolt/native_world.rs").exists())
        self.assertTrue((backend_root / "jolt/runtime.rs").exists())
        self.assertEqual([], tech_stack_boundary_audit(REPO_ROOT)["risks"])

    def test_runtime_06_current_backend_command_and_folder_guard_owners_are_clean(self) -> None:
        report = plugin_surface_lifecycle_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_source_anchors"])
        self.assertEqual([], report["missing_cargo_gate_anchors"])
        self.assertTrue(report["mirror_docs_guard_present"])
        self.assertEqual([], report["risks"])


    def test_registry_policy_is_format_independent_and_requires_zstd_admission(self) -> None:
        from runtime_structure_audits.tech_stack_boundary import _runtime_dependency_policy_errors
        source = (
            '[features]\nnative-host = ["dep:zip", "dep:zstd"]\n[dependencies]\n'
            'zip = { optional = true, features = ["deflate-flate2"], default-features = false, version = "9.0.0-pre2" }\n'
            "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies]\n"
            'zstd = { optional = true, version = "0.13.3" }\n'
        )
        texture_owner = "zircon_plugins/texture_importer/runtime/Cargo.toml"
        texture_source = '[dependencies]\nzstd = "0.13.3"\n'
        def entries(value: str) -> list[tuple[str, str]]:
            return [
                ("Cargo.toml", "[workspace]\n"), ("zircon_runtime/Cargo.toml", value),
                (texture_owner, texture_source),
            ]
        self.assertEqual([], _runtime_dependency_policy_errors(
            entries(source), "zip", "9.0.0-pre2", "native-host", False, ("deflate-flate2",)))
        self.assertEqual([], _runtime_dependency_policy_errors(
            entries(source), "zstd", "0.13.3", "native-host", True, ()))
        unrelated = entries(source) + [
            ("zircon_plugins/example/Cargo.toml",
             "[dependencies]\nplugin_local_sdk = { workspace = true }\n")]
        self.assertEqual([], _runtime_dependency_policy_errors(
            unrelated, "zstd", "0.13.3", "native-host", True, ()))
        for label, drifted in (
            ("version", source.replace("0.13.3", "0.13.2")),
            ("admission", source.replace('"dep:zstd"', '"unrelated"')),
            ("optional", source.replace('zstd = { optional = true', 'zstd = { optional = false')),
        ):
            with self.subTest(zstd=label):
                self.assertTrue(_runtime_dependency_policy_errors(
                    entries(drifted), "zstd", "0.13.3", "native-host", True, ()))

        target_header = "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies]\n"
        section_form = source.replace(
            'zstd = { optional = true, version = "0.13.3" }\n',
            "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies.zstd]\n"
            'version = "0.13.3"\noptional = true\n',
        )
        self.assertEqual([], _runtime_dependency_policy_errors(
            entries(section_form), "zstd", "0.13.3", "native-host", True, ()))
        for label, drifted in (
            ("wrong-root", source.replace(target_header, "")),
            ("wasm-target", source.replace('not(target_arch = "wasm32")', 'target_arch = "wasm32"')),
            ("windows-target", source.replace('not(target_arch = "wasm32")', "windows")),
            ("target-dev", source.replace(target_header, target_header.replace(".dependencies]", ".dev-dependencies]"))),
            ("target-build", source.replace(target_header, target_header.replace(".dependencies]", ".build-dependencies]"))),
            ("target-duplicate", source + 'duplicate = { package = "zstd", version = "0.13.3", optional = true }\n'),
        ):
            with self.subTest(runtime_target=label):
                self.assertTrue(_runtime_dependency_policy_errors(
                    entries(drifted), "zstd", "0.13.3", "native-host", True, ()))

        for missing_owner in ("zircon_runtime/Cargo.toml", texture_owner):
            with self.subTest(missing_owner=missing_owner):
                self.assertTrue(_runtime_dependency_policy_errors(
                    [(path, text) for path, text in entries(source) if path != missing_owner],
                    "zstd", "0.13.3", "native-host", True, ()))
        inline = '[dependencies]\nzstd = { version = "0.13.3", optional = false }\n'
        for texture_value in (texture_source, inline):
            self.assertEqual([], _runtime_dependency_policy_errors(
                [(path, texture_value if path == texture_owner else text)
                 for path, text in entries(source)],
                "zstd", "0.13.3", "native-host", True, ()))
        texture_mutations = {
            "version": texture_source.replace("0.13.3", "0.13.2"),
            "optional": inline.replace("optional = false", "optional = true"),
            "defaults": inline.replace("optional = false", "default-features = false"),
            "features": inline.replace("optional = false", 'features = ["zstdmt"]'),
            "git": inline.replace("optional = false", 'git = "https://example.invalid/zstd"'),
            "path": inline.replace("optional = false", 'path = "../zstd"'),
            "alias": texture_source + 'compressed = { package = "zstd", version = "0.13.3" }\n',
            "canonical-alias": texture_source.replace('zstd = "0.13.3"', 'compressed = { package = "zstd", version = "0.13.3" }'),
            "dev": texture_source.replace("[dependencies]", "[dev-dependencies]"),
            "target": texture_source.replace("[dependencies]", "[target.'cfg(windows)'.dependencies]"),
            "metadata": texture_source.replace("[dependencies]", "[package.metadata]"),
        }
        for label, texture_value in texture_mutations.items():
            with self.subTest(texture=label):
                self.assertTrue(_runtime_dependency_policy_errors(
                    [(path, texture_value if path == texture_owner else text)
                     for path, text in entries(source)],
                    "zstd", "0.13.3", "native-host", True, ()))
        self.assertTrue(_runtime_dependency_policy_errors(
            entries(source) + [("zircon_editor/Cargo.toml", texture_source)],
            "zstd", "0.13.3", "native-host", True, ()))

    def test_zip_policy_rejects_owner_alias_metadata_and_feature_bypasses(self) -> None:
        from runtime_structure_audits.tech_stack_boundary import _runtime_dependency_policy_errors
        source = (
            '[features]\nnative-host = ["dep:zip"]\n[dependencies]\n'
            'zip = { version = "9.0.0-pre2", optional = true, default-features = false, features = ["deflate-flate2"] }\n'
        )
        mutations = {
            "version": source.replace("9.0.0-pre2", "9.0.0"),
            "defaults": source.replace("default-features = false", "default-features = true"),
            "extra-feature": source.replace('["deflate-flate2"]', '["deflate-flate2", "aes-crypto"]'),
            "nonoptional": source.replace("optional = true", "optional = false"),
            "admission": source.replace('["dep:zip"]', "[]"),
            "duplicate-admission": source.replace('["dep:zip"]', '["dep:zip", "dep:zip"]'),
            "alias": source + 'archive = { package = "zip", version = "9.0.0-pre2" }\n',
            "dev-alias": source + '[dev-dependencies]\narchive = { package = "zip", version = "9.0.0-pre2" }\n',
            "target-alias": source + "[target.'cfg(windows)'.build-dependencies]\n" + 'archive = { package = "zip", version = "9.0.0-pre2" }\n',
            "path": source.replace("optional = true", 'path = "../zip", optional = true'),
            "git": source.replace("optional = true", 'git = "https://example.invalid/zip", optional = true'),
            "canonical-alias": source.replace('zip = { version', 'archive = { package = "zip", version').replace('"dep:zip"', '"dep:archive"'),
            "metadata": source.replace("[dependencies]", "[package.metadata]"),
        }
        for label, drifted in mutations.items():
            with self.subTest(label=label):
                self.assertTrue(_runtime_dependency_policy_errors(
                    [("Cargo.toml", "[workspace]\n"), ("zircon_runtime/Cargo.toml", drifted)],
                    "zip", "9.0.0-pre2", "native-host", False, ("deflate-flate2",)))
        self.assertTrue(_runtime_dependency_policy_errors(
            [("Cargo.toml", "[workspace]\n"), ("zircon_runtime/Cargo.toml", source),
             ("zircon_editor/Cargo.toml", '[dependencies]\narchive = { package = "zip", version = "9.0.0-pre2" }\n')],
            "zip", "9.0.0-pre2", "native-host", False, ("deflate-flate2",)))

    def test_sdf_policy_requires_full_chain_and_rejects_retired_fontsdf_identity(self) -> None:
        from runtime_structure_audits.tech_stack_boundary import (
            _runtime_dependency_policy_errors, _runtime_retired_fontsdf_present,
        )
        packages = (("fdsm", "0.8.0"), ("fdsm-ttf-parser", "0.2.0"), ("ttf-parser", "0.25.1"))
        source = (
            '[features]\ntext = ["dep:fdsm", "dep:fdsm-ttf-parser", "dep:ttf-parser"]\n'
            '[dependencies]\n' + "".join(
                f'{name} = {{ version = "{version}", optional = true }}\n'
                for name, version in packages)
        )
        def entries(value: str, workspace: str = "[workspace]\n") -> list[tuple[str, str]]:
            return [("Cargo.toml", workspace), ("zircon_runtime/Cargo.toml", value)]
        for name, version in packages:
            self.assertEqual([], _runtime_dependency_policy_errors(
                entries(source), name, version, "text", True, ()))
            line = f'{name} = {{ version = "{version}", optional = true }}\n'
            for label, drifted in (
                ("version", source.replace(line, line.replace(version, "0.0.0"))),
                ("missing", source.replace(line, "")),
                ("optional", source.replace(line, line.replace("true", "false"))),
                ("admission", source.replace(f'"dep:{name}"', '"unrelated"')),
            ):
                with self.subTest(package=name, label=label):
                    self.assertTrue(_runtime_dependency_policy_errors(
                        entries(drifted), name, version, "text", True, ()))
        self.assertFalse(_runtime_retired_fontsdf_present(entries(source)))
        self.assertFalse(_runtime_retired_fontsdf_present(entries(
            source + '[package.metadata]\nlegacy = { package = "fontsdf", version = "0.5.3" }\n')))
        for suffix in (
            'fontsdf = { version = "0.5.3", optional = true }\n',
            'legacy = { package = "fontsdf", version = "0.5.3", optional = true }\n',
            '[dev-dependencies]\nlegacy = { package = "fontsdf", version = "0.5.3" }\n',
            "[target.'cfg(windows)'.dependencies]\n" + 'legacy = { package = "fontsdf", version = "0.5.3" }\n',
        ):
            with self.subTest(retired=suffix):
                self.assertTrue(_runtime_retired_fontsdf_present(entries(source + suffix)))
        self.assertTrue(_runtime_retired_fontsdf_present(entries(
            source + 'legacy = { workspace = true }\n',
            '[workspace.dependencies]\nlegacy = { package = "fontsdf", version = "0.5.3" }\n')))

    def test_jolt_and_zr_vm_current_gates_reject_removed_admission_and_revision(self) -> None:
        import copy
        boundary = sys.modules["runtime_structure_audits.tech_stack_boundary"]
        real_text, real_toml = boundary._read_text, boundary._read_toml
        backend = REPO_ROOT / "zircon_plugins/physics/runtime/src/backend/mod.rs"
        plugin = REPO_ROOT / "zircon_plugins/zr_vm_language/runtime/Cargo.toml"
        for anchor in (
            '#[cfg(feature = "backend-jolt")]\n#[path = "jolt/mod.rs"]\nmod jolt;',
            '#[cfg(feature = "backend-jolt")]\npub use jolt::JoltPhysicsBackend;',
        ):
            def changed_text(path: Path, anchor: str = anchor) -> str:
                value = real_text(path)
                return value.replace(anchor, anchor.replace('#[cfg(feature = "backend-jolt")]\n', "")) if path == backend else value
            with self.subTest(jolt=anchor), patch.object(boundary, "_read_text", side_effect=changed_text):
                self.assertIn("Physics Jolt backend module gate drifted",
                              tech_stack_boundary_audit(REPO_ROOT)["dependency_boundary_violations"])
        for field in ("revision", "optional", "admission"):
            def changed_toml(path: Path, field: str = field) -> dict[str, object]:
                value = real_toml(path)
                if path == plugin:
                    value = copy.deepcopy(value)
                    if field == "admission":
                        value["features"]["backend-zr-vm"] = []
                    else:
                        for name, spec in value["dependencies"].items():
                            if name.startswith("zr_vm_rust_binding"):
                                spec["rev" if field == "revision" else "optional"] = "wrong-revision" if field == "revision" else False
                return value
            with self.subTest(zr_vm=field), patch.object(boundary, "_read_toml", side_effect=changed_toml):
                report = tech_stack_boundary_audit(REPO_ROOT)
                self.assertTrue(report["dependency_boundary_violations"])
                self.assertFalse(report[
                    {"revision": "zr_vm_plugin_binding_dependencies_external",
                     "optional": "zr_vm_plugin_binding_dependencies_optional",
                     "admission": "zr_vm_plugin_backend_feature_gates_bindings"}[field]])


if __name__ == "__main__":
    unittest.main()
