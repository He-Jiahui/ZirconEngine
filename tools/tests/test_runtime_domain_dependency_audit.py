from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.runtime_domain_dependency_audit import (
    _rust_code_view,
    _rust_use_paths,
    audit_runtime_domain_dependencies,
)


class RuntimeDomainDependencyAuditTests(unittest.TestCase):
    def test_lexes_nested_use_tree_paths_and_aliases(self) -> None:
        source = (
            "use crate::graphics as gfx;\n"
            "use gfx::{text as gt};\n"
            "use crate::graphics::{nested::{A}, text as nested_text};\n"
            "use {wgpu as gpu};\n"
        )

        paths = [
            (path, alias)
            for path, alias, _line in _rust_use_paths(_rust_code_view(source))
        ]

        self.assertEqual(
            paths,
            [
                (("crate", "graphics"), "gfx"),
                (("gfx", "text"), "gt"),
                (("crate", "graphics", "nested", "A"), None),
                (("crate", "graphics", "text"), "nested_text"),
                (("wgpu",), "gpu"),
            ],
        )

    def test_canonicalizes_raw_identifiers_in_paths_and_use_trees(self) -> None:
        source = (
            "use crate::r#graphics::{r#text as raw_text};\n"
            "use crate::r#ui as raw_ui;\n"
        )

        paths = [
            (path, alias)
            for path, alias, _line in _rust_use_paths(_rust_code_view(source))
        ]

        self.assertEqual(
            [
                (("crate", "graphics", "text"), "raw_text"),
                (("crate", "ui"), "raw_ui"),
            ],
            paths,
        )

        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "builtin").mkdir(parents=True)
            (source_root / "builtin" / "raw_domains.rs").write_text(
                "use crate::{r#graphics::Renderer, r#ui as runtime_ui};\n"
                "fn scene() { let _ = crate::r#scene::SceneHandle::default(); }\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(
                ["graphics", "scene", "ui"],
                sorted(row["target_domain"] for row in report["matrix"]),
            )

    def test_reports_shared_text_domain_dependencies(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "text").mkdir(parents=True)
            (source_root / "text" / "mod.rs").write_text(
                "use crate::{graphics::RenderContext, ui::UiTree};\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "text",
                        "target_domain": "graphics",
                        "reference_count": 1,
                    },
                    {
                        "source_domain": "text",
                        "target_domain": "ui",
                        "reference_count": 1,
                    },
                ],
            )

    def test_reports_unique_production_cross_domain_references(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "graphics").mkdir(parents=True)
            (source_root / "ui" / "tests").mkdir(parents=True)
            (source_root / "graphics" / "render.rs").write_text(
                "use crate::ui::TextLayout;\n"
                "fn draw() { let _ = crate::scene::SceneHandle::default(); }\n",
                encoding="utf-8",
            )
            (source_root / "graphics" / "self_ref.rs").write_text(
                "use crate::graphics::Renderer;\n",
                encoding="utf-8",
            )
            (source_root / "ui" / "tests" / "ignored.rs").write_text(
                "use crate::graphics::Renderer;\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 2)
            self.assertEqual(report["domain_edge_count"], 2)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "graphics",
                        "target_domain": "scene",
                        "reference_count": 1,
                    },
                    {
                        "source_domain": "graphics",
                        "target_domain": "ui",
                        "reference_count": 1,
                    },
                ],
            )

    def test_reports_domains_inside_grouped_crate_use_statements(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "asset").mkdir(parents=True)
            (source_root / "asset" / "project.rs").write_text(
                "use crate::{\n"
                "    core::resource::ResourceLocator,\n"
                "    plugin::{ExportProfile, ProjectPluginManifest},\n"
                "    plugin::RuntimeProfileId,\n"
                "};\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 3)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "asset",
                        "target_domain": "core",
                        "reference_count": 1,
                    },
                    {
                        "source_domain": "asset",
                        "target_domain": "plugin",
                        "reference_count": 2,
                    },
                ],
            )

    def test_reports_bare_and_grouped_root_domain_imports(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "builtin").mkdir(parents=True)
            (source_root / "builtin" / "modules.rs").write_text(
                "use crate::graphics;\n"
                "use crate::{asset, foundation, input, platform, scene};\n"
                "use crate::{\n"
                "    core::{asset::AssetId, scene::World},\n"
                "    ui as runtime_ui,\n"
                "};\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 8)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "builtin",
                        "target_domain": target_domain,
                        "reference_count": 1,
                    }
                    for target_domain in (
                        "asset",
                        "core",
                        "foundation",
                        "graphics",
                        "input",
                        "platform",
                        "scene",
                        "ui",
                    )
                ],
            )

    def test_ignores_domain_paths_inside_comments_and_literals(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "graphics").mkdir(parents=True)
            (source_root / "graphics" / "render.rs").write_text(
                "use crate::asset::AssetId;\n"
                "// crate::scene::World\n"
                'const TEXT: &str = "crate::ui::UiTree";\n'
                'const RAW: &str = r#"crate::plugin::Registry"#;\n'
                "/* crate::builtin::BuiltinModule */\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 1)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "graphics",
                        "target_domain": "asset",
                        "reference_count": 1,
                    }
                ],
            )

    def test_preserves_domain_paths_between_rust_lifetimes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "graphics").mkdir(parents=True)
            (source_root / "graphics" / "borrowed.rs").write_text(
                "type Foreign<'a> = crate::ui::UiTree<'a>;\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 1)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "graphics",
                        "target_domain": "ui",
                        "reference_count": 1,
                    }
                ],
            )

    def test_ignores_root_files_and_test_owners(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "ui").mkdir(parents=True)
            (source_root / "lib.rs").write_text(
                "pub use crate::graphics::Renderer;\n", encoding="utf-8"
            )
            (source_root / "ui" / "tests.rs").write_text(
                "use crate::graphics::Renderer;\n", encoding="utf-8"
            )
            (source_root / "ui" / "layout_tests.rs").write_text(
                "use crate::graphics::Renderer;\n", encoding="utf-8"
            )
            (source_root / "ui" / "test_layout.rs").write_text(
                "use crate::graphics::Renderer;\n", encoding="utf-8"
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 0)
            self.assertEqual(report["matrix"], [])

    def test_ignores_inline_cfg_test_items_without_hiding_production_items(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "graphics").mkdir(parents=True)
            (source_root / "graphics" / "render.rs").write_text(
                "use crate::asset::AssetId;\n"
                '#[cfg(all(test, feature = "graphics"))]\n'
                "mod tests {\n"
                "    use crate::scene::SceneHandle;\n"
                "    fn nested() {\n"
                "        let _ = crate::ui::UiTree::default();\n"
                "    }\n"
                "}\n"
                '#[cfg(all(feature = "graphics", test))]\n'
                "mod reverse_tests {\n"
                "    use crate::ui::ReverseUiTree;\n"
                "}\n"
                '#[cfg(all(test, any(feature = "x", feature = "y")))]\n'
                "mod nested_tests {\n"
                "    use crate::ui::NestedUiTree;\n"
                "}\n"
                '#[cfg(any(test, feature = "graphics"))]\n'
                '#[cfg(not(feature = "graphics"))]\n'
                "mod joint_tests {\n"
                "    use crate::ui::JointUiTree;\n"
                "}\n"
                "use crate::render_graph::RenderGraph;\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 2)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "graphics",
                        "target_domain": "asset",
                        "reference_count": 1,
                    },
                    {
                        "source_domain": "graphics",
                        "target_domain": "render_graph",
                        "reference_count": 1,
                    },
                ],
            )

    def test_ignores_support_files_reachable_only_from_cfg_test_modules(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            owner_root = source_root / "graphics" / "registry"
            owner_root.mkdir(parents=True)
            (source_root / "graphics" / "registry.rs").write_text(
                "use crate::asset::AssetId;\n"
                "#[cfg(test)]\n"
                "mod tests;\n",
                encoding="utf-8",
            )
            (owner_root / "tests.rs").write_text(
                '#[path = "support.rs"]\n'
                "mod support;\n",
                encoding="utf-8",
            )
            (owner_root / "support.rs").write_text(
                "use crate::scene::world::World;\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 1)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "graphics",
                        "target_domain": "asset",
                        "reference_count": 1,
                    }
                ],
            )


if __name__ == "__main__":
    unittest.main()
