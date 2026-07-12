from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.processes import current_process_identity
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION
from tools.session_coordinator.supervision.repository_identity import repository_identity
from tools.session_coordinator.supervision.runtime_descriptor import RuntimeDescriptor


class RuntimeDescriptorTests(unittest.TestCase):
    def test_repository_identity_matches_lowercase_resolved_path_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            first = repository_identity(directory)
            second = repository_identity(Path(directory).resolve())

        self.assertEqual(1, first.version)
        self.assertEqual(first, second)
        self.assertEqual(64, len(first.key))
        self.assertEqual(first.key[:10].upper(), first.short_key)

    def test_descriptor_binds_process_repository_schema_and_capability(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            identity = current_process_identity()
            descriptor = RuntimeDescriptor(
                "127.0.0.1",
                43123,
                "secret-runtime-token",
                root,
                repository_identity(root),
                "instance-a",
                "now",
                identity,
            )

            payload = descriptor.to_payload()
            diagnostic = descriptor.diagnostic_payload()

        self.assertEqual(2, payload["descriptor_version"])
        self.assertEqual(os.getpid(), payload["pid"])
        self.assertEqual(identity.creation_time, payload["process_creation_time"])
        self.assertEqual(LATEST_SCHEMA_VERSION, payload["schema_version"])
        self.assertEqual([1], payload["supervision_api_versions"])
        self.assertEqual("secret-runtime-token", payload["token"])
        self.assertNotIn("token", diagnostic)

    def test_descriptor_rejects_non_exact_loopback(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            descriptor = RuntimeDescriptor(
                "localhost",
                1,
                "token",
                root,
                repository_identity(root),
                "instance",
                "now",
                current_process_identity(),
            )
            with self.assertRaises(ValueError):
                descriptor.to_payload()


if __name__ == "__main__":
    unittest.main()
