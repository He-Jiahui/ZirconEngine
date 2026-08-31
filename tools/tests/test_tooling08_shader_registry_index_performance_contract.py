from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools import zircon_build_shader_resource_registry as resource_registry


class _CountingRecords(list[object]):
    def __init__(self, values: list[object]) -> None:
        super().__init__(values)
        self.iteration_count = 0

    def __iter__(self):
        self.iteration_count += 1
        return super().__iter__()


def _resource_record(index: int) -> dict[str, object]:
    locator = f"res://shaders/indexed-{index}.wgsl"
    return {
        "id": f"00000000-0000-0000-0000-{index:012d}",
        "kind": "Shader",
        "primary_locator": locator,
        "artifact_locator": f"lib://shaders/indexed-{index}.spv",
        "revision": 1,
        "state": "Ready",
        "dependency_ids": [],
        "diagnostics": [],
        "source_hash": "",
        "importer_id": "",
        "importer_version": 0,
        "config_hash": "",
    }


class Tooling08ShaderRegistryIndexPerformanceContractTests(unittest.TestCase):
    def test_validation_builds_report_indices_in_the_shape_validation_pass(self) -> None:
        records = _CountingRecords([_resource_record(1), _resource_record(2)])
        report = {
            "written_variants": [
                {"source_label": "res://shaders/indexed-1.wgsl"},
                {"source_label": "res://shaders/indexed-2.wgsl"},
            ]
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "registry.json"
            report_path = Path(temp_dir) / "report.json"
            registry_path.write_text("[]", encoding="utf-8")
            report_path.write_text(json.dumps(report), encoding="utf-8")
            with mock.patch.object(resource_registry.json, "loads") as loads:
                loads.side_effect = [records, report]
                resource_registry.validate_shader_resource_registry_export_contract(
                    registry_path,
                    report_path=report_path,
                    require_usable_shader_records=True,
                    require_report_registry_backed_sources=True,
                )

        self.assertEqual(1, records.iteration_count)

    def test_legacy_structure_guard_helpers_remain_available(self) -> None:
        source = Path(resource_registry.__file__).read_text(encoding="utf-8")
        self.assertIn("def _resource_record_locators(", source)
        self.assertIn("def _usable_shader_resource_record_locators(", source)
        self.assertIn("def _validate_and_index_resource_records(", source)


if __name__ == "__main__":
    unittest.main()
