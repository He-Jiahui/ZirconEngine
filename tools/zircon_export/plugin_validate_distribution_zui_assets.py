"""distribution.assets .zui document checks for plugin validation."""

from __future__ import annotations

from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]


PLUGIN_VALIDATE_ZUI_ASSET_KINDS = ("component", "style", "theme_tokens", "view")


def validate_plugin_distribution_zui_asset(
    *,
    item_label: str,
    relative_source: Path,
    source_path: Path,
    diagnostics: list[str],
) -> None:
    if relative_source.suffix != ".zui":
        return
    relative_asset = relative_source.as_posix()
    try:
        document = tomllib.loads(source_path.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(
            f"{item_label} matched .zui asset {relative_asset} could not be read: {error}"
        )
        return
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            f"{item_label} matched .zui asset {relative_asset} could not be "
            f"parsed as TOML: {error}"
        )
        return

    asset = document.get("asset")
    asset_kind = asset.get("kind") if isinstance(asset, dict) else None
    if not isinstance(asset_kind, str) or not asset_kind:
        diagnostics.append(
            f"{item_label} matched .zui asset {relative_asset} must declare asset.kind"
        )
        return
    if asset_kind not in PLUGIN_VALIDATE_ZUI_ASSET_KINDS:
        diagnostics.append(
            f"{item_label} matched .zui asset {relative_asset} has unsupported "
            f"asset.kind {asset_kind}; expected one of "
            f"{', '.join(PLUGIN_VALIDATE_ZUI_ASSET_KINDS)}"
        )
