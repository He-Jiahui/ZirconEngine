from __future__ import annotations

import hashlib

from tools.zircon_export.export_template_manifest import compute_template_content_hash


def _template_manifest_text(extra: str = "", **overrides: str) -> str:
    template_file_hash = hashlib.sha256(b"<plist>zircon</plist>").hexdigest()
    content_hash = compute_template_content_hash(
        [
            {
                "path": "Info.plist",
                "bundle_path": "Contents/Info.plist",
                "sha256": template_file_hash,
            }
        ]
    )
    fields = {
        "template_id": "fixture-template",
        "engine_version": "0.1.0",
        "target_platform": "windows-x86_64",
        "host_kind": "desktop",
        "host_artifact": "precompiled",
        "resource_strategy": "filesystem_bundle",
        "plugin_strategy": "native_dynamic_allowed",
        "bundle_format": "directory",
        "content_hash": content_hash,
    }
    fields.update(overrides)
    lines = ["format_version = 1\n"]
    lines.extend(f'{field} = "{value}"\n' for field, value in fields.items())
    if "compatible_profiles" not in extra:
        lines.append('compatible_profiles = ["windows-release"]\n')
    if extra:
        lines.append(extra)
    if "[paths]" not in extra:
        lines.extend(
            [
                "\n[paths]\n",
                'host_executable = "Info.plist"\n',
            ]
        )
    if "[[files]]" not in extra:
        lines.extend(
            [
                "\n[[files]]\n",
                'path = "Info.plist"\n',
                'bundle_path = "Contents/Info.plist"\n',
                'purpose = "platform_metadata"\n',
                f'sha256 = "{template_file_hash}"\n',
            ]
        )
    return "".join(lines)
