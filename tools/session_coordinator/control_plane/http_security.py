from __future__ import annotations

from urllib.parse import SplitResult, urlsplit

from ..models import CoordinatorError


def validate_loopback_host(host_header: str | None, port: int) -> None:
    allowed = {f"127.0.0.1:{port}", f"localhost:{port}"}
    if host_header not in allowed:
        raise CoordinatorError(
            "invalid_host", "Control API requests require the bound loopback Host"
        )


def _parse_loopback_http_url(value: str, port: int) -> SplitResult | None:
    try:
        parsed = urlsplit(value)
        parsed_port = parsed.port
    except ValueError:
        return None
    if (
        parsed.scheme != "http"
        or parsed.hostname not in {"127.0.0.1", "localhost"}
        or parsed_port != port
        or parsed.username is not None
        or parsed.password is not None
    ):
        return None
    return parsed


def validate_loopback_origin(origin: str | None, port: int, *, required: bool = True) -> None:
    if not origin:
        if required:
            raise CoordinatorError("origin_required", "Browser control requests require Origin")
        return
    parsed = _parse_loopback_http_url(origin, port)
    if parsed is None or (
        parsed.path not in {"", "/"}
        or parsed.query
        or parsed.fragment
    ):
        raise CoordinatorError(
            "invalid_origin", "Control API requests require the bound loopback Origin"
        )


def validate_browser_read_origin(
    origin: str | None,
    referer: str | None,
    fetch_site: str | None,
    port: int,
) -> None:
    """Authenticate the browser origin signals emitted by safe same-origin GETs."""
    if origin:
        validate_loopback_origin(origin, port)
        return
    if fetch_site != "same-origin" or not referer:
        raise CoordinatorError(
            "origin_required", "Browser control reads require same-origin fetch metadata"
        )
    parsed = _parse_loopback_http_url(referer, port)
    if parsed is None or not parsed.path.startswith("/ui/"):
        raise CoordinatorError(
            "invalid_origin", "Control API requests require a loopback UI referrer"
        )
