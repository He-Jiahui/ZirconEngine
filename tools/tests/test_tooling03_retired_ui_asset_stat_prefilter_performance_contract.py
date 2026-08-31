import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.plugin_validate_retired_ui_assets import (
    _plugin_validate_retired_ui_asset_files,
)


class Tooling03RetiredUiAssetStatPrefilterPerformanceContractTests(
    unittest.TestCase
):
    def test_only_retired_suffix_candidates_require_file_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            assets = root / "assets"
            assets.mkdir()
            for index in range(2048):
                (assets / f"current_{index}.zui").write_text("(panel)", encoding="utf-8")
            retired = (
                assets / "legacy.ui.toml",
                assets / "legacy.v2.ui.toml",
            )
            for path in retired:
                path.write_text('[ui]\nkind = "panel"\n', encoding="utf-8")

            metadata_paths: list[Path] = []
            projected_paths: list[Path] = []
            original_is_file = Path.is_file
            original_relative_to = Path.relative_to

            def is_file(path: Path) -> bool:
                metadata_paths.append(path)
                return original_is_file(path)

            def relative_to(path: Path, *other: object, **kwargs: object) -> Path:
                if path.parent == assets:
                    projected_paths.append(path)
                return original_relative_to(path, *other, **kwargs)

            with (
                mock.patch.object(Path, "is_file", is_file),
                mock.patch.object(Path, "relative_to", relative_to),
            ):
                actual = _plugin_validate_retired_ui_asset_files(
                    root,
                    roots=(assets,),
                )

            self.assertEqual(
                [Path("assets/legacy.ui.toml"), Path("assets/legacy.v2.ui.toml")],
                actual,
            )
            self.assertEqual(set(retired), set(metadata_paths))
            self.assertEqual(2, len(metadata_paths))
            self.assertEqual(set(retired), set(projected_paths))
            self.assertEqual(2, len(projected_paths))


if __name__ == "__main__":
    unittest.main()
