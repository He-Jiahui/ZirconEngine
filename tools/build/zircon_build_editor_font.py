#!/usr/bin/env python3
"""Build the Editor TTC from recorded sources; --check rebuilds without writes.

Requires fonttools==4.64.0. sources.json pins source hashes and provenance.
Google Fonts entries retain their revision, OFL license and source paths;
local Unreal Engine font entries carry separate source and license references.
Use the manifest license_scope for each source; local verification does not
assert redistribution permission. Only actual font faces are packaged.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import tomllib

import fontTools
from fontTools.ttLib import TTCollection, TTFont
from fontTools.varLib.instancer import instantiateVariableFont


FONT_ROOT = Path(__file__).resolve().parents[2] / "zircon_runtime/assets/fonts"
SOURCE_ROOT = FONT_ROOT / "editor-ui-sources"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def build() -> tuple[bytes, list[dict]]:
    manifest = json.loads((SOURCE_ROOT / "sources.json").read_text(encoding="utf-8"))
    require(fontTools.__version__ == manifest["fonttools_version"],
            f"Install fonttools=={manifest['fonttools_version']} for reproducible output")
    asset = tomllib.loads((FONT_ROOT / "editor-ui.font.toml").read_text(encoding="utf-8"))
    faces = []
    report = []
    for entry in manifest["files"]:
        path = SOURCE_ROOT / entry["file"]
        source_bytes = path.read_bytes()
        require(sha256(source_bytes) == entry["sha256"], f"Source hash mismatch: {path}")
        if "family" not in entry:
            continue
        font = TTFont(io.BytesIO(source_bytes), recalcTimestamp=False)
        original_cmap = font.getBestCmap()
        if "instance" in entry:
            font = instantiateVariableFont(font, entry["instance"], inplace=True,
                                           updateFontNames=True)
            require(font.getBestCmap() == original_cmap, "CJK instance lost source glyph coverage")
            require("fvar" not in font, "CJK instance must be a static real weight")
            require(len(original_cmap) >= 30000, "Full Noto Sans SC source is required")
            for codepoint in (0x4E2D, 0x6587, 0x7F16, 0x8F91, 0x5668, 0xFF0C, 0x3002):
                require(codepoint in original_cmap, f"Missing CJK glyph U+{codepoint:04X}")
        family = font["name"].getBestFamilyName()
        weight = font["OS/2"].usWeightClass
        require(family == entry["family"] and weight == entry["weight"],
                f"Unexpected face identity: {family} {weight}")
        member = asset["family_members"][len(faces)]
        require(member == {"family": family, "face_index": len(faces), "weight": weight,
                           "width_class": font["OS/2"].usWidthClass, "style": "normal"},
                f"Asset descriptor disagrees with actual face: {entry['file']}")
        require(not font["head"].macStyle & 2, f"Unexpected italic face: {entry['file']}")
        report.append({"face_index": len(faces), "family": family,
                       "subfamily": font["name"].getBestSubFamilyName(), "weight": weight,
                       "postscript_name": font["name"].getDebugName(6),
                       "mapped_codepoints": len(original_cmap), "source": entry["file"],
                       "source_sha256": sha256(source_bytes)})
        faces.append(font)
    require(len(faces) == len(asset["family_members"]), "Asset face count mismatch")
    collection = TTCollection()
    collection.fonts = faces
    buffer = io.BytesIO()
    collection.save(buffer, shareTables=True)
    payload = buffer.getvalue()
    decoded = TTCollection(io.BytesIO(payload))
    for original, restored in zip(faces, decoded.fonts, strict=True):
        require(original.getBestCmap() == restored.getBestCmap(), "TTC lost glyph coverage")
    collection.close()
    decoded.close()
    return payload, report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Compare a fresh build with editor-ui.ttc")
    args = parser.parse_args()
    payload, report = build()
    output = FONT_ROOT / "editor-ui.ttc"
    if args.check:
        require(output.read_bytes() == payload, "editor-ui.ttc differs from its deterministic rebuild")
    else:
        output.write_bytes(payload)
    print(json.dumps({"output": str(output), "check": args.check, "bytes": len(payload),
                      "sha256": sha256(payload), "faces": report}, indent=2))


if __name__ == "__main__":
    main()
