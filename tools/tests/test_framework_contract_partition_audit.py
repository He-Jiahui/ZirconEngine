import tempfile
import unittest
from collections import Counter
from pathlib import Path
from unittest import mock

from tools.framework_contract_partition_audit import (
    audit_framework_partition,
    classify_rust_source,
)


class FrameworkContractPartitionAuditTests(unittest.TestCase):
    def test_declaration_only_trait_methods_do_not_count_as_behavior(self) -> None:
        source = """
pub struct Request {
    pub value: u32,
}

pub trait Service {
    fn execute(&self, request: Request) -> Result<(), Error>;
}

pub enum Error {
    Rejected,
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "declaration_only")
        self.assertEqual(result.function_body_count, 0)
        self.assertEqual(result.public_function_body_count, 0)
        self.assertEqual(result.restricted_function_body_count, 0)
        self.assertEqual(result.trait_count, 1)
        self.assertEqual(result.public_trait_count, 1)
        self.assertEqual(result.struct_count, 1)
        self.assertEqual(result.public_struct_count, 1)
        self.assertEqual(result.enum_count, 1)
        self.assertEqual(result.public_enum_count, 1)

    def test_default_trait_and_impl_methods_make_a_declaration_file_mixed(self) -> None:
        source = """
pub struct Descriptor(u32);

pub trait DescriptorExt {
    fn normalized(&self) -> u32 {
        self.0.max(1)
    }
}

impl Descriptor {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) async fn resolve(&self)
    where
        Self: Sync,
    {
    }
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "mixed")
        self.assertEqual(result.function_body_count, 3)
        self.assertEqual(result.public_function_body_count, 1)
        self.assertEqual(result.restricted_function_body_count, 1)

    def test_free_functions_without_contract_declarations_are_behavior_only(self) -> None:
        source = """
fn compile() {}

pub fn evaluate() -> bool {
    true
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.function_body_count, 2)
        self.assertEqual(result.public_function_body_count, 1)

    def test_cfg_test_items_comments_and_literals_do_not_change_classification(self) -> None:
        source = r'''
pub struct Snapshot;

// pub fn commented_out() {}
const DOCUMENTATION: &str = "pub fn string_literal() {}";

#[cfg(test)]
fn direct_test() {}

#[cfg(all(feature = "diagnostics", test))]
mod tests {
    pub fn nested_test() {}
}
'''

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "declaration_only")
        self.assertEqual(result.function_body_count, 0)

    def test_cfg_any_test_item_remains_production_capable(self) -> None:
        source = """
#[cfg(any(test, feature = "diagnostics"))]
pub fn production_capable() {}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.function_body_count, 1)

    def test_function_local_types_do_not_make_a_behavior_file_mixed(self) -> None:
        source = """
pub fn execute() {
    struct Scratch;
    enum LocalState { Ready }
    type LocalId = u32;
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.function_body_count, 1)
        self.assertEqual(result.struct_count, 0)
        self.assertEqual(result.enum_count, 0)
        self.assertEqual(result.type_alias_count, 0)

    def test_const_generic_return_expression_is_not_the_function_body(self) -> None:
        source = """
pub fn construct<const N: usize>() -> Value<{ N + 1 }> {
    struct Scratch;
    Value
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.function_body_count, 1)
        self.assertEqual(result.struct_count, 0)

    def test_macro_definition_templates_do_not_inflate_generated_items(self) -> None:
        source = """
macro_rules! generate_api {
    () => {
        pub struct Generated;
        pub fn generated() {}
    };
}

pub fn execute() {}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.function_body_count, 1)
        self.assertEqual(result.struct_count, 0)
        self.assertEqual(result.macro_definition_count, 1)
        self.assertTrue(result.manual_review_required)

    def test_macro_only_file_requires_review_instead_of_guessing_ownership(self) -> None:
        source = """
macro_rules! define_id {
    ($name:ident) => {
        pub struct $name(u64);
        impl $name {
            pub fn raw(self) -> u64 { self.0 }
        }
    };
}

define_id!(RequestId);
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "macro_generated_review")
        self.assertEqual(result.function_body_count, 0)
        self.assertEqual(result.macro_definition_count, 1)
        self.assertTrue(result.manual_review_required)

    def test_external_item_macro_token_tree_is_masked_and_requires_review(self) -> None:
        source = """
bitflags::bitflags! {
    pub struct Flags: u32 {
        const READY = 1;
    }
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "macro_generated_review")
        self.assertEqual(result.function_body_count, 0)
        self.assertEqual(result.struct_count, 0)
        self.assertEqual(result.macro_invocation_review_count, 1)
        self.assertTrue(result.manual_review_required)

    def test_item_macro_function_tokens_do_not_inflate_real_function_count(self) -> None:
        source = """
make_api! {
    pub fn generated() {}
}

