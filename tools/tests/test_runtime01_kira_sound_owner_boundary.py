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
    _manifest_has_exact_single_package_pin,
    _manifest_dependency_owners,
    tech_stack_boundary_audit,
)
from runtime_structure_audits.tech_stack_markdown import (  # noqa: E402
    render_tech_stack_boundary_markdown,
)
from runtime_structure_audits.tech_stack_source_inventory import (  # noqa: E402
    KIRA_DEPENDENCY_LINE,
    KIRA_DEPENDENCY_VERSION,
    KIRA_OWNER_MANIFEST,
)


class Runtime01KiraSoundOwnerBoundaryTests(unittest.TestCase):
    def test_current_kira_dependency_is_pinned_to_the_sound_runtime_owner(self) -> None:
        report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["declared_removed_dependencies"])
        self.assertEqual([KIRA_OWNER_MANIFEST], report["kira_dependency_owners"])
        self.assertTrue(report["kira_owner_version_pinned"])
        self.assertEqual(1, report["kira_owner_dependency_declaration_count"])
        self.assertEqual(
            [KIRA_DEPENDENCY_VERSION],
            report["kira_owner_dependency_versions"],
        )
        self.assertEqual([], report["kira_owner_violations"])
        self.assertEqual([], report["missing_kira_tech_stack_doc_anchors"])
        self.assertNotIn(
            "Kira dependency escaped the Sound runtime owner.",
            report["risks"],
        )
        rendered = render_tech_stack_boundary_markdown(report)
        self.assertIn(
            f"- Kira dependency owners: {KIRA_OWNER_MANIFEST}",
            rendered,
        )
        self.assertIn("- Sound runtime Kira 0.12.2 pin: current", rendered)
        self.assertIn(
            "- Sound runtime Kira declarations: 1 (versions: 0.12.2)",
            rendered,
        )

    def test_kira_owner_scan_reports_every_manifest_instead_of_hiding_leaks(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sound_manifest = root / KIRA_OWNER_MANIFEST
            sound_manifest.parent.mkdir(parents=True)
            sound_manifest.write_text(
                "[dependencies]\n" + KIRA_DEPENDENCY_LINE + "\n",
                encoding="utf-8",
            )
            (root / "Cargo.toml").write_text(
                "[workspace.dependencies]\n"
                'audio_backend = { package = "kira", version = "0.12.2" }\n',
                encoding="utf-8",
            )
            validation_copy = root / ".codex/state/validation-copy/Cargo.toml"
            validation_copy.parent.mkdir(parents=True)
            validation_copy.write_text(
                "[dependencies]\nkira = \"0.11.0\"\n",
                encoding="utf-8",
            )

            self.assertEqual(
                ["Cargo.toml", KIRA_OWNER_MANIFEST],
                _manifest_dependency_owners(root, "kira"),
            )

    def test_runtime_rust_guards_mirror_the_kira_sound_owner_contract(self) -> None:
        dependency_guard = (
            REPO_ROOT
            / "zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs"
        ).read_text(encoding="utf-8")
        mirror_guard = (
            REPO_ROOT
            / "zircon_runtime/src/tests/runtime_absorption/tech_stack/mirror_docs.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            'for removed in ["cosmic-text", "rfd", "arboard"]',
            dependency_guard,
        )
        self.assertIn(KIRA_OWNER_MANIFEST, dependency_guard)
        self.assertIn(KIRA_DEPENDENCY_LINE, dependency_guard)
        self.assertIn('manifest_declares_package(source, "kira")', dependency_guard)
        self.assertIn("expected_non_dependency_count = 4", mirror_guard)
        self.assertIn("kira_dependency_owners = [", mirror_guard)
        self.assertIn("kira_owner_version_pinned = true", mirror_guard)
        self.assertIn("kira_owner_violations = []", mirror_guard)

    def test_kira_pin_cannot_be_faked_by_metadata_or_a_second_alias(self) -> None:
        cases = (
            (
                "[dependencies]\n"
                'audio = { package = "kira", version = "0.11.0" }\n'
                "[package.metadata]\n"
                + KIRA_DEPENDENCY_LINE
                + "\n",
                False,
            ),
            (
                "[dependencies]\n"
                + KIRA_DEPENDENCY_LINE
                + "\n"
                + 'audio = { package = "kira", version = "0.11.0" }\n',
                False,
            ),
            ("[package.metadata]\n" + KIRA_DEPENDENCY_LINE + "\n", False),
            (
                "[target.'cfg(windows)'.dependencies]\n"
                + 'audio = { package = "kira", version = "0.12.2" }\n',
                True,
            ),
            ("[dev-dependencies]\n" + KIRA_DEPENDENCY_LINE + "\n", False),
            ("[build-dependencies]\n" + KIRA_DEPENDENCY_LINE + "\n", False),
            ("[workspace.dependencies]\n" + KIRA_DEPENDENCY_LINE + "\n", False),
            (
                "[target.'cfg(windows)'.dev-dependencies]\n"
                + 'audio = { package = "kira", version = "0.12.2" }\n',
                False,
            ),
        )
        for source, expected in cases:
            with self.subTest(source=source):
                self.assertEqual(
                    expected,
                    _manifest_has_exact_single_package_pin(
                        source,
                        "kira",
                        KIRA_DEPENDENCY_VERSION,
                    ),
                )

    def test_unreadable_product_subtree_fails_the_manifest_scan_closed(self) -> None:
        unreadable = REPO_ROOT / "zircon_editor"
        original_iterdir = Path.iterdir

        def guarded_iterdir(path: Path):
            if path == unreadable:
                raise OSError("injected unreadable product subtree")
            return original_iterdir(path)

        with patch.object(Path, "iterdir", guarded_iterdir):
            report = tech_stack_boundary_audit(REPO_ROOT)

        self.assertEqual([KIRA_OWNER_MANIFEST], report["kira_dependency_owners"])
        self.assertTrue(report["manifest_scan_errors"])
        self.assertTrue(
            any("zircon_editor" in error for error in report["manifest_scan_errors"])
        )
        self.assertIn(
            "Runtime 01 product manifest scan was incomplete.",
            report["risks"],
        )


if __name__ == "__main__":
    unittest.main()
