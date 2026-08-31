import unittest
from pathlib import Path


class RuntimeScriptArgumentViewsOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner = (
            self.repo_root
            / "zircon_runtime/src/core/framework/script/argument_views.rs"
        )
        self.owner_dir = self.owner.with_suffix("")

    def test_argument_views_use_focused_folder_backed_owners(self) -> None:
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 24)
        for declaration in (
            '#[path = "argument_views/argument_source.rs"]\nmod argument_source;',
            '#[path = "argument_views/byte_view.rs"]\nmod byte_view;',
            '#[path = "argument_views/typed_conversion.rs"]\nmod typed_conversion;',
            '#[path = "argument_views/value_ref.rs"]\nmod value_ref;',
            '#[cfg(test)]\n#[path = "argument_views/tests.rs"]\nmod tests;',
        ):
            self.assertIn(declaration, owner_source)

        expected_children = {
            "argument_source.rs": (
                "pub trait ScriptHostArgumentSource",
                "pub struct ScriptHostArguments",
                "pub(crate) struct ScriptHostOwnedArgumentSource",
            ),
            "byte_view.rs": (
                "pub trait ScriptHostByteSource",
                "pub enum ScriptHostByteView",
            ),
            "typed_conversion.rs": ("pub trait ScriptHostFromArgument",),
            "value_ref.rs": ("pub enum ScriptHostValueRef",),
            "tests.rs": (
                "owned_argument_source_lends_text_and_bytes_without_generic_transport_clones",
                "explicit_owned_argument_conversions_record_only_their_business_boundary_copies",
            ),
        }
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            for anchor in anchors:
                self.assertIn(anchor, child_source)

        for forbidden in (
            "pub trait ScriptHostByteSource",
            "pub enum ScriptHostByteView",
            "pub enum ScriptHostValueRef",
            "pub trait ScriptHostArgumentSource",
            "pub struct ScriptHostArguments",
            "pub trait ScriptHostFromArgument",
        ):
            self.assertNotIn(forbidden, owner_source)


if __name__ == "__main__":
    unittest.main()
