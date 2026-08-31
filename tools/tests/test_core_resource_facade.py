import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
INTERFACE_RESOURCE_FACADE = (
    REPO_ROOT / "zircon_runtime_interface" / "src" / "resource" / "mod.rs"
)
RUNTIME_RESOURCE_FACADE = (
    REPO_ROOT / "zircon_runtime" / "src" / "core" / "resource" / "mod.rs"
)


def _public_reexport_symbols(path: Path) -> set[str]:
    source = path.read_text(encoding="utf-8")
    symbols = set(
        re.findall(r"^pub use [A-Za-z_:]+::([A-Z][A-Za-z0-9_]+);", source, re.MULTILINE)
    )
    for reexport in re.finditer(
        r"pub use [A-Za-z_:]+::\{(?P<items>.*?)\};", source, re.DOTALL
    ):
        symbols.update(
            re.findall(r"\b[A-Z][A-Za-z0-9_]+\b", reexport.group("items"))
        )
    return symbols


class CoreResourceFacadeTests(unittest.TestCase):
    def test_resource_facade_explicitly_projects_the_interface_contract(self) -> None:
        facade = RUNTIME_RESOURCE_FACADE.read_text(encoding="utf-8")

        self.assertNotIn("pub use zircon_runtime_interface::resource::*;", facade)
        self.assertEqual(
            _public_reexport_symbols(INTERFACE_RESOURCE_FACADE),
            _public_reexport_symbols(RUNTIME_RESOURCE_FACADE)
            & _public_reexport_symbols(INTERFACE_RESOURCE_FACADE),
        )


if __name__ == "__main__":
    unittest.main()
