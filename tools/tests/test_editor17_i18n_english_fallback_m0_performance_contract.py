from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "zircon_editor" / "src" / "core" / "i18n" / "catalog.rs"
LOCALE = ROOT / "zircon_editor" / "src" / "core" / "i18n" / "locale.rs"


class I18nEnglishFallbackPerformanceContractTests(unittest.TestCase):
    def test_english_fallback_borrows_the_static_locale_tag(self) -> None:
        catalog = CATALOG.read_text(encoding="utf-8")
        locale = LOCALE.read_text(encoding="utf-8")

        self.assertIn('const ENGLISH_TAG: &str = "en";', locale)
        self.assertIn("impl Borrow<str> for EditorLocale", locale)
        self.assertIn("Self(Arc::from(ENGLISH_TAG))", locale)
        self.assertIn(".get(EditorLocale::english_tag())", catalog)
        self.assertNotIn(".get(&EditorLocale::english())", catalog)


if __name__ == "__main__":
    unittest.main()
