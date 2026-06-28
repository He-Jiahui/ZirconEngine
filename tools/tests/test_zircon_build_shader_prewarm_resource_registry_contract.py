import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_build_shader_prewarm import (
    validate_shader_resource_registry_export_contract,
)


class ZirconBuildShaderPrewarmResourceRegistryContractTests(unittest.TestCase):
    def test_validate_registry_export_contract_requires_resource_records(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(json.dumps({}), encoding="utf-8")

            with self.assertRaisesRegex(
                RuntimeError,
                "did not produce a ResourceRecord array",
            ):
                validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_accepts_wrapped_resources(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps({"resources": []}),
                encoding="utf-8",
            )

            validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_accepts_raw_array(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(json.dumps([]), encoding="utf-8")

            validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_rejects_incomplete_resource_record(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "resources": [
                            {
                                "kind": "Shader",
                                "primary_locator": "res://shaders/example",
                                "revision": 1,
                            },
                        ]
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "incomplete ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_rejects_tagged_enum_resource_record(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "resources": [
                            _resource_record(
                                kind={"type": "Shader"},
                                state={"state": "Ready"},
                            ),
                        ]
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "incomplete ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_rejects_u64_revision_overflow(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "resources": [
                            _resource_record(revision=2**64),
                        ]
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "incomplete ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_rejects_u32_importer_version_overflow(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            record = _resource_record()
            record["importer_version"] = 2**32
            registry_path.write_text(
                json.dumps({"resources": [record]}),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "incomplete ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_rejects_invalid_locator_wire_shape(
        self,
    ):
        invalid_locators = (
            "file://shaders/example",
            "res://",
            "res:///absolute",
            "res://C:/outside",
            "res://../outside",
            "res://shaders/example#",
            "package://package-only",
            "package://C:/shader",
            "package://zircon//shader",
        )
        for locator in invalid_locators:
            with self.subTest(locator=locator):
                with tempfile.TemporaryDirectory() as temp_dir:
                    registry_path = Path(temp_dir) / "shader_resource_records.json"
                    registry_path.write_text(
                        json.dumps(
                            {
                                "resources": [
                                    _resource_record(primary_locator=locator),
                                ]
                            }
                        ),
                        encoding="utf-8",
                    )

                    with self.assertRaisesRegex(
                        RuntimeError,
                        "incomplete ResourceRecord",
                    ):
                        validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_accepts_locator_wire_shape_variants(
        self,
    ):
        valid_locators = (
            "res://shaders/example",
            "res://shaders//example",
            "lib://render/shaders/example",
            "package://zircon/shaders/example",
            "builtin://shader/pbr.wgsl",
            "mem://generated/shader",
            "res://shaders/example#fragment",
        )
        for locator in valid_locators:
            with self.subTest(locator=locator):
                with tempfile.TemporaryDirectory() as temp_dir:
                    registry_path = Path(temp_dir) / "shader_resource_records.json"
                    registry_path.write_text(
                        json.dumps(
                            {
                                "resources": [
                                    _resource_record(primary_locator=locator),
                                ]
                            }
                        ),
                        encoding="utf-8",
                    )

                    validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_rejects_invalid_artifact_locator(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            record = _resource_record()
            record["artifact_locator"] = "file://generated/shader.wgsl"
            registry_path.write_text(
                json.dumps({"resources": [record]}),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "incomplete ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(registry_path)

    def test_validate_registry_export_contract_rejects_missing_report_source_locator(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(json.dumps({"resources": []}), encoding="utf-8")
            report_path = _write_report_source(
                temp_dir,
                "res://shaders/example",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "missing ResourceRecord locators",
            ):
                validate_shader_resource_registry_export_contract(
                    registry_path,
                    report_path=report_path,
                )

    def test_validate_registry_export_contract_rejects_missing_written_variant_locator(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(json.dumps({"resources": []}), encoding="utf-8")
            report_path = _write_report_written_variant(
                temp_dir,
                "res://shaders/example",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "missing ResourceRecord locators",
            ):
                validate_shader_resource_registry_export_contract(
                    registry_path,
                    report_path=report_path,
                )

    def test_validate_registry_export_contract_accepts_report_source_locator(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "resources": [
                            _resource_record(),
                        ]
                    }
                ),
                encoding="utf-8",
            )
            report_path = _write_report_source(
                temp_dir,
                "res://shaders/example",
            )

            validate_shader_resource_registry_export_contract(
                registry_path,
                report_path=report_path,
            )

    def test_validate_registry_export_contract_accepts_registry_backed_source_locators(
        self,
    ):
        locators = (
            "res://shaders/example",
            "lib://plugin/shaders/example",
            "package://demo/shaders/example",
            "mem://generated/shader",
        )
        for locator in locators:
            with self.subTest(locator=locator):
                with tempfile.TemporaryDirectory() as temp_dir:
                    registry_path = Path(temp_dir) / "shader_resource_records.json"
                    registry_path.write_text(
                        json.dumps(
                            {
                                "resources": [
                                    _resource_record(primary_locator=locator),
                                ]
                            }
                        ),
                        encoding="utf-8",
                    )
                    report_path = _write_report_source(temp_dir, locator)

                    validate_shader_resource_registry_export_contract(
                        registry_path,
                        report_path=report_path,
                    )

    def test_validate_registry_export_contract_rejects_missing_registry_backed_source_locator(
        self,
    ):
        for locator in (
            "lib://plugin/shaders/example",
            "package://demo/shaders/example",
            "mem://generated/shader",
        ):
            with self.subTest(locator=locator):
                with tempfile.TemporaryDirectory() as temp_dir:
                    registry_path = Path(temp_dir) / "shader_resource_records.json"
                    registry_path.write_text(
                        json.dumps({"resources": []}),
                        encoding="utf-8",
                    )
                    report_path = _write_report_source(temp_dir, locator)

                    with self.assertRaisesRegex(
                        RuntimeError,
                        "missing ResourceRecord locators",
                    ):
                        validate_shader_resource_registry_export_contract(
                            registry_path,
                            report_path=report_path,
                        )

    def test_validate_registry_export_contract_rejects_non_shader_report_source_record(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "resources": [
                            _resource_record(kind="Texture"),
                        ]
                    }
                ),
                encoding="utf-8",
            )
            report_path = _write_report_source(
                temp_dir,
                "res://shaders/example",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "usable shader ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(
                    registry_path,
                    report_path=report_path,
                )

    def test_validate_registry_export_contract_rejects_zero_revision_report_source_record(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "resources": [
                            _resource_record(revision=0),
                        ]
                    }
                ),
                encoding="utf-8",
            )
            report_path = _write_report_source(
                temp_dir,
                "res://shaders/example",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "usable shader ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(
                    registry_path,
                    report_path=report_path,
                )

    def test_validate_registry_export_contract_rejects_non_ready_report_source_record(
        self,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "resources": [
                            _resource_record(state="Error"),
                        ]
                    }
                ),
                encoding="utf-8",
            )
            report_path = _write_report_source(
                temp_dir,
                "res://shaders/example",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "usable shader ResourceRecord",
            ):
                validate_shader_resource_registry_export_contract(
                    registry_path,
                    report_path=report_path,
                )

    def test_validate_registry_export_contract_ignores_builtin_report_sources(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_resource_records.json"
            registry_path.write_text(json.dumps({"resources": []}), encoding="utf-8")
            report_path = _write_report_source(
                temp_dir,
                "builtin://shader/pbr.wgsl",
            )

            validate_shader_resource_registry_export_contract(
                registry_path,
                report_path=report_path,
            )


def _resource_record(
    *,
    id_value: str = "00000000-0000-0000-0000-000000000001",
    kind: object = "Shader",
    primary_locator: str = "res://shaders/example",
    revision: int = 1,
    state: object = "Ready",
) -> dict[str, object]:
    return {
        "id": id_value,
        "kind": kind,
        "primary_locator": primary_locator,
        "artifact_locator": None,
        "revision": revision,
        "state": state,
        "dependency_ids": [],
        "diagnostics": [],
        "source_hash": "shader-source-hash",
        "importer_id": "zircon_shader_importer",
        "importer_version": 1,
        "config_hash": "shader-config-hash",
    }


def _write_report_source(temp_dir: str, source_label: str) -> Path:
    report_path = Path(temp_dir) / "shader_variants_report.json"
    report_path.write_text(
        json.dumps(
            {
                "requested_count": 1,
                "written_count": 1,
                "failed_count": 0,
                "source_provenance": {
                    "source_count": 1,
                    "variant_count": 1,
                    "sources": {
                        f"{source_label}#source-a#template-a": {
                            "source_label": source_label,
                            "source_hash": "source-a",
                            "template_revision": "template-a",
                            "requested_count": 1,
                            "written_count": 1,
                            "failed_count": 0,
                        }
                    },
                },
            }
        ),
        encoding="utf-8",
    )
    return report_path


def _write_report_written_variant(temp_dir: str, source_label: str) -> Path:
    report_path = Path(temp_dir) / "shader_variants_report.json"
    report_path.write_text(
        json.dumps(
            {
                "requested_count": 1,
                "written_count": 1,
                "failed_count": 0,
                "written_variants": [
                    {
                        "shader_id": "material://example",
                        "source_label": source_label,
                        "cache_key": "cache-key",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    return report_path
