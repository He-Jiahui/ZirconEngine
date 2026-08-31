from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.platform_bundle_template_files_materialize import (
    materialize_platform_bundle_template_files,
)


class Tooling03PlatformBundleTemplateHostResolveCachePerformanceContractTests(
    unittest.TestCase
):
    def test_host_path_is_resolved_once_for_many_template_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_dir = root / "template"
            template_dir.mkdir()
            host = root / "zircon_runtime.exe"
            host.write_bytes(b"host")
            files = []
            for index in range(64):
                name = f"file-{index}.json"
                template_dir.joinpath(name).write_text("{}", encoding="utf-8")
                files.append({"path": name})
            original_resolve = Path.resolve
            host_resolves = 0

            def observed_resolve(
                path: Path,
                *args: object,
                **kwargs: object,
            ) -> Path:
                nonlocal host_resolves
                if path == host:
                    host_resolves += 1
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", new=observed_resolve), mock.patch(
                "tools.zircon_export.platform_bundle_template_files_materialize.copy_platform_bundle_template_file",
                return_value=True,
            ):
                fatal, copied = materialize_platform_bundle_template_files(
                    bundle_root=root / "bundle",
                    template_report={
                        "template_dir": str(template_dir),
                        "files": files,
                    },
                    host_executable=host,
                    host_destination=root / "bundle" / host.name,
                    diagnostics=[],
                )

        self.assertFalse(fatal)
        self.assertEqual(len(copied), 64)
        self.assertEqual(host_resolves, 1)

    def test_empty_template_file_list_does_not_resolve_host(self) -> None:
        host = Path("zircon_runtime.exe")
        with mock.patch.object(
            Path,
            "resolve",
            side_effect=AssertionError("unused host path was resolved"),
        ):
            fatal, copied = materialize_platform_bundle_template_files(
                bundle_root=Path("bundle"),
                template_report={"template_dir": "template", "files": []},
                host_executable=host,
                host_destination=Path("bundle") / host.name,
                diagnostics=[],
            )

        self.assertFalse(fatal)
        self.assertEqual(copied, [])


if __name__ == "__main__":
    unittest.main()
