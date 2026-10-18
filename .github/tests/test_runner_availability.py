from __future__ import annotations

import importlib.util
import io
import json
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/check-runner-availability.py"
SPEC = importlib.util.spec_from_file_location("runner_availability", SCRIPT)
availability = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(availability)


def runner(status="online", labels=("self-hosted", "Windows", "X64"), **extra):
    return {"status": status, "labels": [{"name": name} for name in labels], **extra}


class RunnerAvailabilityTests(unittest.TestCase):
    def test_empty_offline_and_wrong_platform_do_not_satisfy_gate(self):
        self.assertEqual(availability.online_matching_runners([]), [])
        self.assertEqual(availability.online_matching_runners([
            runner("offline"), runner(labels=("self-hosted", "Linux", "X64")),
        ]), [])

    def test_busy_online_runner_and_case_insensitive_labels_are_available(self):
        online = runner(labels=("SELF-HOSTED", "windows", "x64"), busy=True)
        self.assertEqual(availability.online_matching_runners([online]), [online])

    def test_malformed_inventory_fails_closed(self):
        with self.assertRaises(ValueError):
            availability.online_matching_runners([{"status": "online", "labels": None}])
        with patch.object(availability, "urlopen", return_value=io.BytesIO(b'{}')):
            with self.assertRaises(ValueError):
                availability.fetch_runners("owner/repo", "test-token")

    def test_missing_token_or_invalid_repository_never_sends_request(self):
        with patch.object(availability, "urlopen") as request:
            for repository, token in (("owner/repo", ""), ("../bad", "test-token")):
                with self.assertRaises(ValueError):
                    availability.fetch_runners(repository, token)
            request.assert_not_called()

    def test_pagination_includes_online_runner_on_next_page(self):
        first = io.BytesIO(json.dumps({"runners": [runner("offline")] * 100}).encode())
        second = io.BytesIO(json.dumps({"runners": [runner()]}).encode())
        with patch.object(availability, "urlopen", side_effect=[first, second]) as request:
            result = availability.fetch_runners("owner/repo", "test-token")
        self.assertEqual(len(availability.online_matching_runners(result)), 1)
        self.assertIn("page=2", request.call_args.args[0].full_url)
        self.assertEqual(request.call_args.kwargs["timeout"], 15)

    def test_no_online_runner_returns_nonzero(self):
        with patch("sys.argv", [str(SCRIPT), "--repository", "owner/repo"]), \
             patch.object(availability, "fetch_runners", return_value=[runner("offline")]), \
             patch("sys.stderr", new_callable=io.StringIO) as stderr:
            self.assertEqual(availability.main(), 1)
        self.assertIn("No online self-hosted Windows X64 runner", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
