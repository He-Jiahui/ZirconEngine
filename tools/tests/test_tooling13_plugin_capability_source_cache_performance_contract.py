from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.plugin_structure_audits import capability


class Tooling13PluginCapabilitySourceCachePerformanceContractTests(
    unittest.TestCase
):
    def test_capability_audit_reads_each_capability_owner_once(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir)
            for root in capability.FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS:
                runtime_src = repo_root / "zircon_plugins" / root / "runtime" / "src"
                runtime_src.mkdir(parents=True)
                (runtime_src / "capability.rs").write_text(
                    "\n".join(
                        [
                            'pub const ROOT_CAPABILITY: &str = "runtime.root";',
                            "pub const RUNTIME_CAPABILITIES: &[&str] = &[ROOT_CAPABILITY];",
                        ]
                    ),
                    encoding="utf-8",
                )
                (repo_root / "zircon_plugins" / root / "plugin.toml").write_text(
                    "\n".join(
                        [
                            'capabilities = ["runtime.root"]',
                            "[[modules]]",
                            'kind = "runtime"',
                            f'crate_name = "zircon_plugin_{root}_runtime"',
                            'capabilities = ["runtime.root"]',
                        ]
                    ),
                    encoding="utf-8",
                )

            original_read_text = Path.read_text
            capability_reads = 0

            def read_text_counted(path: Path, *args: object, **kwargs: object) -> str:
                nonlocal capability_reads
                if path.name == "capability.rs":
                    capability_reads += 1
                return original_read_text(path, *args, **kwargs)

            with mock.patch.object(
                Path,
                "read_text",
                read_text_counted,
            ), mock.patch.object(
                capability,
                "collect_native_abi_projection_violations",
                return_value=[],
            ), mock.patch.object(
                capability,
                "collect_sdk_builder_mirror_violations",
                return_value=[],
            ), mock.patch.object(
                capability,
                "collect_editor_runtime_mirror_violations",
                return_value=[],
            ), mock.patch.object(
                capability,
                "collect_lib_capability_literal_sites",
                return_value=None,
            ):
                audit = capability.audit_plugin_capability_conformance(repo_root)

        self.assertEqual(audit.runtime_capability_mismatch_details(), [])
        self.assertEqual(
            capability_reads,
            len(capability.FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS),
            "the audit must share one source snapshot across both parsers",
        )


if __name__ == "__main__":
    unittest.main()
