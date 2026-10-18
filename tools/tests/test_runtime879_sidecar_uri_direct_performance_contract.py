"""Runtime879 guards direct sidecar URI construction without running Cargo."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_runtime/src/asset/migration/sidecar.rs"
LOWER = ROOT / "zircon_runtime/src/asset/migration/sidecar/tests/direct_uri_tests.rs"


class Runtime879SidecarUriDirectContract(unittest.TestCase):
    def test_uri_uses_one_result_without_component_vector_or_join(self):
        source = OWNER.read_text(encoding="utf-8")
        function = source.split('fn sidecar_resource_uri(relative: &Path) -> String {', 1)[1].split('\nfn ', 1)[0]
        self.assertIn('String::with_capacity(', function)
        self.assertIn('relative.components().enumerate()', function)
        self.assertIn("output.push('/');", function)
        self.assertIn('component.as_os_str().to_string_lossy()', function)
        self.assertNotIn('collect::<Vec<_>>()', function)
        self.assertNotIn('.join(', function)
        self.assertIn('AssetUri::parse(&sidecar_resource_uri(relative))', source)

    def test_lower_parity_and_release_percentiles_are_wired(self):
        source = OWNER.read_text(encoding="utf-8")
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn('mod direct_uri_tests;', source)
        self.assertIn('RUNTIME879_SIDECAR_URI_DIRECT_BENCH_V1', lower)
        self.assertIn('legacy_uri(', lower)
        self.assertIn('101', lower)
        self.assertIn('p99', lower)

    def test_deterministic_intermediate_model(self):
        records, path_components = 4_096, 32
        self.assertEqual(records * path_components, 131_072)
        self.assertEqual(records * 2, 8_192)


if __name__ == '__main__':
    unittest.main()
