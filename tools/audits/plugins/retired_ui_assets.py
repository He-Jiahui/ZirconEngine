from __future__ import annotations

import os
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
        for directory, _subdirectories, file_names in os.walk(root_path):
            for file_name in file_names:
                if is_retired_ui_asset_name(file_name):
                    retired_paths.append(
                        Path(directory, file_name).relative_to(repo_root).as_posix()
                    )
    return RetiredUiAssetAudit(retired_ui_asset_file_paths=sorted(retired_paths))


def is_retired_ui_asset_path(path: Path) -> bool:
    return is_retired_ui_asset_name(path.as_posix())


def is_retired_ui_asset_name(file_name: str) -> bool:
    normalized = file_name.lower()
    return any(normalized.endswith(suffix) for suffix in RETIRED_UI_ASSET_SUFFIXES)
