from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features/manifest.rs"
)

# 读取实现源码约束精确粒子特性标识符：标识符连接预分配借用部分，并特性路径使用精确连接不带格式器增长。
class ExactParticlesFeatureIdentifiersPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_identifier_join_preallocates_borrowed_parts(self) -> None:
        self.assertIn("parts.iter().map(|part| part.len()).sum()", self.source)
        self.assertIn("String::with_capacity(capacity)", self.source)
        self.assertIn("joined.push_str(part)", self.source)

    def test_feature_path_uses_exact_join_without_formatter_growth(self) -> None:
        self.assertIn(
            'join_string_parts(&["particles.", row.id_suffix])',
            self.source,
        )
        self.assertNotIn("format!(", self.production)

    def test_rust_guard_preserves_longest_production_identifier(self) -> None:
        self.assertIn(
            "exact_particles_identifier_join_preserves_feature_id",
            self.source,
        )
        self.assertIn('"particles.animation_control"', self.source)


if __name__ == "__main__":
    unittest.main()
