import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
UI_ASSETS = REPO_ROOT / "zircon_runtime/src/asset/assets/ui.rs"
IMPORTED_ASSET = REPO_ROOT / "zircon_runtime/src/asset/assets/imported.rs"
DEPENDENCY_EXTRACTORS = (
    REPO_ROOT / "zircon_runtime/src/asset/registry/dependency_extractors/mod.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/asset/tests/assets/ui/references.rs"
)


class Runtime863UiV2DirectReferenceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ui_assets = UI_ASSETS.read_text(encoding="utf-8")
        cls.imported_asset = IMPORTED_ASSET.read_text(encoding="utf-8")
        cls.dependency_extractors = DEPENDENCY_EXTRACTORS.read_text(encoding="utf-8")
        cls.lower_regression = LOWER_REGRESSION.read_text(encoding="utf-8")

    def test_all_typed_ui_v2_assets_delegate_to_the_shared_collector(self) -> None:
        asset_types = ("UiV2ViewAsset", "UiV2ComponentAsset", "UiV2StyleAsset")
        for index, asset_type in enumerate(asset_types):
            start = self.ui_assets.index(f"impl {asset_type} {{")
            next_type = asset_types[index + 1] if index + 1 < len(asset_types) else None
            end = (
                self.ui_assets.index(f"impl {next_type} {{", start)
                if next_type is not None
                else self.ui_assets.index("pub fn ui_asset_references", start)
            )
            implementation = self.ui_assets[start:end]
            self.assertIn("pub fn direct_references(&self) -> Vec<AssetReference>", implementation)
            self.assertIn("ui_v2_asset_references(&self.document)", implementation)

    def test_imported_asset_uses_the_typed_reference_contract(self) -> None:
        for variant in ("UiV2View", "UiV2Component", "UiV2Style"):
            self.assertIn(
                f"Self::{variant}(asset) => asset.direct_references()",
                self.imported_asset,
            )
        self.assertNotIn("ui_v2_asset_references,", self.imported_asset)

    def test_registry_dependency_extractors_use_the_same_typed_contract(self) -> None:
        for variant in ("UiV2View", "UiV2Component", "UiV2Style"):
            self.assertIn(
                f"ImportedAsset::{variant}(asset) => asset.direct_references()",
                self.dependency_extractors,
            )

    def test_lower_regression_covers_all_three_typed_wrappers(self) -> None:
        self.assertIn(
            "ui_v2_typed_assets_delegate_direct_references_to_the_shared_collector",
            self.lower_regression,
        )
        for variable in ("view", "component", "style"):
            self.assertIn(f"reference_locators({variable}.direct_references())", self.lower_regression)


if __name__ == "__main__":
    unittest.main()
