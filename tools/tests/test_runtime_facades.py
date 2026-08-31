import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_API_FACADE = (
    REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_api" / "mod.rs"
)
RUNTIME_BUILD_SET_FACADE = (
    REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_build_set" / "mod.rs"
)
RUNTIME_API_DOMAIN_FACADES = {
    "abi": REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_api" / "abi" / "mod.rs",
    "constants": REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_api" / "constants.rs",
    "frame": REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_api" / "frame" / "mod.rs",
    "host": REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_api" / "host" / "mod.rs",
    "session": REPO_ROOT / "zircon_runtime_interface" / "src" / "runtime_api" / "session" / "mod.rs",
}


def _public_symbol_tokens(path: Path) -> set[str]:
    source = path.read_text(encoding="utf-8")
    symbols = set(re.findall(r"^pub const ([A-Za-z0-9_]+)", source, re.MULTILINE))
    for reexport in re.finditer(
        r"pub use [a-z_:]+::\{(?P<items>.*?)\};", source, re.DOTALL
    ):
        symbols.update(
            re.findall(
                r"\b(?:validate_[A-Za-z0-9_]+|[A-Z][A-Za-z0-9_]+)\b",
                reexport.group("items"),
            )
        )
    return symbols


class RuntimeFacadeTests(unittest.TestCase):
    def test_runtime_api_facade_uses_explicit_domain_exports(self) -> None:
        facade = RUNTIME_API_FACADE.read_text(encoding="utf-8")

        self.assertIsNone(re.search(r"pub use \w+::\*;", facade))
        for domain in ("abi", "constants", "frame", "host", "session"):
            self.assertIn(f"pub use {domain}::{{", facade)

        root_symbols = _public_symbol_tokens(RUNTIME_API_FACADE)
        for domain, domain_facade in RUNTIME_API_DOMAIN_FACADES.items():
            with self.subTest(domain=domain):
                self.assertTrue(_public_symbol_tokens(domain_facade) <= root_symbols)

    def test_runtime_build_set_facade_keeps_the_generated_catalog_explicit(self) -> None:
        facade = RUNTIME_BUILD_SET_FACADE.read_text(encoding="utf-8")

        self.assertNotIn("pub use slot_catalog::*;", facade)
        for symbol in (
            "ZR_RUNTIME_INTERFACE_FAMILY_V1",
            "ZR_RUNTIME_INTERFACE_SPEC_VERSION_V1",
            "ZIRCON_RUNTIME_API_VERSION_V8",
            "ZR_RUNTIME_GET_API_SYMBOL_V8",
            "ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES",
            "ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES",
            "ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES",
        ):
            self.assertRegex(facade, rf"\b{symbol}\b")


if __name__ == "__main__":
    unittest.main()
