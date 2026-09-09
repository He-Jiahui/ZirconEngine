"""MkDocs hooks for the repository-owned Wiki navigation manifest."""

from __future__ import annotations

import json
import os
from pathlib import Path
import sys
from typing import Any

from mkdocs.structure.files import File

# MkDocs loads hook files by path; make the sibling helper import explicit
# instead of depending on the caller's current Python import path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from wiki_site import load_navigation, mkdocs_navigation


REPO_ROOT = Path(__file__).resolve().parents[1]


def _normalized_url(value: str) -> str:
    return value.rstrip("/") + "/"


def on_config(config: Any, **_: Any) -> Any:
    navigation = load_navigation(REPO_ROOT)
    config["nav"] = mkdocs_navigation(navigation)
    configured_url = os.environ.get("WIKI_SITE_URL", "").strip()
    if configured_url:
        config["site_url"] = _normalized_url(configured_url)
    configured_site_dir = os.environ.get("WIKI_SITE_DIR", "").strip()
    if configured_site_dir:
        config["site_dir"] = configured_site_dir
    extra = config.setdefault("extra", {})
    extra["wiki_navigation_version"] = navigation.version
    source_revision = os.environ.get("WIKI_SOURCE_REVISION", "").strip()
    if source_revision:
        extra["wiki_source_revision"] = source_revision
    return config


def on_files(files: Any, config: Any, **_: Any) -> Any:
    """Publish machine-readable navigation and build metadata at the site root."""

    if files.get_file_from_path("navigation.yaml") is None:
        navigation_path = REPO_ROOT / "docs" / "wiki" / "navigation.yaml"
        files.append(File.generated(config, "navigation.yaml", abs_src_path=str(navigation_path)))
    if files.get_file_from_path("wiki-build-info.json") is None:
        extra = config.get("extra", {})
        build_info = {
            "generator": "ZirconEngine Wiki",
            "navigation_version": extra.get("wiki_navigation_version"),
            "site_url": config.get("site_url", ""),
            "source_revision": os.environ.get("WIKI_SOURCE_REVISION", "").strip() or None,
        }
        files.append(
            File.generated(
                config,
                "wiki-build-info.json",
                content=json.dumps(build_info, ensure_ascii=False, indent=2) + "\n",
            )
        )
    return files
