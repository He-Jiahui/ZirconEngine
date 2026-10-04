"""asset_importers ResourceKind checks for plugin validation."""

from __future__ import annotations

from typing import Any

Importer = dict[str, Any]
Diagnostics = list[str]

ASSET_IMPORTER_OUTPUT_KINDS = frozenset(
    (
        "Data",
        "Model",
        "Mesh",
        "Material",
        "MaterialGraph",
        "Texture",
        "Shader",
        "Scene",
        "Sound",
        "Font",
        "PhysicsMaterial",
        "NavMesh",
        "NavigationSettings",
        "Terrain",
        "TerrainLayerStack",
        "TileSet",
        "TileMap",
        "Prefab",
        "AnimationSkeleton",
        "AnimationClip",
        "AnimationSequence",
        "AnimationGraph",
        "AnimationStateMachine",
        "UiLayout",
        "UiWidget",
        "UiStyle",
    )
)


def validate_plugin_asset_importer_output_kinds(
    importer: Importer, importer_label: str, diagnostics: Diagnostics
) -> None:
    validate_plugin_asset_importer_known_output_kind(
        importer.get("output_kind"),
        f"{importer_label}.output_kind",
        diagnostics,
    )
    values = importer.get("additional_output_kinds")
    if not isinstance(values, list):
        return
    for index, value in enumerate(values):
        validate_plugin_asset_importer_known_output_kind(
            value,
            f"{importer_label}.additional_output_kinds[{index}]",
            diagnostics,
        )


def validate_plugin_asset_importer_known_output_kind(
    value: Any, label: str, diagnostics: Diagnostics
) -> None:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return
    if value not in ASSET_IMPORTER_OUTPUT_KINDS:
        diagnostics.append(f"{label} must be a known ResourceKind")
