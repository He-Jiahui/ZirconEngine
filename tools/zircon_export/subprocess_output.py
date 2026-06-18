"""Helpers for subprocess output captured by export pipeline stages."""

from __future__ import annotations


def split_subprocess_output(output: str | None) -> list[str]:
    if not output:
        return []
    return output.splitlines()
