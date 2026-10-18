"""Static contract for bounded HTTP response-body collection."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
BACKEND = ROOT / "zircon_plugins/net/features/http/runtime/src/backend.rs"
CLIENT = ROOT / "zircon_plugins/net/features/http/runtime/src/backend/client.rs"
ROUTES = ROOT / "zircon_plugins/net/features/http/runtime/src/tests/routes.rs"


class NetHttpResponseBodyLimitContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.backend = BACKEND.read_text(encoding="utf-8")
        cls.client = CLIENT.read_text(encoding="utf-8")
        cls.routes = ROUTES.read_text(encoding="utf-8")

    def test_both_http_clients_apply_one_response_body_limit(self) -> None:
        self.assertIn("HTTP_RESPONSE_BODY_LIMIT_BYTES", self.backend)
        self.assertIn(
            "Limited::new(response.into_body(), HTTP_RESPONSE_BODY_LIMIT_BYTES)",
            self.client,
        )
        self.assertIn("while let Some(chunk) = response", self.client)
        self.assertIn(".chunk()", self.client)
        self.assertIn("append_response_chunk(&mut body, &chunk)?", self.client)
        self.assertIn("checked_add(chunk.len())", self.client)
        self.assertNotIn(".into_body()\n        .collect()", self.client)
        self.assertNotIn(".bytes()\n        .await", self.client)

    def test_oversized_response_regression_is_wired(self) -> None:
        self.assertIn(
            "http_feature_manager_rejects_oversized_response_body_before_publication",
            self.routes,
        )
        self.assertIn("HTTP_RESPONSE_BODY_LIMIT_BYTES + 1", self.routes)
        self.assertIn("HTTP response body exceeds", self.routes)


if __name__ == "__main__":
    unittest.main()