pub fn execute() {}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.function_body_count, 1)
        self.assertEqual(result.macro_invocation_review_count, 1)
        self.assertTrue(result.manual_review_required)

    def test_expression_macro_inside_function_is_not_an_item_review_surface(self) -> None:
        source = """
pub fn execute() {
    make_expression! { pub fn token_only() {} }
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.function_body_count, 1)
        self.assertEqual(result.macro_invocation_review_count, 0)
        self.assertFalse(result.manual_review_required)

    def test_impl_without_function_body_is_a_contract_binding(self) -> None:
        source = """
impl Iterator for Value {
    type Item = u32;
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "contract_binding_only")
        self.assertEqual(result.impl_block_count, 1)
        self.assertEqual(result.type_alias_count, 0)

    def test_associated_constants_keep_a_declared_contract_declaration_only(self) -> None:
        source = """
pub struct ResourceNames;

impl ResourceNames {
    pub const SCENE_COLOR: &'static str = "scene-color";
}
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "declaration_only")
        self.assertEqual(result.impl_block_count, 1)

    def test_static_state_is_implementation_behavior(self) -> None:
        result = classify_rust_source("static NEXT_ID: AtomicU64 = AtomicU64::new(1);")

        self.assertEqual(result.classification, "behavior_only")
        self.assertEqual(result.static_item_count, 1)

    def test_trait_default_and_impl_methods_have_distinct_counts(self) -> None:
        source = """
pub trait Service {
    fn default_value(&self) -> u32 { 1 }
}

impl Service for RuntimeService {
    fn default_value(&self) -> u32 { 2 }
}

pub fn create() -> RuntimeService { RuntimeService }
"""

        result = classify_rust_source(source)

        self.assertEqual(result.classification, "mixed")
        self.assertEqual(result.trait_default_function_body_count, 1)
        self.assertEqual(result.impl_function_body_count, 1)
        self.assertEqual(result.free_function_body_count, 1)

    def test_audit_reads_only_the_framework_module_inventory(self) -> None:
        temp_parent = Path(__file__).resolve().parents[3]
        with tempfile.TemporaryDirectory(dir=temp_parent) as temp_directory:
            repo_root = Path(temp_directory)
            framework_root = repo_root / "zircon_runtime/src/core/framework"
            graphics_root = repo_root / "zircon_runtime/src/graphics"
            framework_root.mkdir(parents=True)
            graphics_root.mkdir(parents=True)
            (framework_root / "mod.rs").write_text(
                "pub mod visible;\n#[cfg(test)]\nmod hidden;\n",
                encoding="utf-8",
            )
            (framework_root / "visible.rs").write_text(
                "pub struct Visible;\n", encoding="utf-8"
            )
            (framework_root / "hidden.rs").write_text(
                "pub fn test_only() {}\n", encoding="utf-8"
            )
            (graphics_root / "unrelated.rs").write_bytes(b"\xff")

            framework_reads: Counter[Path] = Counter()
            original_read_text = Path.read_text

            def tracked_read_text(path: Path, *args, **kwargs) -> str:
                if framework_root in path.parents:
                    framework_reads[path] += 1
                return original_read_text(path, *args, **kwargs)

            with mock.patch.object(Path, "read_text", tracked_read_text):
                report = audit_framework_partition(repo_root)

        self.assertEqual(report["schema_version"], 2)
        self.assertEqual(report["totals"]["file_count"], 2)
        self.assertEqual(
            [
                "zircon_runtime/src/core/framework/mod.rs",
                "zircon_runtime/src/core/framework/visible.rs",
            ],
            [row["path"] for row in report["files"]],
        )
        self.assertEqual(
            {
                framework_root / "hidden.rs": 1,
                framework_root / "mod.rs": 1,
                framework_root / "visible.rs": 1,
            },
            dict(framework_reads),
        )

    def test_test_only_reachability_is_transitive_without_hiding_shared_modules(
        self,
    ) -> None:
        temp_parent = Path(__file__).resolve().parents[3]
        with tempfile.TemporaryDirectory(dir=temp_parent) as temp_directory:
            repo_root = Path(temp_directory)
            framework_root = repo_root / "zircon_runtime/src/core/framework"
            test_support_root = framework_root / "test_support"
            test_support_root.mkdir(parents=True)
            (framework_root / "mod.rs").write_text(
                "pub mod shared;\n#[cfg(test)]\nmod test_support;\n",
                encoding="utf-8",
            )
            (framework_root / "shared.rs").write_text(
                "pub struct Shared;\n", encoding="utf-8"
            )
            (framework_root / "test_support.rs").write_text(
                'pub mod nested;\n#[path = "shared.rs"]\nmod shared_again;\n',
                encoding="utf-8",
            )
            (test_support_root / "nested.rs").write_text(
                "pub fn test_only() {}\n", encoding="utf-8"
            )

            report = audit_framework_partition(repo_root)

        self.assertEqual(
            [
                "zircon_runtime/src/core/framework/mod.rs",
                "zircon_runtime/src/core/framework/shared.rs",
            ],
            [row["path"] for row in report["files"]],
        )


if __name__ == "__main__":
    unittest.main()
