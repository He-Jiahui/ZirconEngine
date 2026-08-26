from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT_PATH = (
    Path(__file__).resolve().parents[2]
    / "docs"
    / "plans"
    / "performance"
    / "01"
    / "renderdoc_capture_audit.py"
)


class _GPUCounter:
    EventGPUDuration = 7


def _load_audit_module():
    renderdoc = types.ModuleType("renderdoc")
    renderdoc.GPUCounter = _GPUCounter
    renderdoc.ResultCode = types.SimpleNamespace(Succeeded=0)
    spec = importlib.util.spec_from_file_location("renderdoc_capture_audit_test", SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    with patch.dict(sys.modules, {"renderdoc": renderdoc}):
        spec.loader.exec_module(module)
    return module


class _Controller:
    def __init__(self, counters, results):
        self.counters = counters
        self.results = results
        self.fetch_count = 0

    def EnumerateCounters(self):
        return self.counters

    def DescribeCounter(self, counter):
        return types.SimpleNamespace(name=f"counter-{counter}")

    def FetchCounters(self, counters):
        self.fetch_count += 1
        return self.results


def _sample(event_id: int, seconds: float):
    return types.SimpleNamespace(
        eventId=event_id,
        value=types.SimpleNamespace(d=seconds),
    )


class RenderDocCaptureAuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.audit = _load_audit_module()

    def test_gpu_duration_reports_counter_not_exposed(self):
        controller = _Controller([], [])

        descriptions, report = self.audit._gpu_duration_report(controller, {})

        self.assertEqual([], descriptions)
        self.assertEqual("unavailable_counter_not_exposed", report["status"])
        self.assertEqual(0, report["sample_count"])
        self.assertIsNone(report["total_ms"])
        self.assertEqual([], report["top_25"])
        self.assertEqual(0, controller.fetch_count)

    def test_gpu_duration_reports_exposed_counter_without_samples(self):
        controller = _Controller([_GPUCounter.EventGPUDuration], [])

        descriptions, report = self.audit._gpu_duration_report(controller, {})

        self.assertEqual(
            [{"id": _GPUCounter.EventGPUDuration, "name": "counter-7"}],
            descriptions,
        )
        self.assertEqual("unavailable_no_samples", report["status"])
        self.assertEqual(0, report["sample_count"])
        self.assertIsNone(report["total_ms"])
        self.assertEqual([], report["top_25"])
        self.assertEqual(1, controller.fetch_count)

    def test_gpu_duration_reports_measured_samples(self):
        controller = _Controller(
            [_GPUCounter.EventGPUDuration],
            [_sample(2, 0.001), _sample(1, 0.003)],
        )
        actions = {
            1: {"name": "slow", "flags": "Drawcall"},
            2: {"name": "fast", "flags": "Dispatch"},
        }

        _, report = self.audit._gpu_duration_report(controller, actions)

        self.assertEqual("available", report["status"])
        self.assertEqual(2, report["sample_count"])
        self.assertAlmostEqual(4.0, report["total_ms"])
        self.assertEqual([1, 2], [row["event_id"] for row in report["top_25"]])
        self.assertEqual("slow", report["top_25"][0]["name"])


if __name__ == "__main__":
    unittest.main()
