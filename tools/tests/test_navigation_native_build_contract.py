from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BUILD_RS = ROOT / "zircon_plugins/navigation/native/build.rs"


class NavigationNativeBuildContractTests(unittest.TestCase):
    def test_msvc_build_forces_cxx17_for_std_clamp(self) -> None:
        source = BUILD_RS.read_text(encoding="utf-8")

        self.assertIn('if cfg!(target_env = "msvc")', source)
        self.assertIn('build.flag("/std:c++17");', source)
        self.assertIn('build.flag("-std=c++17");', source)


if __name__ == "__main__":
    unittest.main()
