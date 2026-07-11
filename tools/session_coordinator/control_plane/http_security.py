from __future__ import annotations

from urllib.parse import urlsplit

from ..models import CoordinatorError


def validate_loopback_host(host_header: str | None, port: int) -> None:
    allowed = {f"127.0.0.1:{port}", f"localhost:{port}"}
    if host_header not in allowed:
        raise CoordinatorError(
            "invalid_host", "Control API requests require the bound loopback Host"
        )


def validate_loopback_origin(origin: str | None, port: int, *, required: bool = True) -> None:
    if not origin:
        if required:
            raise CoordinatorError("origin_required", "Browser control requests require Origin")
        return
    parsed = urlsplit(origin)
    if (
        parsed.scheme != "http"
        or parsed.path not in {"", "/"}
        or parsed.query
        or parsed.fragment
        or parsed.hostname not in {"127.0.0.1", "localhost"}
        or parsed.port != port
    ):
        raise CoordinatorError(
            "invalid_origin", "Control API requests require the bound loopback Origin"
        )
