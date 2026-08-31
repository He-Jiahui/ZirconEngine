import os
import re
import unittest
from pathlib import Path

from tools.runtime_domain_dependency_audit import _rust_code_view


REPO_ROOT = Path(__file__).resolve().parents[2]
CONTRACT_POLICY = (
    REPO_ROOT / "zircon_runtime/src/core/framework/time/product_policy.rs"
)
CONTRACT_MOD = REPO_ROOT / "zircon_runtime/src/core/framework/time/mod.rs"
CONTRACT_CLOCK = REPO_ROOT / "zircon_runtime/src/core/framework/time/clock.rs"
CONTRACT_FIXED = REPO_ROOT / "zircon_runtime/src/core/framework/time/fixed.rs"
KERNEL_POLICY = REPO_ROOT / "zircon_runtime/src/core/runtime/time/product_policy.rs"
KERNEL_MOD = REPO_ROOT / "zircon_runtime/src/core/runtime/time.rs"
CORE_FACADE = REPO_ROOT / "zircon_runtime/src/core/mod.rs"
PRELUDE = REPO_ROOT / "zircon_runtime/src/prelude.rs"
PRODUCT_SOURCE_ROOTS = (
    REPO_ROOT / "zircon_runtime/src",
)
OLD_INHERENT_BEHAVIOR = re.compile(
    r"\bProductTimePolicy\s*::\s*(?:client|server|editor|test)\b"
)
CFG_TEST = re.compile(r"(?m)^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
EXCLUDED_DIRECTORY_NAMES = {
    ".git",
    "build",
    "cache",
    "generated",
    "node_modules",
    "target",
    "tests",
    "vendor",
}


def production_prefix(source: str) -> str:
    marker = CFG_TEST.search(source)
    return source if marker is None else source[: marker.start()]


def product_rust_files() -> list[Path]:
    result = []
    for root in PRODUCT_SOURCE_ROOTS:
        for directory, child_directories, file_names in os.walk(root):
            directory_path = Path(directory)
            child_directories[:] = [
                name
                for name in child_directories
                if name not in EXCLUDED_DIRECTORY_NAMES
                and not (directory_path / name).is_symlink()
            ]
            for file_name in file_names:
                if file_name.endswith(".rs"):
                    result.append(directory_path / file_name)
    return result


class Frameworks01TimeProductPolicyOwnerBoundaryTests(unittest.TestCase):
    def test_production_clock_contract_exposes_observation_not_mutation_authority(self) -> None:
        clock_source = CONTRACT_CLOCK.read_text(encoding="utf-8")
        fixed_source = CONTRACT_FIXED.read_text(encoding="utf-8")
        production_clock = _rust_code_view(production_prefix(clock_source))
        production_fixed = _rust_code_view(production_prefix(fixed_source))
        public_mutation = re.compile(
            r"(?m)^\s*pub\s+fn\s+"
            r"(?:context_mut|advance_by|advance_from_real_delta|accumulate_overstep|"
            r"pause|unpause|drain_steps)\b"
        )

        self.assertEqual([], public_mutation.findall(production_clock))
        self.assertEqual([], public_mutation.findall(production_fixed))
        self.assertRegex(
            clock_source,
            r"(?s)#\[cfg\(test\)\]\s+pub\(crate\)\s+fn\s+drain_steps\b",
        )

    def test_product_sources_do_not_use_batch_fixed_step_drain(self) -> None:
        offenders = []
        for path in product_rust_files():
            source = production_prefix(path.read_text(encoding="utf-8"))
            if ".drain_steps(" in _rust_code_view(source):
                offenders.append(path.relative_to(REPO_ROOT).as_posix())

        self.assertEqual([], offenders)

    def test_contract_owner_contains_only_product_time_policy_dto_behavior(self) -> None:
        source = _rust_code_view(CONTRACT_POLICY.read_text(encoding="utf-8"))
        forbidden = (
            "blake3",
            "ProductTimePolicyDigest",
            "ProductTimePolicies",
            "normal_time_policy",
            "canonical_f64_bits",
            "fn digest",
            "fn client",
            "fn server",
            "fn editor",
            "fn test",
        )

        self.assertEqual([], [token for token in forbidden if token in source])

    def test_kernel_owner_is_the_only_product_preset_and_digest_implementation(self) -> None:
        self.assertTrue(KERNEL_POLICY.is_file())
        policy = _rust_code_view(KERNEL_POLICY.read_text(encoding="utf-8"))
        for token in (
            "pub struct ProductTimePolicies",
            "pub struct ProductTimePolicyDigest",
            "pub fn for_profile",
            "blake3::Hasher",
        ):
            self.assertIn(token, policy)

        kernel_mod = _rust_code_view(KERNEL_MOD.read_text(encoding="utf-8"))
        self.assertRegex(kernel_mod, r"(?m)^\s*mod\s+product_policy\s*;")
        self.assertRegex(
            kernel_mod,
            r"(?s)pub\s+use\s+product_policy\s*::\s*\{[^}]*"
            r"ProductTimePolicies[^}]*ProductTimePolicyDigest[^}]*\}\s*;",
        )

    def test_public_facades_export_contract_and_kernel_time_policy_owners_separately(self) -> None:
        contract_mod = _rust_code_view(CONTRACT_MOD.read_text(encoding="utf-8"))
        self.assertNotIn("ProductTimePolicyDigest", contract_mod)
        self.assertNotIn("ProductTimePolicies", contract_mod)
        for token in (
            "ProductTimePolicy",
            "ProductTimePolicyError",
            "ProductTimeProfile",
        ):
            self.assertIn(token, contract_mod)

        facade = _rust_code_view(CORE_FACADE.read_text(encoding="utf-8"))
        prelude = _rust_code_view(PRELUDE.read_text(encoding="utf-8"))
        for source in (facade, prelude):
            self.assertIn("ProductTimePolicy", source)
            self.assertIn("ProductTimePolicyError", source)
            self.assertIn("ProductTimeProfile", source)
            self.assertIn("ProductTimePolicies", source)
            self.assertIn("ProductTimePolicyDigest", source)

    def test_product_sources_do_not_call_deleted_inherent_policy_behavior(self) -> None:
        violations = []
        for path in product_rust_files():
            source = production_prefix(path.read_text(encoding="utf-8"))
            code_view = _rust_code_view(source)
            for match in OLD_INHERENT_BEHAVIOR.finditer(code_view):
                line = code_view.count("\n", 0, match.start()) + 1
                violations.append(f"{path.relative_to(REPO_ROOT).as_posix()}:{line}")

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
