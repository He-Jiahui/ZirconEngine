from __future__ import annotations

import unittest

from tools.session_coordinator.control_plane.http_security import (
    validate_browser_read_origin,
    validate_loopback_host,
    validate_loopback_origin,
)
from tools.session_coordinator.models import CoordinatorError


class ControlSecurityTests(unittest.TestCase):
    def test_exact_loopback_hosts_and_origins_are_allowed(self) -> None:
        validate_loopback_host("127.0.0.1:14250", 14250)
        validate_loopback_host("localhost:14250", 14250)
        validate_loopback_origin("http://127.0.0.1:14250", 14250)
        validate_loopback_origin("http://localhost:14250", 14250)

    def test_non_loopback_or_wrong_port_values_are_rejected(self) -> None:
        for host in ("example.com:14250", "127.0.0.1:80", "localhost.evil:14250"):
            with self.subTest(host=host), self.assertRaises(CoordinatorError):
                validate_loopback_host(host, 14250)
        for origin in ("https://127.0.0.1:14250", "http://example.com:14250", "null"):
            with self.subTest(origin=origin), self.assertRaises(CoordinatorError):
                validate_loopback_origin(origin, 14250)

    def test_same_origin_browser_get_metadata_replaces_missing_origin(self) -> None:
        validate_browser_read_origin(
            None,
            "http://127.0.0.1:14250/ui/workflows/run-a",
            "same-origin",
            14250,
        )
        for referer, site in (
            ("http://example.com/ui/", "same-origin"),
            ("http://127.0.0.1:14250/ui/", "cross-site"),
            (None, "same-origin"),
        ):
            with self.subTest(referer=referer, site=site), self.assertRaises(CoordinatorError):
                validate_browser_read_origin(None, referer, site, 14250)


if __name__ == "__main__":
    unittest.main()
