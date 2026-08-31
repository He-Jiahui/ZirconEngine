import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZR_MATH_ROOT = REPO_ROOT / "zircon_runtime/crates/zr_math"
INTERFACE_MATH_ROOT = REPO_ROOT / "zircon_runtime_interface/src/math"


def load_toml(relative_path: str) -> dict:
    with (REPO_ROOT / relative_path).open("rb") as stream:
        return tomllib.load(stream)


class Frameworks01MathCrateBoundaryTests(unittest.TestCase):
    def test_workspace_and_product_crates_use_the_canonical_math_owner(self) -> None:
        workspace = load_toml("Cargo.toml")
        runtime = load_toml("zircon_runtime/Cargo.toml")
        runtime_interface = load_toml("zircon_runtime_interface/Cargo.toml")
        math = load_toml("zircon_runtime/crates/zr_math/Cargo.toml")

        self.assertIn("zircon_runtime/crates/zr_math", workspace["workspace"]["members"])
        self.assertEqual(
            {
                "path": "zircon_runtime/crates/zr_math",
                "default-features": False,
            },
            workspace["workspace"]["dependencies"]["zr_math"],
        )
        self.assertEqual({"workspace": True}, runtime["dependencies"]["zr_math"])
        self.assertEqual(
            {"workspace": True}, runtime_interface["dependencies"]["zr_math"]
        )
        self.assertEqual("zr_math", math["package"]["name"])
        self.assertFalse(math["package"]["publish"])
        self.assertEqual(
            {"glam", "serde", "thiserror"}, set(math["dependencies"])
        )
        self.assertNotIn("zircon_runtime", math["dependencies"])
        self.assertNotIn("zircon_runtime_interface", math["dependencies"])

    def test_math_implementation_is_physically_owned_by_zr_math(self) -> None:
        lib = (ZR_MATH_ROOT / "src/lib.rs").read_text(encoding="utf-8")
        for module in (
            "conventions",
            "fallible",
            "numeric_policy",
            "render_conversion",
            "space",
            "transform",
        ):
            self.assertIn(f"mod {module};", lib)

        for retired_name in (
            "fallible.rs",
            "numeric_policy.rs",
            "render_conversion.rs",
            "space.rs",
            "transform.rs",
        ):
            self.assertFalse((INTERFACE_MATH_ROOT / retired_name).exists())

        math_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted((ZR_MATH_ROOT / "src").rglob("*.rs"))
        )
        self.assertNotIn("SchemaId", math_sources)
        self.assertNotIn("zircon_runtime_interface", math_sources)
        self.assertNotIn("zircon_runtime::", math_sources)

    def test_interface_owns_only_versioned_math_schema_dtos_and_projection(self) -> None:
        interface_root = (
            REPO_ROOT / "zircon_runtime_interface/src/math.rs"
        ).read_text(encoding="utf-8")
        schema = (INTERFACE_MATH_ROOT / "schema.rs").read_text(encoding="utf-8")

        self.assertIn("pub use zr_math::{", interface_root)
        self.assertNotIn("pub use zr_math::*;", interface_root)
        self.assertIn("mod schema;", interface_root)
        self.assertNotIn("mod fallible;", interface_root)
        self.assertIn("use crate::serialization::SchemaId;", schema)
        self.assertIn("use zr_math::{", schema)
        for primitive in (
            "Axis3",
            "DepthDirection",
            "SpaceKind",
            "ScalarPrecision",
        ):
            self.assertNotIn(f"pub enum {primitive}", schema)
        for dto in ("CoordinateSchema", "UnitSchema", "PrecisionProfile"):
            self.assertIn(f"pub struct {dto}", schema)

    def test_runtime_math_surface_is_a_curated_product_projection(self) -> None:
        runtime_math = (
            REPO_ROOT / "zircon_runtime/src/core/math/mod.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub use zr_math::{", runtime_math)
        self.assertNotIn("pub use zr_math::*;", runtime_math)
        self.assertIn("zircon_runtime_interface::math::{", runtime_math)
        self.assertNotIn("pub use zircon_runtime_interface::math::*;", runtime_math)


if __name__ == "__main__":
    unittest.main()
