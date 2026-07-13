import json
import unittest

from tools.zircon_export.cli_arguments import parse_args
from tools.zircon_export.preset_contract import load_export_preset


def write_preset(path, profile="desktop_windows"):
    path.write_text(
        json.dumps(
            {
                "$zircon": {
                    "header": {
                        "schema_id": "zircon.export-preset",
                        "schema_version": 0,
                    },
                    "payload": {
                        "profile_ref": profile,
                        "target_mode": "client_runtime",
                        "include_filter": "assets/**",
                    },
                }
            }
        ),
        encoding="utf-8",
    )


class PresetContractTests(unittest.TestCase):
    def setUp(self):
        import tempfile

        self.temp = tempfile.TemporaryDirectory()
        from pathlib import Path

        self.preset = Path(self.temp.name) / "desktop.zpreset"
        write_preset(self.preset)

    def tearDown(self):
        self.temp.cleanup()

    def test_load_export_preset_requires_the_versioned_envelope(self):
        self.assertEqual(
            load_export_preset(str(self.preset))["profile_ref"],
            "desktop_windows",
        )

    def test_cli_projects_preset_profile_and_include_filter(self):
        args = parse_args(
            [
                "--profile",
                "desktop_windows",
                "--preset",
                str(self.preset),
                "--stage",
                "validate",
            ]
        )
        self.assertEqual(args.export_preset["target_mode"], "client_runtime")
        self.assertEqual(args.asset_filter, "assets/**")
        self.assertFalse(args.preset_debug)
        self.assertEqual(args.preset_cook["compression"], "zstd")

    def test_cli_rejects_profile_that_disagrees_with_preset(self):
        with self.assertRaises(SystemExit):
            parse_args(
                [
                    "--profile",
                    "server_linux",
                    "--preset",
                    str(self.preset),
                ]
            )

    def test_contract_rejects_unknown_and_wrongly_typed_payload_fields(self):
        document = json.loads(self.preset.read_text(encoding="utf-8"))
        document["$zircon"]["payload"]["debug"] = "false"
        self.preset.write_text(json.dumps(document), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "debug must be a boolean"):
            load_export_preset(str(self.preset))

        document["$zircon"]["payload"]["debug"] = False
        document["$zircon"]["payload"]["legacy_profile"] = "desktop"
        self.preset.write_text(json.dumps(document), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "unknown fields"):
            load_export_preset(str(self.preset))

    def test_contract_rejects_boolean_version_and_absolute_rel_path(self):
        document = json.loads(self.preset.read_text(encoding="utf-8"))
        document["$zircon"]["header"]["schema_version"] = False
        self.preset.write_text(json.dumps(document), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "schema_version"):
            load_export_preset(str(self.preset))

        document["$zircon"]["header"]["schema_version"] = 0
        document["$zircon"]["payload"]["customized_files"] = {
            "C:/outside.txt": "include"
        }
        self.preset.write_text(json.dumps(document), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "must be relative"):
            load_export_preset(str(self.preset))
