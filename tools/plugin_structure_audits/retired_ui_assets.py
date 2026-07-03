from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any


RETIRED_UI_ASSET_SCAN_ROOTS = ("zircon_editor", "zircon_plugins", "zircon_runtime")
RETIRED_UI_ASSET_SUFFIXES = (".v2.ui.toml", ".ui.toml")


@dataclass(frozen=True)
class RetiredUiAssetAudit:
    retired_ui_asset_file_paths: list[str]

    def to_json(self) -> dict[str, Any]:
        retired_ui_asset_files = len(self.retired_ui_asset_file_paths)
        return {
            "retired_ui_asset_files": retired_ui_asset_files,
            "retired_ui_asset_file_paths": self.retired_ui_asset_file_paths,
            "zui_only_layout_status": (
                "zui-only-clean"
                if retired_ui_asset_files == 0
                else "retired-ui-assets-present"
            ),
        }


def audit_retired_ui_asset_conformance(repo_root: Path) -> RetiredUiAssetAudit:
    retired_paths: list[str] = []
    for scan_root in RETIRED_UI_ASSET_SCAN_ROOTS:
        root_path = repo_root / scan_root
        if not root_path.exists():
            continue
        retired_paths.extend(
            path.relative_to(repo_root).as_posix()
            for path in root_path.rglob("*")
            if path.is_file() and is_retired_ui_asset_path(path)
        )
    return RetiredUiAssetAudit(retired_ui_asset_file_paths=sorted(retired_paths))


def is_retired_ui_asset_path(path: Path) -> bool:
    normalized = path.as_posix().lower()
    return any(normalized.endswith(suffix) for suffix in RETIRED_UI_ASSET_SUFFIXES)
