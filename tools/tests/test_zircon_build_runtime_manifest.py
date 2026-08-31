import hashlib
import json
import tempfile
import types
import unittest
from pathlib import Path


class ZirconBuildRuntimeManifestTests(unittest.TestCase):
    def test_runtime_manifest_binds_library_and_staged_host_artifacts_to_one_build_set(self):
        from tools.zircon_build_runtime_manifest import (
            runtime_artifact_manifest_path,
            runtime_host_file_names,
            runtime_library_file_name,
            write_runtime_artifact_manifest,
        )

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo_root = root / "repo"
            engine_root = root / "out" / "ZirconEngine"
            library_name = runtime_library_file_name()
            self._write(
                repo_root
                / "zircon_runtime_interface"
                / "src"
                / "runtime_build_set"
                / "interface_spec_v1.json",
                json.dumps(
                    {
                        "family": "zircon.runtime.internal",
                        "spec_version": 1,
                        "runtime_api_version": 7,
                        "entry_symbol": "zircon_runtime_get_api_v7",
                        "runtime_api_required_slots": [],
                        "runtime_api_optional_slots": [],
                        "host_api_optional_slots": [],
                    },
                    separators=(",", ":"),
                ).encode(),
            )
            self._write(
                repo_root
                / "zircon_runtime_interface"
                / "src"
                / "runtime_build_set"
                / "payload_schema_set_v1.json",
                b'{"family":"zircon.runtime.payload.internal","spec_version":1,"encoding":"utf-8","serialization":"entry-local-json","schema_status":"migration-baseline"}',
            )
            self._write(engine_root / library_name, b"runtime-library")
            host_names = runtime_host_file_names()
            self._write(engine_root / host_names[0], b"editor-host")
            self._write(engine_root / host_names[1], b"runtime-host")
            config = types.SimpleNamespace(
                repo_root=repo_root,
                engine_root=engine_root,
                mode="debug",
                runtime_features=("target-client",),
                dry_run=False,
            )

            manifest_path = write_runtime_artifact_manifest(config)
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

            self.assertEqual(
                runtime_artifact_manifest_path(engine_root / library_name), manifest_path
            )
            self.assertEqual(1, manifest["schema_version"])
            self.assertEqual("debug", manifest["build_mode"])
            self.assertEqual(["target-client"], manifest["runtime_features"])
            self.assertEqual(library_name, manifest["artifact"]["file_name"])
            self.assertEqual(
                hashlib.sha256(b"runtime-library").hexdigest(),
                manifest["artifact"]["sha256"],
            )
            self.assertEqual(
                hashlib.sha256(
                    b'{"family":"zircon.runtime.payload.internal","spec_version":1,"encoding":"utf-8","serialization":"entry-local-json","schema_status":"migration-baseline"}'
                ).hexdigest(),
                manifest["payload_schema_digest"],
            )
            self.assertEqual(
                set(host_names),
                {host["file_name"] for host in manifest["host_artifacts"]},
            )
            self.assertEqual(64, len(manifest["build_set_id"]))
            build_set_identity = {
                "artifact": manifest["artifact"],
                "build_mode": manifest["build_mode"],
                "capabilities": manifest["capabilities"],
                "host_artifacts": manifest["host_artifacts"],
                "interface_spec_digest": manifest["interface_spec_digest"],
                "payload_schema_digest": manifest["payload_schema_digest"],
                "runtime_features": manifest["runtime_features"],
                "target": {
                    "architecture": manifest["target"]["architecture"],
                    "operating_system": manifest["target"]["operating_system"],
                    "pointer_width": manifest["target"]["pointer_width"],
                    "endian": manifest["target"]["endian"],
                },
            }
            self.assertEqual(
                hashlib.sha256(
                    json.dumps(
                        build_set_identity, separators=(",", ":"), ensure_ascii=True
                    ).encode("utf-8")
                ).hexdigest(),
                manifest["build_set_id"],
            )
            self.assertEqual(
                "zircon.runtime.internal", manifest["interface_spec"]["family"]
            )

    def test_runtime_manifest_refuses_a_staged_runtime_without_a_host_executable(self):
        from tools.zircon_build_runtime_manifest import (
            runtime_library_file_name,
            write_runtime_artifact_manifest,
        )

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo_root = root / "repo"
            engine_root = root / "out" / "ZirconEngine"
            self._write(engine_root / runtime_library_file_name(), b"runtime-library")
            config = types.SimpleNamespace(
                repo_root=repo_root,
                engine_root=engine_root,
                mode="debug",
                runtime_features=("target-client",),
                dry_run=False,
            )

            with self.assertRaisesRegex(SystemExit, "host executable"):
                write_runtime_artifact_manifest(config)

    @staticmethod
    def _write(path: Path, contents: bytes) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(contents)


if __name__ == "__main__":
    unittest.main()
