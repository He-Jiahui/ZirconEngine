from __future__ import annotations

import mimetypes
import re
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from urllib.parse import unquote, urlsplit

from ..models import CoordinatorError


_HASHED_ASSET = re.compile(r"(?:^|[-.])[0-9A-Za-z_-]{8,}(?:[.-]|$)")


@dataclass(frozen=True, slots=True)
class BinaryResponse:
    status: int
    body: bytes
    headers: dict[str, str]


class StaticAssetService:
    """Resolves only production console assets and never exposes a directory."""

    def __init__(self, dist_root: Path):
        self.dist_root = dist_root.resolve()

    def resolve(self, raw_path: str) -> BinaryResponse | None:
        path = unquote(urlsplit(raw_path).path)
        if path.startswith("/ui/bootstrap/"):
            return None
        if not path.startswith("/ui/"):
            raise CoordinatorError("not_found", "Unknown UI asset")
        relative_text = path.removeprefix("/ui/")
        relative = PurePosixPath(relative_text)
        if any(part in {"", ".", ".."} for part in relative.parts if relative_text):
            raise CoordinatorError("not_found", "Unknown UI asset")
        candidate = (self.dist_root / Path(*relative.parts)).resolve()
        if not candidate.is_relative_to(self.dist_root):
            raise CoordinatorError("not_found", "Unknown UI asset")
        if candidate.is_file():
            return self._response(candidate, index=False)
        if relative.suffix:
            raise CoordinatorError("not_found", "Unknown UI asset")
        index = self.dist_root / "index.html"
        if not index.is_file():
            raise CoordinatorError("control_assets_missing", "Control console has not been built")
        return self._response(index, index=True)

    @staticmethod
    def _content_type(path: Path) -> str:
        content_type, _encoding = mimetypes.guess_type(path.name)
        if path.suffix == ".js":
            return "text/javascript; charset=utf-8"
        if path.suffix in {".html", ".css", ".json", ".svg"}:
            return f"{content_type or 'text/plain'}; charset=utf-8"
        return content_type or "application/octet-stream"

    def _response(self, path: Path, *, index: bool) -> BinaryResponse:
        cache = "no-store"
        if not index and _HASHED_ASSET.search(path.name):
            cache = "public,max-age=31536000,immutable"
        headers = {
            "Content-Type": self._content_type(path),
            "Cache-Control": cache,
            "X-Content-Type-Options": "nosniff",
        }
        if index:
            headers.update(
                {
                    "X-Frame-Options": "DENY",
                    "Referrer-Policy": "same-origin",
                    "Permissions-Policy": (
                        "camera=(), geolocation=(), microphone=(), payment=(), usb=()"
                    ),
                }
            )
        return BinaryResponse(
            200,
            path.read_bytes(),
            headers,
        )
